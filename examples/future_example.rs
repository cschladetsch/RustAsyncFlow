use async_flow::*;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let kernel = AsyncKernel::new();
    let root = kernel.root();

    let future1 = FlowFactory::new_future_with_name::<String>("DataFuture");
    let future2 = FlowFactory::new_future_with_name::<u32>("CountFuture");

    let data_producer = FlowFactory::new_async_coroutine_with_name(
        "DataProducer",
        {
            let future1 = future1.clone();
            async move {
                println!("Producing data...");
                sleep(Duration::from_millis(400)).await;
                future1.set_value("Hello AsyncFlow!".to_string()).await;
                println!("Data produced");
                Ok(())
            }
        }
    );

    let count_producer = FlowFactory::new_async_coroutine_with_name(
        "CountProducer", 
        {
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
        }
    );

    let consumer = FlowFactory::new_async_coroutine_with_name(
        "Consumer",
        {
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
        }
    );

    let producer_barrier = FlowFactory::new_barrier_with_name("ProducerBarrier");
    producer_barrier.add_child(data_producer).await;
    producer_barrier.add_child(count_producer).await;

    let sequence = FlowFactory::new_sequence_with_name("FutureSequence");
    sequence.add_child(producer_barrier).await;
    sequence.add_child(consumer).await;

    root.add_child(sequence).await;

    println!("Starting AsyncFlow future example...");
    kernel.run_until_complete().await?;
    println!("Example completed!");

    Ok(())
}