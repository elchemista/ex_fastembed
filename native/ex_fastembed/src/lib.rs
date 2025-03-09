use fastembed::{
    EmbeddingModel, InitOptions, RerankInitOptions, RerankerModel, TextEmbedding, TextRerank,
};
use std::sync::OnceLock;

static EMBED_MODEL: OnceLock<TextEmbedding> = OnceLock::new();
static RERANKER: OnceLock<TextRerank> = OnceLock::new();

#[rustler::nif(schedule = "DirtyCpu")]
fn load(model_name: String) -> Result<i64, String> {
    let model_enum = match model_name.as_str() {
        "BAAI/bge-small-en-v1.5" => EmbeddingModel::BGESmallENV15,
        "sentence-transformers/all-MiniLM-L6-v2" => EmbeddingModel::AllMiniLML6V2,
        "sentence-transformers/all-MiniLM-L12-v2" => EmbeddingModel::AllMiniLML12V2,
        "mixedbread-ai/mxbai-embed-large-v1" => EmbeddingModel::MxbaiEmbedLargeV1,
        "Qdrant/clip-ViT-B-32-text" => EmbeddingModel::ClipVitB32,
        "BAAI/bge-large-en-v1.5" => EmbeddingModel::BGELargeENV15,
        "BAAI/bge-small-zh-v1.5" => EmbeddingModel::BGESmallZHV15,
        "BAAI/bge-base-en-v1.5" => EmbeddingModel::BGEBaseENV15,
        "sentence-transformers/paraphrase-MiniLM-L12-v2" => EmbeddingModel::ParaphraseMLMiniLML12V2,
        "sentence-transformers/paraphrase-multilingual-mpnet-base-v2" => {
            EmbeddingModel::ParaphraseMLMpnetBaseV2
        }
        "lightonai/ModernBERT-embed-large" => EmbeddingModel::ModernBertEmbedLarge,
        "nomic-ai/nomic-embed-text-v1" => EmbeddingModel::NomicEmbedTextV1,
        "nomic-ai/nomic-embed-text-v1.5" => EmbeddingModel::NomicEmbedTextV15,
        "intfloat/multilingual-e5-small" => EmbeddingModel::MultilingualE5Small,
        "intfloat/multilingual-e5-base" => EmbeddingModel::MultilingualE5Base,
        "intfloat/multilingual-e5-large" => EmbeddingModel::MultilingualE5Large,
        "Alibaba-NLP/gte-base-en-v1.5" => EmbeddingModel::GTEBaseENV15,
        "Alibaba-NLP/gte-large-en-v1.5" => EmbeddingModel::GTELargeENV15,
        other => return Err(format!("Model not recognized or not implemented: {other}")),
    };
    let info = TextEmbedding::get_model_info(&model_enum)
        .map_err(|_| format!("No recognized info for {model_name}"))?;
    let dimension = info.dim as i64;
    let text_embedding =
        TextEmbedding::try_new(InitOptions::new(model_enum)).map_err(|e| e.to_string())?;
    EMBED_MODEL
        .set(text_embedding)
        .map_err(|_| "Model already loaded!".to_string())?;
    Ok(dimension)
}

#[rustler::nif(schedule = "DirtyCpu")]
fn embed_text(texts: Vec<String>) -> Result<Vec<Vec<f32>>, String> {
    let model = EMBED_MODEL
        .get()
        .ok_or("No model loaded. Call load/1 first.")?;
    let embeddings = model.embed(texts, None).map_err(|e| e.to_string())?;
    Ok(embeddings)
}

#[rustler::nif(schedule = "DirtyCpu")]
fn load_reranker(model_name: String) -> Result<bool, String> {
    let model_enum = match model_name.as_str() {
        "BAAI/bge-reranker-base" => RerankerModel::BGERerankerBase,
        "BAAI/bge-reranker-v2-m3" => RerankerModel::BGERerankerV2M3,
        "jinaai/jina-reranker-v1-turbo-en" => RerankerModel::JINARerankerV1TurboEn,
        "jinaai/jina-reranker-v2-base-multiligual" => RerankerModel::JINARerankerV2BaseMultiligual,
        other => return Err(format!("Reranker model not recognized: {other}")),
    };
    let reranker =
        TextRerank::try_new(RerankInitOptions::new(model_enum).with_show_download_progress(true))
            .map_err(|e| e.to_string())?;
    RERANKER
        .set(reranker)
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
        .ok_or("No reranker loaded. Call load_reranker/1 first.")?;
    let doc_refs: Vec<&String> = documents.iter().collect();
    let results = reranker
        .rerank(&query, doc_refs, return_docs, None)
        .map_err(|e| e.to_string())?;
    let converted = results
        .into_iter()
        .map(|sd| (sd.index, sd.score, sd.document))
        .collect();
    Ok(converted)
}

rustler::init!("Elixir.ExFastembed");
