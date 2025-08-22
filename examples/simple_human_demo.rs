use async_flow::*;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    println!("🚀 AsyncFlow Simple Demo");
    println!("========================\n");

    // Demo 1: Sequential Tasks
    println!("📋 Demo 1: Sequential Document Processing");
    println!("------------------------------------------");
    
    let kernel = AsyncKernel::new();
    let sequence = Arc::new(Sequence::new()).named("DocumentProcessing");
    
    let upload_task = Arc::new(AsyncCoroutine::new(async {
        println!("📄 Uploading document...");
        sleep(Duration::from_millis(1500)).await;
        println!("✅ Document uploaded successfully");
        Ok(())
    })).named("UploadDocument");
    
    let scan_task = Arc::new(AsyncCoroutine::new(async {
        println!("🔍 Scanning document for viruses...");
        sleep(Duration::from_millis(1200)).await;
        println!("✅ Document is clean");
        Ok(())
    })).named("ScanDocument");
    
    let process_task = Arc::new(AsyncCoroutine::new(async {
        println!("⚙️  Processing document content...");
        sleep(Duration::from_millis(1800)).await;
        println!("✅ Document processed and ready");
        Ok(())
    })).named("ProcessDocument");
    
    sequence.add_child(upload_task).await;
    sequence.add_child(scan_task).await;
    sequence.add_child(process_task).await;
    
    kernel.root().add_child(sequence).await;
    kernel.run_until_complete().await?;
    
    println!("\n");
    sleep(Duration::from_millis(800)).await;

    // Demo 2: Parallel Processing
    println!("🔄 Demo 2: Parallel File Downloads");
    println!("-----------------------------------");
    
    let kernel2 = AsyncKernel::new();
    let barrier = Arc::new(Barrier::new()).named("ParallelDownloads");
    
    let download1 = Arc::new(AsyncCoroutine::new(async {
        println!("📥 Starting download: report.pdf (2.1MB)");
        sleep(Duration::from_millis(2100)).await;
        println!("✅ Completed: report.pdf");
        Ok(())
    })).named("DownloadFile1");
    
    let download2 = Arc::new(AsyncCoroutine::new(async {
        println!("📥 Starting download: data.xlsx (1.8MB)");
        sleep(Duration::from_millis(1800)).await;
        println!("✅ Completed: data.xlsx");
        Ok(())
    })).named("DownloadFile2");
    
    let download3 = Arc::new(AsyncCoroutine::new(async {
        println!("📥 Starting download: presentation.pptx (3.2MB)");
        sleep(Duration::from_millis(2400)).await;
        println!("✅ Completed: presentation.pptx");
        Ok(())
    })).named("DownloadFile3");
    
    barrier.add_child(download1).await;
    barrier.add_child(download2).await; 
    barrier.add_child(download3).await;
    
    let compress_task = Arc::new(AsyncCoroutine::new(async {
        println!("\n🗜️  All downloads complete! Creating archive...");
        sleep(Duration::from_millis(1200)).await;
        println!("✅ Archive created: documents.zip (5.8MB)");
        Ok(())
    })).named("CompressFiles");
    
    let download_sequence = Arc::new(Sequence::new()).named("DownloadAndCompress");
    download_sequence.add_child(barrier).await;
    download_sequence.add_child(compress_task).await;
    
    kernel2.root().add_child(download_sequence).await;
    kernel2.run_until_complete().await?;
    
    println!("\n");
    sleep(Duration::from_millis(800)).await;

    // Demo 3: Future-based Communication
    println!("📡 Demo 3: Service Coordination with Futures");
    println!("--------------------------------------------");
    
    let kernel3 = AsyncKernel::new();
    let config_future = Arc::new(AsyncFuture::<String>::new()).named("ConfigData");
    let auth_future = Arc::new(AsyncFuture::<u32>::new()).named("AuthToken");
    
    let config_loader = Arc::new(AsyncCoroutine::new({
        let config_future = config_future.clone();
        async move {
            println!("⚙️  Loading system configuration...");
            sleep(Duration::from_millis(1400)).await;
            config_future.set_value("production-v2.1.3".to_string()).await;
            println!("✅ Configuration loaded");
            Ok(())
        }
    })).named("ConfigLoader");
    
    let auth_service = Arc::new(AsyncCoroutine::new({
        let auth_future = auth_future.clone();
        async move {
            println!("🔐 Authenticating with remote service...");
            sleep(Duration::from_millis(1800)).await;
            auth_future.set_value(87654321).await;
            println!("✅ Authentication successful");
            Ok(())
        }
    })).named("AuthService");
    
    let main_service = Arc::new(AsyncCoroutine::new({
        let config_future = config_future.clone();
        let auth_future = auth_future.clone();
        async move {
            println!("⏳ Main service waiting for dependencies...");
            
            let config = config_future.wait().await;
            println!("📋 Using config: {}", config);
            
            let token = auth_future.wait().await;
            println!("🎫 Using auth token: {}", token);
            
            println!("🚀 Starting main application service...");
            sleep(Duration::from_millis(1000)).await;
            println!("✅ Application is now running!");
            
            Ok(())
        }
    })).named("MainService");
    
    let startup_barrier = Arc::new(Barrier::new()).named("ServiceStartup");
    startup_barrier.add_child(config_loader).await;
    startup_barrier.add_child(auth_service).await;
    startup_barrier.add_child(main_service).await;
    
    kernel3.root().add_child(startup_barrier).await;
    kernel3.run_until_complete().await?;
    
    println!("\n🎊 Demo Complete!");
    println!("==================");
    println!("AsyncFlow successfully demonstrated:");
    println!("  ✅ Sequential task execution");
    println!("  ✅ Parallel processing with barriers"); 
    println!("  ✅ Future-based inter-task communication");
    println!("  ✅ Zero threads - pure async/await coordination!");
    
    Ok(())
}