use crate::runtime::Identity;
use fastembed::{ModelInfo, RerankerModelInfo, TextEmbedding, TextRerank};
use hf_hub::{api::sync::ApiBuilder, Cache, CacheRepo, Repo, RepoType};
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

#[rustler::nif(schedule = "DirtyIo")]
fn models(cache_dir: String) -> Vec<ModelStatus> {
    let cache = Cache::new(effective_cache_dir(cache_dir));
    let mut models: Vec<_> = TextEmbedding::list_supported_models()
        .into_iter()
        .map(|info| embedding_status(&info, &cache))
        .chain(
            TextRerank::list_supported_models()
                .into_iter()
                .map(|info| reranker_status(info, &cache)),
        )
        .collect();
    models.sort_by(|left, right| (left.kind, &left.name).cmp(&(right.kind, &right.name)));
    models
}

#[rustler::nif(schedule = "DirtyIo")]
fn model_info(name: String, kind: ModelKind, cache_dir: String) -> Result<ModelStatus, String> {
    lookup_model(name, kind, cache_dir)
}

fn lookup_model(name: String, kind: ModelKind, cache_dir: String) -> Result<ModelStatus, String> {
    let cache = Cache::new(effective_cache_dir(cache_dir));
    match kind {
        ModelKind::Embedding => {
            let model = super::resolve_embedding_model(&name)?;
            TextEmbedding::get_model_info(&model)
                .map(|info| embedding_status(info, &cache))
                .map_err(|error| error.to_string())
        }
        ModelKind::Reranker => {
            let model = super::resolve_reranker_model(&name)?;
            Ok(reranker_status(TextRerank::get_model_info(&model), &cache))
        }
    }
}

fn embedding_status(info: &ModelInfo<fastembed::EmbeddingModel>, cache: &Cache) -> ModelStatus {
    status(
        info.model.to_string(),
        ModelKind::Embedding,
        &info.model_code,
        Some(info.dim),
        &info.model_file,
        &info.additional_files,
        cache,
    )
}

fn reranker_status(info: RerankerModelInfo, cache: &Cache) -> ModelStatus {
    status(
        format!("{:?}", info.model),
        ModelKind::Reranker,
        &info.model_code,
        None,
        &info.model_file,
        &info.additional_files,
        cache,
    )
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

fn loaded_identity(kind: ModelKind) -> Result<Option<Identity>, String> {
    match kind {
        ModelKind::Embedding => super::EMBED_MODEL.identity(),
        ModelKind::Reranker => super::RERANKER.identity(),
    }
}

fn status(
    name: String,
    kind: ModelKind,
    repository: &str,
    dimension: Option<usize>,
    model_file: &str,
    additional: &[String],
    cache: &Cache,
) -> ModelStatus {
    let files = required_files(model_file, additional);
    let repo = cache.model(repository.into());
    let path = cache
        .path()
        .join(Repo::model(repository.into()).folder_name());
    let file_details: Vec<_> = files
        .iter()
        .map(|name| {
            let path = repo.get(name);
            let size_bytes = path
                .as_ref()
                .and_then(|p| p.metadata().ok())
                .filter(|m| m.is_file() && m.len() > 0)
                .map(|m| m.len());
            FileStatus {
                name: name.clone(),
                path: path.map(|p| p.to_string_lossy().into_owned()),
                size_bytes,
            }
        })
        .collect();
    let variant_bytes = file_details.iter().map(|f| f.size_bytes).sum();
    let loaded = loaded_identity(kind)
        .ok()
        .flatten()
        .is_some_and(|id| id.name == name && id.cache_dir == *cache.path());
    ModelStatus {
        name,
        kind,
        repository: repository.into(),
        dimension,
        cached: file_details.iter().all(|f| f.size_bytes.is_some()),
        loaded,
        cache_dir: cache.path().to_string_lossy().into_owned(),
        path: path.to_string_lossy().into_owned(),
        revision: std::fs::read_to_string(path.join("refs/main"))
            .ok()
            .map(|s| s.trim().to_string()),
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

#[rustler::nif(schedule = "DirtyIo")]
fn cache_directory(cache_dir: String) -> String {
    effective_cache_dir(cache_dir)
        .to_string_lossy()
        .into_owned()
}

#[rustler::nif(schedule = "DirtyIo")]
fn loaded_models() -> Result<Vec<ModelStatus>, String> {
    let mut models = Vec::new();
    for kind in [ModelKind::Embedding, ModelKind::Reranker] {
        if let Some(identity) = loaded_identity(kind)? {
            models.push(lookup_model(
                identity.name,
                kind,
                identity.cache_dir.to_string_lossy().into_owned(),
            )?);
        }
    }
    Ok(models)
}

#[rustler::nif(schedule = "DirtyIo")]
fn delete_model(name: String, kind: ModelKind, cache_dir: String) -> Result<bool, String> {
    // Hold across inspection, unload, and deletion so a concurrent loader cannot
    // recreate the cache or keep a session backed by files being removed.
    let _guard = super::CACHE_ACCESS.write().map_err(|e| e.to_string())?;
    let info = lookup_model(name, kind, cache_dir)?;
    let directory = PathBuf::from(&info.cache_dir);
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
    let repo = cache.model(repository.to_string());
    std::iter::once(model_file)
        .chain(TOKENIZER_FILES)
        .chain(additional.iter().map(String::as_str))
        .all(|file| file_cached(&repo, file))
}

fn file_cached(repo: &CacheRepo, file: &str) -> bool {
    repo.get(file)
        .and_then(|path| path.metadata().ok())
        .is_some_and(|metadata| metadata.is_file() && metadata.len() > 0)
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
    let directory = effective_cache_dir(directory);
    let cache = Cache::new(directory.clone());
    if files_cached(&cache, repository, model_file, additional) {
        return Ok(directory);
    }

    let download = || -> Result<(), Box<dyn std::error::Error>> {
        let api = ApiBuilder::new()
            .with_cache_dir(directory.clone())
            .with_endpoint(
                std::env::var("HF_ENDPOINT").unwrap_or_else(|_| "https://huggingface.co".into()),
            )
            .with_progress(true)
            .build()?;
        let main = Repo::model(repository.into());
        let reference = directory.join(main.folder_name()).join("refs/main");
        if !reference.is_file() {
            api.model(repository.into()).download(model_file)?;
        }

        // Pin missing files to the cached revision. Fetching each file from main
        // can advance refs/main midway and mix weights and tokenizer revisions.
        let revision = std::fs::read_to_string(reference)?;
        let pinned = Repo::with_revision(repository.into(), RepoType::Model, revision.clone());
        let pinned_cache = cache.repo(pinned.clone());
        pinned_cache.create_ref(&revision)?;
        let pinned_api = api.repo(pinned);
        for file in std::iter::once(model_file)
            .chain(TOKENIZER_FILES)
            .chain(additional.iter().map(String::as_str))
        {
            if !file_cached(&pinned_cache, file) {
                pinned_api.download(file)?;
            }
        }
        cache.model(repository.into()).create_ref(&revision)?;
        Ok(())
    };
    download().map_err(|error| error.to_string())?;
    Ok(directory)
}

#[cfg(test)]
#[path = "cache_tests.rs"]
mod tests;
