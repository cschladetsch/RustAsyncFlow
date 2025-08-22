use async_flow::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let kernel = AsyncKernel::new();
    let root = kernel.root();

    println!("=== AsyncFlow Timed Barrier Demo ===");
    println!("This demo showcases barriers with timed components\n");

    // Demo 1: Barrier with multiple timed tasks
    demo_barrier_with_timed_tasks(&root).await?;
    
    // Demo 2: Staged timed barriers
    demo_staged_timed_barriers(&root).await?;
    
    // Demo 3: Barrier with mixed timer types
    demo_barrier_with_mixed_timers(&root).await?;

    println!("Starting all timed barrier demos...");
    kernel.run_until_complete().await?;
    println!("\n=== All Timed Barrier Demos Completed! ===");

    Ok(())
}

async fn demo_barrier_with_timed_tasks(root: &Arc<Node>) -> Result<()> {
    println!("--- Demo 1: Barrier with Multiple Timed Tasks ---");
    
    let barrier = Arc::new(Barrier::new()).named("Demo1TimedBarrier");
    
    // Create multiple timers with different durations
    let timers_data = vec![
        ("Timer_500ms", 500),
        ("Timer_300ms", 300),
        ("Timer_800ms", 800),
        ("Timer_200ms", 200),
    ];
    
    for (name, duration_ms) in timers_data {
        let timer = Arc::new(Timer::new(Duration::from_millis(duration_ms)))
            .named(name);
        
        let timer_name = name.to_string();
        timer.set_elapsed_callback(move || {
            println!("  ⏰ {} completed!", timer_name);
        }).await;
        
        barrier.add_child(timer).await;
    }
    
    // Create a task that runs after all timers complete
    let after_barrier = Arc::new(AsyncCoroutine::new(async {
        println!("  ✓ All timed tasks in barrier completed!");
        println!("  🎯 Demo 1 finished - barrier synchronized {} timers\n", 4);
        Ok(())
    })).named("Demo1AfterBarrier");
    
    // Sequence the barrier and after-task
    let sequence = Arc::new(Sequence::new()).named("Demo1Sequence");
    sequence.add_child(barrier).await;
    sequence.add_child(after_barrier).await;
    
    root.add_child(sequence).await;
    Ok(())
}

async fn demo_staged_timed_barriers(root: &Arc<Node>) -> Result<()> {
    println!("--- Demo 2: Staged Timed Barriers ---");
    
    // Stage 1: Fast timers (100-300ms)
    let stage1_barrier = Arc::new(Barrier::new()).named("Demo2Stage1Barrier");
    
    let stage1_timers = vec![
        ("Stage1_Fast1", 100),
        ("Stage1_Fast2", 200),
        ("Stage1_Fast3", 150),
    ];
    
    for (name, duration_ms) in stage1_timers {
        let timer = Arc::new(Timer::new(Duration::from_millis(duration_ms)))
            .named(name);
        
        let timer_name = name.to_string();
        timer.set_elapsed_callback(move || {
            println!("  🏃 Stage 1 {} completed ({}ms)", timer_name, duration_ms);
        }).await;
        
        stage1_barrier.add_child(timer).await;
    }
    
    // Stage 2: Medium timers (400-600ms)
    let stage2_barrier = Arc::new(Barrier::new()).named("Demo2Stage2Barrier");
    
    let stage2_timers = vec![
        ("Stage2_Med1", 400),
        ("Stage2_Med2", 500),
        ("Stage2_Med3", 600),
    ];
    
    for (name, duration_ms) in stage2_timers {
        let timer = Arc::new(Timer::new(Duration::from_millis(duration_ms)))
            .named(name);
        
        let timer_name = name.to_string();
        timer.set_elapsed_callback(move || {
            println!("  🚶 Stage 2 {} completed ({}ms)", timer_name, duration_ms);
        }).await;
        
        stage2_barrier.add_child(timer).await;
    }
    
    // Transition tasks
    let stage1_complete = Arc::new(AsyncCoroutine::new(async {
        println!("  ✓ Stage 1 barrier completed - all fast timers done!");
        Ok(())
    })).named("Stage1Complete");
    
    let stage2_complete = Arc::new(AsyncCoroutine::new(async {
        println!("  ✓ Stage 2 barrier completed - all medium timers done!");
        println!("  🎯 Demo 2 finished - staged barriers completed\n");
        Ok(())
    })).named("Stage2Complete");
    
    // Create sequence for staged execution
    let staged_sequence = Arc::new(Sequence::new()).named("Demo2StagedSequence");
    staged_sequence.add_child(stage1_barrier).await;
    staged_sequence.add_child(stage1_complete).await;
    staged_sequence.add_child(stage2_barrier).await;
    staged_sequence.add_child(stage2_complete).await;
    
    root.add_child(staged_sequence).await;
    Ok(())
}

