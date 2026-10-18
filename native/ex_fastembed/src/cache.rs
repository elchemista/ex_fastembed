use crate::runtime::Identity;
use fastembed::{ModelInfo, RerankerModelInfo, TextEmbedding, TextRerank};
use hf_hub::{
    api::sync::{ApiBuilder, ApiError},
    Cache, Repo, RepoType,
};
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, rustler::NifUnitEnum)]
pub(crate) enum ModelKind {
    Embedding,
    Reranker,
}

#[derive(rustler::NifMap)]
pub(crate) struct ModelStatus {
    pub name: String,
    pub kind: ModelKind,
    pub repository: String,
    pub description: String,
    pub dimension: Option<usize>,
    pub cached: bool,
    pub loaded: bool,
    pub cache_dir: String,
    pub path: String,
    pub revision: Option<String>,
    pub files: Vec<String>,
    pub file_details: Vec<FileStatus>,
    pub variant_bytes: Option<u64>,
    pub disk_bytes: Option<u64>,
}

#[derive(rustler::NifMap)]
pub(crate) struct FileStatus {
    pub name: String,
    pub path: Option<String>,
    pub size_bytes: Option<u64>,
}

const TOKENIZER_FILES: [&str; 4] = [
    "tokenizer.json",
    "config.json",
    "special_tokens_map.json",
    "tokenizer_config.json",
];

struct CatalogEntry {
    name: String,
    kind: ModelKind,
    repository: String,
    description: String,
    dimension: Option<usize>,
    model_file: String,
    additional: Vec<String>,
}

impl CatalogEntry {
    fn embedding(info: &ModelInfo<fastembed::EmbeddingModel>) -> Self {
        Self {
            name: info.model.to_string(),
            kind: ModelKind::Embedding,
            repository: info.model_code.clone(),
            description: info.description.clone(),
            dimension: Some(info.dim),
            model_file: info.model_file.clone(),
            additional: info.additional_files.clone(),
        }
    }

    fn reranker(info: RerankerModelInfo) -> Self {
        Self {
            name: format!("{:?}", info.model),
            kind: ModelKind::Reranker,
            repository: info.model_code,
            description: info.description,
            dimension: None,
            model_file: info.model_file,
            additional: info.additional_files,
        }
    }
}

fn lookup_entry(name: &str, kind: ModelKind) -> Result<CatalogEntry, String> {
    match kind {
        ModelKind::Embedding => {
            let model = super::resolve_embedding_model(name)?;
            TextEmbedding::get_model_info(&model)
                .map(CatalogEntry::embedding)
                .map_err(|error| error.to_string())
        }
        ModelKind::Reranker => {
            let model = super::resolve_reranker_model(name)?;
            Ok(CatalogEntry::reranker(TextRerank::get_model_info(&model)))
        }
    }
}

#[cfg_attr(not(test), rustler::nif(schedule = "DirtyIo"))]
fn models(cache_dir: String) -> Vec<ModelStatus> {
    let cache = Cache::new(effective_cache_dir(cache_dir));
    // Snapshot each family once; walking the catalog never reacquires slot locks.
    let embedding = loaded_identity(ModelKind::Embedding);
    let reranker = loaded_identity(ModelKind::Reranker);
    let mut models: Vec<_> = TextEmbedding::list_supported_models()
        .iter()
        .map(|info| status(CatalogEntry::embedding(info), &cache, embedding.as_ref()))
        .chain(
            TextRerank::list_supported_models()
                .into_iter()
                .map(|info| status(CatalogEntry::reranker(info), &cache, reranker.as_ref())),
        )
        .collect();
    models.sort_by(|left, right| (left.kind, &left.name).cmp(&(right.kind, &right.name)));
    models
}

#[cfg_attr(not(test), rustler::nif(schedule = "DirtyIo"))]
fn model_info(name: String, kind: ModelKind, cache_dir: String) -> Result<ModelStatus, String> {
    let entry = lookup_entry(&name, kind)?;
    let cache = Cache::new(effective_cache_dir(cache_dir));
    Ok(status(entry, &cache, loaded_identity(kind).as_ref()))
}

