use fastembed::{
    EmbeddingModel, RerankInitOptions, RerankerModel, TextEmbedding, TextInitOptions, TextRerank,
};
use runtime::{Identity, Slot};
use std::sync::RwLock;

mod cache;
mod runtime;

static EMBED_MODEL: Slot<TextEmbedding> = Slot::new();
static RERANKER: Slot<TextRerank> = Slot::new();
// Loads share this lock; repository deletion is exclusive across both families.
static CACHE_ACCESS: RwLock<()> = RwLock::new(());

const LEGACY_EMBEDDING_ALIASES: &[(&str, EmbeddingModel)] = &[
    ("BAAI/bge-small-en-v1.5", EmbeddingModel::BGESmallENV15),
    ("BAAI/bge-base-en-v1.5", EmbeddingModel::BGEBaseENV15),
    ("BAAI/bge-large-en-v1.5", EmbeddingModel::BGELargeENV15),
    ("BAAI/bge-small-zh-v1.5", EmbeddingModel::BGESmallZHV15),
    ("BAAI/bge-large-zh-v1.5", EmbeddingModel::BGELargeZHV15),
    (
        "sentence-transformers/all-MiniLM-L6-v2",
        EmbeddingModel::AllMiniLML6V2,
    ),
    (
        "sentence-transformers/all-MiniLM-L12-v2",
        EmbeddingModel::AllMiniLML12V2,
    ),
    (
        "sentence-transformers/all-mpnet-base-v2",
        EmbeddingModel::AllMpnetBaseV2,
    ),
    (
        "sentence-transformers/paraphrase-MiniLM-L12-v2",
        EmbeddingModel::ParaphraseMLMiniLML12V2,
    ),
    (
        "sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2",
        EmbeddingModel::ParaphraseMLMiniLML12V2,
    ),
    (
        "sentence-transformers/paraphrase-multilingual-mpnet-base-v2",
        EmbeddingModel::ParaphraseMLMpnetBaseV2,
    ),
    (
        "intfloat/multilingual-e5-large",
        EmbeddingModel::MultilingualE5Large,
    ),
    (
        "lightonai/ModernBERT-embed-large",
        EmbeddingModel::ModernBertEmbedLarge,
    ),
    (
        "snowflake/snowflake-arctic-embed-m",
        EmbeddingModel::SnowflakeArcticEmbedM,
    ),
];

const LEGACY_RERANKER_ALIASES: &[(&str, RerankerModel)] = &[
    ("BAAI/bge-reranker-v2-m3", RerankerModel::BGERerankerV2M3),
    (
        "jinaai/jina-reranker-v2-base-multiligual",
        RerankerModel::JINARerankerV2BaseMultiligual,
    ),
];

#[rustler::nif]
fn embed_models() -> Vec<String> {
    supported_embedding_model_names()
}

fn supported_embedding_model_names() -> Vec<String> {
    let supported_models = TextEmbedding::list_supported_models();
    let mut names = Vec::with_capacity(2 * supported_models.len() + LEGACY_EMBEDDING_ALIASES.len());

    for model_info in supported_models {
        names.push(model_info.model_code);
        names.push(model_info.model.to_string());
    }

    names.extend(
        LEGACY_EMBEDDING_ALIASES
            .iter()
            .map(|(name, _model)| (*name).to_string()),
    );

    sorted_unique_names(names)
}

#[rustler::nif]
fn reranker_models() -> Vec<String> {
    supported_reranker_model_names()
}

fn supported_reranker_model_names() -> Vec<String> {
    let supported_models = TextRerank::list_supported_models();
    let mut names = Vec::with_capacity(2 * supported_models.len() + LEGACY_RERANKER_ALIASES.len());

    for model_info in supported_models {
        names.push(model_info.model_code);
        names.push(format!("{:?}", model_info.model));
    }

    names.extend(
        LEGACY_RERANKER_ALIASES
            .iter()
            .map(|(name, _model)| (*name).to_string()),
    );

    sorted_unique_names(names)
}

#[rustler::nif(schedule = "DirtyIo")]
fn unload() -> Result<bool, String> {
    EMBED_MODEL.unload()
}

#[rustler::nif(schedule = "DirtyIo")]
fn unload_reranker() -> Result<bool, String> {
    RERANKER.unload()
}

