use async_flow::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_basic_kernel_operations() {
    let kernel = AsyncKernel::new();
    
    assert!(kernel.is_active());
    assert!(kernel.is_running());
    assert!(!kernel.is_completed());
    assert!(!kernel.is_breaking().await);
}

#[tokio::test]
async fn test_node_child_management() {
    let node = Arc::new(Node::new()).named("TestNode");
    
    assert_eq!(node.child_count().await, 0);
    
    let child = Arc::new(Node::new()).named("Child");
    node.add_child(child.clone()).await;
    
    assert_eq!(node.child_count().await, 1);
    
    node.remove_child(child.id()).await;
    assert_eq!(node.child_count().await, 0);
}

#[tokio::test]
async fn test_sequence_execution() {
    let sequence = Arc::new(Sequence::new()).named("TestSequence");
    let executed_order = Arc::new(tokio::sync::Mutex::new(Vec::<u32>::new()));
    
    for i in 1..=3 {
        let order = executed_order.clone();
        let task = Arc::new(AsyncCoroutine::new(async move {
            let mut order = order.lock().await;
            order.push(i);
            Ok(())
        })).named(format!("Task{}", i));
        sequence.add_child(task).await;
    }
    
    let kernel = AsyncKernel::new();
    kernel.root().add_child(sequence.clone()).await;
    
    kernel.run_until_complete().await.unwrap();
    
    let order = executed_order.lock().await;
    assert_eq!(*order, vec![1, 2, 3]);
}

#[tokio::test]
async fn test_barrier_execution() {
    let barrier = Arc::new(Barrier::new()).named("TestBarrier");
    let completed_tasks = Arc::new(AtomicU32::new(0));
    
    for i in 1..=3 {
        let completed = completed_tasks.clone();
        let delay = i * 50; // Different delays to test concurrent execution
        let task = Arc::new(AsyncCoroutine::new(async move {
            sleep(Duration::from_millis(delay)).await;
            completed.fetch_add(1, Ordering::Relaxed);
            Ok(())
        })).named(format!("Task{}", i));
        barrier.add_child(task).await;
    }
    
    let kernel = AsyncKernel::new();
    kernel.root().add_child(barrier.clone()).await;
    
    kernel.run_until_complete().await.unwrap();
    
    assert_eq!(completed_tasks.load(Ordering::Relaxed), 3);
}

#[tokio::test]
async fn test_timer_functionality() {
    let timer = Arc::new(Timer::new(Duration::from_millis(100))).named("TestTimer");
    let elapsed = Arc::new(AtomicBool::new(false));
    
    timer.set_elapsed_callback({
        let elapsed = elapsed.clone();
        move || {
            elapsed.store(true, Ordering::Relaxed);
        }
    }).await;
    
    let kernel = AsyncKernel::new();
    kernel.root().add_child(timer.clone()).await;
    
    kernel.run_for(Duration::from_millis(200)).await.unwrap();
    
    assert!(elapsed.load(Ordering::Relaxed));
    assert!(timer.is_completed());
}

#[tokio::test]
async fn test_periodic_timer_functionality() {
    let timer = Arc::new(PeriodicTimer::new(Duration::from_millis(50))).named("TestPeriodicTimer");
    let tick_count = Arc::new(AtomicU32::new(0));
    
    timer.set_elapsed_callback({
        let tick_count = tick_count.clone();
        move || {
            tick_count.fetch_add(1, Ordering::Relaxed);
        }
    }).await;
    
    let kernel = AsyncKernel::new();
    kernel.root().add_child(timer.clone()).await;
    
    kernel.run_for(Duration::from_millis(250)).await.unwrap();
    
    let final_count = tick_count.load(Ordering::Relaxed);
    assert!(final_count >= 4 && final_count <= 6); // Should tick ~5 times in 250ms
}

#[tokio::test]
async fn test_trigger_functionality() {
    let condition_met = Arc::new(AtomicBool::new(false));
    let trigger_fired = Arc::new(AtomicBool::new(false));
    
    let trigger = Arc::new(Trigger::new({
        let condition_met = condition_met.clone();
        move || condition_met.load(Ordering::Relaxed)
    })).named("TestTrigger");
    
    trigger.set_triggered_callback({
        let trigger_fired = trigger_fired.clone();
        move || {
            trigger_fired.store(true, Ordering::Relaxed);
        }
    }).await;
    
    let kernel = AsyncKernel::new();
    kernel.root().add_child(trigger.clone()).await;
    
    // Run for a bit without condition met
    kernel.run_for(Duration::from_millis(50)).await.unwrap();
    assert!(!trigger_fired.load(Ordering::Relaxed));
    
    // Set condition and run again
    condition_met.store(true, Ordering::Relaxed);
    kernel.run_for(Duration::from_millis(50)).await.unwrap();
    
    assert!(trigger_fired.load(Ordering::Relaxed));
    assert!(trigger.is_completed());
}

