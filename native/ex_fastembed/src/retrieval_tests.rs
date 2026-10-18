use super::*;
use fastembed::{EmbeddingModel::*, TextEmbedding};
use std::collections::HashSet;

fn cases() -> Vec<(Vec<EmbeddingModel>, &'static str, &'static str)> {
    vec![
        (
            vec![MultilingualE5Small, MultilingualE5Base, MultilingualE5Large],
            "query: ",
            "passage: ",
        ),
        (
            vec![
                NomicEmbedTextV1,
                NomicEmbedTextV15,
                NomicEmbedTextV15Q,
                ModernBertEmbedLarge,
            ],
            "search_query: ",
            "search_document: ",
        ),
        (
            vec![
                BGESmallENV15,
                BGESmallENV15Q,
                BGEBaseENV15,
                BGEBaseENV15Q,
                BGELargeENV15,
                BGELargeENV15Q,
                SnowflakeArcticEmbedXS,
                SnowflakeArcticEmbedXSQ,
                SnowflakeArcticEmbedS,
                SnowflakeArcticEmbedSQ,
                SnowflakeArcticEmbedM,
                SnowflakeArcticEmbedMQ,
                SnowflakeArcticEmbedMLong,
                SnowflakeArcticEmbedMLongQ,
                SnowflakeArcticEmbedL,
                SnowflakeArcticEmbedLQ,
                MxbaiEmbedLargeV1,
                MxbaiEmbedLargeV1Q,
            ],
            "Represent this sentence for searching relevant passages: ",
            "",
        ),
        (
            vec![BGESmallZHV15, BGELargeZHV15],
            "为这个句子生成表示以用于检索相关文章：",
            "",
        ),
        (
            vec![
                EmbeddingGemma300M,
                EmbeddingGemma300MQ,
                EmbeddingGemma300MQ4,
            ],
            "task: search result | query: ",
            "title: none | text: ",
        ),
        (
            vec![
                AllMiniLML6V2,
                AllMiniLML6V2Q,
                AllMiniLML12V2,
                AllMiniLML12V2Q,
                AllMpnetBaseV2,
                ParaphraseMLMiniLML12V2,
                ParaphraseMLMiniLML12V2Q,
                ParaphraseMLMpnetBaseV2,
                BGEM3,
                GTEBaseENV15,
                GTEBaseENV15Q,
                GTELargeENV15,
                GTELargeENV15Q,
                ClipVitB32,
                JinaEmbeddingsV2BaseCode,
                JinaEmbeddingsV2BaseEN,
            ],
            "",
            "",
        ),
    ]
}

#[test]
fn retrieval_prefixes_cover_language_task_and_quantized_variants() {
    for (models, query, document) in cases() {
        for model in models {
            for (input, prefix) in [
                (Input::Query, query),
                (Input::Document, document),
                (Input::Raw, ""),
            ] {
                assert_eq!(
                    prepare(vec!["Città 城市".into(), "second".into()], &model, input),
                    vec![format!("{prefix}Città 城市"), format!("{prefix}second")],
                    "{model}"
                );
            }
        }
    }
}

#[test]
fn prefix_expectations_account_for_every_catalog_variant_exactly_once() {
    let expected: Vec<_> = cases()
        .into_iter()
        .flat_map(|(models, _, _)| models)
        .collect();
    let unique: HashSet<_> = expected.iter().cloned().collect();
    assert_eq!(
        expected.len(),
        unique.len(),
        "duplicate variant in prefix expectations"
    );
    let catalog: HashSet<_> = TextEmbedding::list_supported_models()
        .into_iter()
        .map(|info| info.model)
        .collect();
    assert_eq!(
        unique, catalog,
        "update prefix expectations when the catalog changes"
    );
}

#[test]
fn prefixes_are_always_added_and_empty_batches_stay_empty() {
    assert_eq!(
        prepare(
            vec!["query: already prefixed".into()],
            &MultilingualE5Small,
            Input::Query
        ),
        vec!["query: query: already prefixed"]
    );
    assert!(prepare(vec![], &NomicEmbedTextV15, Input::Document).is_empty());
    assert_eq!(
        prepare(
            vec!["A title | text: content".into()],
            &EmbeddingGemma300M,
            Input::Document
        ),
        vec!["title: none | text: A title | text: content"]
    );
}
