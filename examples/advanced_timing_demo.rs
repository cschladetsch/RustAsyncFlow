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

    println!("=== AsyncFlow Advanced Timing Patterns Demo ===");
    println!("This demo showcases complex timing patterns and combinations\n");

    // Demo 1: Timeout pattern - race between work and timeout
    demo_timeout_pattern(&root).await?;
    
    // Demo 2: Heartbeat with health monitoring
    demo_heartbeat_monitoring(&root).await?;
    
    // Demo 3: Cascading timer chain
    demo_cascading_timer_chain(&root).await?;
    
    // Demo 4: Timer orchestration with dynamic barriers
    demo_timer_orchestration(&root).await?;

    println!("Starting all advanced timing demos...");
    kernel.run_until_complete().await?;
    println!("\n=== All Advanced Timing Demos Completed! ===");

    Ok(())
}

async fn demo_timeout_pattern(root: &Arc<Node>) -> Result<()> {
    println!("--- Demo 1: Timeout Pattern (Race Condition) ---");
    
    let work_completed = Arc::new(AtomicBool::new(false));
    let timeout_occurred = Arc::new(AtomicBool::new(false));
    
    // Simulate work that might take variable time
    let work_task = FlowFactory::new_async_coroutine_with_name(
        "WorkTask",
        {
            let work_completed = work_completed.clone();
            async move {
                // Simulate work that takes 700ms
                sleep(Duration::from_millis(700)).await;
                work_completed.store(true, Ordering::Relaxed);
                println!("  🔨 Work task completed (700ms)");
                Ok(())
            }
        }
    );
    
    // Timeout timer (500ms)
    let timeout_timer = FlowFactory::new_timer_with_name(
        "TimeoutTimer",
        Duration::from_millis(500)
    );
    
    let timeout_occurred_clone = timeout_occurred.clone();
    timeout_timer.set_elapsed_callback(move || {
        timeout_occurred_clone.store(true, Ordering::Relaxed);
        println!("  ⏰ Timeout occurred (500ms)");
    }).await;
    
    // Success trigger (work completes before timeout)
    let success_trigger = FlowFactory::new_trigger_with_name(
        "SuccessTrigger",
        {
            let work_completed = work_completed.clone();
            let timeout_occurred = timeout_occurred.clone();
            move || work_completed.load(Ordering::Relaxed) && !timeout_occurred.load(Ordering::Relaxed)
        }
    );
    
    success_trigger.set_triggered_callback(|| {
        println!("  ✅ Success: Work completed before timeout!");
    }).await;
    
    // Timeout trigger (timeout occurs before work completes)
    let timeout_trigger = FlowFactory::new_trigger_with_name(
        "TimeoutTrigger",
        {
            let work_completed = work_completed.clone();
            let timeout_occurred = timeout_occurred.clone();
            move || timeout_occurred.load(Ordering::Relaxed) && !work_completed.load(Ordering::Relaxed)
        }
    );
    
    timeout_trigger.set_triggered_callback(|| {
        println!("  ⏳ Timeout: Work did not complete in time!");
    }).await;
    
    // Completion trigger (either outcome reached)
    let completion_trigger = FlowFactory::new_trigger_with_name(
        "Demo1CompletionTrigger",
        {
            let work_completed = work_completed.clone();
            let timeout_occurred = timeout_occurred.clone();
            move || work_completed.load(Ordering::Relaxed) || timeout_occurred.load(Ordering::Relaxed)
        }
    );
    
    completion_trigger.set_triggered_callback(|| {
        println!("  🎯 Demo 1 finished - timeout pattern demonstrated\n");
    }).await;
    
    root.add_child(work_task).await;
    root.add_child(timeout_timer).await;
    root.add_child(success_trigger).await;
    root.add_child(timeout_trigger).await;
    root.add_child(completion_trigger).await;
    
    Ok(())
}

