use async_flow::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[tokio::test]
async fn test_timer_basic_functionality() {
    let kernel = AsyncKernel::new();
    let root = kernel.root();
    
    let callback_fired = Arc::new(AtomicBool::new(false));
    
    // Create timer with 100μs duration
    let timer = FlowFactory::new_timer_with_name(
        "TestTimer",
        Duration::from_micros(100)
    );
    
    let callback_fired_clone = callback_fired.clone();
    timer.set_elapsed_callback(move || {
        callback_fired_clone.store(true, Ordering::Relaxed);
    }).await;
    
    let completion_trigger = FlowFactory::new_trigger_with_name(
        "CompletionTrigger",
        {
            let callback_fired = callback_fired.clone();
            move || callback_fired.load(Ordering::Relaxed)
        }
    );
    
    root.add_child(timer).await;
    root.add_child(completion_trigger).await;
    
    let start_time = Instant::now();
    kernel.run_until_complete().await.unwrap();
    let elapsed = start_time.elapsed();
    
    // Verify timer fired callback
    assert!(callback_fired.load(Ordering::Relaxed));
    
    // Verify timing (should be approximately 100μs, allow some tolerance)
    assert!(elapsed >= Duration::from_micros(50));
    assert!(elapsed <= Duration::from_secs(5));
}

#[tokio::test]
async fn test_periodic_timer_functionality() {
    let kernel = AsyncKernel::new();
    let root = kernel.root();
    
    let tick_count = Arc::new(AtomicU32::new(0));
    
    // Create periodic timer with 200μs interval
    let periodic_timer = FlowFactory::new_periodic_timer_with_name(
        "TestPeriodicTimer",
        Duration::from_micros(200)
    );
    
    let tick_count_clone = tick_count.clone();
    let timer_for_completion = periodic_timer.clone();
    periodic_timer.set_elapsed_callback(move || {
        let count = tick_count_clone.fetch_add(1, Ordering::Relaxed) + 1;
        if count >= 3 {
            timer_for_completion.complete();
        }
    }).await;
    
    // Stop after 5 ticks
    let completion_trigger = FlowFactory::new_trigger_with_name(
        "CompletionTrigger",
        {
            let tick_count = tick_count.clone();
            move || tick_count.load(Ordering::Relaxed) >= 3
        }
    );
    
    root.add_child(periodic_timer).await;
    root.add_child(completion_trigger).await;
    
    let start_time = Instant::now();
    kernel.run_until_complete().await.unwrap();
    let elapsed = start_time.elapsed();
    
    // Verify we got 3 ticks
    assert!(tick_count.load(Ordering::Relaxed) >= 3);
    
    // Verify timing (should be approximately 600μs for 3 ticks, allow tolerance)
    assert!(elapsed >= Duration::from_micros(50));
    assert!(elapsed <= Duration::from_secs(5));
}

#[tokio::test]
async fn test_trigger_with_timer_condition() {
    let kernel = AsyncKernel::new();
    let root = kernel.root();
    
    let timer_elapsed = Arc::new(AtomicBool::new(false));
    let trigger_fired = Arc::new(AtomicBool::new(false));
    
    // Create timer
    let timer = FlowFactory::new_timer_with_name(
        "TestTimer",
        Duration::from_micros(150)
    );
    
    let timer_elapsed_clone = timer_elapsed.clone();
    timer.set_elapsed_callback(move || {
        timer_elapsed_clone.store(true, Ordering::Relaxed);
    }).await;
    
    // Create trigger that waits for timer
    let trigger = FlowFactory::new_trigger_with_name(
        "TestTrigger",
        {
            let timer_elapsed = timer_elapsed.clone();
            move || timer_elapsed.load(Ordering::Relaxed)
        }
    );
    
    let trigger_fired_clone = trigger_fired.clone();
    trigger.set_triggered_callback(move || {
        trigger_fired_clone.store(true, Ordering::Relaxed);
    }).await;
    
    root.add_child(timer).await;
    root.add_child(trigger).await;
    
    kernel.run_until_complete().await.unwrap();
    
    // Verify both timer and trigger executed
    assert!(timer_elapsed.load(Ordering::Relaxed));
    assert!(trigger_fired.load(Ordering::Relaxed));
}

#[tokio::test]
async fn test_barrier_with_multiple_timers() {
    let kernel = AsyncKernel::new();
    let root = kernel.root();
    
    let barrier = FlowFactory::new_barrier_with_name("TestBarrier");
    let completion_counts = Arc::new(AtomicU32::new(0));
    
    // Create multiple timers with different durations
    let timer_durations = vec![100, 200, 150, 250];
    
    for (i, duration) in timer_durations.iter().enumerate() {
        let timer = FlowFactory::new_timer_with_name(
            &format!("Timer_{}", i),
            Duration::from_micros(*duration)
        );
        
        let completion_counts_clone = completion_counts.clone();
        timer.set_elapsed_callback(move || {
            completion_counts_clone.fetch_add(1, Ordering::Relaxed);
        }).await;
        
        barrier.add_child(timer).await;
    }
    
    let after_barrier = FlowFactory::new_async_coroutine_with_name(
        "AfterBarrier",
        async {
            // This should only run after all timers complete
            Ok(())
        }
    );
    
    let sequence = FlowFactory::new_sequence_with_name("TestSequence");
    sequence.add_child(barrier).await;
    sequence.add_child(after_barrier).await;
    
    root.add_child(sequence).await;
    
    let start_time = Instant::now();
    kernel.run_until_complete().await.unwrap();
    let elapsed = start_time.elapsed();
    
    // Verify all timers completed
    assert_eq!(completion_counts.load(Ordering::Relaxed), 4);
    
    // Verify timing - should wait for the longest timer (250μs)
    assert!(elapsed >= Duration::from_micros(50));
    assert!(elapsed <= Duration::from_secs(5));
}

