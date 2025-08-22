use async_flow::*;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let kernel = AsyncKernel::new();
    let root = kernel.root();

    let future1 = Arc::new(AsyncFuture::<String>::new()).named("DataFuture");
    let future2 = Arc::new(AsyncFuture::<u32>::new()).named("CountFuture");

    let data_producer = Arc::new(AsyncCoroutine::new({
        let future1 = future1.clone();
        async move {
            println!("Producing data...");
            sleep(Duration::from_millis(400)).await;
            future1.set_value("Hello AsyncFlow!".to_string()).await;
            println!("Data produced");
            Ok(())
        }
    })).named("DataProducer");

    let count_producer = Arc::new(AsyncCoroutine::new({
        let future2 = future2.clone();
        async move {
            println!("Counting...");
            for i in 1..=5 {
                println!("Count: {}", i);
                sleep(Duration::from_millis(100)).await;
            }
            future2.set_value(42).await;
            println!("Counting complete");
            Ok(())
        }
    })).named("CountProducer");

    let consumer = Arc::new(AsyncCoroutine::new({
        let future1 = future1.clone();
        let future2 = future2.clone();
        async move {
            println!("Waiting for futures...");
            
            let data = future1.wait().await;
            println!("Received data: {}", data);
            
            let count = future2.wait().await;
            println!("Received count: {}", count);
            
            println!("All data received!");
            Ok(())
        }
    })).named("Consumer");

    let producer_barrier = Arc::new(Barrier::new()).named("ProducerBarrier");
    producer_barrier.add_child(data_producer).await;
    producer_barrier.add_child(count_producer).await;

    let sequence = Arc::new(Sequence::new()).named("FutureSequence");
    sequence.add_child(producer_barrier).await;
    sequence.add_child(consumer).await;

    root.add_child(sequence).await;

    println!("Starting AsyncFlow future example...");
    kernel.run_until_complete().await?;
    println!("Example completed!");

    Ok(())
}