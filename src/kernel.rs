use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::{sleep, Instant};
use uuid::Uuid;
use crate::flow::{Generator, GeneratorBase, Node};
use crate::{Logger, TimeFrame, Result};

#[derive(Clone)]
pub struct AsyncKernel {
    base: GeneratorBase,
    root: Arc<Node>,
    time_frame: Arc<RwLock<TimeFrame>>,
    break_flag: Arc<RwLock<bool>>,
    wait_until: Arc<RwLock<Option<Instant>>>,
}

impl AsyncKernel {
    pub fn new() -> Self {
        Self {
            base: GeneratorBase::with_name("AsyncKernel"),
            root: Arc::new(Node::with_name("Root")),
            time_frame: Arc::new(RwLock::new(TimeFrame::new())),
            break_flag: Arc::new(RwLock::new(false)),
            wait_until: Arc::new(RwLock::new(None)),
        }
    }

    pub fn root(&self) -> Arc<Node> {
        self.root.clone()
    }

    pub async fn time_frame(&self) -> TimeFrame {
        let time_frame = self.time_frame.read().await;
        time_frame.clone()
    }

    pub async fn break_flow(&self) {
        let mut break_flag = self.break_flag.write().await;
        *break_flag = true;
    }

    pub async fn is_breaking(&self) -> bool {
        let break_flag = self.break_flag.read().await;
        *break_flag
    }

    pub async fn wait(&self, duration: Duration) {
        let mut wait_until = self.wait_until.write().await;
        *wait_until = Some(Instant::now() + duration);
    }

    pub async fn is_waiting(&self) -> bool {
        let wait_until = self.wait_until.read().await;
        if let Some(until) = *wait_until {
            Instant::now() < until
        } else {
            false
        }
    }

    pub async fn clear_wait(&self) {
        let mut wait_until = self.wait_until.write().await;
        *wait_until = None;
    }

    pub async fn update(&self, delta_time: Duration) -> Result<()> {
        {
            let mut time_frame = self.time_frame.write().await;
            time_frame.update_with_delta(delta_time);
        }

        self.step().await
    }

    pub async fn update_real_time(&self) -> Result<()> {
        {
            let mut time_frame = self.time_frame.write().await;
            time_frame.update();
        }

        self.step().await
    }

    pub async fn run_until_complete(&self) -> Result<()> {
        while self.is_running() && !self.is_breaking().await {
            if self.is_waiting().await {
                sleep(Duration::from_millis(1)).await;
                continue;
            }

            self.update_real_time().await?;
            
            if self.root.child_count().await == 0 {
                break;
            }

            sleep(Duration::from_millis(1)).await;
        }
        
        Ok(())
    }

    pub async fn run_for(&self, duration: Duration) -> Result<()> {
        let start_time = Instant::now();
        
        while self.is_running() && !self.is_breaking().await {
            if start_time.elapsed() >= duration {
                break;
            }

            if self.is_waiting().await {
                sleep(Duration::from_millis(1)).await;
                continue;
            }

            self.update_real_time().await?;
            sleep(Duration::from_millis(1)).await;
        }
        
        Ok(())
    }
}

impl Default for AsyncKernel {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Generator for AsyncKernel {
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

        if self.is_breaking().await {
            return Ok(());
        }

        if self.is_waiting().await {
            return Ok(());
        }

        let child_count = self.root.child_count().await;
        if child_count > 0 {
            self.logger().verbose(4, format!("Stepping kernel with {} root children", child_count));
        }

        self.root.step().await?;
        self.root.clear_completed().await;

        Ok(())
    }

    fn logger(&self) -> &Logger {
        self.base.logger()
    }
}