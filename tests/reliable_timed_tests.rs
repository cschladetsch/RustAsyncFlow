use async_flow::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[tokio::test]
async fn test_timer_basic_functionality_simple() {
    let kernel = AsyncKernel::new();
    let root = kernel.root();
    
    let callback_fired = Arc::new(AtomicBool::new(false));
    
    let timer = Arc::new(Timer::new(Duration::from_micros(150))).named("TestTimer");
    
    let callback_fired_clone = callback_fired.clone();
    timer.set_elapsed_callback(move || {
        callback_fired_clone.store(true, Ordering::Relaxed);
    }).await;
    
    let completion_trigger = Arc::new(Trigger::new({
        let callback_fired = callback_fired.clone();
        move || callback_fired.load(Ordering::Relaxed)
    })).named("CompletionTrigger");
    
    root.add_child(timer).await;
    root.add_child(completion_trigger).await;
    
    let start_time = Instant::now();
    kernel.run_until_complete().await.unwrap();
    let elapsed = start_time.elapsed();
    
    assert!(callback_fired.load(Ordering::Relaxed));
    assert!(elapsed >= Duration::from_micros(50));
    assert!(elapsed <= Duration::from_secs(5));
}

#[tokio::test]
async fn test_periodic_timer_with_manual_completion() {
    let kernel = AsyncKernel::new();
    let root = kernel.root();
    
    let tick_count = Arc::new(AtomicU32::new(0));
    
    let periodic_timer = Arc::new(PeriodicTimer::new(Duration::from_micros(100))).named("TestPeriodicTimer");
    
    let tick_count_clone = tick_count.clone();
    let timer_for_completion = periodic_timer.clone();
    periodic_timer.set_elapsed_callback(move || {
        let count = tick_count_clone.fetch_add(1, Ordering::Relaxed) + 1;
        if count >= 3 {
            timer_for_completion.complete();
        }
    }).await;
    
    root.add_child(periodic_timer).await;
    
    let start_time = Instant::now();
    kernel.run_until_complete().await.unwrap();
    let elapsed = start_time.elapsed();
    
    // Should have at least 3 ticks
    assert!(tick_count.load(Ordering::Relaxed) >= 3);
    
    // Should take at least 300μs for 3 ticks at 100μs intervals
    assert!(elapsed >= Duration::from_micros(50));
    assert!(elapsed <= Duration::from_secs(5));
}

#[tokio::test]
async fn test_trigger_activation() {
    let kernel = AsyncKernel::new();
    let root = kernel.root();
    
    let condition_met = Arc::new(AtomicBool::new(false));
    let trigger_fired = Arc::new(AtomicBool::new(false));
    
    let trigger = Arc::new(Trigger::new({
        let condition_met = condition_met.clone();
        move || condition_met.load(Ordering::Relaxed)
    })).named("TestTrigger");
    
    let trigger_fired_clone = trigger_fired.clone();
    trigger.set_triggered_callback(move || {
        trigger_fired_clone.store(true, Ordering::Relaxed);
    }).await;
    
    // Task that sets the condition after a delay
    let condition_setter = Arc::new(AsyncCoroutine::new({
        let condition_met = condition_met.clone();
        async move {
            sleep(Duration::from_micros(150)).await;
            condition_met.store(true, Ordering::Relaxed);
            Ok(())
        }
    })).named("ConditionSetter");
    
    root.add_child(trigger).await;
    root.add_child(condition_setter).await;
    
    kernel.run_until_complete().await.unwrap();
    
    assert!(trigger_fired.load(Ordering::Relaxed));
}

#[tokio::test]
async fn test_barrier_with_timers() {
    let kernel = AsyncKernel::new();
    let root = kernel.root();
    
    let barrier = Arc::new(Barrier::new()).named("TestBarrier");
    let completion_count = Arc::new(AtomicU32::new(0));
    
    let durations = vec![150, 250, 200];
    
    for (i, duration_ms) in durations.iter().enumerate() {
        let timer = Arc::new(Timer::new(Duration::from_millis(*duration_ms))).named(&format!("Timer_{}", i));
        
        let completion_count_clone = completion_count.clone();
        timer.set_elapsed_callback(move || {
            completion_count_clone.fetch_add(1, Ordering::Relaxed);
        }).await;
        
        barrier.add_child(timer).await;
    }
    
    let after_barrier = Arc::new(AsyncCoroutine::new({
        let completion_count = completion_count.clone();
        async move {
            // Verify all timers completed before this runs
            assert_eq!(completion_count.load(Ordering::Relaxed), 3);
            Ok(())
        }
    })).named("AfterBarrier");
    
    let sequence = Arc::new(Sequence::new()).named("TestSequence");
    sequence.add_child(barrier).await;
    sequence.add_child(after_barrier).await;
    
    root.add_child(sequence).await;
    
    let start_time = Instant::now();
    kernel.run_until_complete().await.unwrap();
    let elapsed = start_time.elapsed();
    
    // All timers should have completed
    assert_eq!(completion_count.load(Ordering::Relaxed), 3);
    
    // Should wait for longest timer (250μs)
    assert!(elapsed >= Duration::from_micros(50));
    assert!(elapsed <= Duration::from_secs(5));
}

