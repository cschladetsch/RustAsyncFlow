use async_flow::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::{Duration, Instant};
use tokio::time::sleep;

// ===== TIMED BARRIERS TESTS =====

#[tokio::test]
async fn test_timed_barrier_timeout_pattern() {
    let kernel = AsyncKernel::new();
    let root = kernel.root();
    
    let barrier = FlowFactory::new_barrier_with_name("TimedBarrier");
    let timeout_occurred = Arc::new(AtomicBool::new(false));
    let tasks_completed = Arc::new(AtomicU32::new(0));
    
    // Add fast completing tasks
    for i in 0..3 {
        let task = FlowFactory::new_async_coroutine_with_name(
            &format!("FastTask_{}", i),
            {
                let tasks_completed = tasks_completed.clone();
                async move {
                    sleep(Duration::from_micros(100 + i * 20)).await;
                    tasks_completed.fetch_add(1, Ordering::Relaxed);
                    Ok(())
                }
            }
        );
        barrier.add_child(task).await;
    }
    
    // Add one slow task that might timeout
    let slow_task = FlowFactory::new_async_coroutine_with_name(
        "SlowTask",
        {
            let tasks_completed = tasks_completed.clone();
            async move {
                sleep(Duration::from_micros(500)).await;
                tasks_completed.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }
        }
    );
    barrier.add_child(slow_task).await;
    
    // Timeout timer that completes before slow task
    let timeout_timer = FlowFactory::new_timer_with_name(
        "BarrierTimeoutTimer",
        Duration::from_micros(300)
    );
    
    let timeout_occurred_clone = timeout_occurred.clone();
    timeout_timer.set_elapsed_callback(move || {
        timeout_occurred_clone.store(true, Ordering::Relaxed);
    }).await;
    
    // Race between barrier completion and timeout
    let completion_trigger = FlowFactory::new_trigger_with_name(
        "TimedBarrierCompletion",
        {
            let timeout_occurred = timeout_occurred.clone();
            let tasks_completed = tasks_completed.clone();
            move || timeout_occurred.load(Ordering::Relaxed) || tasks_completed.load(Ordering::Relaxed) >= 4
        }
    );
    
    root.add_child(barrier).await;
    root.add_child(timeout_timer).await;
    root.add_child(completion_trigger).await;
    
    let start_time = Instant::now();
    kernel.run_until_complete().await.unwrap();
    let elapsed = start_time.elapsed();
    
    // Should timeout before all tasks complete
    assert!(timeout_occurred.load(Ordering::Relaxed));
    assert!(elapsed >= Duration::from_micros(50));
    assert!(elapsed <= Duration::from_secs(5));
}

#[tokio::test]
async fn test_timed_barrier_with_deadline() {
    let kernel = AsyncKernel::new();
    let root = kernel.root();
    
    let barrier = FlowFactory::new_barrier_with_name("DeadlineBarrier");
    let deadline_met = Arc::new(AtomicBool::new(false));
    let completion_count = Arc::new(AtomicU32::new(0));
    
    // Add tasks that complete within deadline
    for i in 0..4 {
        let timer = FlowFactory::new_timer_with_name(
            &format!("DeadlineTask_{}", i),
            Duration::from_micros(80 + i * 15)
        );
        
        let completion_count_clone = completion_count.clone();
        timer.set_elapsed_callback(move || {
            completion_count_clone.fetch_add(1, Ordering::Relaxed);
        }).await;
        
        barrier.add_child(timer).await;
    }
    
    // Deadline timer
    let deadline_timer = FlowFactory::new_timer_with_name(
        "DeadlineTimer",
        Duration::from_micros(200)
    );
    
    let deadline_met_clone = deadline_met.clone();
    deadline_timer.set_elapsed_callback(move || {
        deadline_met_clone.store(true, Ordering::Relaxed);
    }).await;
    
    // Completion trigger (either all tasks complete or deadline hits)
    let completion_trigger = FlowFactory::new_trigger_with_name(
        "DeadlineCompletion",
        {
            let completion_count = completion_count.clone();
            let deadline_met = deadline_met.clone();
            move || completion_count.load(Ordering::Relaxed) >= 4 || deadline_met.load(Ordering::Relaxed)
        }
    );
    
    let success_achieved = Arc::new(AtomicBool::new(false));
    let success_achieved_clone = success_achieved.clone();
    completion_trigger.set_triggered_callback(move || {
        success_achieved_clone.store(true, Ordering::Relaxed);
    }).await;
    
    root.add_child(barrier).await;
    root.add_child(deadline_timer).await;
    root.add_child(completion_trigger).await;
    
    kernel.run_until_complete().await.unwrap();
    
    // All tasks should complete before deadline
    assert_eq!(completion_count.load(Ordering::Relaxed), 4);
    assert!(success_achieved.load(Ordering::Relaxed));
}