#[cfg_attr(not(test), rustler::nif(schedule = "DirtyIo"))]
fn download(name: String, kind: ModelKind, cache_dir: String) -> Result<bool, String> {
    let entry = lookup_entry(&name, kind)?;
    // Serialize cache preparation with loads and deletion: two first downloads
    // must not choose different refs/main revisions before initialization reads it.
    // Existing inference sessions are not locked or changed.
    let _guard = super::cache_write();
    prepare_model_files(
        cache_dir,
        &entry.repository,
        &entry.model_file,
        &entry.additional,
    )?;
    Ok(true)
}

fn required_files(model_file: &str, additional: &[String]) -> Vec<String> {
    let mut files: Vec<_> = std::iter::once(model_file)
        .chain(TOKENIZER_FILES)
        .chain(additional.iter().map(String::as_str))
        .map(String::from)
        .collect();
    files.sort();
    files.dedup();
    files
}

fn loaded_identity(kind: ModelKind) -> Option<Identity> {
    match kind {
        ModelKind::Embedding => super::EMBED_MODEL.identity(),
        ModelKind::Reranker => super::RERANKER.identity(),
    }
}

fn status(entry: CatalogEntry, cache: &Cache, identity: Option<&Identity>) -> ModelStatus {
    let files = required_files(&entry.model_file, &entry.additional);
    let repo = cache.model(entry.repository.clone());
    let path = cache
        .path()
        .join(Repo::model(entry.repository.clone()).folder_name());
    let revision = cached_revision(&path.join("refs/main")).ok();
    let file_details: Vec<_> = files
        .iter()
        .map(|name| {
            // Use the validated, trimmed revision instead of CacheRepo::get, which
            // rereads refs/main verbatim and mishandles trailing whitespace.
            let path = revision
                .as_ref()
                .map(|revision| repo.pointer_path(revision).join(name))
                .filter(|path| path.exists());
            let size_bytes = path.as_ref().and_then(|path| file_size(path));
            FileStatus {
                name: name.clone(),
                path: path.map(|path| path.to_string_lossy().into_owned()),
                size_bytes,
            }
        })
        .collect();
    let variant_bytes = file_details.iter().map(|file| file.size_bytes).sum();
    let loaded = identity.is_some_and(|id| id.name == entry.name && id.cache_dir == *cache.path());
    ModelStatus {
        name: entry.name,
        kind: entry.kind,
        repository: entry.repository,
        description: entry.description,
        dimension: entry.dimension,
        cached: file_details.iter().all(|file| file.size_bytes.is_some()),
        loaded,
        cache_dir: cache.path().to_string_lossy().into_owned(),
        path: path.to_string_lossy().into_owned(),
        revision,
        files,
        file_details,
        variant_bytes,
        disk_bytes: directory_bytes(&path).ok(),
    }
}

// Do not follow snapshot symlinks: their blobs are counted exactly once.
fn directory_bytes(path: &Path) -> std::io::Result<u64> {
    let metadata = match path.symlink_metadata() {
        Ok(metadata) => metadata,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(0),
        Err(e) => return Err(e),
    };
    if metadata.is_symlink() {
        return Ok(0);
    }
    if metadata.is_file() {
        return Ok(metadata.len());
    }
    if !metadata.is_dir() {
        return Ok(0);
    }
    std::fs::read_dir(path)?.try_fold(0, |size, entry| Ok(size + directory_bytes(&entry?.path())?))
}

#[cfg_attr(not(test), rustler::nif(schedule = "DirtyIo"))]
fn cache_directory(cache_dir: String) -> String {
    effective_cache_dir(cache_dir)
        .to_string_lossy()
        .into_owned()
}