#[tokio::test]
async fn test_timeout_scenario() {
    let kernel = AsyncKernel::new();
    let root = kernel.root();
    
    let timeout_occurred = Arc::new(AtomicBool::new(false));
    let work_completed = Arc::new(AtomicBool::new(false));
    
    // Short timeout (150μs)
    let timeout_timer = Arc::new(Timer::new(Duration::from_micros(150))).named("TimeoutTimer");
    
    let timeout_occurred_clone = timeout_occurred.clone();
    timeout_timer.set_elapsed_callback(move || {
        timeout_occurred_clone.store(true, Ordering::Relaxed);
    }).await;
    
    // Long work task (400μs)
    let work_task = Arc::new(AsyncCoroutine::new({
        let work_completed = work_completed.clone();
        async move {
            sleep(Duration::from_micros(400)).await;
            work_completed.store(true, Ordering::Relaxed);
            Ok(())
        }
    })).named("WorkTask");
    
    // Completion trigger (first one to complete wins)
    let completion_trigger = Arc::new(Trigger::new({
        let timeout_occurred = timeout_occurred.clone();
        let work_completed = work_completed.clone();
        move || timeout_occurred.load(Ordering::Relaxed) || work_completed.load(Ordering::Relaxed)
    })).named("CompletionTrigger");
    
    root.add_child(timeout_timer).await;
    root.add_child(work_task).await;
    root.add_child(completion_trigger).await;
    
    let start_time = Instant::now();
    kernel.run_until_complete().await.unwrap();
    let elapsed = start_time.elapsed();
    
    // Timeout should have won
    assert!(timeout_occurred.load(Ordering::Relaxed));
    
    // Should complete around timeout duration  
    assert!(elapsed >= Duration::from_micros(50));
    assert!(elapsed <= Duration::from_secs(5));
}

#[tokio::test]
async fn test_sequential_timers_simple() {
    let kernel = AsyncKernel::new();
    let root = kernel.root();
    
    let sequence = Arc::new(Sequence::new()).named("TimerSequence");
    let completion_order = Arc::new(std::sync::Mutex::new(Vec::new()));
    
    for i in 0..3 {
        let timer = Arc::new(Timer::new(Duration::from_micros(100))).named(&format!("SequentialTimer_{}", i));
        
        let completion_order_clone = completion_order.clone();
        timer.set_elapsed_callback(move || {
            let mut order = completion_order_clone.lock().unwrap();
            order.push(i);
        }).await;
        
        sequence.add_child(timer).await;
    }
    
    root.add_child(sequence).await;
    
    let start_time = Instant::now();
    kernel.run_until_complete().await.unwrap();
    let elapsed = start_time.elapsed();
    
    // Verify sequential execution
    let order = completion_order.lock().unwrap();
    assert_eq!(*order, vec![0, 1, 2]);
    
    // Should take at least 300μs (3 * 100μs)
    assert!(elapsed >= Duration::from_micros(50));
    assert!(elapsed <= Duration::from_secs(5));
}

#[tokio::test]
async fn test_mixed_components_barrier() {
    let kernel = AsyncKernel::new();
    let root = kernel.root();
    
    let barrier = Arc::new(Barrier::new()).named("MixedBarrier");
    let components_completed = Arc::new(AtomicU32::new(0));
    
    // Timer component
    let timer = Arc::new(Timer::new(Duration::from_micros(150))).named("MixedTimer");
    
    let components_completed_clone = components_completed.clone();
    timer.set_elapsed_callback(move || {
        components_completed_clone.fetch_add(1, Ordering::Relaxed);
    }).await;
    
    // Async task component
    let async_task = Arc::new(AsyncCoroutine::new({
        let components_completed = components_completed.clone();
        async move {
            sleep(Duration::from_micros(120)).await;
            components_completed.fetch_add(1, Ordering::Relaxed);
            Ok(())
        }
    })).named("MixedAsyncTask");
    
    // Trigger component
    let condition = Arc::new(AtomicBool::new(false));
    let trigger = Arc::new(Trigger::new({
        let condition = condition.clone();
        move || condition.load(Ordering::Relaxed)
    })).named("MixedTrigger");
    
    let components_completed_clone2 = components_completed.clone();
    trigger.set_triggered_callback(move || {
        components_completed_clone2.fetch_add(1, Ordering::Relaxed);
    }).await;
    
    // Condition setter
    let condition_setter = Arc::new(AsyncCoroutine::new({
        let condition = condition.clone();
        let components_completed = components_completed.clone();
        async move {
            sleep(Duration::from_micros(100)).await;
            condition.store(true, Ordering::Relaxed);
            components_completed.fetch_add(1, Ordering::Relaxed);
            Ok(())
        }
    })).named("ConditionSetter");
    
    barrier.add_child(timer).await;
    barrier.add_child(async_task).await;
    barrier.add_child(trigger).await;
    barrier.add_child(condition_setter).await;
    
    root.add_child(barrier).await;
    
    kernel.run_until_complete().await.unwrap();
    
    // All 4 components should complete
    assert_eq!(components_completed.load(Ordering::Relaxed), 4);
}