#[rustler::nif(schedule = "DirtyCpu")]
fn load(model_name: String, cache_dir: String) -> Result<i64, String> {
    let model = resolve_embedding_model(&model_name)?;
    let info = TextEmbedding::get_model_info(&model).map_err(|e| e.to_string())?;
    let dimension = info.dim as i64;
    let _cache_guard = CACHE_ACCESS.read().map_err(|e| e.to_string())?;
    let identity = Identity {
        name: model.to_string(),
        repository: info.model_code.clone(),
        cache_dir: cache::effective_cache_dir(cache_dir.clone()),
    };
    EMBED_MODEL.load(identity, || {
        let cache_dir = cache::prepare_model_files(
            cache_dir,
            &info.model_code,
            &info.model_file,
            &info.additional_files,
        )?;
        TextEmbedding::try_new(TextInitOptions::new(model.clone()).with_cache_dir(cache_dir))
            .map_err(|e| e.to_string())
    })?;
    Ok(dimension)
}

#[rustler::nif(schedule = "DirtyCpu")]
fn embed_text(texts: Vec<String>) -> Result<Vec<Vec<f32>>, String> {
    if texts.is_empty() {
        return Ok(Vec::new());
    }
    EMBED_MODEL.with_model("No model loaded. Call load/1 first.", |model| {
        model.embed(texts, None).map_err(|e| e.to_string())
    })
}

#[rustler::nif(schedule = "DirtyCpu")]
fn load_reranker(model_name: String, cache_dir: String) -> Result<bool, String> {
    let model = resolve_reranker_model(&model_name)?;
    let info = TextRerank::get_model_info(&model);
    let _cache_guard = CACHE_ACCESS.read().map_err(|e| e.to_string())?;
    let identity = Identity {
        name: format!("{:?}", model),
        repository: info.model_code.clone(),
        cache_dir: cache::effective_cache_dir(cache_dir.clone()),
    };
    RERANKER.load(identity, || {
        let cache_dir = cache::prepare_model_files(
            cache_dir,
            &info.model_code,
            &info.model_file,
            &info.additional_files,
        )?;
        TextRerank::try_new(RerankInitOptions::new(model).with_cache_dir(cache_dir))
            .map_err(|e| e.to_string())
    })?;
    Ok(true)
}

#[rustler::nif(schedule = "DirtyCpu")]
fn rerank(
    query: String,
    documents: Vec<String>,
    return_docs: bool,
) -> Result<Vec<(usize, f32, Option<String>)>, String> {
    if documents.is_empty() {
        return Ok(Vec::new());
    }
    RERANKER.with_model("No reranker loaded. Call load_reranker/1 first.", |model| {
        model
            .rerank(query, documents, return_docs, None)
            .map(|results| {
                results
                    .into_iter()
                    .map(|result| (result.index, result.score, result.document))
                    .collect()
            })
            .map_err(|e| e.to_string())
    })
}

fn resolve_embedding_model(model_name: &str) -> Result<EmbeddingModel, String> {
    if let Some((_alias, model)) = LEGACY_EMBEDDING_ALIASES
        .iter()
        .find(|(alias, _model)| alias.eq_ignore_ascii_case(model_name))
    {
        return Ok(model.clone());
    }

    if let Ok(model) = model_name.parse::<EmbeddingModel>() {
        return Ok(model);
    }

    TextEmbedding::list_supported_models()
        .into_iter()
        .filter(|model_info| model_info.model_code.eq_ignore_ascii_case(model_name))
        .min_by_key(|model_info| {
            (
                is_quantized_model(&model_info.model),
                model_info.model.to_string(),
            )
        })
        .map(|model_info| model_info.model)
        .ok_or_else(|| format!("Model not recognized or not implemented: {model_name}"))
}

fn resolve_reranker_model(model_name: &str) -> Result<RerankerModel, String> {
    if let Some((_alias, model)) = LEGACY_RERANKER_ALIASES
        .iter()
        .find(|(alias, _model)| alias.eq_ignore_ascii_case(model_name))
    {
        return Ok(model.clone());
    }

    TextRerank::list_supported_models()
        .into_iter()
        .find(|model_info| {
            model_info.model_code.eq_ignore_ascii_case(model_name)
                || format!("{:?}", model_info.model).eq_ignore_ascii_case(model_name)
        })
        .map(|model_info| model_info.model)
        .ok_or_else(|| format!("Reranker model not recognized: {model_name}"))
}

fn sorted_unique_names(mut names: Vec<String>) -> Vec<String> {
    names.sort_by(|left, right| {
        left.to_ascii_lowercase()
            .cmp(&right.to_ascii_lowercase())
            .then_with(|| left.cmp(right))
    });
    names.dedup();
    names
}

fn is_quantized_model(model: &EmbeddingModel) -> bool {
    model
        .to_string()
        .rsplit_once('Q')
        .is_some_and(|(prefix, suffix)| {
            !prefix.is_empty() && suffix.chars().all(|character| character.is_ascii_digit())
        })
}

rustler::init!("Elixir.ExFastembed.Native");

#[cfg(test)]
mod tests;
