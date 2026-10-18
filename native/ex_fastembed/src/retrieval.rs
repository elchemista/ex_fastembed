//! Retrieval prompts from the E5, BGE v1.5 and Nomic model cards.
//! Other variants and raw embedding calls pass text through unchanged.

#[derive(Clone, Copy, PartialEq, Eq, rustler::NifUnitEnum)]
pub(crate) enum Input {
    Raw,
    Query,
    Document,
}

pub(crate) fn prepare(texts: Vec<String>, model: &str, input: Input) -> Vec<String> {
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

fn prefix(model: &str, input: Input) -> &'static str {
    match (model, input) {
        (_, Input::Raw) => "",
        ("MultilingualE5Small" | "MultilingualE5Base" | "MultilingualE5Large", Input::Query) => {
            "query: "
        }
        ("MultilingualE5Small" | "MultilingualE5Base" | "MultilingualE5Large", Input::Document) => {
            "passage: "
        }
        ("NomicEmbedTextV1" | "NomicEmbedTextV15" | "NomicEmbedTextV15Q", Input::Query) => {
            "search_query: "
        }
        ("NomicEmbedTextV1" | "NomicEmbedTextV15" | "NomicEmbedTextV15Q", Input::Document) => {
            "search_document: "
        }
        (
            "BGESmallENV15" | "BGESmallENV15Q" | "BGEBaseENV15" | "BGEBaseENV15Q" | "BGELargeENV15"
            | "BGELargeENV15Q",
            Input::Query,
        ) => "Represent this sentence for searching relevant passages: ",
        ("BGESmallZHV15" | "BGELargeZHV15", Input::Query) => {
            "为这个句子生成表示以用于检索相关文章："
        }
        _ => "",
    }
}

#[cfg(test)]
#[path = "retrieval_tests.rs"]
mod tests;
