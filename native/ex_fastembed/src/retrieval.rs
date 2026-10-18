//! Retrieval prompts from the upstream model cards linked in the lifecycle guide.
use fastembed::EmbeddingModel;

#[derive(Clone, Copy, PartialEq, Eq, rustler::NifUnitEnum)]
pub(crate) enum Input {
    Raw,
    Query,
    Document,
}

pub(crate) fn prepare(texts: Vec<String>, model: &EmbeddingModel, input: Input) -> Vec<String> {
    let prefix = prefix(model, input);
    if prefix.is_empty() {
        texts
    } else {
        texts
            .into_iter()
            .map(|text| format!("{prefix}{text}"))
            .collect()
    }
}

fn prefix(model: &EmbeddingModel, input: Input) -> &'static str {
    use EmbeddingModel::*;
    // Deliberately exhaustive: dependency upgrades must account for new or
    // renamed variants, including those that need no retrieval instruction.
    let (query, document) = match model {
        MultilingualE5Small | MultilingualE5Base | MultilingualE5Large => ("query: ", "passage: "),
        NomicEmbedTextV1 | NomicEmbedTextV15 | NomicEmbedTextV15Q | ModernBertEmbedLarge => {
            ("search_query: ", "search_document: ")
        }
        BGESmallENV15
        | BGESmallENV15Q
        | BGEBaseENV15
        | BGEBaseENV15Q
        | BGELargeENV15
        | BGELargeENV15Q
        | SnowflakeArcticEmbedXS
        | SnowflakeArcticEmbedXSQ
        | SnowflakeArcticEmbedS
        | SnowflakeArcticEmbedSQ
        | SnowflakeArcticEmbedM
        | SnowflakeArcticEmbedMQ
        | SnowflakeArcticEmbedMLong
        | SnowflakeArcticEmbedMLongQ
        | SnowflakeArcticEmbedL
        | SnowflakeArcticEmbedLQ
        | MxbaiEmbedLargeV1
        | MxbaiEmbedLargeV1Q => (
            "Represent this sentence for searching relevant passages: ",
            "",
        ),
        BGESmallZHV15 | BGELargeZHV15 => ("为这个句子生成表示以用于检索相关文章：", ""),
        EmbeddingGemma300M | EmbeddingGemma300MQ | EmbeddingGemma300MQ4 => {
            ("task: search result | query: ", "title: none | text: ")
        }
        AllMiniLML6V2
        | AllMiniLML6V2Q
        | AllMiniLML12V2
        | AllMiniLML12V2Q
        | AllMpnetBaseV2
        | ParaphraseMLMiniLML12V2
        | ParaphraseMLMiniLML12V2Q
        | ParaphraseMLMpnetBaseV2
        | BGEM3
        | GTEBaseENV15
        | GTEBaseENV15Q
        | GTELargeENV15
        | GTELargeENV15Q
        | ClipVitB32
        | JinaEmbeddingsV2BaseCode
        | JinaEmbeddingsV2BaseEN => ("", ""),
    };
    match input {
        Input::Raw => "",
        Input::Query => query,
        Input::Document => document,
    }
}

#[cfg(test)]
#[path = "retrieval_tests.rs"]
mod tests;
