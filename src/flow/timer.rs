use async_trait::async_trait;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;
use crate::flow::{Generator, GeneratorBase};
use crate::{Logger, Result};

pub struct Timer {
    base: GeneratorBase,
    duration: Duration,
    start_time: Arc<RwLock<Option<Instant>>>,
    elapsed_callback: Arc<RwLock<Option<Box<dyn Fn() + Send + Sync>>>>,
}

impl Timer {
    pub fn new(duration: Duration) -> Self {
        Self {
            base: GeneratorBase::new(),
            duration,
            start_time: Arc::new(RwLock::new(None)),
            elapsed_callback: Arc::new(RwLock::new(None)),
        }
    }

    pub fn with_name(name: impl Into<String>, duration: Duration) -> Self {
        Self {
            base: GeneratorBase::with_name(name),
            duration,
            start_time: Arc::new(RwLock::new(None)),
            elapsed_callback: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn set_elapsed_callback<F>(&self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        let mut elapsed_callback = self.elapsed_callback.write().await;
        *elapsed_callback = Some(Box::new(callback));
    }

    pub async fn is_elapsed(&self) -> bool {
        let start_time = self.start_time.read().await;
        if let Some(start) = *start_time {
            start.elapsed() >= self.duration
        } else {
            false
        }
    }

    async fn start_if_needed(&self) {
        let mut start_time = self.start_time.write().await;
        if start_time.is_none() {
            *start_time = Some(Instant::now());
        }
    }
}

#[async_trait]
impl Generator for Timer {
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

        self.start_if_needed().await;

        if self.is_elapsed().await {
            let elapsed_callback = self.elapsed_callback.read().await;
            if let Some(ref callback) = *elapsed_callback {
                callback();
            }
            self.complete();
        }

        Ok(())
    }

    fn logger(&self) -> &Logger {
        self.base.logger()
    }
}

pub struct PeriodicTimer {
    base: GeneratorBase,
    interval: Duration,
    last_trigger: Arc<RwLock<Option<Instant>>>,
    elapsed_callback: Arc<RwLock<Option<Box<dyn Fn() + Send + Sync>>>>,
}

impl PeriodicTimer {
    pub fn new(interval: Duration) -> Self {
        Self {
            base: GeneratorBase::new(),
            interval,
            last_trigger: Arc::new(RwLock::new(None)),
            elapsed_callback: Arc::new(RwLock::new(None)),
        }
    }

    pub fn with_name(name: impl Into<String>, interval: Duration) -> Self {
        Self {
            base: GeneratorBase::with_name(name),
            interval,
            last_trigger: Arc::new(RwLock::new(None)),
            elapsed_callback: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn set_elapsed_callback<F>(&self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        let mut elapsed_callback = self.elapsed_callback.write().await;
        *elapsed_callback = Some(Box::new(callback));
    }

    async fn should_trigger(&self) -> bool {
        let last_trigger = self.last_trigger.read().await;
        if let Some(last) = *last_trigger {
            last.elapsed() >= self.interval
        } else {
            true
        }
    }

    async fn trigger(&self) {
        let mut last_trigger = self.last_trigger.write().await;
        *last_trigger = Some(Instant::now());
    }
}

#[async_trait]
impl Generator for PeriodicTimer {
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

        if self.should_trigger().await {
            let elapsed_callback = self.elapsed_callback.read().await;
            if let Some(ref callback) = *elapsed_callback {
                callback();
            }
            self.trigger().await;
        }

        Ok(())
    }

    fn logger(&self) -> &Logger {
        self.base.logger()
    }
}