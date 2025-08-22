use async_trait::async_trait;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::sync::{RwLock, Notify};
use uuid::Uuid;
use crate::flow::{Generator, GeneratorBase};
use crate::{Logger, Result};

pub struct AsyncFuture<T> {
    base: GeneratorBase,
    inner: Arc<RwLock<Option<T>>>,
    notify: Arc<Notify>,
}

impl<T: Send + Sync + 'static> AsyncFuture<T> {
    pub fn new() -> Self {
        Self {
            base: GeneratorBase::new(),
            inner: Arc::new(RwLock::new(None)),
            notify: Arc::new(Notify::new()),
        }
    }

    pub fn with_name(name: impl Into<String>) -> Self {
        Self {
            base: GeneratorBase::with_name(name),
            inner: Arc::new(RwLock::new(None)),
            notify: Arc::new(Notify::new()),
        }
    }

    pub async fn set_value(&self, value: T) {
        let mut inner = self.inner.write().await;
        *inner = Some(value);
        self.notify.notify_waiters();
        self.complete();
    }

    pub async fn get_value(&self) -> Option<T> 
    where
        T: Clone,
    {
        let inner = self.inner.read().await;
        inner.clone()
    }

    pub async fn take_value(&self) -> Option<T> {
        let mut inner = self.inner.write().await;
        inner.take()
    }

    pub async fn wait(&self) -> T 
    where
        T: Clone,
    {
        loop {
            {
                let inner = self.inner.read().await;
                if let Some(ref value) = *inner {
                    return value.clone();
                }
            }
            
            self.notify.notified().await;
        }
    }

    pub fn is_ready(&self) -> bool {
        self.is_completed()
    }
}

impl<T: Send + Sync + 'static> Default for AsyncFuture<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl<T: Send + Sync + 'static> Generator for AsyncFuture<T> {
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

        let inner = self.inner.read().await;
        if inner.is_some() {
            self.complete();
        }

        Ok(())
    }

    fn logger(&self) -> &Logger {
        self.base.logger()
    }
}

impl<T: Send + Sync + 'static + Clone> Future for AsyncFuture<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let notify = self.notify.clone();
        let inner = self.inner.clone();
        
        let waker = cx.waker().clone();
        tokio::spawn(async move {
            notify.notified().await;
            waker.wake();
        });
        
        match futures::executor::block_on(async {
            let inner = inner.read().await;
            inner.clone()
        }) {
            Some(value) => Poll::Ready(value),
            None => Poll::Pending,
        }
    }
}