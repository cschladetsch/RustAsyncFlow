use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use crate::flow::{Generator, GeneratorBase};
use crate::{Logger, Result};

pub struct Trigger {
    base: GeneratorBase,
    condition: Arc<RwLock<Box<dyn Fn() -> bool + Send + Sync>>>,
    triggered_callback: Arc<RwLock<Option<Box<dyn Fn() + Send + Sync>>>>,
    triggered: Arc<RwLock<bool>>,
}

impl Trigger {
    pub fn new<F>(condition: F) -> Self
    where
        F: Fn() -> bool + Send + Sync + 'static,
    {
        Self {
            base: GeneratorBase::new(),
            condition: Arc::new(RwLock::new(Box::new(condition))),
            triggered_callback: Arc::new(RwLock::new(None)),
            triggered: Arc::new(RwLock::new(false)),
        }
    }

    pub fn with_name<F>(name: impl Into<String>, condition: F) -> Self
    where
        F: Fn() -> bool + Send + Sync + 'static,
    {
        Self {
            base: GeneratorBase::with_name(name),
            condition: Arc::new(RwLock::new(Box::new(condition))),
            triggered_callback: Arc::new(RwLock::new(None)),
            triggered: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn set_triggered_callback<F>(&self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        let mut triggered_callback = self.triggered_callback.write().await;
        *triggered_callback = Some(Box::new(callback));
    }

    pub async fn is_triggered(&self) -> bool {
        *self.triggered.read().await
    }

    async fn check_condition(&self) -> bool {
        let condition = self.condition.read().await;
        condition()
    }

    async fn trigger(&self) {
        let mut triggered = self.triggered.write().await;
        *triggered = true;
    }
}

#[async_trait]
impl Generator for Trigger {
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

        if self.check_condition().await {
            if !self.is_triggered().await {
                let triggered_callback = self.triggered_callback.read().await;
                if let Some(ref callback) = *triggered_callback {
                    callback();
                }
                self.trigger().await;
            }
            self.complete();
        }

        Ok(())
    }

    fn logger(&self) -> &Logger {
        self.base.logger()
    }
}