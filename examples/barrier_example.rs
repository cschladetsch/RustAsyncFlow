use async_flow::*;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

async fn async_task(name: &str, delay_ms: u64) -> Result<()> {
    println!("Task {} starting...", name);
    sleep(Duration::from_millis(delay_ms)).await;
    println!("Task {} completed after {}ms", name, delay_ms);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let kernel = AsyncKernel::new();
    let root = kernel.root();

    let barrier = Arc::new(Barrier::new()).named("TaskBarrier");

    let task1 = Arc::new(AsyncCoroutine::new(async_task("Alpha", 300)))
        .named("Task1");

    let task2 = Arc::new(AsyncCoroutine::new(async_task("Beta", 500)))
        .named("Task2");

    let task3 = Arc::new(AsyncCoroutine::new(async_task("Gamma", 200)))
        .named("Task3");

    barrier.add_child(task1).await;
    barrier.add_child(task2).await;
    barrier.add_child(task3).await;

    let after_barrier = Arc::new(AsyncCoroutine::new(async {
        println!("All barrier tasks completed! Proceeding...");
        sleep(Duration::from_millis(100)).await;
        println!("Cleanup done");
        Ok(())
    })).named("AfterBarrier");

    let sequence = Arc::new(Sequence::new()).named("BarrierSequence");
    sequence.add_child(barrier).await;
    sequence.add_child(after_barrier).await;

    root.add_child(sequence).await;

    println!("Starting AsyncFlow barrier example...");
    kernel.run_until_complete().await?;
    println!("Example completed!");

    Ok(())
}