#[cfg_attr(not(test), rustler::nif(schedule = "DirtyIo"))]
fn loaded_models() -> Result<Vec<ModelStatus>, String> {
    let mut models = Vec::new();
    for kind in [ModelKind::Embedding, ModelKind::Reranker] {
        if let Some(identity) = loaded_identity(kind) {
            let entry = lookup_entry(&identity.name, kind)?;
            let cache = Cache::new(identity.cache_dir.clone());
            models.push(status(entry, &cache, Some(&identity)));
        }
    }
    Ok(models)
}

#[cfg_attr(not(test), rustler::nif(schedule = "DirtyIo"))]
fn delete_model(name: String, kind: ModelKind, cache_dir: String) -> Result<bool, String> {
    // Hold across inspection, unload, and deletion so a concurrent loader cannot
    // recreate the cache or keep a session backed by files being removed.
    let _guard = super::cache_write();
    let info = lookup_entry(&name, kind)?;
    let directory = effective_cache_dir(cache_dir);
    let path = repository_path(&directory, &info.repository)?;
    match path.symlink_metadata() {
        Ok(metadata) if metadata.is_dir() && !metadata.is_symlink() => (),
        Ok(_) => {
            return Err("The model cache is not a regular directory. No files were removed.".into())
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
        Err(e) => return Err(e.to_string()),
    }
    super::EMBED_MODEL.unload_repository(&info.repository, &directory)?;
    super::RERANKER.unload_repository(&info.repository, &directory)?;
    remove_repository(&path)?;
    Ok(true)
}

fn repository_path(cache_dir: &Path, repository: &str) -> Result<PathBuf, String> {
    let parts: Vec<_> = repository.split('/').collect();
    if parts.len() != 2
        || parts.iter().any(|part| {
            part.is_empty()
                || *part == "."
                || *part == ".."
                || !part
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
        })
    {
        return Err("Invalid model repository.".into());
    }
    Ok(cache_dir.join(Repo::model(repository.into()).folder_name()))
}

fn remove_repository(path: &Path) -> Result<(), String> {
    match path.symlink_metadata() {
        Ok(metadata) if metadata.is_dir() && !metadata.is_symlink() => {
            std::fs::remove_dir_all(path).map_err(|e| e.to_string())
        }
        Ok(_) => Err("The model cache is not a regular directory. No files were removed.".into()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

fn files_cached(cache: &Cache, repository: &str, model_file: &str, additional: &[String]) -> bool {
    let reference = cache
        .path()
        .join(Repo::model(repository.into()).folder_name())
        .join("refs/main");
    let Ok(revision) = cached_revision(&reference) else {
        return false;
    };
    let snapshot = cache.model(repository.into()).pointer_path(&revision);
    std::iter::once(model_file)
        .chain(TOKENIZER_FILES)
        .chain(additional.iter().map(String::as_str))
        .all(|file| file_size(&snapshot.join(file)).is_some())
}

fn file_size(path: &Path) -> Option<u64> {
    path.metadata()
        .ok()
        .filter(|metadata| metadata.is_file() && metadata.len() > 0)
        .map(|metadata| metadata.len())
}

// hf-hub keeps an existing snapshot entry, even if it is an empty regular file.
// Unlink unusable entries before downloading; never remove symlink targets or
// directories that happen to occupy a required file's path.
fn prepare_download_destination(path: &Path) -> std::io::Result<()> {
    match path.metadata() {
        Ok(metadata) if metadata.is_file() && metadata.len() > 0 => return Ok(()),
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => return Err(error),
        _ => (),
    }
    match path.symlink_metadata() {
        Ok(metadata) if metadata.is_file() || metadata.is_symlink() => std::fs::remove_file(path),
        Ok(_) => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Model file path is not a regular file: {}", path.display()),
        )),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

fn cached_revision(reference: &Path) -> std::io::Result<String> {
    let contents = std::fs::read_to_string(reference)?;
    validate_revision(&contents)
}

fn validate_revision(contents: &str) -> std::io::Result<String> {
    let revision = contents.trim();
    if revision.is_empty()
        || matches!(revision, "." | "..")
        || !revision
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Invalid cached model revision: expected a single snapshot directory name",
        ));
    }
    Ok(revision.to_string())
}

pub(crate) fn effective_cache_dir(default: String) -> PathBuf {
    // FastEmbed's HF_HOME override is read from the native process environment.
    let path = PathBuf::from(std::env::var("HF_HOME").unwrap_or(default));
    path.canonicalize()
        .unwrap_or_else(|_| std::path::absolute(&path).unwrap_or(path))
}

pub(crate) fn prepare_model_files(
    directory: String,
    repository: &str,
    model_file: &str,
    additional: &[String],
) -> Result<PathBuf, String> {
    prepare_files(
        effective_cache_dir(directory),
        repository,
        model_file,
        additional,
        std::env::var("HF_ENDPOINT").unwrap_or_else(|_| "https://huggingface.co".into()),
    )
}

fn prepare_files(
    directory: PathBuf,
    repository: &str,
    model_file: &str,
    additional: &[String],
    endpoint: String,
) -> Result<PathBuf, String> {
    let cache = Cache::new(directory.clone());
    let main = cache.model(repository.into());
    let reference = directory
        .join(Repo::model(repository.into()).folder_name())
        .join("refs/main");
    if files_cached(&cache, repository, model_file, additional) {
        // FastEmbed still uses CacheRepo::get: normalize whitespace before it
        // reads the ref, but leave already canonical/read-only caches untouched.
        let revision = cached_revision(&reference).map_err(|error| error.to_string())?;
        if std::fs::read_to_string(&reference).map_err(|error| error.to_string())? != revision {
            main.create_ref(&revision)
                .map_err(|error| error.to_string())?;
        }
        return Ok(directory);
    }

    let download = || -> Result<(), Box<dyn std::error::Error>> {
        let api = ApiBuilder::new()
            .with_cache_dir(directory.clone())
            .with_endpoint(endpoint)
            .with_progress(true)
            .build()?;
        // Invalid or missing refs are resolved from main; permission and other
        // I/O failures must not silently select a different revision.
        let revision = resolve_revision(&reference, || {
            let metadata = api.metadata(&api.model(repository.into()).url(model_file))?;
            Ok(validate_revision(metadata.commit_hash())?)
        })?;
        let pinned = Repo::with_revision(repository.into(), RepoType::Model, revision.clone());
        let snapshot = main.pointer_path(&revision);
        // Persist before downloads so retries keep the selected commit.
        main.create_ref(&revision)?;
        let pinned_api = api.repo(pinned);
        for file in std::iter::once(model_file)
            .chain(TOKENIZER_FILES)
            .chain(additional.iter().map(String::as_str))
        {
            let path = snapshot.join(file);
            if file_size(&path).is_none() {
                prepare_download_destination(&path)?;
                pinned_api.download(file)?;
            }
        }
        Ok(())
    };
    download().map_err(|error| download_error(error.as_ref()))?;
    Ok(directory)
}

fn resolve_revision(
    reference: &Path,
    remote: impl FnOnce() -> Result<String, Box<dyn std::error::Error>>,
) -> Result<String, Box<dyn std::error::Error>> {
    match cached_revision(reference) {
        Ok(revision) => Ok(revision),
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::NotFound | std::io::ErrorKind::InvalidData
            ) =>
        {
            remote()
        }
        Err(error) => Err(error.into()),
    }
}

fn download_error(error: &(dyn std::error::Error + 'static)) -> String {
    if error.downcast_ref::<ApiError>().is_some_and(is_not_found) {
        format!("{error}. The requested file or revision was not found (404). If the saved revision is no longer available, use ExFastembed.delete_model/2 and retry (all cached variants in that repository are deleted).")
    } else {
        error.to_string()
    }
}

fn is_not_found(error: &ApiError) -> bool {
    match error {
        ApiError::RequestError(error) => matches!(error.as_ref(), ureq::Error::StatusCode(404)),
        ApiError::TooManyRetries(error) => is_not_found(error),
        _ => false,
    }
}

#[cfg(test)]
#[path = "cache_tests.rs"]
mod tests;
