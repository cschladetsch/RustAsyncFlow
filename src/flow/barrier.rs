use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use crate::flow::{Generator, GeneratorBase};
use crate::{Logger, Result};

pub struct Barrier {
    base: GeneratorBase,
    children: Arc<RwLock<Vec<Arc<dyn Generator>>>>,
}

impl Barrier {
    pub fn new() -> Self {
        Self {
            base: GeneratorBase::new(),
            children: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn with_name(name: impl Into<String>) -> Self {
        Self {
            base: GeneratorBase::with_name(name),
            children: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn add_child(&self, child: Arc<dyn Generator>) {
        let mut children = self.children.write().await;
        children.push(child);
    }

    pub async fn child_count(&self) -> usize {
        let children = self.children.read().await;
        children.len()
    }

    async fn all_children_completed(&self) -> bool {
        let children = self.children.read().await;
        children.iter().all(|child| child.is_completed())
    }
}

impl Default for Barrier {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Generator for Barrier {
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

        let children = self.children.read().await;
        if children.is_empty() {
            self.complete();
            return Ok(());
        }

        for child in children.iter() {
            if child.is_active() && child.is_running() && !child.is_completed() {
                if let Err(e) = child.step().await {
                    self.logger().error(format!("Child step failed in barrier: {}", e));
                }
            }
        }

        if self.all_children_completed().await {
            self.complete();
        }

        Ok(())
    }

    fn logger(&self) -> &Logger {
        self.base.logger()
    }
}