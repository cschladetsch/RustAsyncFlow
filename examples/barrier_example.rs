use async_flow::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
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

    let barrier = FlowFactory::new_barrier_with_name("TaskBarrier");

    let task1 = FlowFactory::new_async_coroutine_with_name(
        "Task1",
        async_task("Alpha", 300)
    );

    let task2 = FlowFactory::new_async_coroutine_with_name(
        "Task2",
        async_task("Beta", 500)
    );

    let task3 = FlowFactory::new_async_coroutine_with_name(
        "Task3",
        async_task("Gamma", 200)
    );

    barrier.add_child(task1).await;
    barrier.add_child(task2).await;
    barrier.add_child(task3).await;

    let after_barrier = FlowFactory::new_async_coroutine_with_name(
        "AfterBarrier",
        async {
            println!("All barrier tasks completed! Proceeding...");
            sleep(Duration::from_millis(100)).await;
            println!("Cleanup done");
            Ok(())
        }
    );

    let sequence = FlowFactory::new_sequence_with_name("BarrierSequence");
    sequence.add_child(barrier).await;
    sequence.add_child(after_barrier).await;

    root.add_child(sequence).await;

    println!("Starting AsyncFlow barrier example...");
    kernel.run_until_complete().await?;
    println!("Example completed!");

    Ok(())
}