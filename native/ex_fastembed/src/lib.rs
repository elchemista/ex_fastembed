use fastembed::{
    EmbeddingModel, RerankInitOptions, RerankerModel, TextEmbedding, TextInitOptions, TextRerank,
};
use std::sync::{Mutex, OnceLock};

static EMBED_MODEL: OnceLock<Mutex<Option<TextEmbedding>>> = OnceLock::new();
static RERANKER: OnceLock<Mutex<Option<TextRerank>>> = OnceLock::new();

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
    let mut names = Vec::with_capacity(supported_models.len() + LEGACY_EMBEDDING_ALIASES.len());

    for model_info in supported_models {
        names.push(model_info.model_code);

        if is_quantized_model(&model_info.model) {
            names.push(model_info.model.to_string());
        }
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
    let mut names = Vec::with_capacity(supported_models.len() + LEGACY_RERANKER_ALIASES.len());

    for model_info in supported_models {
        names.push(model_info.model_code);
    }

    names.extend(
        LEGACY_RERANKER_ALIASES
            .iter()
            .map(|(name, _model)| (*name).to_string()),
    );

    sorted_unique_names(names)
}

#[rustler::nif(schedule = "DirtyCpu")]
fn load(model_name: String) -> Result<i64, String> {
    let model = resolve_embedding_model(&model_name)?;
    let info = TextEmbedding::get_model_info(&model)
        .map_err(|_| format!("No recognized info for {model_name}"))?;
    let dimension = info.dim as i64;
    let text_embedding =
        TextEmbedding::try_new(TextInitOptions::new(model)).map_err(|error| error.to_string())?;
    let model_slot = EMBED_MODEL.get_or_init(|| Mutex::new(None));
    let mut model_slot = model_slot.lock().map_err(|error| error.to_string())?;

    *model_slot = Some(text_embedding);

    Ok(dimension)
}

#[rustler::nif(schedule = "DirtyCpu")]
fn embed_text(texts: Vec<String>) -> Result<Vec<Vec<f32>>, String> {
    if texts.is_empty() {
        return Ok(Vec::new());
    }

    let model_slot = EMBED_MODEL
        .get()
        .ok_or_else(|| "No model loaded. Call load/1 first.".to_string())?;
    let mut model_slot = model_slot.lock().map_err(|error| error.to_string())?;
    let model = model_slot
        .as_mut()
        .ok_or_else(|| "No model loaded. Call load/1 first.".to_string())?;

    model.embed(texts, None).map_err(|error| error.to_string())
}

#[rustler::nif(schedule = "DirtyCpu")]
fn load_reranker(model_name: String) -> Result<bool, String> {
    let model = resolve_reranker_model(&model_name)?;
    let reranker =
        TextRerank::try_new(RerankInitOptions::new(model).with_show_download_progress(true))
            .map_err(|error| error.to_string())?;
    let reranker_slot = RERANKER.get_or_init(|| Mutex::new(None));
    let mut reranker_slot = reranker_slot.lock().map_err(|error| error.to_string())?;

    *reranker_slot = Some(reranker);

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

    let reranker_slot = RERANKER
        .get()
        .ok_or_else(|| "No reranker loaded. Call load_reranker/1 first.".to_string())?;
    let mut reranker_slot = reranker_slot.lock().map_err(|error| error.to_string())?;
    let reranker = reranker_slot
        .as_mut()
        .ok_or_else(|| "No reranker loaded. Call load_reranker/1 first.".to_string())?;
    let document_refs: Vec<&String> = documents.iter().collect();

    reranker
        .rerank(&query, document_refs, return_docs, None)
        .map(|results| {
            results
                .into_iter()
                .map(|result| (result.index, result.score, result.document))
                .collect()
        })
        .map_err(|error| error.to_string())
}

fn resolve_embedding_model(model_name: &str) -> Result<EmbeddingModel, String> {
    if let Some((_alias, model)) = LEGACY_EMBEDDING_ALIASES
        .iter()
        .find(|(alias, _model)| alias.eq_ignore_ascii_case(model_name))
    {
        return Ok(model.clone());
    }

    if let Ok(model) = model_name.parse::<EmbeddingModel>() {
        if is_quantized_model(&model) {
            return Ok(model);
        }
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
        .find(|model_info| model_info.model_code.eq_ignore_ascii_case(model_name))
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
mod tests {
    use super::*;

    #[test]
    fn advertised_embedding_models_are_sorted_unique_and_resolvable() {
        let models = supported_embedding_model_names();

        assert_eq!(models, sorted_unique_names(models.clone()));

        for model in models {
            assert!(
                resolve_embedding_model(&model).is_ok(),
                "advertised embedding model did not resolve: {model}"
            );
        }
    }

    #[test]
    fn advertised_reranker_models_are_sorted_unique_and_resolvable() {
        let models = supported_reranker_model_names();

        assert_eq!(models, sorted_unique_names(models.clone()));

        for model in models {
            assert!(
                resolve_reranker_model(&model).is_ok(),
                "advertised reranker model did not resolve: {model}"
            );
        }
    }

    #[test]
    fn canonical_ambiguous_name_prefers_the_non_quantized_model() {
        assert_eq!(
            resolve_embedding_model("Xenova/all-MiniLM-L12-v2"),
            Ok(EmbeddingModel::AllMiniLML12V2)
        );

        assert_eq!(
            resolve_embedding_model("onnx-community/embeddinggemma-300m-ONNX"),
            Ok(EmbeddingModel::EmbeddingGemma300M)
        );
    }

    #[test]
    fn quantized_model_detection_includes_numbered_variants() {
        assert!(is_quantized_model(&EmbeddingModel::EmbeddingGemma300MQ));
        assert!(is_quantized_model(&EmbeddingModel::EmbeddingGemma300MQ4));
        assert!(!is_quantized_model(&EmbeddingModel::EmbeddingGemma300M));
    }
}
