use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use crate::flow::{Generator, GeneratorBase};
use crate::{Logger, Result};

pub struct Sequence {
    base: GeneratorBase,
    children: Arc<RwLock<Vec<Arc<dyn Generator>>>>,
    current_index: Arc<RwLock<usize>>,
}

impl Sequence {
    pub fn new() -> Self {
        Self {
            base: GeneratorBase::new(),
            children: Arc::new(RwLock::new(Vec::new())),
            current_index: Arc::new(RwLock::new(0)),
        }
    }

    pub fn with_name(name: impl Into<String>) -> Self {
        Self {
            base: GeneratorBase::with_name(name),
            children: Arc::new(RwLock::new(Vec::new())),
            current_index: Arc::new(RwLock::new(0)),
        }
    }

    pub async fn add_child(&self, child: Arc<dyn Generator>) {
        let mut children = self.children.write().await;
        children.push(child);
    }

    pub async fn current_index(&self) -> usize {
        *self.current_index.read().await
    }

    pub async fn child_count(&self) -> usize {
        let children = self.children.read().await;
        children.len()
    }
}

impl Default for Sequence {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Generator for Sequence {
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

        let mut current_index = self.current_index.write().await;
        
        if *current_index >= children.len() {
            self.complete();
            return Ok(());
        }

        let current_child = &children[*current_index];
        
        if current_child.is_completed() {
            *current_index += 1;
            if *current_index >= children.len() {
                self.complete();
            }
        } else if current_child.is_active() && current_child.is_running() {
            if let Err(e) = current_child.step().await {
                self.logger().error(format!("Child step failed in sequence: {}", e));
            }
        }

        Ok(())
    }

    fn logger(&self) -> &Logger {
        self.base.logger()
    }
}