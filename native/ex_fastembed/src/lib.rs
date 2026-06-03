use fastembed::{
    EmbeddingModel, RerankInitOptions, RerankerModel, TextEmbedding, TextInitOptions, TextRerank,
};
use std::sync::{Mutex, OnceLock};

static EMBED_MODEL: OnceLock<Mutex<TextEmbedding>> = OnceLock::new();
static RERANKER: OnceLock<Mutex<TextRerank>> = OnceLock::new();

#[rustler::nif]
fn embed_models() -> Vec<String> {
    embedding_aliases()
        .into_iter()
        .map(|(name, _model)| name.to_string())
        .collect()
}

#[rustler::nif]
fn reranker_models() -> Vec<String> {
    reranker_aliases()
        .into_iter()
        .map(|(name, _model)| name.to_string())
        .collect()
}

#[rustler::nif(schedule = "DirtyCpu")]
fn load(model_name: String) -> Result<i64, String> {
    let model = resolve_embedding_model(&model_name)?;
    let info = TextEmbedding::get_model_info(&model)
        .map_err(|_| format!("No recognized info for {model_name}"))?;
    let dimension = info.dim as i64;
    let text_embedding =
        TextEmbedding::try_new(TextInitOptions::new(model)).map_err(|error| error.to_string())?;

    EMBED_MODEL
        .set(Mutex::new(text_embedding))
        .map_err(|_| "Model already loaded!".to_string())?;

    Ok(dimension)
}

#[rustler::nif(schedule = "DirtyCpu")]
fn embed_text(texts: Vec<String>) -> Result<Vec<Vec<f32>>, String> {
    let model = EMBED_MODEL
        .get()
        .ok_or_else(|| "No model loaded. Call load/1 first.".to_string())?;
    let mut model = model.lock().map_err(|error| error.to_string())?;

    model.embed(texts, None).map_err(|error| error.to_string())
}

#[rustler::nif(schedule = "DirtyCpu")]
fn load_reranker(model_name: String) -> Result<bool, String> {
    let model = resolve_reranker_model(&model_name)?;
    let reranker =
        TextRerank::try_new(RerankInitOptions::new(model).with_show_download_progress(true))
            .map_err(|error| error.to_string())?;

    RERANKER
        .set(Mutex::new(reranker))
        .map_err(|_| "Reranker already loaded!".to_string())?;

    Ok(true)
}

#[rustler::nif(schedule = "DirtyCpu")]
fn rerank(
    query: String,
    documents: Vec<String>,
    return_docs: bool,
) -> Result<Vec<(usize, f32, Option<String>)>, String> {
    let reranker = RERANKER
        .get()
        .ok_or_else(|| "No reranker loaded. Call load_reranker/1 first.".to_string())?;
    let mut reranker = reranker.lock().map_err(|error| error.to_string())?;
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
    embedding_aliases()
        .into_iter()
        .find(|(alias, _model)| alias.eq_ignore_ascii_case(model_name))
        .map(|(_alias, model)| model)
        .ok_or_else(|| format!("Model not recognized or not implemented: {model_name}"))
}

fn resolve_reranker_model(model_name: &str) -> Result<RerankerModel, String> {
    reranker_aliases()
        .into_iter()
        .find(|(alias, _model)| alias.eq_ignore_ascii_case(model_name))
        .map(|(_alias, model)| model)
        .ok_or_else(|| format!("Reranker model not recognized: {model_name}"))
}