async fn demo_heartbeat_monitoring(root: &Arc<Node>) -> Result<()> {
    println!("--- Demo 2: Heartbeat with Health Monitoring ---");
    
    let heartbeat_count = Arc::new(AtomicU32::new(0));
    let system_healthy = Arc::new(AtomicBool::new(true));
    
    // Heartbeat timer (every 200ms)
    let heartbeat_timer = FlowFactory::new_periodic_timer_with_name(
        "HeartbeatTimer",
        Duration::from_millis(200)
    );
    
    let heartbeat_count_clone = heartbeat_count.clone();
    heartbeat_timer.set_elapsed_callback(move || {
        let count = heartbeat_count_clone.fetch_add(1, Ordering::Relaxed) + 1;
        println!("  💓 Heartbeat #{}", count);
    }).await;
    
    // Health check timer (every 600ms)
    let health_check_timer = FlowFactory::new_periodic_timer_with_name(
        "HealthCheckTimer",
        Duration::from_millis(600)
    );
    
    let system_healthy_clone = system_healthy.clone();
    health_check_timer.set_elapsed_callback(move || {
        // Simulate system becoming unhealthy after some time
        let current_health = system_healthy_clone.load(Ordering::Relaxed);
        if current_health && rand::random::<f32>() < 0.3 {
            system_healthy_clone.store(false, Ordering::Relaxed);
            println!("  🔴 Health check failed - system unhealthy!");
        } else if current_health {
            println!("  🟢 Health check passed - system healthy");
        }
    }).await;
    
    // Unhealthy system trigger
    let unhealthy_trigger = FlowFactory::new_trigger_with_name(
        "UnhealthyTrigger",
        {
            let system_healthy = system_healthy.clone();
            move || !system_healthy.load(Ordering::Relaxed)
        }
    );
    
    unhealthy_trigger.set_triggered_callback(|| {
        println!("  🚨 System unhealthy detected - initiating shutdown...");
    }).await;
    
    // Maximum heartbeats trigger (stop after 8 heartbeats)
    let max_heartbeats_trigger = FlowFactory::new_trigger_with_name(
        "MaxHeartbeatsTrigger",
        {
            let heartbeat_count = heartbeat_count.clone();
            move || heartbeat_count.load(Ordering::Relaxed) >= 8
        }
    );
    
    max_heartbeats_trigger.set_triggered_callback(|| {
        println!("  ⏹️  Maximum heartbeats reached - stopping monitoring");
    }).await;
    
    // Completion trigger (unhealthy OR max heartbeats)
    let monitoring_complete_trigger = FlowFactory::new_trigger_with_name(
        "Demo2CompletionTrigger",
        {
            let heartbeat_count = heartbeat_count.clone();
            let system_healthy = system_healthy.clone();
            move || !system_healthy.load(Ordering::Relaxed) || heartbeat_count.load(Ordering::Relaxed) >= 8
        }
    );
    
    monitoring_complete_trigger.set_triggered_callback(|| {
        println!("  🎯 Demo 2 finished - health monitoring completed\n");
    }).await;
    
    root.add_child(heartbeat_timer).await;
    root.add_child(health_check_timer).await;
    root.add_child(unhealthy_trigger).await;
    root.add_child(max_heartbeats_trigger).await;
    root.add_child(monitoring_complete_trigger).await;
    
    Ok(())
}

async fn demo_cascading_timer_chain(root: &Arc<Node>) -> Result<()> {
    println!("--- Demo 3: Cascading Timer Chain ---");
    
    let chain_stages = Arc::new(AtomicU32::new(0));
    let chain_complete = Arc::new(AtomicBool::new(false));
    
    // Stage 1 Timer (300ms)
    let stage1_timer = FlowFactory::new_timer_with_name(
        "ChainStage1Timer",
        Duration::from_millis(300)
    );
    
    let chain_stages_clone = chain_stages.clone();
    stage1_timer.set_elapsed_callback(move || {
        chain_stages_clone.store(1, Ordering::Relaxed);
        println!("  🔗 Chain Stage 1 completed (300ms)");
    }).await;
    
    // Stage 2 Timer - starts after Stage 1
    let stage2_timer = FlowFactory::new_timer_with_name(
        "ChainStage2Timer",
        Duration::from_millis(250)
    );
    
    let stage2_started = Arc::new(AtomicBool::new(false));
    let chain_stages_clone2 = chain_stages.clone();
    let stage2_started_clone = stage2_started.clone();
    stage2_timer.set_elapsed_callback(move || {
        if stage2_started_clone.load(Ordering::Relaxed) {
            chain_stages_clone2.store(2, Ordering::Relaxed);
            println!("  🔗 Chain Stage 2 completed (250ms)");
        }
    }).await;
    
    // Stage 2 trigger - activates Stage 2 timer when Stage 1 completes
    let stage2_trigger = FlowFactory::new_trigger_with_name(
        "Stage2StartTrigger",
        {
            let chain_stages = chain_stages.clone();
            move || chain_stages.load(Ordering::Relaxed) >= 1
        }
    );
    
    let stage2_started_clone2 = stage2_started.clone();
    stage2_trigger.set_triggered_callback(move || {
        stage2_started_clone2.store(true, Ordering::Relaxed);
        println!("  ⚡ Stage 2 timer activated by Stage 1 completion");
    }).await;
    
    // Stage 3 Timer - starts after Stage 2
    let stage3_timer = FlowFactory::new_timer_with_name(
        "ChainStage3Timer",
        Duration::from_millis(200)
    );
    
    let stage3_started = Arc::new(AtomicBool::new(false));
    let chain_complete_clone = chain_complete.clone();
    let stage3_started_clone = stage3_started.clone();
    stage3_timer.set_elapsed_callback(move || {
        if stage3_started_clone.load(Ordering::Relaxed) {
            chain_complete_clone.store(true, Ordering::Relaxed);
            println!("  🔗 Chain Stage 3 completed (200ms) - Full chain complete!");
        }
    }).await;
    
    // Stage 3 trigger - activates Stage 3 timer when Stage 2 completes
    let stage3_trigger = FlowFactory::new_trigger_with_name(
        "Stage3StartTrigger",
        {
            let chain_stages = chain_stages.clone();
            move || chain_stages.load(Ordering::Relaxed) >= 2
        }
    );
    
    let stage3_started_clone2 = stage3_started.clone();
    stage3_trigger.set_triggered_callback(move || {
        stage3_started_clone2.store(true, Ordering::Relaxed);
        println!("  ⚡ Stage 3 timer activated by Stage 2 completion");
    }).await;
    
    // Final completion trigger
    let chain_completion_trigger = FlowFactory::new_trigger_with_name(
        "Demo3CompletionTrigger",
        {
            let chain_complete = chain_complete.clone();
            move || chain_complete.load(Ordering::Relaxed)
        }
    );
    
    chain_completion_trigger.set_triggered_callback(|| {
        println!("  🎯 Demo 3 finished - cascading timer chain completed\n");
    }).await;
    
    root.add_child(stage1_timer).await;
    root.add_child(stage2_timer).await;
    root.add_child(stage2_trigger).await;
    root.add_child(stage3_timer).await;
    root.add_child(stage3_trigger).await;
    root.add_child(chain_completion_trigger).await;
    
    Ok(())
}

