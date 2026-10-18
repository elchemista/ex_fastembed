use super::*;

#[test]
fn cached_revisions_reject_paths_before_repairing_or_downloading_files() {
    let root = tempfile::tempdir().unwrap();
    let reference = root.path().join("main");
    assert!(cached_revision(&reference).is_err());
    for revision in [
        "",
        " ",
        ".",
        "..",
        "../../outside",
        "/outside",
        "a\\b",
        "a\nb",
    ] {
        std::fs::write(&reference, revision).unwrap();
        assert!(cached_revision(&reference).is_err(), "{revision:?}");
    }
    std::fs::write(&reference, "0123456789abcdef\n").unwrap();
    assert_eq!(cached_revision(&reference).unwrap(), "0123456789abcdef");
}

#[test]
fn incomplete_download_destinations_are_repaired_without_removing_valid_files() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("config.json");
    prepare_download_destination(&file).unwrap();
    assert!(!file.exists());
    std::fs::write(&file, b"").unwrap();
    prepare_download_destination(&file).unwrap();
    assert!(!file.exists());
    std::fs::write(&file, b"valid").unwrap();
    prepare_download_destination(&file).unwrap();
    assert_eq!(std::fs::read(&file).unwrap(), b"valid");
    std::fs::remove_file(&file).unwrap();
    std::fs::create_dir(&file).unwrap();
    std::fs::write(file.join("keep"), b"unrelated").unwrap();
    assert!(prepare_download_destination(&file).is_err());
    assert_eq!(std::fs::read(file.join("keep")).unwrap(), b"unrelated");
}

#[cfg(unix)]
#[test]
fn incomplete_snapshot_links_are_unlinked_without_touching_their_targets() {
    use std::os::unix::fs::symlink;
    let root = tempfile::tempdir().unwrap();
    let target = root.path().join("blob");
    let link = root.path().join("config.json");
    symlink(&target, &link).unwrap();
    prepare_download_destination(&link).unwrap();
    assert!(link.symlink_metadata().is_err());
    std::fs::write(&target, b"").unwrap();
    symlink(&target, &link).unwrap();
    prepare_download_destination(&link).unwrap();
    assert!(link.symlink_metadata().is_err());
    assert_eq!(std::fs::read(&target).unwrap(), b"");
    std::fs::write(&target, b"valid").unwrap();
    symlink(&target, &link).unwrap();
    prepare_download_destination(&link).unwrap();
    assert!(link.symlink_metadata().unwrap().is_symlink());
    assert_eq!(std::fs::read(&target).unwrap(), b"valid");
}

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

#[test]
fn revision_resolution_recovers_missing_corrupt_and_non_utf8_refs_only() {
    let root = tempfile::tempdir().unwrap();
    let reference = root.path().join("main");
    assert_eq!(
        resolve_revision(&reference, || Ok("remote".into())).unwrap(),
        "remote"
    );
    for bytes in [b"../../outside".as_slice(), b"", &[255]] {
        std::fs::write(&reference, bytes).unwrap();
        assert_eq!(
            resolve_revision(&reference, || Ok("remote".into())).unwrap(),
            "remote"
        );
    }
    std::fs::write(&reference, "saved\n").unwrap();
    assert_eq!(
        resolve_revision(&reference, || panic!("must not consult main")).unwrap(),
        "saved"
    );
    std::fs::remove_file(&reference).unwrap();
    std::fs::create_dir(&reference).unwrap();
    assert!(resolve_revision(&reference, || panic!("must preserve I/O error")).is_err());
}

#[test]
fn only_http_404_suggests_deleting_cached_variants() {
    for error in [
        ApiError::RequestError(Box::new(ureq::Error::StatusCode(404))),
        ApiError::TooManyRetries(Box::new(ApiError::RequestError(Box::new(
            ureq::Error::StatusCode(404),
        )))),
    ] {
        assert!(download_error(&error).contains("ExFastembed.delete_model/2"));
    }
    for error in [
        ApiError::RequestError(Box::new(ureq::Error::StatusCode(401))),
        ApiError::RequestError(Box::new(ureq::Error::StatusCode(500))),
        ApiError::IoError(std::io::Error::new(std::io::ErrorKind::TimedOut, "timeout")),
        ApiError::TooManyRetries(Box::new(ApiError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotConnected,
            "offline",
        )))),
    ] {
        assert!(!download_error(&error).contains("delete_model"));
        assert_eq!(download_error(&error), error.to_string());
    }
}

fn complete_snapshot(root: &Path, revision: &str) {
    let snapshot = root.join("models--test--model/snapshots").join(revision);
    std::fs::create_dir_all(&snapshot).unwrap();
    for file in std::iter::once("model.onnx").chain(TOKENIZER_FILES) {
        std::fs::write(snapshot.join(file), b"fixture").unwrap();
    }
}

#[test]
fn trailing_newline_cache_is_complete_and_normalized_without_network() {
    let root = tempfile::tempdir().unwrap();
    let cache = Cache::new(root.path().to_path_buf());
    cache
        .model("test/model".into())
        .create_ref("saved")
        .unwrap();
    let reference = root.path().join("models--test--model/refs/main");
    std::fs::write(&reference, "saved\n").unwrap();
    complete_snapshot(root.path(), "saved");
    assert!(files_cached(&cache, "test/model", "model.onnx", &[]));
    prepare_files(
        root.path().to_path_buf(),
        "test/model",
        "model.onnx",
        &[],
        "http://127.0.0.1:1".into(),
    )
    .unwrap();
    assert_eq!(std::fs::read_to_string(reference).unwrap(), "saved");
}

#[test]
fn corrupt_reference_is_resolved_from_remote_without_deleting_snapshots() {
    use std::io::{Read, Write};
    use std::time::{Duration, Instant};
    let root = tempfile::tempdir().unwrap();
    let cache = Cache::new(root.path().to_path_buf());
    cache
        .model("test/model".into())
        .create_ref("saved")
        .unwrap();
    let reference = root.path().join("models--test--model/refs/main");
    std::fs::write(&reference, "../../outside").unwrap();
    std::fs::write(root.path().join("outside"), b"keep").unwrap();
    complete_snapshot(root.path(), "old");
    complete_snapshot(root.path(), "resolved");
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let endpoint = format!("http://{}", listener.local_addr().unwrap());
    let server = std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(Instant::now() < deadline, "metadata request missing");
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(error) => panic!("{error}"),
            }
        };
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut request = Vec::new();
        let mut buffer = [0; 1024];
        while !request.windows(4).any(|part| part == b"\r\n\r\n") {
            let size = stream.read(&mut buffer).unwrap();
            assert!(size > 0);
            request.extend_from_slice(&buffer[..size]);
        }
        assert!(String::from_utf8_lossy(&request)
            .starts_with("GET /test/model/resolve/main/model.onnx "));
        stream.write_all(b"HTTP/1.1 206 Partial Content\r\nX-Repo-Commit: resolved\r\nETag: fixture\r\nContent-Range: bytes 0-0/7\r\nContent-Length: 1\r\nConnection: close\r\n\r\nx").unwrap();
    });
    let result = prepare_files(
        root.path().to_path_buf(),
        "test/model",
        "model.onnx",
        &[],
        endpoint,
    );
    server.join().unwrap();
    result.unwrap();
    assert_eq!(std::fs::read_to_string(reference).unwrap(), "resolved");
    assert_eq!(std::fs::read(root.path().join("outside")).unwrap(), b"keep");
    assert!(root
        .path()
        .join("models--test--model/snapshots/old/model.onnx")
        .is_file());
}
