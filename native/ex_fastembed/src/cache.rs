use fastembed::{ModelInfo, RerankerModelInfo, TextEmbedding, TextRerank};
use hf_hub::{api::sync::ApiBuilder, Cache, CacheRepo, Repo, RepoType};
use std::path::PathBuf;

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
    ModelStatus {
        cached: files_cached(
            cache,
            &info.model_code,
            &info.model_file,
            &info.additional_files,
        ),
        name: info.model.to_string(),
        kind: ModelKind::Embedding,
        repository: info.model_code.clone(),
        dimension: Some(info.dim),
    }
}

fn reranker_status(info: RerankerModelInfo, cache: &Cache) -> ModelStatus {
    ModelStatus {
        cached: files_cached(
            cache,
            &info.model_code,
            &info.model_file,
            &info.additional_files,
        ),
        name: format!("{:?}", info.model),
        kind: ModelKind::Reranker,
        repository: info.model_code,
        dimension: None,
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

fn effective_cache_dir(default: String) -> PathBuf {
    // FastEmbed's HF_HOME override is read from the native process environment.
    std::env::var("HF_HOME").unwrap_or(default).into()
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