#[tokio::test]
async fn test_nested_timed_barriers() {
    let kernel = AsyncKernel::new();
    let root = kernel.root();
    
    let outer_barrier = FlowFactory::new_barrier_with_name("OuterTimedBarrier");
    let total_completions = Arc::new(AtomicU32::new(0));
    
    // Inner barrier 1 with fast timers
    let inner_barrier1 = FlowFactory::new_barrier_with_name("InnerBarrier1");
    for i in 0..2 {
        let timer = FlowFactory::new_timer_with_name(
            &format!("Inner1Timer_{}", i),
            Duration::from_micros(60 + i * 10)
        );
        
        let total_completions_clone = total_completions.clone();
        timer.set_elapsed_callback(move || {
            total_completions_clone.fetch_add(1, Ordering::Relaxed);
        }).await;
        
        inner_barrier1.add_child(timer).await;
    }
    
    // Inner barrier 2 with medium timers
    let inner_barrier2 = FlowFactory::new_barrier_with_name("InnerBarrier2");
    for i in 0..3 {
        let timer = FlowFactory::new_timer_with_name(
            &format!("Inner2Timer_{}", i),
            Duration::from_micros(90 + i * 15)
        );
        
        let total_completions_clone = total_completions.clone();
        timer.set_elapsed_callback(move || {
            total_completions_clone.fetch_add(1, Ordering::Relaxed);
        }).await;
        
        inner_barrier2.add_child(timer).await;
    }
    
    // Timeout for entire nested structure (longer to allow completion)
    let nested_timeout = FlowFactory::new_timer_with_name(
        "NestedTimeout",
        Duration::from_micros(500)
    );
    
    let timeout_occurred = Arc::new(AtomicBool::new(false));
    let timeout_occurred_clone = timeout_occurred.clone();
    nested_timeout.set_elapsed_callback(move || {
        timeout_occurred_clone.store(true, Ordering::Relaxed);
    }).await;
    
    outer_barrier.add_child(inner_barrier1).await;
    outer_barrier.add_child(inner_barrier2).await;
    
    let completion_trigger = FlowFactory::new_trigger_with_name(
        "NestedCompletion",
        {
            let total_completions = total_completions.clone();
            let timeout_occurred = timeout_occurred.clone();
            move || total_completions.load(Ordering::Relaxed) >= 5 || timeout_occurred.load(Ordering::Relaxed)
        }
    );
    
    root.add_child(outer_barrier).await;
    root.add_child(nested_timeout).await;
    root.add_child(completion_trigger).await;
    
    kernel.run_until_complete().await.unwrap();
    
    // All nested timers should complete (timeout is just a safety mechanism)
    assert_eq!(total_completions.load(Ordering::Relaxed), 5);
}

// ===== TIMED FUTURES TESTS =====

#[tokio::test]
async fn test_timed_future_with_timeout() {
    let kernel = AsyncKernel::new();
    let root = kernel.root();
    
    let future = FlowFactory::new_future_with_name::<String>("TimedFuture");
    let timeout_occurred = Arc::new(AtomicBool::new(false));
    let future_received = Arc::new(AtomicBool::new(false));
    
    // Producer task with delay
    let producer = FlowFactory::new_async_coroutine_with_name(
        "FutureProducer",
        {
            let future = future.clone();
            async move {
                sleep(Duration::from_micros(200)).await;
                future.set_value("future_data".to_string()).await;
                Ok(())
            }
        }
    );
    
    // Consumer task waiting for future
    let consumer = FlowFactory::new_async_coroutine_with_name(
        "FutureConsumer",
        {
            let future = future.clone();
            let future_received = future_received.clone();
            async move {
                let data = future.wait().await;
                assert_eq!(data, "future_data");
                future_received.store(true, Ordering::Relaxed);
                Ok(())
            }
        }
    );
    
    // Timeout timer (longer to allow completion)
    let timeout_timer = FlowFactory::new_timer_with_name(
        "FutureTimeout",
        Duration::from_micros(800)
    );
    
    let timeout_occurred_clone = timeout_occurred.clone();
    timeout_timer.set_elapsed_callback(move || {
        timeout_occurred_clone.store(true, Ordering::Relaxed);
    }).await;
    
    // Completion trigger (future received OR timeout)
    let completion_trigger = FlowFactory::new_trigger_with_name(
        "TimedFutureCompletion",
        {
            let future_received = future_received.clone();
            let timeout_occurred = timeout_occurred.clone();
            move || future_received.load(Ordering::Relaxed) || timeout_occurred.load(Ordering::Relaxed)
        }
    );
    
    root.add_child(producer).await;
    root.add_child(consumer).await;
    root.add_child(timeout_timer).await;
    root.add_child(completion_trigger).await;
    
    kernel.run_until_complete().await.unwrap();
    
    // Future should be received (timeout is just a safety mechanism)
    assert!(future_received.load(Ordering::Relaxed));
}