fn embedding_aliases() -> Vec<(&'static str, EmbeddingModel)> {
    vec![
        ("BAAI/bge-small-en-v1.5", EmbeddingModel::BGESmallENV15),
        ("Xenova/bge-small-en-v1.5", EmbeddingModel::BGESmallENV15),
        (
            "Qdrant/bge-small-en-v1.5-onnx-Q",
            EmbeddingModel::BGESmallENV15Q,
        ),
        ("BGESmallENV15Q", EmbeddingModel::BGESmallENV15Q),
        ("BAAI/bge-base-en-v1.5", EmbeddingModel::BGEBaseENV15),
        ("Xenova/bge-base-en-v1.5", EmbeddingModel::BGEBaseENV15),
        (
            "Qdrant/bge-base-en-v1.5-onnx-Q",
            EmbeddingModel::BGEBaseENV15Q,
        ),
        ("BGEBaseENV15Q", EmbeddingModel::BGEBaseENV15Q),
        ("BAAI/bge-large-en-v1.5", EmbeddingModel::BGELargeENV15),
        ("Xenova/bge-large-en-v1.5", EmbeddingModel::BGELargeENV15),
        (
            "Qdrant/bge-large-en-v1.5-onnx-Q",
            EmbeddingModel::BGELargeENV15Q,
        ),
        ("BGELargeENV15Q", EmbeddingModel::BGELargeENV15Q),
        ("BAAI/bge-small-zh-v1.5", EmbeddingModel::BGESmallZHV15),
        ("Xenova/bge-small-zh-v1.5", EmbeddingModel::BGESmallZHV15),
        ("BAAI/bge-large-zh-v1.5", EmbeddingModel::BGELargeZHV15),
        ("Xenova/bge-large-zh-v1.5", EmbeddingModel::BGELargeZHV15),
        ("BAAI/bge-m3", EmbeddingModel::BGEM3),
        (
            "sentence-transformers/all-MiniLM-L6-v2",
            EmbeddingModel::AllMiniLML6V2,
        ),
        (
            "Qdrant/all-MiniLM-L6-v2-onnx",
            EmbeddingModel::AllMiniLML6V2,
        ),
        ("Xenova/all-MiniLM-L6-v2", EmbeddingModel::AllMiniLML6V2Q),
        ("AllMiniLML6V2Q", EmbeddingModel::AllMiniLML6V2Q),
        (
            "sentence-transformers/all-MiniLM-L12-v2",
            EmbeddingModel::AllMiniLML12V2,
        ),
        ("Xenova/all-MiniLM-L12-v2", EmbeddingModel::AllMiniLML12V2),
        ("AllMiniLML12V2Q", EmbeddingModel::AllMiniLML12V2Q),
        (
            "sentence-transformers/all-mpnet-base-v2",
            EmbeddingModel::AllMpnetBaseV2,
        ),
        ("Xenova/all-mpnet-base-v2", EmbeddingModel::AllMpnetBaseV2),
        (
            "sentence-transformers/paraphrase-MiniLM-L12-v2",
            EmbeddingModel::ParaphraseMLMiniLML12V2,
        ),
        (
            "sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2",
            EmbeddingModel::ParaphraseMLMiniLML12V2,
        ),
        (
            "Xenova/paraphrase-multilingual-MiniLM-L12-v2",
            EmbeddingModel::ParaphraseMLMiniLML12V2,
        ),
        (
            "Qdrant/paraphrase-multilingual-MiniLM-L12-v2-onnx-Q",
            EmbeddingModel::ParaphraseMLMiniLML12V2Q,
        ),
        (
            "ParaphraseMLMiniLML12V2Q",
            EmbeddingModel::ParaphraseMLMiniLML12V2Q,
        ),
        (
            "sentence-transformers/paraphrase-multilingual-mpnet-base-v2",
            EmbeddingModel::ParaphraseMLMpnetBaseV2,
        ),
        (
            "Xenova/paraphrase-multilingual-mpnet-base-v2",
            EmbeddingModel::ParaphraseMLMpnetBaseV2,
        ),
        (
            "lightonai/ModernBERT-embed-large",
            EmbeddingModel::ModernBertEmbedLarge,
        ),
        (
            "lightonai/modernbert-embed-large",
            EmbeddingModel::ModernBertEmbedLarge,
        ),
        (
            "nomic-ai/nomic-embed-text-v1",
            EmbeddingModel::NomicEmbedTextV1,
        ),
        (
            "nomic-ai/nomic-embed-text-v1.5",
            EmbeddingModel::NomicEmbedTextV15,
        ),
        ("NomicEmbedTextV15Q", EmbeddingModel::NomicEmbedTextV15Q),
        (
            "intfloat/multilingual-e5-small",
            EmbeddingModel::MultilingualE5Small,
        ),
        (
            "intfloat/multilingual-e5-base",
            EmbeddingModel::MultilingualE5Base,
        ),
        (
            "intfloat/multilingual-e5-large",
            EmbeddingModel::MultilingualE5Large,
        ),
        (
            "Qdrant/multilingual-e5-large-onnx",
            EmbeddingModel::MultilingualE5Large,
        ),
        (
            "mixedbread-ai/mxbai-embed-large-v1",
            EmbeddingModel::MxbaiEmbedLargeV1,
        ),
        ("MxbaiEmbedLargeV1Q", EmbeddingModel::MxbaiEmbedLargeV1Q),
        ("Alibaba-NLP/gte-base-en-v1.5", EmbeddingModel::GTEBaseENV15),
        ("GTEBaseENV15Q", EmbeddingModel::GTEBaseENV15Q),
        (
            "Alibaba-NLP/gte-large-en-v1.5",
            EmbeddingModel::GTELargeENV15,
        ),
        ("GTELargeENV15Q", EmbeddingModel::GTELargeENV15Q),
        ("Qdrant/clip-ViT-B-32-text", EmbeddingModel::ClipVitB32),
        (
            "jinaai/jina-embeddings-v2-base-code",
            EmbeddingModel::JinaEmbeddingsV2BaseCode,
        ),
        (
            "jinaai/jina-embeddings-v2-base-en",
            EmbeddingModel::JinaEmbeddingsV2BaseEN,
        ),
        (
            "onnx-community/embeddinggemma-300m-ONNX",
            EmbeddingModel::EmbeddingGemma300M,
        ),
        (
            "snowflake/snowflake-arctic-embed-xs",
            EmbeddingModel::SnowflakeArcticEmbedXS,
        ),
        (
            "SnowflakeArcticEmbedXSQ",
            EmbeddingModel::SnowflakeArcticEmbedXSQ,
        ),
        (
            "snowflake/snowflake-arctic-embed-s",
            EmbeddingModel::SnowflakeArcticEmbedS,
        ),
        (
            "SnowflakeArcticEmbedSQ",
            EmbeddingModel::SnowflakeArcticEmbedSQ,
        ),
        (
            "Snowflake/snowflake-arctic-embed-m",
            EmbeddingModel::SnowflakeArcticEmbedM,
        ),
        (
            "snowflake/snowflake-arctic-embed-m",
            EmbeddingModel::SnowflakeArcticEmbedM,
        ),
        (
            "SnowflakeArcticEmbedMQ",
            EmbeddingModel::SnowflakeArcticEmbedMQ,
        ),
        (
            "snowflake/snowflake-arctic-embed-m-long",
            EmbeddingModel::SnowflakeArcticEmbedMLong,
        ),
        (
            "SnowflakeArcticEmbedMLongQ",
            EmbeddingModel::SnowflakeArcticEmbedMLongQ,
        ),
        (
            "snowflake/snowflake-arctic-embed-l",
            EmbeddingModel::SnowflakeArcticEmbedL,
        ),
        (
            "SnowflakeArcticEmbedLQ",
            EmbeddingModel::SnowflakeArcticEmbedLQ,
        ),
    ]
}

fn reranker_aliases() -> Vec<(&'static str, RerankerModel)> {
    vec![
        ("BAAI/bge-reranker-base", RerankerModel::BGERerankerBase),
        ("BAAI/bge-reranker-v2-m3", RerankerModel::BGERerankerV2M3),
        ("rozgo/bge-reranker-v2-m3", RerankerModel::BGERerankerV2M3),
        (
            "jinaai/jina-reranker-v1-turbo-en",
            RerankerModel::JINARerankerV1TurboEn,
        ),
        (
            "jinaai/jina-reranker-v2-base-multiligual",
            RerankerModel::JINARerankerV2BaseMultiligual,
        ),
        (
            "jinaai/jina-reranker-v2-base-multilingual",
            RerankerModel::JINARerankerV2BaseMultiligual,
        ),
    ]
}

rustler::init!("Elixir.ExFastembed.Native");