async fn demo_timer_orchestration(root: &Arc<Node>) -> Result<()> {
    println!("--- Demo 4: Timer Orchestration with Dynamic Barriers ---");
    
    let _orchestration_complete = Arc::new(AtomicBool::new(false));
    
    // Create first wave of timers
    let wave1_barrier = FlowFactory::new_barrier_with_name("Wave1Barrier");
    let wave1_complete = Arc::new(AtomicBool::new(false));
    
    for i in 1..=3 {
        let timer = FlowFactory::new_timer_with_name(
            &format!("Wave1Timer_{}", i),
            Duration::from_millis(200 + i * 50)
        );
        
        timer.set_elapsed_callback(move || {
            println!("  🌊 Wave 1 Timer {} completed", i);
        }).await;
        
        wave1_barrier.add_child(timer).await;
    }
    
    // Wave 1 completion task
    let wave1_completion = FlowFactory::new_async_coroutine_with_name(
        "Wave1Completion",
        {
            let wave1_complete = wave1_complete.clone();
            async move {
                wave1_complete.store(true, Ordering::Relaxed);
                println!("  ✅ Wave 1 completed - triggering Wave 2");
                Ok(())
            }
        }
    );
    
    // Create second wave of timers (triggered by Wave 1 completion)
    let wave2_barrier = FlowFactory::new_barrier_with_name("Wave2Barrier");
    let wave2_complete = Arc::new(AtomicBool::new(false));
    
    for i in 1..=2 {
        let timer = FlowFactory::new_timer_with_name(
            &format!("Wave2Timer_{}", i),
            Duration::from_millis(150 + i * 75)
        );
        
        timer.set_elapsed_callback(move || {
            println!("  🌊 Wave 2 Timer {} completed", i);
        }).await;
        
        wave2_barrier.add_child(timer).await;
    }
    
    // Wave 2 activation trigger
    let wave2_trigger = FlowFactory::new_trigger_with_name(
        "Wave2ActivationTrigger",
        {
            let wave1_complete = wave1_complete.clone();
            move || wave1_complete.load(Ordering::Relaxed)
        }
    );
    
    wave2_trigger.set_triggered_callback(|| {
        println!("  ⚡ Wave 2 barrier activated");
    }).await;
    
    // Wave 2 completion task
    let wave2_completion = FlowFactory::new_async_coroutine_with_name(
        "Wave2Completion",
        {
            let wave2_complete = wave2_complete.clone();
            async move {
                wave2_complete.store(true, Ordering::Relaxed);
                println!("  ✅ Wave 2 completed - orchestration finished");
                Ok(())
            }
        }
    );
    
    // Final orchestration completion trigger
    let orchestration_trigger = FlowFactory::new_trigger_with_name(
        "OrchestrationCompletionTrigger",
        {
            let wave2_complete = wave2_complete.clone();
            move || wave2_complete.load(Ordering::Relaxed)
        }
    );
    
    orchestration_trigger.set_triggered_callback(move || {
        println!("  🎯 Demo 4 finished - timer orchestration completed\n");
    }).await;
    
    // Create sequences for proper ordering
    let wave1_sequence = FlowFactory::new_sequence_with_name("Wave1Sequence");
    wave1_sequence.add_child(wave1_barrier).await;
    wave1_sequence.add_child(wave1_completion).await;
    
    let wave2_sequence = FlowFactory::new_sequence_with_name("Wave2Sequence");
    wave2_sequence.add_child(wave2_barrier).await;
    wave2_sequence.add_child(wave2_completion).await;
    
    root.add_child(wave1_sequence).await;
    root.add_child(wave2_trigger).await;
    root.add_child(wave2_sequence).await;
    root.add_child(orchestration_trigger).await;
    
    Ok(())
}