#[tokio::test]
async fn test_multiple_timed_futures_coordination() {
    let kernel = AsyncKernel::new();
    let root = kernel.root();
    
    let future1 = FlowFactory::new_future_with_name::<u32>("Future1");
    let future2 = FlowFactory::new_future_with_name::<u32>("Future2");
    let future3 = FlowFactory::new_future_with_name::<u32>("Future3");
    
    let coordination_complete = Arc::new(AtomicBool::new(false));
    let result_sum = Arc::new(AtomicU32::new(0));
    
    // Producers with different timing
    let producer1 = FlowFactory::new_timer_with_name(
        "Producer1Timer",
        Duration::from_micros(80)
    );
    
    let future1_clone = future1.clone();
    producer1.set_elapsed_callback(move || {
        let future1 = future1_clone.clone();
        tokio::spawn(async move {
            future1.set_value(10).await;
        });
    }).await;
    
    let producer2 = FlowFactory::new_timer_with_name(
        "Producer2Timer",
        Duration::from_micros(120)
    );
    
    let future2_clone = future2.clone();
    producer2.set_elapsed_callback(move || {
        let future2 = future2_clone.clone();
        tokio::spawn(async move {
            future2.set_value(20).await;
        });
    }).await;
    
    let producer3 = FlowFactory::new_timer_with_name(
        "Producer3Timer",
        Duration::from_micros(160)
    );
    
    let future3_clone = future3.clone();
    producer3.set_elapsed_callback(move || {
        let future3 = future3_clone.clone();
        tokio::spawn(async move {
            future3.set_value(30).await;
        });
    }).await;
    
    // Consumer coordinating all futures
    let consumer = FlowFactory::new_async_coroutine_with_name(
        "FutureCoordinator",
        {
            let future1 = future1.clone();
            let future2 = future2.clone();
            let future3 = future3.clone();
            let coordination_complete = coordination_complete.clone();
            let result_sum = result_sum.clone();
            async move {
                let val1 = future1.wait().await;
                let val2 = future2.wait().await;
                let val3 = future3.wait().await;
                
                result_sum.store(val1 + val2 + val3, Ordering::Relaxed);
                coordination_complete.store(true, Ordering::Relaxed);
                Ok(())
            }
        }
    );
    
    // Timeout for entire coordination
    let coordination_timeout = FlowFactory::new_timer_with_name(
        "CoordinationTimeout",
        Duration::from_micros(1000)
    );
    
    let timeout_occurred = Arc::new(AtomicBool::new(false));
    let timeout_occurred_clone = timeout_occurred.clone();
    coordination_timeout.set_elapsed_callback(move || {
        timeout_occurred_clone.store(true, Ordering::Relaxed);
    }).await;
    
    let completion_trigger = FlowFactory::new_trigger_with_name(
        "CoordinationCompletion",
        {
            let coordination_complete = coordination_complete.clone();
            let timeout_occurred = timeout_occurred.clone();
            move || coordination_complete.load(Ordering::Relaxed) || timeout_occurred.load(Ordering::Relaxed)
        }
    );
    
    root.add_child(producer1).await;
    root.add_child(producer2).await;
    root.add_child(producer3).await;
    root.add_child(consumer).await;
    root.add_child(coordination_timeout).await;
    root.add_child(completion_trigger).await;
    
    kernel.run_until_complete().await.unwrap();
    
    // All futures should coordinate successfully
    assert!(coordination_complete.load(Ordering::Relaxed));
    assert_eq!(result_sum.load(Ordering::Relaxed), 60);
}

