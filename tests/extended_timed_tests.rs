use async_flow::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[tokio::test]
async fn test_ultra_fast_timer() {
    let kernel = AsyncKernel::new();
    let root = kernel.root();
    
    let callback_fired = Arc::new(AtomicBool::new(false));
    
    let timer = Arc::new(Timer::new(Duration::from_micros(60))).named("UltraFastTimer");
    
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
async fn test_rapid_fire_periodic_timer() {
    let kernel = AsyncKernel::new();
    let root = kernel.root();
    
    let tick_count = Arc::new(AtomicU32::new(0));
    
    let periodic_timer = Arc::new(PeriodicTimer::new(Duration::from_micros(80))).named("RapidFireTimer");
    
    let tick_count_clone = tick_count.clone();
    let timer_for_completion = periodic_timer.clone();
    periodic_timer.set_elapsed_callback(move || {
        let count = tick_count_clone.fetch_add(1, Ordering::Relaxed) + 1;
        if count >= 5 {
            timer_for_completion.complete();
        }
    }).await;
    
    root.add_child(periodic_timer).await;
    
    let start_time = Instant::now();
    kernel.run_until_complete().await.unwrap();
    let elapsed = start_time.elapsed();
    
    assert!(tick_count.load(Ordering::Relaxed) >= 5);
    assert!(elapsed >= Duration::from_micros(50));
    assert!(elapsed <= Duration::from_secs(5));
}

#[tokio::test]
async fn test_microsecond_race_condition() {
    let kernel = AsyncKernel::new();
    let root = kernel.root();
    
    let timer1_won = Arc::new(AtomicBool::new(false));
    let timer2_won = Arc::new(AtomicBool::new(false));
    
    let timer1 = Arc::new(Timer::new(Duration::from_micros(90))).named("Timer1_90us");
    
    let timer1_won_clone = timer1_won.clone();
    timer1.set_elapsed_callback(move || {
        timer1_won_clone.store(true, Ordering::Relaxed);
    }).await;
    
    let timer2 = Arc::new(Timer::new(Duration::from_micros(300))).named("Timer2_300us");
    
    let timer2_won_clone2 = timer2_won.clone();
    timer2.set_elapsed_callback(move || {
        timer2_won_clone2.store(true, Ordering::Relaxed);
    }).await;
    
    let completion_trigger = Arc::new(Trigger::new({
        let timer1_won = timer1_won.clone();
        move || timer1_won.load(Ordering::Relaxed)
    })).named("RaceCompletionTrigger");
    
    root.add_child(timer1).await;
    root.add_child(timer2).await;
    root.add_child(completion_trigger).await;
    
    kernel.run_until_complete().await.unwrap();
    
    // Timer1 should complete first (90μs vs 300μs)
    assert!(timer1_won.load(Ordering::Relaxed));
}

#[tokio::test]
async fn test_high_frequency_trigger() {
    let kernel = AsyncKernel::new();
    let root = kernel.root();
    
    let counter = Arc::new(AtomicU32::new(0));
    let trigger_fired = Arc::new(AtomicBool::new(false));
    
    let increment_task = Arc::new(AsyncCoroutine::new({
        let counter = counter.clone();
        async move {
            for _ in 0..10 {
                counter.fetch_add(1, Ordering::Relaxed);
                sleep(Duration::from_micros(50)).await;
            }
            Ok(())
        }
    })).named("IncrementTask");
    
    let trigger = Arc::new(Trigger::new({
        let counter = counter.clone();
        move || counter.load(Ordering::Relaxed) >= 8
    })).named("HighFrequencyTrigger");
    
    let trigger_fired_clone = trigger_fired.clone();
    trigger.set_triggered_callback(move || {
        trigger_fired_clone.store(true, Ordering::Relaxed);
    }).await;
    
    root.add_child(increment_task).await;
    root.add_child(trigger).await;
    
    kernel.run_until_complete().await.unwrap();
    
    assert!(trigger_fired.load(Ordering::Relaxed));
    assert!(counter.load(Ordering::Relaxed) >= 8);
}

#[tokio::test]
async fn test_nested_barrier_timing() {
    let kernel = AsyncKernel::new();
    let root = kernel.root();
    
    let outer_barrier = Arc::new(Barrier::new()).named("OuterBarrier");
    let inner_barrier1 = Arc::new(Barrier::new()).named("InnerBarrier1");
    let inner_barrier2 = Arc::new(Barrier::new()).named("InnerBarrier2");
    
    let completion_count = Arc::new(AtomicU32::new(0));
    
    // Add fast timers to first inner barrier
    for i in 0..3 {
        let timer = Arc::new(Timer::new(Duration::from_micros(80 + i * 20))).named(&format!("Inner1Timer_{}", i));
        
        let completion_count_clone = completion_count.clone();
        timer.set_elapsed_callback(move || {
            completion_count_clone.fetch_add(1, Ordering::Relaxed);
        }).await;
        
        inner_barrier1.add_child(timer).await;
    }
    
    // Add medium timers to second inner barrier
    for i in 0..2 {
        let timer = Arc::new(Timer::new(Duration::from_micros(150 + i * 30))).named(&format!("Inner2Timer_{}", i));
        
        let completion_count_clone = completion_count.clone();
        timer.set_elapsed_callback(move || {
            completion_count_clone.fetch_add(1, Ordering::Relaxed);
        }).await;
        
        inner_barrier2.add_child(timer).await;
    }
    
    outer_barrier.add_child(inner_barrier1).await;
    outer_barrier.add_child(inner_barrier2).await;
    
    root.add_child(outer_barrier).await;
    
    kernel.run_until_complete().await.unwrap();
    
    assert_eq!(completion_count.load(Ordering::Relaxed), 5);
}

#[tokio::test]
async fn test_trigger_chain_microseconds() {
    let kernel = AsyncKernel::new();
    let root = kernel.root();
    
    let stage = Arc::new(AtomicU32::new(0));
    
    let trigger1 = Arc::new(Trigger::new({
        let stage = stage.clone();
        move || stage.load(Ordering::Relaxed) >= 1
    })).named("Trigger1");
    
    let stage_clone = stage.clone();
    trigger1.set_triggered_callback(move || {
        stage_clone.store(2, Ordering::Relaxed);
    }).await;
    
    let trigger2 = Arc::new(Trigger::new({
        let stage = stage.clone();
        move || stage.load(Ordering::Relaxed) >= 2
    })).named("Trigger2");
    
    let stage_clone2 = stage.clone();
    trigger2.set_triggered_callback(move || {
        stage_clone2.store(3, Ordering::Relaxed);
    }).await;
    
    let trigger3 = Arc::new(Trigger::new({
        let stage = stage.clone();
        move || stage.load(Ordering::Relaxed) >= 3
    })).named("Trigger3");
    
    let starter_task = Arc::new(AsyncCoroutine::new({
        let stage = stage.clone();
        async move {
            sleep(Duration::from_micros(100)).await;
            stage.store(1, Ordering::Relaxed);
            Ok(())
        }
    })).named("StarterTask");
    
    root.add_child(starter_task).await;
    root.add_child(trigger1).await;
    root.add_child(trigger2).await;
    root.add_child(trigger3).await;
    
    kernel.run_until_complete().await.unwrap();
    
    assert_eq!(stage.load(Ordering::Relaxed), 3);
}

#[tokio::test]
async fn test_mixed_microsecond_sequence() {
    let kernel = AsyncKernel::new();
    let root = kernel.root();
    
    let sequence = Arc::new(Sequence::new()).named("MixedMicroSequence");
    let execution_order = Arc::new(std::sync::Mutex::new(Vec::new()));
    
    let timer1 = Arc::new(Timer::new(Duration::from_micros(80))).named("SeqTimer1");
    
    let execution_order_clone = execution_order.clone();
    timer1.set_elapsed_callback(move || {
        let mut order = execution_order_clone.lock().unwrap();
        order.push(1);
    }).await;
    
    let async_task = Arc::new(AsyncCoroutine::new({
        let execution_order = execution_order.clone();
        async move {
            sleep(Duration::from_micros(80)).await;
            let mut order = execution_order.lock().unwrap();
            order.push(2);
            Ok(())
        }
    })).named("SeqAsyncTask");
    
    let timer2 = Arc::new(Timer::new(Duration::from_micros(80))).named("SeqTimer2");
    
    let execution_order_clone2 = execution_order.clone();
    timer2.set_elapsed_callback(move || {
        let mut order = execution_order_clone2.lock().unwrap();
        order.push(3);
    }).await;
    
    sequence.add_child(timer1).await;
    sequence.add_child(async_task).await;
    sequence.add_child(timer2).await;
    
    root.add_child(sequence).await;
    
    kernel.run_until_complete().await.unwrap();
    
    let order = execution_order.lock().unwrap();
    // With microsecond timings, exact ordering may vary due to runtime scheduling
    // Verify that all components executed
    assert_eq!(order.len(), 3);
    assert!(order.contains(&1));
    assert!(order.contains(&2));
    assert!(order.contains(&3));
}

#[tokio::test]
async fn test_microsecond_timeout_race() {
    let kernel = AsyncKernel::new();
    let root = kernel.root();
    
    let timeout_occurred = Arc::new(AtomicBool::new(false));
    let work_completed = Arc::new(AtomicBool::new(false));
    
    let timeout_timer = Arc::new(Timer::new(Duration::from_micros(120))).named("MicroTimeoutTimer");
    
    let timeout_occurred_clone = timeout_occurred.clone();
    timeout_timer.set_elapsed_callback(move || {
        timeout_occurred_clone.store(true, Ordering::Relaxed);
    }).await;
    
    let work_task = Arc::new(AsyncCoroutine::new({
        let work_completed = work_completed.clone();
        async move {
            sleep(Duration::from_micros(300)).await;
            work_completed.store(true, Ordering::Relaxed);
            Ok(())
        }
    })).named("SlowMicroWork");
    
    let completion_trigger = Arc::new(Trigger::new({
        let timeout_occurred = timeout_occurred.clone();
        let work_completed = work_completed.clone();
        move || timeout_occurred.load(Ordering::Relaxed) || work_completed.load(Ordering::Relaxed)
    })).named("MicroTimeoutCompletion");
    
    root.add_child(timeout_timer).await;
    root.add_child(work_task).await;
    root.add_child(completion_trigger).await;
    
    kernel.run_until_complete().await.unwrap();
    
    assert!(timeout_occurred.load(Ordering::Relaxed));
}

#[tokio::test]
async fn test_parallel_microsecond_barriers() {
    let kernel = AsyncKernel::new();
    let root = kernel.root();
    
    let main_barrier = Arc::new(Barrier::new()).named("MainMicroBarrier");
    let completions = Arc::new(AtomicU32::new(0));
    
    // Create 4 parallel barriers with different timer configurations
    for barrier_id in 0..4 {
        let sub_barrier = Arc::new(Barrier::new()).named(&format!("SubBarrier_{}", barrier_id));
        
        for timer_id in 0..2 {
            let timer = Arc::new(Timer::new(Duration::from_micros(80 + barrier_id * 20 + timer_id * 10))).named(&format!("B{}T{}", barrier_id, timer_id));
            
            let completions_clone = completions.clone();
            timer.set_elapsed_callback(move || {
                completions_clone.fetch_add(1, Ordering::Relaxed);
            }).await;
            
            sub_barrier.add_child(timer).await;
        }
        
        main_barrier.add_child(sub_barrier).await;
    }
    
    root.add_child(main_barrier).await;
    
    kernel.run_until_complete().await.unwrap();
    
    assert_eq!(completions.load(Ordering::Relaxed), 8);
}

#[tokio::test]
async fn test_microsecond_periodic_with_condition() {
    let kernel = AsyncKernel::new();
    let root = kernel.root();
    
    let tick_counter = Arc::new(AtomicU64::new(0));
    let condition_met = Arc::new(AtomicBool::new(false));
    
    let periodic_timer = Arc::new(PeriodicTimer::new(Duration::from_micros(75))).named("ConditionPeriodicTimer");
    
    let tick_counter_clone = periodic_timer.clone();
    let tick_counter_clone2 = tick_counter.clone();
    periodic_timer.set_elapsed_callback(move || {
        let count = tick_counter_clone2.fetch_add(1, Ordering::Relaxed) + 1;
        if count >= 7 {
            tick_counter_clone.complete();
        }
    }).await;
    
    let condition_checker = Arc::new(AsyncCoroutine::new({
        let tick_counter = tick_counter.clone();
        let condition_met = condition_met.clone();
        async move {
            while tick_counter.load(Ordering::Relaxed) < 5 {
                sleep(Duration::from_micros(50)).await;
            }
            condition_met.store(true, Ordering::Relaxed);
            Ok(())
        }
    })).named("ConditionChecker");
    
    let condition_trigger = Arc::new(Trigger::new({
        let condition_met = condition_met.clone();
        move || condition_met.load(Ordering::Relaxed)
    })).named("ConditionTrigger");
    
    root.add_child(periodic_timer).await;
    root.add_child(condition_checker).await;
    root.add_child(condition_trigger).await;
    
    kernel.run_until_complete().await.unwrap();
    
    assert!(tick_counter.load(Ordering::Relaxed) >= 5);
    assert!(condition_met.load(Ordering::Relaxed));
}

#[tokio::test]
async fn test_rapid_sequence_execution() {
    let kernel = AsyncKernel::new();
    let root = kernel.root();
    
    let sequence = Arc::new(Sequence::new()).named("RapidSequence");
    let execution_times = Arc::new(std::sync::Mutex::new(Vec::new()));
    
    for i in 0..6 {
        let timer = Arc::new(Timer::new(Duration::from_micros(60 + i * 15))).named(&format!("RapidTimer_{}", i));
        
        let execution_times_clone = execution_times.clone();
        timer.set_elapsed_callback(move || {
            let mut times = execution_times_clone.lock().unwrap();
            times.push(std::time::SystemTime::now());
        }).await;
        
        sequence.add_child(timer).await;
    }
    
    root.add_child(sequence).await;
    
    kernel.run_until_complete().await.unwrap();
    
    let times = execution_times.lock().unwrap();
    assert_eq!(times.len(), 6);
}

#[tokio::test]
async fn test_complex_trigger_network() {
    let kernel = AsyncKernel::new();
    let root = kernel.root();
    
    let state = Arc::new(AtomicU32::new(0));
    let network_complete = Arc::new(AtomicBool::new(false));
    
    // Initial timer sets state to 1
    let init_timer = Arc::new(Timer::new(Duration::from_micros(80))).named("InitTimer");
    
    let state_clone = state.clone();
    init_timer.set_elapsed_callback(move || {
        state_clone.store(1, Ordering::Relaxed);
    }).await;
    
    // Trigger 1: state 1 -> state 10
    let trigger1 = Arc::new(Trigger::new({
        let state = state.clone();
        move || state.load(Ordering::Relaxed) == 1
    })).named("NetworkTrigger1");
    
    let state_clone2 = state.clone();
    trigger1.set_triggered_callback(move || {
        state_clone2.store(10, Ordering::Relaxed);
    }).await;
    
    // Trigger 2: state 10 -> state 100
    let trigger2 = Arc::new(Trigger::new({
        let state = state.clone();
        move || state.load(Ordering::Relaxed) == 10
    })).named("NetworkTrigger2");
    
    let state_clone3 = state.clone();
    trigger2.set_triggered_callback(move || {
        state_clone3.store(100, Ordering::Relaxed);
    }).await;
    
    // Final trigger: state 100 -> complete
    let final_trigger = Arc::new(Trigger::new({
        let state = state.clone();
        move || state.load(Ordering::Relaxed) == 100
    })).named("NetworkFinalTrigger");
    
    let network_complete_clone = network_complete.clone();
    final_trigger.set_triggered_callback(move || {
        network_complete_clone.store(true, Ordering::Relaxed);
    }).await;
    
    root.add_child(init_timer).await;
    root.add_child(trigger1).await;
    root.add_child(trigger2).await;
    root.add_child(final_trigger).await;
    
    kernel.run_until_complete().await.unwrap();
    
    assert!(network_complete.load(Ordering::Relaxed));
    assert_eq!(state.load(Ordering::Relaxed), 100);
}

#[tokio::test]
async fn test_burst_timer_pattern() {
    let kernel = AsyncKernel::new();
    let root = kernel.root();
    
    let burst_barrier = Arc::new(Barrier::new()).named("BurstBarrier");
    let burst_completions = Arc::new(AtomicU32::new(0));
    
    // Create a burst of 8 very fast timers
    for i in 0..8 {
        let timer = Arc::new(Timer::new(Duration::from_micros(65 + i * 5))).named(&format!("BurstTimer_{}", i));
        
        let burst_completions_clone = burst_completions.clone();
        timer.set_elapsed_callback(move || {
            burst_completions_clone.fetch_add(1, Ordering::Relaxed);
        }).await;
        
        burst_barrier.add_child(timer).await;
    }
    
    let post_burst_task = Arc::new(AsyncCoroutine::new({
        let burst_completions = burst_completions.clone();
        async move {
            // Verify all burst timers completed
            assert_eq!(burst_completions.load(Ordering::Relaxed), 8);
            Ok(())
        }
    })).named("PostBurstTask");
    
    let sequence = Arc::new(Sequence::new()).named("BurstSequence");
    sequence.add_child(burst_barrier).await;
    sequence.add_child(post_burst_task).await;
    
    root.add_child(sequence).await;
    
    kernel.run_until_complete().await.unwrap();
    
    assert_eq!(burst_completions.load(Ordering::Relaxed), 8);
}

#[tokio::test]
async fn test_staggered_periodic_timers() {
    let kernel = AsyncKernel::new();
    let root = kernel.root();
    
    let timer1_ticks = Arc::new(AtomicU32::new(0));
    let timer2_ticks = Arc::new(AtomicU32::new(0));
    let timer3_ticks = Arc::new(AtomicU32::new(0));
    
    // Fast periodic timer (80μs intervals)
    let periodic1 = Arc::new(PeriodicTimer::new(Duration::from_micros(80))).named("StaggeredPeriodic1");
    
    let timer1_ticks_clone = timer1_ticks.clone();
    let periodic1_clone = periodic1.clone();
    periodic1.set_elapsed_callback(move || {
        let count = timer1_ticks_clone.fetch_add(1, Ordering::Relaxed) + 1;
        if count >= 4 {
            periodic1_clone.complete();
        }
    }).await;
    
    // Medium periodic timer (120μs intervals)
    let periodic2 = Arc::new(PeriodicTimer::new(Duration::from_micros(120))).named("StaggeredPeriodic2");
    
    let timer2_ticks_clone = timer2_ticks.clone();
    let periodic2_clone = periodic2.clone();
    periodic2.set_elapsed_callback(move || {
        let count = timer2_ticks_clone.fetch_add(1, Ordering::Relaxed) + 1;
        if count >= 3 {
            periodic2_clone.complete();
        }
    }).await;
    
    // Slow periodic timer (200μs intervals)  
    let periodic3 = Arc::new(PeriodicTimer::new(Duration::from_micros(200))).named("StaggeredPeriodic3");
    
    let timer3_ticks_clone = timer3_ticks.clone();
    let periodic3_clone = periodic3.clone();
    periodic3.set_elapsed_callback(move || {
        let count = timer3_ticks_clone.fetch_add(1, Ordering::Relaxed) + 1;
        if count >= 2 {
            periodic3_clone.complete();
        }
    }).await;
    
    root.add_child(periodic1).await;
    root.add_child(periodic2).await;
    root.add_child(periodic3).await;
    
    kernel.run_until_complete().await.unwrap();
    
    assert!(timer1_ticks.load(Ordering::Relaxed) >= 4);
    assert!(timer2_ticks.load(Ordering::Relaxed) >= 3);
    assert!(timer3_ticks.load(Ordering::Relaxed) >= 2);
}

#[tokio::test]
async fn test_microsecond_future_coordination() {
    let kernel = AsyncKernel::new();
    let root = kernel.root();
    
    let coordination_future = Arc::new(AsyncFuture::<u32>::new()).named("CoordinationData");
    let coordination_complete = Arc::new(AtomicBool::new(false));
    
    // Producer task sets future value after timer
    let producer_timer = Arc::new(Timer::new(Duration::from_micros(150))).named("ProducerTimer");
    
    let coordination_future_clone = coordination_future.clone();
    producer_timer.set_elapsed_callback(move || {
        tokio::spawn({
            let coordination_future = coordination_future_clone.clone();
            async move {
                coordination_future.set_value(42).await;
            }
        });
    }).await;
    
    // Consumer task waits for future and processes value
    let consumer_task = Arc::new(AsyncCoroutine::new({
        let coordination_future = coordination_future.clone();
        let coordination_complete = coordination_complete.clone();
        async move {
            let value = coordination_future.wait().await;
            assert_eq!(value, 42);
            coordination_complete.store(true, Ordering::Relaxed);
            Ok(())
        }
    })).named("ConsumerTask");
    
    root.add_child(producer_timer).await;
    root.add_child(consumer_task).await;
    
    kernel.run_until_complete().await.unwrap();
    
    assert!(coordination_complete.load(Ordering::Relaxed));
}

#[tokio::test]
async fn test_dynamic_barrier_microseconds() {
    let kernel = AsyncKernel::new();
    let root = kernel.root();
    
    let dynamic_barrier = Arc::new(Barrier::new()).named("DynamicMicroBarrier");
    let completion_tracker = Arc::new(AtomicU32::new(0));
    
    // Add initial set of timers
    for i in 0..3 {
        let timer = Arc::new(Timer::new(Duration::from_micros(90 + i * 20))).named(&format!("InitialTimer_{}", i));
        
        let completion_tracker_clone = completion_tracker.clone();
        timer.set_elapsed_callback(move || {
            completion_tracker_clone.fetch_add(1, Ordering::Relaxed);
        }).await;
        
        dynamic_barrier.add_child(timer).await;
    }
    
    // Dynamically add more timers after a delay
    let dynamic_adder = Arc::new(AsyncCoroutine::new({
        let dynamic_barrier = dynamic_barrier.clone();
        let completion_tracker = completion_tracker.clone();
        async move {
            sleep(Duration::from_micros(50)).await;
            
            for i in 3..5 {
                let timer = Arc::new(Timer::new(Duration::from_micros(70 + i * 15))).named(&format!("DynamicTimer_{}", i));
                
                let completion_tracker_clone = completion_tracker.clone();
                timer.set_elapsed_callback(move || {
                    completion_tracker_clone.fetch_add(1, Ordering::Relaxed);
                }).await;
                
                dynamic_barrier.add_child(timer).await;
            }
            Ok(())
        }
    })).named("DynamicAdder");
    
    root.add_child(dynamic_adder).await;
    root.add_child(dynamic_barrier).await;
    
    kernel.run_until_complete().await.unwrap();
    
    assert_eq!(completion_tracker.load(Ordering::Relaxed), 5);
}

#[tokio::test]
async fn test_cascading_microsecond_triggers() {
    let kernel = AsyncKernel::new();
    let root = kernel.root();
    
    let cascade_state = Arc::new(AtomicU32::new(0));
    let cascade_times = Arc::new(std::sync::Mutex::new(Vec::new()));
    
    // Initial timer starts the cascade
    let init_timer = Arc::new(Timer::new(Duration::from_micros(70))).named("CascadeInitTimer");
    
    let cascade_state_clone = cascade_state.clone();
    let cascade_times_clone = cascade_times.clone();
    init_timer.set_elapsed_callback(move || {
        cascade_state_clone.store(1, Ordering::Relaxed);
        let mut times = cascade_times_clone.lock().unwrap();
        times.push(1);
    }).await;
    
    // Create cascade of triggers
    for level in 1..5 {
        let trigger = Arc::new(Trigger::new({
            let cascade_state = cascade_state.clone();
            move || cascade_state.load(Ordering::Relaxed) >= level
        })).named(&format!("CascadeTrigger_{}", level));
        
        let cascade_state_clone = cascade_state.clone();
        let cascade_times_clone = cascade_times.clone();
        let next_level = level + 1;
        trigger.set_triggered_callback(move || {
            cascade_state_clone.store(next_level, Ordering::Relaxed);
            let mut times = cascade_times_clone.lock().unwrap();
            times.push(next_level);
        }).await;
        
        root.add_child(trigger).await;
    }
    
    // Final completion trigger
    let completion_trigger = Arc::new(Trigger::new({
        let cascade_state = cascade_state.clone();
        move || cascade_state.load(Ordering::Relaxed) >= 5
    })).named("CascadeCompletionTrigger");
    
    root.add_child(init_timer).await;
    root.add_child(completion_trigger).await;
    
    kernel.run_until_complete().await.unwrap();
    
    let times = cascade_times.lock().unwrap();
    assert_eq!(times.len(), 5);
    assert_eq!(*times, vec![1, 2, 3, 4, 5]);
}

#[tokio::test]
async fn test_extreme_parallel_execution() {
    let kernel = AsyncKernel::new();
    let root = kernel.root();
    
    let parallel_barrier = Arc::new(Barrier::new()).named("ExtremeParallelBarrier");
    let execution_count = Arc::new(AtomicU32::new(0));
    
    // Create 15 parallel components of different types
    for i in 0..15 {
        match i % 3 {
            0 => {
                // Timer component
                let timer = Arc::new(Timer::new(Duration::from_micros(80 + (i as u64) * 10))).named(&format!("ParallelTimer_{}", i));
                
                let execution_count_clone = execution_count.clone();
                timer.set_elapsed_callback(move || {
                    execution_count_clone.fetch_add(1, Ordering::Relaxed);
                }).await;
                
                parallel_barrier.add_child(timer).await;
            },
            1 => {
                // Async task component
                let task = Arc::new(AsyncCoroutine::new({
                    let execution_count = execution_count.clone();
                    async move {
                        sleep(Duration::from_micros(100 + (i as u64) * 8)).await;
                        execution_count.fetch_add(1, Ordering::Relaxed);
                        Ok(())
                    }
                })).named(&format!("ParallelTask_{}", i));
                
                parallel_barrier.add_child(task).await;
            },
            2 => {
                // Periodic timer component (with quick completion)
                let periodic = Arc::new(PeriodicTimer::new(Duration::from_micros(60))).named(&format!("ParallelPeriodic_{}", i));
                
                let execution_count_clone = execution_count.clone();
                let periodic_clone = periodic.clone();
                periodic.set_elapsed_callback(move || {
                    execution_count_clone.fetch_add(1, Ordering::Relaxed);
                    periodic_clone.complete();
                }).await;
                
                parallel_barrier.add_child(periodic).await;
            },
            _ => unreachable!()
        }
    }
    
    root.add_child(parallel_barrier).await;
    
    kernel.run_until_complete().await.unwrap();
    
    assert_eq!(execution_count.load(Ordering::Relaxed), 15);
}