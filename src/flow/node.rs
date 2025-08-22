use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use crate::flow::{Generator, GeneratorBase};
use crate::{Logger, Result};

pub struct Node {
    base: GeneratorBase,
    children: Arc<RwLock<Vec<Arc<dyn Generator>>>>,
}

impl Node {
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

    pub async fn remove_child(&self, id: Uuid) -> bool {
        let mut children = self.children.write().await;
        if let Some(pos) = children.iter().position(|c| c.id() == id) {
            children.remove(pos);
            return true;
        }
        false
    }

    pub async fn child_count(&self) -> usize {
        let children = self.children.read().await;
        children.len()
    }

    pub async fn clear_completed(&self) {
        let mut children = self.children.write().await;
        children.retain(|child| !child.is_completed());
    }
}

impl Default for Node {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Generator for Node {
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
            return Ok(());
        }

        if children.len() > 0 {
            self.logger().verbose(4, format!("Stepping node with {} children", children.len()));
        }

        for child in children.iter() {
            if child.is_active() && child.is_running() && !child.is_completed() {
                if let Err(e) = child.step().await {
                    self.logger().error(format!("Child step failed: {}", e));
                }
            }
        }

        Ok(())
    }

    fn logger(&self) -> &Logger {
        self.base.logger()
    }
}