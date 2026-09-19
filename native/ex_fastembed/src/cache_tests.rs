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

#[test]
fn required_files_are_unique_and_sorted() {
    assert_eq!(
        required_files("model.onnx", &["model.onnx".into(), "weights.data".into()]),
        vec![
            "config.json",
            "model.onnx",
            "special_tokens_map.json",
            "tokenizer.json",
            "tokenizer_config.json",
            "weights.data"
        ]
    );
}

#[test]
fn repository_paths_reject_traversal_and_invalid_names() {
    let root = Path::new("/cache");
    for repository in [
        "../model",
        "test/..",
        "test/../model",
        "/absolute",
        "test/a\\b",
        "test/",
        "",
        "model",
    ] {
        assert!(repository_path(root, repository).is_err(), "{repository}");
    }
    assert_eq!(
        repository_path(root, "test/model-v1.5").unwrap(),
        root.join("models--test--model-v1.5")
    );
}

#[test]
fn removal_is_idempotent_and_keeps_sibling_repositories() {
    let root = tempfile::tempdir().unwrap();
    let selected = root.path().join("models--test--selected");
    let other = root.path().join("models--test--other");
    std::fs::create_dir_all(selected.join("snapshots/revision")).unwrap();
    std::fs::write(selected.join("snapshots/revision/model.onnx"), b"weights").unwrap();
    std::fs::write(&other, b"unrelated").unwrap();
    assert_eq!(directory_bytes(&selected).unwrap(), 7);
    remove_repository(&selected).unwrap();
    remove_repository(&selected).unwrap();
    assert!(!selected.exists());
    assert_eq!(std::fs::read(&other).unwrap(), b"unrelated");
    assert!(remove_repository(&other).is_err());
}

#[cfg(unix)]
#[test]
fn sizes_and_removal_do_not_follow_links_or_delete_external_targets() {
    use std::os::unix::fs::symlink;
    let root = tempfile::tempdir().unwrap();
    let external = tempfile::tempdir().unwrap();
    std::fs::write(external.path().join("keep"), b"important").unwrap();
    let repo = root.path().join("repository");
    std::fs::create_dir_all(repo.join("blobs")).unwrap();
    std::fs::write(repo.join("blobs/weights"), b"weights").unwrap();
    symlink(repo.join("blobs/weights"), repo.join("model.onnx")).unwrap();
    symlink(external.path(), repo.join("outside")).unwrap();
    symlink(&repo, root.path().join("repository-link")).unwrap();
    assert_eq!(directory_bytes(&repo).unwrap(), 7);
    assert!(remove_repository(&root.path().join("repository-link")).is_err());
    remove_repository(&repo).unwrap();
    assert_eq!(
        std::fs::read(external.path().join("keep")).unwrap(),
        b"important"
    );
}
