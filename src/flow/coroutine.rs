use async_trait::async_trait;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use uuid::Uuid;
use crate::flow::{Generator, GeneratorBase};
use crate::{Logger, Result};

pub struct AsyncCoroutine {
    base: GeneratorBase,
    handle: Arc<Mutex<Option<JoinHandle<Result<()>>>>>,
}

impl AsyncCoroutine {
    pub fn new<F>(future: F) -> Self
    where
        F: Future<Output = Result<()>> + Send + 'static,
    {
        let handle = tokio::spawn(future);
        Self {
            base: GeneratorBase::new(),
            handle: Arc::new(Mutex::new(Some(handle))),
        }
    }

    pub fn with_name<F>(name: impl Into<String>, future: F) -> Self
    where
        F: Future<Output = Result<()>> + Send + 'static,
    {
        let handle = tokio::spawn(future);
        Self {
            base: GeneratorBase::with_name(name),
            handle: Arc::new(Mutex::new(Some(handle))),
        }
    }

    async fn is_handle_finished(&self) -> bool {
        let mut handle_lock = self.handle.lock().await;
        if let Some(ref handle) = *handle_lock {
            handle.is_finished()
        } else {
            true
        }
    }
}

#[async_trait]
impl Generator for AsyncCoroutine {
    fn id(&self) -> Uuid {
        self.base.id()
    }

    fn name(&self) -> Option<&str> {
        self.base.name()
    }

    fn set_name(&mut self, name: String) {
        self.base.set_name(name);
    }

    fn is_active(&self) -> bool {
        self.base.is_active()
    }

    fn is_running(&self) -> bool {
        self.base.is_running()
    }

    fn is_completed(&self) -> bool {
        self.base.is_completed()
    }

    fn activate(&self) {
        self.base.activate();
    }

    fn deactivate(&self) {
        self.base.deactivate();
    }

    fn complete(&self) {
        self.base.complete();
    }

    async fn step(&self) -> Result<()> {
        if !self.is_active() || !self.is_running() || self.is_completed() {
            return Ok(());
        }

        if let Some(ref name) = self.base.name() {
            self.logger().verbose(4, format!("Stepping coroutine: {}", name));
        }

        if self.is_handle_finished().await {
            let mut handle_lock = self.handle.lock().await;
            if let Some(handle) = handle_lock.take() {
                match handle.await {
                    Ok(result) => {
                        if let Err(e) = result {
                            self.logger().error(format!("Coroutine failed: {}", e));
                        }
                    }
                    Err(e) => {
                        self.logger().error(format!("Coroutine join failed: {}", e));
                    }
                }
            }
            self.complete();
        }

        Ok(())
    }

    fn logger(&self) -> &Logger {
        self.base.logger()
    }
}

pub struct SyncCoroutine<T> {
    base: GeneratorBase,
    step_fn: Option<Box<dyn Fn() -> Option<T> + Send + Sync>>,
    value: Option<T>,
}

impl<T: Send + Sync + 'static> SyncCoroutine<T> {
    pub fn new<F>(step_fn: F) -> Self
    where
        F: Fn() -> Option<T> + Send + Sync + 'static,
    {
        Self {
            base: GeneratorBase::new(),
            step_fn: Some(Box::new(step_fn)),
            value: None,
        }
    }

    pub fn value(&self) -> Option<&T> {
        self.value.as_ref()
    }
}

#[async_trait]
impl<T: Send + Sync + 'static> Generator for SyncCoroutine<T> {
    fn id(&self) -> Uuid {
        self.base.id()
    }

    fn name(&self) -> Option<&str> {
        self.base.name()
    }

    fn set_name(&mut self, name: String) {
        self.base.set_name(name);
    }

    fn is_active(&self) -> bool {
        self.base.is_active()
    }

    fn is_running(&self) -> bool {
        self.base.is_running()
    }

    fn is_completed(&self) -> bool {
        self.base.is_completed()
    }

    fn activate(&self) {
        self.base.activate();
    }

    fn deactivate(&self) {
        self.base.deactivate();
    }

    fn complete(&self) {
        self.base.complete();
    }

    async fn step(&self) -> Result<()> {
        if !self.is_active() || !self.is_running() || self.is_completed() {
            return Ok(());
        }

        if let Some(ref step_fn) = self.step_fn {
            let result = step_fn();
            if result.is_none() {
                self.complete();
            }
        }

        Ok(())
    }

    fn logger(&self) -> &Logger {
        self.base.logger()
    }
}