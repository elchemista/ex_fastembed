//! Shared native sessions. Applications own request queues and admission control.
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Clone)]
pub(crate) struct Identity {
    pub name: String,
    pub repository: String,
    pub cache_dir: PathBuf,
}

struct Loaded<T> {
    model: T,
    identity: Identity,
}

pub(crate) struct Slot<T> {
    model: Mutex<Option<Loaded<T>>>,
}

impl<T> Slot<T> {
    pub const fn new() -> Self {
        Self {
            model: Mutex::new(None),
        }
    }

    pub fn load(
        &self,
        mut identity: Identity,
        initialize: impl FnOnce() -> Result<T, String>,
    ) -> Result<(), String> {
        let mut slot = self.model.lock().map_err(|e| e.to_string())?;
        // Keep the previous model alive until initialization succeeds.
        let model = initialize()?;
        identity.cache_dir = identity
            .cache_dir
            .canonicalize()
            .unwrap_or(identity.cache_dir);
        *slot = Some(Loaded { model, identity });
        Ok(())
    }

    pub fn with_model<R>(
        &self,
        missing: &str,
        infer: impl FnOnce(&mut T) -> Result<R, String>,
    ) -> Result<R, String> {
        let mut slot = self.model.lock().map_err(|e| e.to_string())?;
        let loaded = slot.as_mut().ok_or_else(|| missing.to_string())?;
        infer(&mut loaded.model)
    }

    pub fn unload(&self) -> Result<bool, String> {
        let mut slot = self.model.lock().map_err(|e| e.to_string())?;
        // Drop while holding the lock: success means destruction has completed.
        *slot = None;
        Ok(true)
    }

    pub fn identity(&self) -> Result<Option<Identity>, String> {
        let slot = self.model.lock().map_err(|e| e.to_string())?;
        Ok(slot.as_ref().map(|loaded| loaded.identity.clone()))
    }

    pub fn unload_repository(
        &self,
        repository: &str,
        cache_dir: &std::path::Path,
    ) -> Result<(), String> {
        // Caller holds the cache write lock, so no loader can change identity.
        if self
            .identity()?
            .is_some_and(|id| id.repository == repository && id.cache_dir == cache_dir)
        {
            self.unload()?;
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "runtime_tests.rs"]
mod tests;
