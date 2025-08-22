use async_trait::async_trait;
use std::sync::atomic::{AtomicBool, Ordering};
use uuid::Uuid;
use crate::Logger;

#[async_trait]
pub trait Generator: Send + Sync {
    fn id(&self) -> Uuid;
    fn name(&self) -> Option<&str>;
    fn set_name(&mut self, name: String);
    fn is_active(&self) -> bool;
    fn is_running(&self) -> bool;
    fn is_completed(&self) -> bool;
    fn activate(&self);
    fn deactivate(&self);
    fn complete(&self);
    async fn step(&self) -> crate::Result<()>;
    fn logger(&self) -> &Logger;
}

pub struct GeneratorBase {
    id: Uuid,
    name: Option<String>,
    active: AtomicBool,
    running: AtomicBool,
    completed: AtomicBool,
    logger: Logger,
}

impl Clone for GeneratorBase {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            name: self.name.clone(),
            active: AtomicBool::new(self.active.load(Ordering::Relaxed)),
            running: AtomicBool::new(self.running.load(Ordering::Relaxed)),
            completed: AtomicBool::new(self.completed.load(Ordering::Relaxed)),
            logger: self.logger.clone(),
        }
    }
}

impl GeneratorBase {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: None,
            active: AtomicBool::new(true),
            running: AtomicBool::new(true),
            completed: AtomicBool::new(false),
            logger: Logger::default(),
        }
    }

    pub fn with_name(name: impl Into<String>) -> Self {
        let mut base = Self::new();
        base.name = Some(name.into());
        base
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn set_name(&mut self, name: String) {
        self.name = Some(name);
    }

    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Relaxed)
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    pub fn is_completed(&self) -> bool {
        self.completed.load(Ordering::Relaxed)
    }

    pub fn activate(&self) {
        self.active.store(true, Ordering::Relaxed);
    }

    pub fn deactivate(&self) {
        self.active.store(false, Ordering::Relaxed);
    }

    pub fn complete(&self) {
        self.completed.store(true, Ordering::Relaxed);
        self.running.store(false, Ordering::Relaxed);
    }

    pub fn logger(&self) -> &Logger {
        &self.logger
    }
}