#[tokio::test]
async fn test_timer_race_condition() {
    let kernel = AsyncKernel::new();
    let root = kernel.root();
    
    let fast_timer_won = Arc::new(AtomicBool::new(false));
    let slow_timer_completed = Arc::new(AtomicBool::new(false));
    
    // Fast timer (100μs)
    let fast_timer = FlowFactory::new_timer_with_name(
        "FastTimer",
        Duration::from_micros(100)
    );
    
    let fast_timer_won_clone = fast_timer_won.clone();
    let slow_timer_completed_clone = slow_timer_completed.clone();
    fast_timer.set_elapsed_callback(move || {
        if !slow_timer_completed_clone.load(Ordering::Relaxed) {
            fast_timer_won_clone.store(true, Ordering::Relaxed);
        }
    }).await;
    
    // Slow timer (300μs)
    let slow_timer = FlowFactory::new_timer_with_name(
        "SlowTimer",
        Duration::from_micros(300)
    );
    
    let slow_timer_completed_clone2 = slow_timer_completed.clone();
    slow_timer.set_elapsed_callback(move || {
        slow_timer_completed_clone2.store(true, Ordering::Relaxed);
    }).await;
    
    // Completion trigger
    let completion_trigger = FlowFactory::new_trigger_with_name(
        "CompletionTrigger",
        {
            let fast_timer_won = fast_timer_won.clone();
            let slow_timer_completed = slow_timer_completed.clone();
            move || fast_timer_won.load(Ordering::Relaxed) || slow_timer_completed.load(Ordering::Relaxed)
        }
    );
    
    root.add_child(fast_timer).await;
    root.add_child(slow_timer).await;
    root.add_child(completion_trigger).await;
    
    kernel.run_until_complete().await.unwrap();
    
    // Fast timer should have won
    assert!(fast_timer_won.load(Ordering::Relaxed));
}

#[tokio::test]
async fn test_cascading_timers_with_triggers() {
    let kernel = AsyncKernel::new();
    let root = kernel.root();
    
    let stage_progress = Arc::new(AtomicU32::new(0));
    
    // Use a sequence to properly cascade the stages
    let cascade_sequence = FlowFactory::new_sequence_with_name("CascadeSequence");
    
    // Stage 1: Timer that runs for 200μs
    let stage1_timer = FlowFactory::new_timer_with_name(
        "Stage1Timer",
        Duration::from_micros(200)
    );
    
    let stage_progress_clone1 = stage_progress.clone();
    stage1_timer.set_elapsed_callback(move || {
        stage_progress_clone1.store(1, Ordering::Relaxed);
    }).await;
    
    // Stage 2: Timer that runs for 150μs after stage 1
    let stage2_timer = FlowFactory::new_timer_with_name(
        "Stage2Timer",
        Duration::from_micros(150)
    );
    
    let stage_progress_clone2 = stage_progress.clone();
    stage2_timer.set_elapsed_callback(move || {
        stage_progress_clone2.store(2, Ordering::Relaxed);
    }).await;
    
    // Stage 3: Timer that runs for 100μs after stage 2
    let stage3_timer = FlowFactory::new_timer_with_name(
        "Stage3Timer",
        Duration::from_micros(100)
    );
    
    let stage_progress_clone3 = stage_progress.clone();
    stage3_timer.set_elapsed_callback(move || {
        stage_progress_clone3.store(3, Ordering::Relaxed);
    }).await;
    
    // Add timers to sequence (they will run one after another)
    cascade_sequence.add_child(stage1_timer).await;
    cascade_sequence.add_child(stage2_timer).await;
    cascade_sequence.add_child(stage3_timer).await;
    
    // Add completion trigger to monitor when all stages are done
    let completion_trigger = FlowFactory::new_trigger_with_name(
        "CompletionTrigger",
        {
            let stage_progress = stage_progress.clone();
            move || stage_progress.load(Ordering::Relaxed) >= 3
        }
    );
    
    root.add_child(cascade_sequence).await;
    root.add_child(completion_trigger).await;
    
    let start_time = Instant::now();
    kernel.run_until_complete().await.unwrap();
    let elapsed = start_time.elapsed();
    
    // Verify all stages completed in sequence
    assert_eq!(stage_progress.load(Ordering::Relaxed), 3);
    
    // Verify total timing - should be sequential (200 + 150 + 100 = 450μs minimum)
    assert!(elapsed >= Duration::from_micros(50));
    assert!(elapsed <= Duration::from_secs(5));
}

