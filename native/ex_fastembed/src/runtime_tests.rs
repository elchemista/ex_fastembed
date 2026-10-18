use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{mpsc, Arc};
use std::time::Duration;

struct Model(Arc<AtomicUsize>);
impl Drop for Model {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

fn identity(name: &str) -> Identity {
    Identity {
        name: name.into(),
        repository: "test/model".into(),
        cache_dir: "/cache".into(),
    }
}

#[test]
fn unload_drops_native_resources_and_is_idempotent() {
    let slot = Slot::new();
    let drops = Arc::new(AtomicUsize::new(0));
    slot.load(identity("first"), || Ok(Model(drops.clone())))
        .unwrap();
    assert_eq!(slot.identity().unwrap().name, "first");
    assert!(slot.unload().unwrap());
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    assert!(slot.identity().is_none());
    assert_eq!(
        slot.with_model("missing", |_| Ok(())).unwrap_err(),
        "missing"
    );
    assert!(slot.unload().unwrap());
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}

#[test]
fn failed_load_preserves_model_and_replacement_drops_previous_session() {
    let slot = Slot::new();
    let drops = Arc::new(AtomicUsize::new(0));
    slot.load(identity("first"), || Ok(Model(drops.clone())))
        .unwrap();
    assert!(slot
        .load(identity("bad"), || Err("bad model".into()))
        .is_err());
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    assert_eq!(slot.identity().unwrap().name, "first");
    slot.load(identity("second"), || Ok(Model(drops.clone())))
        .unwrap();
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    assert_eq!(slot.identity().unwrap().name, "second");
}

#[test]
fn initialization_panic_preserves_the_previous_session_and_allows_retry() {
    let slot = Slot::new();
    slot.load(identity("first"), || Ok(42)).unwrap();
    let error = slot.load(identity("bad"), || panic!("initialization failed"));
    assert!(error.unwrap_err().contains("initialization panicked"));
    assert_eq!(slot.identity().unwrap().name, "first");
    assert_eq!(slot.with_model("missing", |model| Ok(*model)), Ok(42));
    assert!(!slot.model.is_poisoned());
    slot.load(identity("retry"), || Ok(43)).unwrap();
    assert_eq!(slot.with_model("missing", |model| Ok(*model)), Ok(43));
}

#[test]
fn inference_panic_drops_the_session_and_allows_unload_and_reload() {
    let slot = Slot::new();
    let drops = Arc::new(AtomicUsize::new(0));
    slot.load(identity("first"), || Ok(Model(drops.clone())))
        .unwrap();
    let error = slot.with_model::<()>("missing", |_| panic!("inference failed"));
    assert!(error.unwrap_err().contains("model unloaded"));
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    assert!(slot.identity().is_none());
    assert!(!slot.model.is_poisoned());
    assert!(slot.unload().unwrap());
    slot.load(identity("retry"), || Ok(Model(drops.clone())))
        .unwrap();
    assert!(slot.with_model("missing", |_| Ok(())).is_ok());
}

#[test]
fn ordinary_inference_errors_keep_the_session_loaded() {
    let slot = Slot::new();
    slot.load(identity("first"), || Ok(42)).unwrap();
    assert_eq!(
        slot.with_model::<()>("missing", |_| Err("invalid input".into())),
        Err("invalid input".into())
    );
    assert_eq!(slot.with_model("missing", |model| Ok(*model)), Ok(42));
}

#[test]
fn poisoned_slot_does_not_hide_its_identity_or_prevent_cleanup() {
    let slot = Slot::new();
    slot.load(identity("first"), || Ok(42)).unwrap();
    let _ = catch_unwind(AssertUnwindSafe(|| {
        let _guard = slot.model.lock().unwrap();
        panic!("unexpected panic while locked");
    }));
    assert!(slot.model.is_poisoned());
    assert_eq!(slot.identity().unwrap().name, "first");
    assert!(slot.unload().unwrap());
    slot.load(identity("retry"), || Ok(43)).unwrap();
    assert_eq!(slot.with_model("missing", |model| Ok(*model)), Ok(43));
}

struct PanickingDrop(bool);

impl Drop for PanickingDrop {
    fn drop(&mut self) {
        assert!(!self.0, "cleanup failed");
    }
}

#[test]
fn cleanup_panic_leaves_an_empty_slot_that_can_be_reused() {
    let slot = Slot::new();
    slot.load(identity("first"), || Ok(PanickingDrop(true)))
        .unwrap();
    assert!(slot.unload().unwrap_err().contains("session was removed"));
    assert!(slot.identity().is_none());
    assert!(!slot.model.is_poisoned());
    assert!(slot.unload().unwrap());
    slot.load(identity("retry"), || Ok(PanickingDrop(false)))
        .unwrap();
}

#[test]
fn replacement_cleanup_panic_keeps_the_successfully_initialized_replacement() {
    let slot = Slot::new();
    slot.load(identity("first"), || Ok(PanickingDrop(true)))
        .unwrap();
    let error = slot.load(identity("second"), || Ok(PanickingDrop(false)));
    assert!(error.unwrap_err().contains("replacement model is loaded"));
    assert_eq!(slot.identity().unwrap().name, "second");
    assert!(slot.unload().unwrap());
}

#[test]
fn unloading_repository_requires_matching_cache_root() {
    let slot = Slot::new();
    slot.load(identity("first"), || Ok(())).unwrap();
    slot.unload_repository("test/other", PathBuf::from("/cache").as_path())
        .unwrap();
    slot.unload_repository("test/model", PathBuf::from("/other").as_path())
        .unwrap();
    assert!(slot.identity().is_some());
    slot.unload_repository("test/model", PathBuf::from("/cache").as_path())
        .unwrap();
    assert!(slot.identity().is_none());
}

#[test]
fn unload_waits_for_inference_before_dropping_session() {
    let slot = Arc::new(Slot::new());
    let drops = Arc::new(AtomicUsize::new(0));
    slot.load(identity("first"), || Ok(Model(drops.clone())))
        .unwrap();
    let (started_tx, started_rx) = mpsc::channel();
    let (finish_tx, finish_rx) = mpsc::channel();
    let infer_slot = slot.clone();
    let inference = std::thread::spawn(move || {
        infer_slot.with_model("missing", |_| {
            started_tx.send(()).unwrap();
            finish_rx.recv_timeout(Duration::from_secs(5)).unwrap();
            Ok(42)
        })
    });
    started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    let unload_slot = slot.clone();
    let (unloaded_tx, unloaded_rx) = mpsc::channel();
    let unload = std::thread::spawn(move || {
        let result = unload_slot.unload();
        unloaded_tx.send(()).unwrap();
        result
    });
    assert!(unloaded_rx.recv_timeout(Duration::from_millis(30)).is_err());
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    finish_tx.send(()).unwrap();
    assert_eq!(inference.join().unwrap().unwrap(), 42);
    assert!(unload.join().unwrap().unwrap());
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}

#[test]
fn unload_waits_for_initialization_so_it_cannot_restore_a_dropped_session() {
    let slot = Arc::new(Slot::new());
    let (started_tx, started_rx) = mpsc::channel();
    let (finish_tx, finish_rx) = mpsc::channel();
    let load_slot = slot.clone();
    let load = std::thread::spawn(move || {
        load_slot.load(identity("first"), || {
            started_tx.send(()).unwrap();
            finish_rx.recv_timeout(Duration::from_secs(5)).unwrap();
            Ok(())
        })
    });
    started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    let unload_slot = slot.clone();
    let (unloaded_tx, unloaded_rx) = mpsc::channel();
    let unload = std::thread::spawn(move || {
        let result = unload_slot.unload();
        unloaded_tx.send(()).unwrap();
        result
    });
    assert!(unloaded_rx.recv_timeout(Duration::from_millis(30)).is_err());
    finish_tx.send(()).unwrap();
    load.join().unwrap().unwrap();
    unload.join().unwrap().unwrap();
    assert!(slot.identity().is_none());
}

#[test]
fn inference_and_state_inspection_wait_for_replacement_to_finish() {
    let slot = Arc::new(Slot::new());
    slot.load(identity("first"), || Ok(42)).unwrap();
    let (started_tx, started_rx) = mpsc::channel();
    let (finish_tx, finish_rx) = mpsc::channel();
    let load_slot = slot.clone();
    let load = std::thread::spawn(move || {
        load_slot.load(identity("second"), || {
            started_tx.send(()).unwrap();
            finish_rx.recv_timeout(Duration::from_secs(5)).unwrap();
            Ok(43)
        })
    });
    started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    let (inferred_tx, inferred_rx) = mpsc::channel();
    let infer_slot = slot.clone();
    let infer = std::thread::spawn(move || {
        inferred_tx
            .send(infer_slot.with_model("missing", |value| Ok(*value)))
            .unwrap();
    });
    let (inspected_tx, inspected_rx) = mpsc::channel();
    let inspect_slot = slot.clone();
    let inspect = std::thread::spawn(move || {
        inspected_tx
            .send(inspect_slot.identity().unwrap().name)
            .unwrap();
    });
    assert!(inferred_rx.recv_timeout(Duration::from_millis(30)).is_err());
    assert!(inspected_rx
        .recv_timeout(Duration::from_millis(30))
        .is_err());
    finish_tx.send(()).unwrap();
    load.join().unwrap().unwrap();
    infer.join().unwrap();
    inspect.join().unwrap();
    assert_eq!(inferred_rx.recv().unwrap(), Ok(43));
    assert_eq!(inspected_rx.recv().unwrap(), "second");
}

#[test]
fn inference_observes_the_identity_of_the_session_it_uses() {
    let slot = Slot::new();
    for name in ["first", "second"] {
        slot.load(identity(name), || Ok(name.to_string())).unwrap();
        slot.with_model_identity("missing", |model, identity| {
            assert_eq!(*model, identity.name);
            Ok(())
        })
        .unwrap();
    }
}
