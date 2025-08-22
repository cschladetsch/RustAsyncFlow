use async_flow::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let kernel = AsyncKernel::new();
    let root = kernel.root();

    println!("=== AsyncFlow Timed Trigger Demo ===");
    println!("This demo showcases various combinations of timers and triggers\n");

    // Demo 1: Basic timer with completion trigger
    demo_timer_with_trigger(&root).await?;
    
    // Demo 2: Periodic timer with count-based trigger
    demo_periodic_timer_with_counter_trigger(&root).await?;
    
    // Demo 3: Multiple timers with conditional triggers
    demo_multiple_timers_with_conditional_triggers(&root).await?;

    println!("Starting all timed trigger demos...");
    kernel.run_until_complete().await?;
    println!("\n=== All Timed Trigger Demos Completed! ===");

    Ok(())
}

async fn demo_timer_with_trigger(root: &Arc<Node>) -> Result<()> {
    println!("--- Demo 1: Basic Timer with Completion Trigger ---");
    
    // Create a timer that runs for 2 seconds
    let demo_timer = FlowFactory::new_timer_with_name(
        "Demo1Timer",
        Duration::from_secs(2)
    );
    
    let timer_completed = Arc::new(AtomicBool::new(false));
    
    // Set callback for when timer completes
    let timer_completed_clone = timer_completed.clone();
    demo_timer.set_elapsed_callback(move || {
        println!("  ✓ Timer elapsed after 2 seconds!");
        timer_completed_clone.store(true, Ordering::Relaxed);
    }).await;
    
    // Create trigger that waits for timer completion
    let completion_trigger = FlowFactory::new_trigger_with_name(
        "Demo1CompletionTrigger",
        {
            let timer_completed = timer_completed.clone();
            move || timer_completed.load(Ordering::Relaxed)
        }
    );
    
    completion_trigger.set_triggered_callback(|| {
        println!("  ✓ Completion trigger fired - Demo 1 finished!\n");
    }).await;
    
    root.add_child(demo_timer).await;
    root.add_child(completion_trigger).await;
    
    Ok(())
}

async fn demo_periodic_timer_with_counter_trigger(root: &Arc<Node>) -> Result<()> {
    println!("--- Demo 2: Periodic Timer with Counter Trigger ---");
    
    let tick_counter = Arc::new(AtomicU32::new(0));
    
    // Create periodic timer that ticks every 300ms
    let periodic_timer = FlowFactory::new_periodic_timer_with_name(
        "Demo2PeriodicTimer",
        Duration::from_millis(300)
    );
    
    let tick_counter_clone = tick_counter.clone();
    let periodic_timer_clone = periodic_timer.clone();
    periodic_timer.set_elapsed_callback(move || {
        let count = tick_counter_clone.fetch_add(1, Ordering::Relaxed) + 1;
        println!("  ⏱️  Periodic tick #{}", count);
        if count >= 4 {
            periodic_timer_clone.complete();
        }
    }).await;
    
    // Create trigger that fires after 4 ticks
    let counter_trigger = FlowFactory::new_trigger_with_name(
        "Demo2CounterTrigger",
        {
            let tick_counter = tick_counter.clone();
            move || tick_counter.load(Ordering::Relaxed) >= 4
        }
    );
    
    counter_trigger.set_triggered_callback(|| {
        println!("  ✓ Counter trigger fired after 4 ticks - Demo 2 finished!\n");
    }).await;
    
    root.add_child(periodic_timer).await;
    root.add_child(counter_trigger).await;
    
    Ok(())
}

async fn demo_multiple_timers_with_conditional_triggers(root: &Arc<Node>) -> Result<()> {
    println!("--- Demo 3: Multiple Timers with Conditional Triggers ---");
    
    let fast_timer_done = Arc::new(AtomicBool::new(false));
    let slow_timer_done = Arc::new(AtomicBool::new(false));
    
    // Fast timer (800ms)
    let fast_timer = FlowFactory::new_timer_with_name(
        "Demo3FastTimer",
        Duration::from_millis(800)
    );
    
    let fast_timer_done_clone = fast_timer_done.clone();
    fast_timer.set_elapsed_callback(move || {
        println!("  🏃 Fast timer completed (800ms)!");
        fast_timer_done_clone.store(true, Ordering::Relaxed);
    }).await;
    
    // Slow timer (1500ms)
    let slow_timer = FlowFactory::new_timer_with_name(
        "Demo3SlowTimer",
        Duration::from_millis(1500)
    );
    
    let slow_timer_done_clone = slow_timer_done.clone();
    slow_timer.set_elapsed_callback(move || {
        println!("  🐌 Slow timer completed (1500ms)!");
        slow_timer_done_clone.store(true, Ordering::Relaxed);
    }).await;
    
    // Trigger for when fast timer completes first
    let fast_trigger = FlowFactory::new_trigger_with_name(
        "Demo3FastTrigger",
        {
            let fast_done = fast_timer_done.clone();
            let slow_done = slow_timer_done.clone();
            move || fast_done.load(Ordering::Relaxed) && !slow_done.load(Ordering::Relaxed)
        }
    );
    
    fast_trigger.set_triggered_callback(|| {
        println!("  ⚡ Fast trigger fired - fast timer won the race!");
    }).await;
    
    // Trigger for when both timers complete
    let both_complete_trigger = FlowFactory::new_trigger_with_name(
        "Demo3BothCompleteTrigger",
        {
            let fast_done = fast_timer_done.clone();
            let slow_done = slow_timer_done.clone();
            move || fast_done.load(Ordering::Relaxed) && slow_done.load(Ordering::Relaxed)
        }
    );
    
    both_complete_trigger.set_triggered_callback(|| {
        println!("  ✓ Both timers completed - Demo 3 finished!\n");
    }).await;
    
    root.add_child(fast_timer).await;
    root.add_child(slow_timer).await;
    root.add_child(fast_trigger).await;
    root.add_child(both_complete_trigger).await;
    
    Ok(())
}