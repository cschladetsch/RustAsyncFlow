use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use crate::flow::*;

pub struct FlowFactory;

impl FlowFactory {
    pub fn new_node() -> Arc<Node> {
        Arc::new(Node::new())
    }

    pub fn new_node_with_name(name: impl Into<String>) -> Arc<Node> {
        Arc::new(Node::with_name(name))
    }

    pub fn new_sequence() -> Arc<Sequence> {
        Arc::new(Sequence::new())
    }

    pub fn new_sequence_with_name(name: impl Into<String>) -> Arc<Sequence> {
        Arc::new(Sequence::with_name(name))
    }

    pub fn new_barrier() -> Arc<Barrier> {
        Arc::new(Barrier::new())
    }

    pub fn new_barrier_with_name(name: impl Into<String>) -> Arc<Barrier> {
        Arc::new(Barrier::with_name(name))
    }

    pub fn new_timer(duration: Duration) -> Arc<Timer> {
        Arc::new(Timer::new(duration))
    }

    pub fn new_timer_with_name(name: impl Into<String>, duration: Duration) -> Arc<Timer> {
        Arc::new(Timer::with_name(name, duration))
    }

    pub fn new_periodic_timer(interval: Duration) -> Arc<PeriodicTimer> {
        Arc::new(PeriodicTimer::new(interval))
    }

    pub fn new_periodic_timer_with_name(name: impl Into<String>, interval: Duration) -> Arc<PeriodicTimer> {
        Arc::new(PeriodicTimer::with_name(name, interval))
    }

    pub fn new_trigger<F>(condition: F) -> Arc<Trigger>
    where
        F: Fn() -> bool + Send + Sync + 'static,
    {
        Arc::new(Trigger::new(condition))
    }

    pub fn new_trigger_with_name<F>(name: impl Into<String>, condition: F) -> Arc<Trigger>
    where
        F: Fn() -> bool + Send + Sync + 'static,
    {
        Arc::new(Trigger::with_name(name, condition))
    }

    pub fn new_async_coroutine<F>(future: F) -> Arc<AsyncCoroutine>
    where
        F: Future<Output = crate::Result<()>> + Send + 'static,
    {
        Arc::new(AsyncCoroutine::new(future))
    }

    pub fn new_async_coroutine_with_name<F>(name: impl Into<String>, future: F) -> Arc<AsyncCoroutine>
    where
        F: Future<Output = crate::Result<()>> + Send + 'static,
    {
        Arc::new(AsyncCoroutine::with_name(name, future))
    }

    pub fn new_sync_coroutine<T, F>(step_fn: F) -> Arc<SyncCoroutine<T>>
    where
        T: Send + Sync + 'static,
        F: Fn() -> Option<T> + Send + Sync + 'static,
    {
        Arc::new(SyncCoroutine::new(step_fn))
    }

    pub fn new_future<T>() -> Arc<AsyncFuture<T>>
    where
        T: Send + Sync + 'static,
    {
        Arc::new(AsyncFuture::new())
    }

    pub fn new_future_with_name<T>(name: impl Into<String>) -> Arc<AsyncFuture<T>>
    where
        T: Send + Sync + 'static,
    {
        Arc::new(AsyncFuture::with_name(name))
    }
}

pub trait FlowExtensions {
    fn named(self, name: impl Into<String>) -> Self;
}

impl<T: Generator> FlowExtensions for Arc<T> {
    fn named(mut self, name: impl Into<String>) -> Self {
        if let Some(gen) = Arc::get_mut(&mut self) {
            gen.set_name(name.into());
        }
        self
    }
}