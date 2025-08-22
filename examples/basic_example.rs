use async_flow::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let kernel = AsyncKernel::new();
    let root = kernel.root();

    let counter = Arc::new(AtomicU32::new(0));
    
    let periodic_timer = Arc::new(PeriodicTimer::new(Duration::from_millis(500)))
        .named("HeartbeatTimer");
    
    let counter_clone = counter.clone();
    periodic_timer.set_elapsed_callback(move || {
        let count = counter_clone.fetch_add(1, Ordering::Relaxed);
        println!("Heartbeat #{}", count + 1);
    }).await;

    let completion_trigger = Arc::new(Trigger::new({
        let counter = counter.clone();
        move || counter.load(Ordering::Relaxed) >= 5
    })).named("CompletionTrigger");

    completion_trigger.set_triggered_callback(|| {
        println!("Completion trigger fired! Stopping...");
    }).await;

    root.add_child(periodic_timer).await;
    root.add_child(completion_trigger).await;

    println!("Starting AsyncFlow basic example...");
    kernel.run_until_complete().await?;
    println!("Example completed!");

    Ok(())
}