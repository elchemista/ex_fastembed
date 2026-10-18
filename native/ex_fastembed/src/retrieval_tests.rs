use super::*;

#[test]
fn retrieval_prefixes_cover_language_task_and_quantized_variants() {
    let cases: &[(&[&str], &str, &str)] = &[
        (
            &[
                "MultilingualE5Small",
                "MultilingualE5Base",
                "MultilingualE5Large",
            ],
            "query: ",
            "passage: ",
        ),
        (
            &[
                "NomicEmbedTextV1",
                "NomicEmbedTextV15",
                "NomicEmbedTextV15Q",
            ],
            "search_query: ",
            "search_document: ",
        ),
        (
            &[
                "BGESmallENV15",
                "BGESmallENV15Q",
                "BGEBaseENV15",
                "BGEBaseENV15Q",
                "BGELargeENV15",
                "BGELargeENV15Q",
            ],
            "Represent this sentence for searching relevant passages: ",
            "",
        ),
        (
            &["BGESmallZHV15", "BGELargeZHV15"],
            "为这个句子生成表示以用于检索相关文章：",
            "",
        ),
        (
            &[
                "BGEM3",
                "AllMiniLML6V2",
                "AllMiniLML6V2Q",
                "EmbeddingGemma300M",
            ],
            "",
            "",
        ),
    ];
    for (models, query, document) in cases {
        for model in *models {
            for (input, prefix) in [
                (Input::Query, query),
                (Input::Document, document),
                (Input::Raw, &""),
            ] {
                assert_eq!(
                    prepare(vec!["Città 城市".into(), "second".into()], model, input),
                    vec![format!("{prefix}Città 城市"), format!("{prefix}second")],
                    "{model}"
                );
            }
        }
    }
}

#[test]
fn prefixes_are_always_added_and_empty_batches_stay_empty() {
    assert_eq!(
        prepare(
            vec!["query: already prefixed".into()],
            "MultilingualE5Small",
            Input::Query
        ),
        vec!["query: query: already prefixed"]
    );
    assert!(prepare(vec![], "NomicEmbedTextV15", Input::Document).is_empty());
}
