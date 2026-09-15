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
fn every_embedding_variant_can_be_selected_explicitly() {
    let names = supported_embedding_model_names();

    for info in TextEmbedding::list_supported_models() {
        let name = info.model.to_string();
        assert!(names.contains(&name));
        assert_eq!(resolve_embedding_model(&name), Ok(info.model.clone()));
        assert_eq!(
            resolve_embedding_model(&name.to_lowercase()),
            Ok(info.model)
        );
    }
}

#[test]
fn every_reranker_variant_can_be_selected_explicitly() {
    let names = supported_reranker_model_names();

    for info in TextRerank::list_supported_models() {
        let name = format!("{:?}", info.model);
        assert!(names.contains(&name));
        assert_eq!(resolve_reranker_model(&name), Ok(info.model.clone()));
        assert_eq!(resolve_reranker_model(&name.to_lowercase()), Ok(info.model));
    }
}

#[test]
fn legacy_aliases_preserve_their_models_case_insensitively() {
    for (name, model) in LEGACY_EMBEDDING_ALIASES {
        assert_eq!(
            resolve_embedding_model(&name.to_uppercase()),
            Ok(model.clone())
        );
    }

    for (name, model) in LEGACY_RERANKER_ALIASES {
        assert_eq!(
            resolve_reranker_model(&name.to_uppercase()),
            Ok(model.clone())
        );
    }
}

#[test]
fn all_shared_repositories_prefer_a_non_quantized_variant() {
    for info in TextEmbedding::list_supported_models() {
        if !is_quantized_model(&info.model) {
            assert_eq!(resolve_embedding_model(&info.model_code), Ok(info.model));
        }
    }
}

#[test]
fn quantized_model_detection_includes_numbered_variants() {
    assert!(is_quantized_model(&EmbeddingModel::EmbeddingGemma300MQ));
    assert!(is_quantized_model(&EmbeddingModel::EmbeddingGemma300MQ4));
    assert!(!is_quantized_model(&EmbeddingModel::EmbeddingGemma300M));
}
