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
    assert_eq!(slot.identity().unwrap().unwrap().name, "first");
    assert!(slot.unload().unwrap());
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    assert!(slot.identity().unwrap().is_none());
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
    assert_eq!(slot.identity().unwrap().unwrap().name, "first");
    slot.load(identity("second"), || Ok(Model(drops.clone())))
        .unwrap();
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    assert_eq!(slot.identity().unwrap().unwrap().name, "second");
}

#[test]
fn unloading_repository_requires_matching_cache_root() {
    let slot = Slot::new();
    slot.load(identity("first"), || Ok(())).unwrap();
    slot.unload_repository("test/other", PathBuf::from("/cache").as_path())
        .unwrap();
    slot.unload_repository("test/model", PathBuf::from("/other").as_path())
        .unwrap();
    assert!(slot.identity().unwrap().is_some());
    slot.unload_repository("test/model", PathBuf::from("/cache").as_path())
        .unwrap();
    assert!(slot.identity().unwrap().is_none());
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
    assert!(slot.identity().unwrap().is_none());
}