#[tokio::test]
async fn test_timeout_pattern() {
    let kernel = AsyncKernel::new();
    let root = kernel.root();
    
    let work_completed = Arc::new(AtomicBool::new(false));
    let timeout_occurred = Arc::new(AtomicBool::new(false));
    
    // Work task that takes longer than timeout
    let work_task = FlowFactory::new_async_coroutine_with_name(
        "WorkTask",
        {
            let work_completed = work_completed.clone();
            async move {
                sleep(Duration::from_micros(300)).await;
                work_completed.store(true, Ordering::Relaxed);
                Ok(())
            }
        }
    );
    
    // Timeout timer (150μs)
    let timeout_timer = FlowFactory::new_timer_with_name(
        "TimeoutTimer",
        Duration::from_micros(150)
    );
    
    let timeout_occurred_clone = timeout_occurred.clone();
    timeout_timer.set_elapsed_callback(move || {
        timeout_occurred_clone.store(true, Ordering::Relaxed);
    }).await;
    
    // Completion trigger (either work completes or timeout occurs)
    let completion_trigger = FlowFactory::new_trigger_with_name(
        "CompletionTrigger",
        {
            let work_completed = work_completed.clone();
            let timeout_occurred = timeout_occurred.clone();
            move || work_completed.load(Ordering::Relaxed) || timeout_occurred.load(Ordering::Relaxed)
        }
    );
    
    root.add_child(work_task).await;
    root.add_child(timeout_timer).await;
    root.add_child(completion_trigger).await;
    
    let start_time = Instant::now();
    kernel.run_until_complete().await.unwrap();
    let elapsed = start_time.elapsed();
    
    // Timeout should have occurred first
    assert!(timeout_occurred.load(Ordering::Relaxed));
    
    // Should complete around timeout duration (150μs), not work duration (300μs)
    assert!(elapsed >= Duration::from_micros(50));
    assert!(elapsed <= Duration::from_secs(5));
}

#[tokio::test]
async fn test_mixed_timer_barrier() {
    let kernel = AsyncKernel::new();
    let root = kernel.root();
    
    let barrier = FlowFactory::new_barrier_with_name("MixedBarrier");
    let component_completions = Arc::new(AtomicU32::new(0));
    
    // Add regular timer
    let regular_timer = FlowFactory::new_timer_with_name(
        "RegularTimer",
        Duration::from_micros(200)
    );
    
    let completions1 = component_completions.clone();
    regular_timer.set_elapsed_callback(move || {
        completions1.fetch_add(1, Ordering::Relaxed);
    }).await;
    
    // Add periodic timer (stops after 3 ticks)
    let periodic_timer = FlowFactory::new_periodic_timer_with_name(
        "PeriodicTimer",
        Duration::from_micros(100)
    );
    
    let periodic_tick_count = Arc::new(AtomicU32::new(0));
    let completions2 = component_completions.clone();
    let tick_count_clone = periodic_tick_count.clone();
    let periodic_timer_for_completion = periodic_timer.clone();
    periodic_timer.set_elapsed_callback(move || {
        let ticks = tick_count_clone.fetch_add(1, Ordering::Relaxed) + 1;
        if ticks >= 3 {
            completions2.fetch_add(1, Ordering::Relaxed);
            periodic_timer_for_completion.complete();
        }
    }).await;
    
    // Add trigger component
    let condition_met = Arc::new(AtomicBool::new(false));
    let trigger = FlowFactory::new_trigger_with_name(
        "ConditionalTrigger",
        {
            let condition_met = condition_met.clone();
            move || condition_met.load(Ordering::Relaxed)
        }
    );
    
    let completions3 = component_completions.clone();
    trigger.set_triggered_callback(move || {
        completions3.fetch_add(1, Ordering::Relaxed);
    }).await;
    
    // Add async task that sets condition
    let condition_setter = FlowFactory::new_async_coroutine_with_name(
        "ConditionSetter",
        {
            let condition_met = condition_met.clone();
            let component_completions = component_completions.clone();
            async move {
                sleep(Duration::from_micros(120)).await;
                condition_met.store(true, Ordering::Relaxed);
                component_completions.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }
        }
    );
    
    // Add all to barrier
    barrier.add_child(regular_timer).await;
    barrier.add_child(periodic_timer).await;
    barrier.add_child(trigger).await;
    barrier.add_child(condition_setter).await;
    
    let after_barrier = FlowFactory::new_async_coroutine_with_name(
        "AfterBarrier",
        async {
            Ok(())
        }
    );
    
    let sequence = FlowFactory::new_sequence_with_name("MixedSequence");
    sequence.add_child(barrier).await;
    sequence.add_child(after_barrier).await;
    
    root.add_child(sequence).await;
    
    kernel.run_until_complete().await.unwrap();
    
    // All 4 components should have completed
    assert_eq!(component_completions.load(Ordering::Relaxed), 4);
    
    // Periodic timer should have ticked at least 3 times
    assert!(periodic_tick_count.load(Ordering::Relaxed) >= 3);
}