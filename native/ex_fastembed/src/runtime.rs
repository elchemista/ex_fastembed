//! Shared native sessions. Applications own request queues and admission control.
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, PoisonError};

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
        let mut slot = self.lock();
        // Keep the previous model alive until initialization succeeds.
        let model = catch_unwind(AssertUnwindSafe(initialize)).map_err(|_| {
            "Native model initialization panicked; previous model preserved.".to_string()
        })??;
        identity.cache_dir = identity
            .cache_dir
            .canonicalize()
            .unwrap_or(identity.cache_dir);
        // Commit the replacement before destroying the old session, so even a
        // destructor panic cannot leave a partially dropped value in the slot.
        let previous = slot.replace(Loaded { model, identity });
        drop_session(previous).map_err(|_| {
            "Previous model cleanup panicked; replacement model is loaded.".to_string()
        })
    }

    pub fn with_model<R>(
        &self,
        missing: &str,
        infer: impl FnOnce(&mut T) -> Result<R, String>,
    ) -> Result<R, String> {
        self.with_model_identity(missing, |model, _identity| infer(model))
    }

    pub fn with_model_identity<R>(
        &self,
        missing: &str,
        infer: impl FnOnce(&mut T, &Identity) -> Result<R, String>,
    ) -> Result<R, String> {
        let mut slot = self.lock();
        let loaded = slot.as_mut().ok_or_else(|| missing.to_string())?;
        match catch_unwind(AssertUnwindSafe(|| {
            infer(&mut loaded.model, &loaded.identity)
        })) {
            Ok(result) => result,
            Err(_) => {
                // Inference has mutable access to the session. Its internal
                // state is unknown after a panic, so do not reuse it.
                let _ = drop_session(slot.take());
                Err("Native inference panicked; model unloaded. Reload it before retrying.".into())
            }
        }
    }

    pub fn unload(&self) -> Result<bool, String> {
        let mut slot = self.lock();
        // Drop while holding the lock: success means destruction has completed.
        drop_session(slot.take())?;
        Ok(true)
    }

    pub fn identity(&self) -> Option<Identity> {
        self.lock().as_ref().map(|loaded| loaded.identity.clone())
    }

    fn lock(&self) -> MutexGuard<'_, Option<Loaded<T>>> {
        // Panics in initialization, inference, and destruction are contained
        // above, with the slot restored to a known state before returning.
        self.model.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub fn unload_repository(
        &self,
        repository: &str,
        cache_dir: &std::path::Path,
    ) -> Result<(), String> {
        // Caller holds the cache write lock, so no loader can change identity.
        if self
            .identity()
            .is_some_and(|id| id.repository == repository && id.cache_dir == cache_dir)
        {
            self.unload()?;
        }
        Ok(())
    }
}

fn drop_session<T>(session: Option<Loaded<T>>) -> Result<(), String> {
    catch_unwind(AssertUnwindSafe(|| drop(session)))
        .map_err(|_| "Native model cleanup panicked; session was removed.".into())
}

#[cfg(test)]
#[path = "runtime_tests.rs"]
mod tests;
