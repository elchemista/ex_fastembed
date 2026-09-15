use super::*;

#[test]
fn cache_requires_model_tokenizer_and_external_weights_for_the_selected_variant() {
    let directory = tempfile::tempdir().unwrap();
    let cache = Cache::new(directory.path().to_path_buf());
    let repo = "test/model";
    let additional = vec!["onnx/model.onnx_data".to_string()];
    assert!(!files_cached(&cache, repo, "onnx/model.onnx", &additional));

    cache
        .model(repo.to_string())
        .create_ref("revision")
        .unwrap();
    let snapshot = directory
        .path()
        .join("models--test--model/snapshots/revision");
    for file in std::iter::once("onnx/model.onnx").chain(TOKENIZER_FILES) {
        let path = snapshot.join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, b"fixture").unwrap();
    }
    assert!(!files_cached(&cache, repo, "onnx/model.onnx", &additional));
    std::fs::write(snapshot.join(&additional[0]), b"weights").unwrap();
    assert!(files_cached(&cache, repo, "onnx/model.onnx", &additional));
    assert!(!files_cached(
        &cache,
        repo,
        "onnx/model_quantized.onnx",
        &[]
    ));

    let config = snapshot.join("tokenizer_config.json");
    std::fs::write(&config, b"").unwrap();
    assert!(!files_cached(&cache, repo, "onnx/model.onnx", &additional));
    std::fs::remove_file(&config).unwrap();
    std::fs::create_dir(&config).unwrap();
    assert!(!files_cached(&cache, repo, "onnx/model.onnx", &additional));
    std::fs::remove_dir(&config).unwrap();
    assert!(!files_cached(&cache, repo, "onnx/model.onnx", &additional));
}

#[cfg(unix)]
#[test]
fn cache_follows_hub_symlinks_and_rejects_broken_links() {
    let directory = tempfile::tempdir().unwrap();
    let cache = Cache::new(directory.path().to_path_buf());
    cache
        .model("test/model".into())
        .create_ref("revision")
        .unwrap();
    let snapshot = directory
        .path()
        .join("models--test--model/snapshots/revision");
    std::fs::create_dir_all(&snapshot).unwrap();
    let blob = directory.path().join("blob");
    std::fs::write(&blob, b"fixture").unwrap();
    for file in std::iter::once("model.onnx").chain(TOKENIZER_FILES) {
        std::os::unix::fs::symlink(&blob, snapshot.join(file)).unwrap();
    }
    assert!(files_cached(&cache, "test/model", "model.onnx", &[]));
    std::fs::remove_file(blob).unwrap();
    assert!(!files_cached(&cache, "test/model", "model.onnx", &[]));
}