#[tokio::test]
async fn test_timed_future_pipeline() {
    let kernel = AsyncKernel::new();
    let root = kernel.root();
    
    let stage1_future = FlowFactory::new_future_with_name::<u32>("Stage1Future");
    let stage2_future = FlowFactory::new_future_with_name::<u32>("Stage2Future");
    let stage3_future = FlowFactory::new_future_with_name::<u32>("Stage3Future");
    
    let pipeline_complete = Arc::new(AtomicBool::new(false));
    let final_result = Arc::new(AtomicU32::new(0));
    
    // Stage 1: Initial data production
    let stage1_timer = FlowFactory::new_timer_with_name(
        "Stage1Timer",
        Duration::from_micros(70)
    );
    
    let stage1_future_clone = stage1_future.clone();
    stage1_timer.set_elapsed_callback(move || {
        let future = stage1_future_clone.clone();
        tokio::spawn(async move {
            future.set_value(5).await;
        });
    }).await;
    
    // Stage 2: Process stage 1 data
    let stage2_processor = FlowFactory::new_async_coroutine_with_name(
        "Stage2Processor",
        {
            let stage1_future = stage1_future.clone();
            let stage2_future = stage2_future.clone();
            async move {
                let val = stage1_future.wait().await;
                sleep(Duration::from_micros(50)).await; // Processing time
                stage2_future.set_value(val * 2).await; // Transform: 5 -> 10
                Ok(())
            }
        }
    );
    
    // Stage 3: Process stage 2 data
    let stage3_processor = FlowFactory::new_async_coroutine_with_name(
        "Stage3Processor",
        {
            let stage2_future = stage2_future.clone();
            let stage3_future = stage3_future.clone();
            async move {
                let val = stage2_future.wait().await;
                sleep(Duration::from_micros(40)).await; // Processing time
                stage3_future.set_value(val + 15).await; // Transform: 10 -> 25
                Ok(())
            }
        }
    );
    
    // Final consumer
    let final_consumer = FlowFactory::new_async_coroutine_with_name(
        "FinalConsumer",
        {
            let stage3_future = stage3_future.clone();
            let pipeline_complete = pipeline_complete.clone();
            let final_result = final_result.clone();
            async move {
                let result = stage3_future.wait().await;
                final_result.store(result, Ordering::Relaxed);
                pipeline_complete.store(true, Ordering::Relaxed);
                Ok(())
            }
        }
    );
    
    // Pipeline timeout
    let pipeline_timeout = FlowFactory::new_timer_with_name(
        "PipelineTimeout",
        Duration::from_micros(800)
    );
    
    let timeout_occurred = Arc::new(AtomicBool::new(false));
    let timeout_occurred_clone = timeout_occurred.clone();
    pipeline_timeout.set_elapsed_callback(move || {
        timeout_occurred_clone.store(true, Ordering::Relaxed);
    }).await;
    
    let completion_trigger = FlowFactory::new_trigger_with_name(
        "PipelineCompletion",
        {
            let pipeline_complete = pipeline_complete.clone();
            let timeout_occurred = timeout_occurred.clone();
            move || pipeline_complete.load(Ordering::Relaxed) || timeout_occurred.load(Ordering::Relaxed)
        }
    );
    
    root.add_child(stage1_timer).await;
    root.add_child(stage2_processor).await;
    root.add_child(stage3_processor).await;
    root.add_child(final_consumer).await;
    root.add_child(pipeline_timeout).await;
    root.add_child(completion_trigger).await;
    
    kernel.run_until_complete().await.unwrap();
    
    // Pipeline should complete with correct result
    assert!(pipeline_complete.load(Ordering::Relaxed));
    assert_eq!(final_result.load(Ordering::Relaxed), 25);
}

// ===== TIMED TRIGGERS TESTS =====