async fn demo_barrier_with_mixed_timers(root: &Arc<Node>) -> Result<()> {
    println!("--- Demo 3: Barrier with Mixed Timer Types ---");
    
    let mixed_barrier = Arc::new(Barrier::new()).named("Demo3MixedTimerBarrier");
    let completion_counter = Arc::new(AtomicU32::new(0));
    
    // Add a regular timer
    let regular_timer = Arc::new(Timer::new(Duration::from_millis(600)))
        .named("MixedRegularTimer");
    
    let counter1 = completion_counter.clone();
    regular_timer.set_elapsed_callback(move || {
        counter1.fetch_add(1, Ordering::Relaxed);
        println!("  ⏰ Regular timer completed (600ms)");
    }).await;
    
    // Add a periodic timer (but we'll let it complete via trigger)
    let periodic_timer = Arc::new(PeriodicTimer::new(Duration::from_millis(150)))
        .named("MixedPeriodicTimer");
    
    let tick_count = Arc::new(AtomicU32::new(0));
    let counter2 = completion_counter.clone();
    let tick_count_clone = tick_count.clone();
    
    periodic_timer.set_elapsed_callback(move || {
        let ticks = tick_count_clone.fetch_add(1, Ordering::Relaxed) + 1;
        println!("  ⏱️  Periodic timer tick #{}", ticks);
        
        // Complete after 4 ticks
        if ticks >= 4 {
            counter2.fetch_add(1, Ordering::Relaxed);
            println!("  ✓ Periodic timer completed after 4 ticks (600ms total)");
        }
    }).await;
    
    // Add a trigger that waits for external condition
    let condition_met = Arc::new(AtomicBool::new(false));
    let condition_trigger = Arc::new(Trigger::new({
        let condition_met = condition_met.clone();
        move || condition_met.load(Ordering::Relaxed)
    })).named("MixedConditionTrigger");
    
    let counter3 = completion_counter.clone();
    let _condition_met_clone = condition_met.clone();
    condition_trigger.set_triggered_callback(move || {
        counter3.fetch_add(1, Ordering::Relaxed);
        println!("  ⚡ Condition trigger fired!");
    }).await;
    
    // Add an async task that simulates work and sets the condition
    let condition_setter = Arc::new(AsyncCoroutine::new({
        let condition_met = condition_met.clone();
        let completion_counter = completion_counter.clone();
        async move {
            sleep(Duration::from_millis(450)).await;
            condition_met.store(true, Ordering::Relaxed);
            completion_counter.fetch_add(1, Ordering::Relaxed);
            println!("  🔧 Async task completed (450ms) - condition set!");
            Ok(())
        }
    })).named("MixedConditionSetter");
    
    // Add all components to the barrier
    mixed_barrier.add_child(regular_timer).await;
    mixed_barrier.add_child(periodic_timer).await;
    mixed_barrier.add_child(condition_trigger).await;
    mixed_barrier.add_child(condition_setter).await;
    
    // Final completion task
    let mixed_complete = Arc::new(AsyncCoroutine::new(async {
        println!("  ✓ All mixed timer components completed!");
        println!("  🎯 Demo 3 finished - mixed timer barrier synchronized\n");
        Ok(())
    })).named("Demo3MixedComplete");
    
    let mixed_sequence = Arc::new(Sequence::new()).named("Demo3MixedSequence");
    mixed_sequence.add_child(mixed_barrier).await;
    mixed_sequence.add_child(mixed_complete).await;
    
    root.add_child(mixed_sequence).await;
    Ok(())
}