#[tokio::test]
async fn test_future_functionality() {
    let future = Arc::new(AsyncFuture::<String>::new()).named("TestFuture");
    let result = Arc::new(tokio::sync::Mutex::new(String::new()));
    
    let producer = Arc::new(AsyncCoroutine::new({
        let future = future.clone();
        async move {
            sleep(Duration::from_millis(100)).await;
            future.set_value("Hello Future!".to_string()).await;
            Ok(())
        }
    })).named("Producer");
    
    let consumer = Arc::new(AsyncCoroutine::new({
        let future = future.clone();
        let result = result.clone();
        async move {
            let value = future.wait().await;
            let mut result = result.lock().await;
            *result = value;
            Ok(())
        }
    })).named("Consumer");
    
    let barrier = Arc::new(Barrier::new()).named("FutureBarrier");
    barrier.add_child(producer).await;
    barrier.add_child(consumer).await;
    
    let kernel = AsyncKernel::new();
    kernel.root().add_child(barrier).await;
    
    kernel.run_until_complete().await.unwrap();
    
    let final_result = result.lock().await;
    assert_eq!(*final_result, "Hello Future!");
}

#[tokio::test]
async fn test_kernel_break_functionality() {
    let kernel = AsyncKernel::new();
    let break_triggered = Arc::new(AtomicBool::new(false));
    
    let long_running_task = Arc::new(AsyncCoroutine::new({
        let kernel = kernel.clone();
        let break_triggered = break_triggered.clone();
        async move {
            for i in 0..100 {
                if i == 5 {
                    kernel.break_flow().await;
                    break_triggered.store(true, Ordering::Relaxed);
                }
                sleep(Duration::from_millis(10)).await;
            }
            Ok(())
        }
    })).named("LongRunningTask");
    
    kernel.root().add_child(long_running_task).await;
    kernel.run_until_complete().await.unwrap();
    
    assert!(break_triggered.load(Ordering::Relaxed));
    assert!(kernel.is_breaking().await);
}

#[tokio::test] 
async fn test_complex_flow_composition() {
    let kernel = AsyncKernel::new();
    let execution_log = Arc::new(tokio::sync::Mutex::new(Vec::<String>::new()));
    
    // Setup phase
    let setup_sequence = Arc::new(Sequence::new()).named("Setup");
    
    let init_task = Arc::new(AsyncCoroutine::new({
        let log = execution_log.clone();
        async move {
            let mut log = log.lock().await;
            log.push("Init".to_string());
            Ok(())
        }
    })).named("Init");
    
    let config_task = Arc::new(AsyncCoroutine::new({
        let log = execution_log.clone();
        async move {
            let mut log = log.lock().await;
            log.push("Config".to_string());
            Ok(())
        }
    })).named("Config");
    
    setup_sequence.add_child(init_task).await;
    setup_sequence.add_child(config_task).await;
    
    // Parallel processing phase
    let parallel_barrier = Arc::new(Barrier::new()).named("Parallel");
    
    for i in 1..=3 {
        let task = Arc::new(AsyncCoroutine::new({
            let log = execution_log.clone();
            async move {
                let mut log = log.lock().await;
                log.push(format!("Parallel{}", i));
                Ok(())
            }
        })).named(format!("Parallel{}", i));
        parallel_barrier.add_child(task).await;
    }
    
    // Cleanup phase
    let cleanup_task = Arc::new(AsyncCoroutine::new({
        let log = execution_log.clone();
        async move {
            let mut log = log.lock().await;
            log.push("Cleanup".to_string());
            Ok(())
        }
    })).named("Cleanup");
    
    // Main flow
    let main_sequence = Arc::new(Sequence::new()).named("Main");
    main_sequence.add_child(setup_sequence).await;
    main_sequence.add_child(parallel_barrier).await;
    main_sequence.add_child(cleanup_task).await;
    
    kernel.root().add_child(main_sequence).await;
    kernel.run_until_complete().await.unwrap();
    
    let log = execution_log.lock().await;
    
    // Verify setup ran in sequence
    assert_eq!(log[0], "Init");
    assert_eq!(log[1], "Config");
    
    // Verify parallel tasks ran
    let mut parallel_tasks: Vec<String> = log[2..5].to_vec();
    parallel_tasks.sort();
    assert_eq!(parallel_tasks, vec!["Parallel1", "Parallel2", "Parallel3"]);
    
    // Verify cleanup ran last
    assert_eq!(log[5], "Cleanup");
}