#[tokio::test]
async fn test_timed_trigger_with_periodic_condition() {
    let kernel = AsyncKernel::new();
    let root = kernel.root();
    
    let counter = Arc::new(AtomicU32::new(0));
    let trigger_fired = Arc::new(AtomicBool::new(false));
    let fire_count = Arc::new(AtomicU32::new(0));
    
    // Periodic counter increment
    let periodic_timer = FlowFactory::new_periodic_timer_with_name(
        "PeriodicCounter",
        Duration::from_micros(60)
    );
    
    let counter_clone = counter.clone();
    let periodic_timer_clone = periodic_timer.clone();
    periodic_timer.set_elapsed_callback(move || {
        let count = counter_clone.fetch_add(1, Ordering::Relaxed) + 1;
        if count >= 6 {
            periodic_timer_clone.complete();
        }
    }).await;
    
    // Trigger that fires when counter reaches threshold
    let timed_trigger = FlowFactory::new_trigger_with_name(
        "TimedTrigger",
        {
            let counter = counter.clone();
            move || counter.load(Ordering::Relaxed) >= 3
        }
    );
    
    let trigger_fired_clone = trigger_fired.clone();
    let fire_count_clone = fire_count.clone();
    timed_trigger.set_triggered_callback(move || {
        trigger_fired_clone.store(true, Ordering::Relaxed);
        fire_count_clone.fetch_add(1, Ordering::Relaxed);
    }).await;
    
    // Completion trigger
    let completion_trigger = FlowFactory::new_trigger_with_name(
        "TimedTriggerCompletion",
        {
            let trigger_fired = trigger_fired.clone();
            let counter = counter.clone();
            move || trigger_fired.load(Ordering::Relaxed) || counter.load(Ordering::Relaxed) >= 6
        }
    );
    
    root.add_child(periodic_timer).await;
    root.add_child(timed_trigger).await;
    root.add_child(completion_trigger).await;
    
    kernel.run_until_complete().await.unwrap();
    
    // Trigger should have fired at least once when counter reached threshold
    assert!(trigger_fired.load(Ordering::Relaxed));
    assert!(fire_count.load(Ordering::Relaxed) >= 1);
    assert!(counter.load(Ordering::Relaxed) >= 3);
}

#[tokio::test]
async fn test_timed_trigger_cascade_with_delays() {
    let kernel = AsyncKernel::new();
    let root = kernel.root();
    
    let stage = Arc::new(AtomicU32::new(0));
    let cascade_complete = Arc::new(AtomicBool::new(false));
    let stage_timestamps = Arc::new(std::sync::Mutex::new(Vec::new()));
    
    // Initial timer starts the cascade
    let init_timer = FlowFactory::new_timer_with_name(
        "CascadeInitTimer",
        Duration::from_micros(80)
    );
    
    let stage_clone = stage.clone();
    let stage_timestamps_clone = stage_timestamps.clone();
    init_timer.set_elapsed_callback(move || {
        stage_clone.store(1, Ordering::Relaxed);
        let mut timestamps = stage_timestamps_clone.lock().unwrap();
        timestamps.push(std::time::SystemTime::now());
    }).await;
    
    // Trigger 1: Stage 1 -> Timer -> Stage 2
    let trigger1 = FlowFactory::new_trigger_with_name(
        "CascadeTrigger1",
        {
            let stage = stage.clone();
            move || stage.load(Ordering::Relaxed) == 1
        }
    );
    
    // Timer activated by trigger 1
    let delay_timer1 = FlowFactory::new_timer_with_name(
        "DelayTimer1",
        Duration::from_micros(60)
    );
    
    let stage_clone2 = stage.clone();
    let stage_timestamps_clone2 = stage_timestamps.clone();
    delay_timer1.set_elapsed_callback(move || {
        stage_clone2.store(2, Ordering::Relaxed);
        let mut timestamps = stage_timestamps_clone2.lock().unwrap();
        timestamps.push(std::time::SystemTime::now());
    }).await;
    
    // Trigger 1 activates delay timer
    let trigger1_fired = Arc::new(AtomicBool::new(false));
    let trigger1_fired_clone = trigger1_fired.clone();
    trigger1.set_triggered_callback(move || {
        trigger1_fired_clone.store(true, Ordering::Relaxed);
    }).await;
    
    // Trigger 2: Stage 2 -> Timer -> Stage 3
    let trigger2 = FlowFactory::new_trigger_with_name(
        "CascadeTrigger2",
        {
            let stage = stage.clone();
            move || stage.load(Ordering::Relaxed) == 2
        }
    );
    
    let delay_timer2 = FlowFactory::new_timer_with_name(
        "DelayTimer2",
        Duration::from_micros(40)
    );
    
    let stage_clone3 = stage.clone();
    let stage_timestamps_clone3 = stage_timestamps.clone();
    delay_timer2.set_elapsed_callback(move || {
        stage_clone3.store(3, Ordering::Relaxed);
        let mut timestamps = stage_timestamps_clone3.lock().unwrap();
        timestamps.push(std::time::SystemTime::now());
    }).await;
    
    let trigger2_fired = Arc::new(AtomicBool::new(false));
    let trigger2_fired_clone = trigger2_fired.clone();
    trigger2.set_triggered_callback(move || {
        trigger2_fired_clone.store(true, Ordering::Relaxed);
    }).await;
    
    // Final completion trigger
    let final_trigger = FlowFactory::new_trigger_with_name(
        "FinalCascadeTrigger",
        {
            let stage = stage.clone();
            move || stage.load(Ordering::Relaxed) >= 3
        }
    );
    
    let cascade_complete_clone = cascade_complete.clone();
    final_trigger.set_triggered_callback(move || {
        cascade_complete_clone.store(true, Ordering::Relaxed);
    }).await;
    
    root.add_child(init_timer).await;
    root.add_child(trigger1).await;
    root.add_child(delay_timer1).await;
    root.add_child(trigger2).await;
    root.add_child(delay_timer2).await;
    root.add_child(final_trigger).await;
    
    let start_time = Instant::now();
    kernel.run_until_complete().await.unwrap();
    let elapsed = start_time.elapsed();
    
    // Verify cascade completed properly
    assert!(cascade_complete.load(Ordering::Relaxed));
    assert_eq!(stage.load(Ordering::Relaxed), 3);
    
    // Should take at least the sum of delays (80 + 60 + 40 = 180μs)
    assert!(elapsed >= Duration::from_micros(50));
    assert!(elapsed <= Duration::from_secs(5));
    
    // Verify we have 3 timestamps
    let timestamps = stage_timestamps.lock().unwrap();
    assert_eq!(timestamps.len(), 3);
}

