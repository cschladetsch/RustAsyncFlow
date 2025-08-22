use async_flow::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    println!("🚀 AsyncFlow Human-Speed Demo");
    println!("============================\n");

    let kernel = AsyncKernel::new();
    let root = kernel.root();

    // Demo 1: Sequential Tasks
    println!("📋 Demo 1: Sequential Task Processing");
    println!("--------------------------------------");
    
    let sequence = FlowFactory::new_sequence_with_name("DocumentProcessing");
    
    let upload_task = FlowFactory::new_async_coroutine_with_name(
        "UploadDocument",
        async {
            println!("📄 Uploading document...");
            sleep(Duration::from_secs(4)).await;
            println!("✅ Document uploaded successfully");
            Ok(())
        }
    );
    
    let scan_task = FlowFactory::new_async_coroutine_with_name(
        "ScanDocument", 
        async {
            println!("🔍 Scanning document for viruses...");
            sleep(Duration::from_secs(3)).await;
            println!("✅ Document is clean");
            Ok(())
        }
    );
    
    let process_task = FlowFactory::new_async_coroutine_with_name(
        "ProcessDocument",
        async {
            println!("⚙️  Processing document content...");
            sleep(Duration::from_secs(4)).await;
            println!("✅ Document processed and indexed");
            Ok(())
        }
    );
    
    sequence.add_child(upload_task).await;
    sequence.add_child(scan_task).await;
    sequence.add_child(process_task).await;
    
    root.add_child(sequence).await;
    kernel.run_until_complete().await?;
    
    println!("\n");
    sleep(Duration::from_secs(1)).await;
    
    // Demo 2: Parallel Processing with Barrier
    println!("🔄 Demo 2: Parallel Processing with Barrier");
    println!("-------------------------------------------");
    
    let kernel2 = AsyncKernel::new();
    let root2 = kernel2.root();
    
    let barrier = FlowFactory::new_barrier_with_name("ParallelTasks");
    
    let download1 = FlowFactory::new_async_coroutine_with_name(
        "DownloadFile1",
        async {
            println!("📥 Downloading file1.pdf...");
            sleep(Duration::from_secs(5)).await;
            println!("✅ file1.pdf downloaded (2.5MB)");
            Ok(())
        }
    );
    
    let download2 = FlowFactory::new_async_coroutine_with_name(
        "DownloadFile2", 
        async {
            println!("📥 Downloading file2.docx...");
            sleep(Duration::from_secs(4)).await;
            println!("✅ file2.docx downloaded (1.8MB)");
            Ok(())
        }
    );
    
    let download3 = FlowFactory::new_async_coroutine_with_name(
        "DownloadFile3",
        async {
            println!("📥 Downloading file3.xlsx...");
            sleep(Duration::from_secs(4)).await;
            println!("✅ file3.xlsx downloaded (3.2MB)");
            Ok(())
        }
    );
    
    barrier.add_child(download1).await;
    barrier.add_child(download2).await; 
    barrier.add_child(download3).await;
    
    let compress_task = FlowFactory::new_async_coroutine_with_name(
        "CompressFiles",
        async {
            println!("🗜️  Compressing all files into archive...");
            sleep(Duration::from_secs(3)).await;
            println!("✅ Archive created: documents.zip (4.1MB)");
            Ok(())
        }
    );
    
    let final_sequence = FlowFactory::new_sequence_with_name("DownloadAndCompress");
    final_sequence.add_child(barrier).await;
    final_sequence.add_child(compress_task).await;
    
    root2.add_child(final_sequence).await;
    kernel2.run_until_complete().await?;
    
    println!("\n");
    sleep(Duration::from_secs(1)).await;
    
    // Demo 3: Timer-based System with Progress
    println!("⏰ Demo 3: Timer-based Progress System");
    println!("--------------------------------------");
    
    let kernel3 = AsyncKernel::new();
    let root3 = kernel3.root();
    
    let progress_counter = Arc::new(AtomicU32::new(0));
    
    let progress_timer = FlowFactory::new_periodic_timer_with_name(
        "ProgressUpdater",
        Duration::from_secs(2)
    );
    
    progress_timer.set_elapsed_callback({
        let counter = progress_counter.clone();
        move || {
            let current = counter.fetch_add(1, Ordering::Relaxed) + 1;
            let percentage = (current * 10).min(100);
            let bar = "█".repeat((percentage / 10) as usize);
            let space = "░".repeat(10 - (percentage / 10) as usize);
            println!("📊 Processing... [{}{}] {}%", bar, space, percentage);
        }
    }).await;
    
    let completion_trigger = FlowFactory::new_trigger_with_name(
        "CompletionTrigger",
        {
            let counter = progress_counter.clone();
            move || counter.load(Ordering::Relaxed) >= 10
        }
    );
    
    completion_trigger.set_triggered_callback({
        let timer = progress_timer.clone();
        move || {
            println!("🎉 Processing complete! All tasks finished successfully.");
            timer.complete();
        }
    }).await;
    
    // Background work simulation
    let work_task = FlowFactory::new_async_coroutine_with_name(
        "BackgroundWork",
        async {
            println!("🔧 Starting background processing...");
            sleep(Duration::from_secs(20)).await;
            Ok(())
        }
    );
    
    root3.add_child(progress_timer).await;
    root3.add_child(completion_trigger).await;
    root3.add_child(work_task).await;
    
    kernel3.run_until_complete().await?;
    
    println!("\n");
    sleep(Duration::from_secs(1)).await;
    
    // Demo 4: Future-based Communication
    println!("📡 Demo 4: Future-based Inter-task Communication");
    println!("------------------------------------------------");
    
    let kernel4 = AsyncKernel::new();
    let root4 = kernel4.root();
    
    let config_future = FlowFactory::new_future_with_name::<String>("ConfigData");
    let auth_future = FlowFactory::new_future_with_name::<u32>("AuthToken");
    
    let config_loader = FlowFactory::new_async_coroutine_with_name(
        "ConfigLoader",
        {
            let config_future = config_future.clone();
            async move {
                println!("⚙️  Loading configuration from server...");
                sleep(Duration::from_secs(3)).await;
                config_future.set_value("production-config-v2.1".to_string()).await;
                println!("✅ Configuration loaded");
                Ok(())
            }
        }
    );
    
    let auth_service = FlowFactory::new_async_coroutine_with_name(
        "AuthService",
        {
            let auth_future = auth_future.clone();
            async move {
                println!("🔐 Authenticating with service...");
                sleep(Duration::from_secs(4)).await;
                auth_future.set_value(12345678).await;
                println!("✅ Authentication successful");
                Ok(())
            }
        }
    );
    
    let main_service = FlowFactory::new_async_coroutine_with_name(
        "MainService",
        {
            let config_future = config_future.clone();
            let auth_future = auth_future.clone();
            async move {
                println!("⏳ Waiting for configuration and authentication...");
                
                let config = config_future.wait().await;
                println!("📋 Received config: {}", config);
                
                let token = auth_future.wait().await;
                println!("🎫 Received auth token: {}", token);
                
                println!("🚀 Starting main service with config and auth...");
                sleep(Duration::from_secs(2)).await;
                println!("✅ Main service is now running!");
                
                Ok(())
            }
        }
    );
    
    let startup_barrier = FlowFactory::new_barrier_with_name("StartupServices");
    startup_barrier.add_child(config_loader).await;
    startup_barrier.add_child(auth_service).await;
    startup_barrier.add_child(main_service).await;
    
    root4.add_child(startup_barrier).await;
    kernel4.run_until_complete().await?;
    
    println!("\n🎊 All demos completed successfully!");
    println!("AsyncFlow demonstrated:");
    println!("  • Sequential task execution");
    println!("  • Parallel processing with barriers");
    println!("  • Timer-based progress tracking");
    println!("  • Future-based communication");
    println!("  • Zero threads used - all async/await coordination!");
    
    Ok(())
}