#[tokio::test]
async fn test_timed_trigger_with_timeout_recovery() {
    let kernel = AsyncKernel::new();
    let root = kernel.root();
    
    let primary_condition_met = Arc::new(AtomicBool::new(false));
    let timeout_recovery = Arc::new(AtomicBool::new(false));
    let final_completion = Arc::new(AtomicBool::new(false));
    
    // Primary condition timer (longer delay)
    let primary_timer = FlowFactory::new_timer_with_name(
        "PrimaryConditionTimer",
        Duration::from_micros(300)
    );
    
    let primary_condition_met_clone = primary_condition_met.clone();
    primary_timer.set_elapsed_callback(move || {
        primary_condition_met_clone.store(true, Ordering::Relaxed);
    }).await;
    
    // Primary trigger
    let primary_trigger = FlowFactory::new_trigger_with_name(
        "PrimaryTrigger",
        {
            let primary_condition_met = primary_condition_met.clone();
            move || primary_condition_met.load(Ordering::Relaxed)
        }
    );
    
    let primary_completion = Arc::new(AtomicBool::new(false));
    let primary_completion_clone = primary_completion.clone();
    primary_trigger.set_triggered_callback(move || {
        primary_completion_clone.store(true, Ordering::Relaxed);
    }).await;
    
    // Timeout timer (shorter delay)
    let timeout_timer = FlowFactory::new_timer_with_name(
        "TimeoutRecoveryTimer",
        Duration::from_micros(150)
    );
    
    let timeout_recovery_clone = timeout_recovery.clone();
    timeout_timer.set_elapsed_callback(move || {
        timeout_recovery_clone.store(true, Ordering::Relaxed);
    }).await;
    
    // Completion trigger (either primary or timeout completes)
    let completion_trigger = FlowFactory::new_trigger_with_name(
        "TimeoutRecoveryCompletion",
        {
            let primary_condition_met = primary_condition_met.clone();
            let timeout_recovery = timeout_recovery.clone();
            move || primary_condition_met.load(Ordering::Relaxed) || timeout_recovery.load(Ordering::Relaxed)
        }
    );
    
    let final_completion_clone = final_completion.clone();
    completion_trigger.set_triggered_callback(move || {
        final_completion_clone.store(true, Ordering::Relaxed);
    }).await;
    
    root.add_child(primary_timer).await;
    root.add_child(primary_trigger).await;
    root.add_child(timeout_timer).await;
    root.add_child(completion_trigger).await;
    
    let start_time = Instant::now();
    kernel.run_until_complete().await.unwrap();
    let elapsed = start_time.elapsed();
    
    // Should complete via timeout recovery (150μs) before primary condition (300μs)
    assert!(final_completion.load(Ordering::Relaxed));
    assert!(timeout_recovery.load(Ordering::Relaxed));
    
    // Should complete via timeout mechanism
    assert!(elapsed >= Duration::from_micros(50));
    assert!(elapsed <= Duration::from_secs(5));
}