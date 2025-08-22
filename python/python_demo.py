"""
AsyncFlow Python Demo - Cross-platform demonstration
Works on Windows, macOS, and Linux with Python 3.7+
"""

import asyncio
import sys
import os

# Add python_asyncflow to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'python_asyncflow'))

from python_asyncflow import AsyncKernel, FlowFactory


async def demo_task(name: str, delay: float):
    """Simple async task for demonstration"""
    print(f"📄 Starting task: {name}")
    await asyncio.sleep(delay)
    print(f"✅ Completed task: {name}")


async def main():
    """Main demo function"""
    print("🚀 AsyncFlow Python Demo")
    print("=========================")
    print(f"Python version: {sys.version}")
    print(f"Platform: {sys.platform}")
    print()

    # Demo 1: Sequential Processing
    print("📋 Demo 1: Sequential Task Processing")
    print("--------------------------------------")
    
    kernel = AsyncKernel()
    sequence = FlowFactory.new_sequence("DocumentProcessing")
    
    # Create tasks
    upload_task = FlowFactory.new_async_coroutine(
        demo_task("Upload Document", 1.5),
        "UploadTask"
    )
    
    scan_task = FlowFactory.new_async_coroutine(
        demo_task("Scan Document", 1.2), 
        "ScanTask"
    )
    
    process_task = FlowFactory.new_async_coroutine(
        demo_task("Process Document", 1.8),
        "ProcessTask"
    )
    
    # Add to sequence
    sequence.add_child(upload_task)
    sequence.add_child(scan_task)
    sequence.add_child(process_task)
    
    kernel.root.add_child(sequence)
    await kernel.run_until_complete()
    
    print()
    await asyncio.sleep(0.8)

    # Demo 2: Parallel Processing
    print("🔄 Demo 2: Parallel File Processing")
    print("------------------------------------")
    
    kernel2 = AsyncKernel()
    barrier = FlowFactory.new_barrier("ParallelTasks")
    
    # Create parallel tasks
    task1 = FlowFactory.new_async_coroutine(
        demo_task("Download file1.pdf (2.1MB)", 2.1),
        "Download1"
    )
    
    task2 = FlowFactory.new_async_coroutine(
        demo_task("Download file2.docx (1.8MB)", 1.8),
        "Download2"
    )
    
    task3 = FlowFactory.new_async_coroutine(
        demo_task("Download file3.xlsx (2.4MB)", 2.4),
        "Download3"
    )
    
    # Add to barrier
    barrier.add_child(task1)
    barrier.add_child(task2)
    barrier.add_child(task3)
    
    # Compression after all downloads
    compress_task = FlowFactory.new_async_coroutine(
        demo_task("Compress all files", 1.2),
        "CompressTask"
    )
    
    # Create sequence: barrier then compress
    download_sequence = FlowFactory.new_sequence("DownloadAndCompress")
    download_sequence.add_child(barrier)
    download_sequence.add_child(compress_task)
    
    kernel2.root.add_child(download_sequence)
    await kernel2.run_until_complete()
    
    print()
    await asyncio.sleep(0.8)

    # Demo 3: Timer-based Processing
    print("⏰ Demo 3: Timer-based Progress System")
    print("--------------------------------------")
    
    kernel3 = AsyncKernel()
    counter = [0]  # Use list for mutable reference
    
    def progress_callback():
        counter[0] += 1
        percentage = min(counter[0] * 20, 100)
        bar = "█" * (percentage // 10)
        space = "░" * (10 - percentage // 10)
        print(f"📊 Progress: [{bar}{space}] {percentage}%")
    
    # Create progress timer
    progress_timer = FlowFactory.new_periodic_timer(0.8, "ProgressTimer")
    progress_timer.set_elapsed_callback(progress_callback)
    
    # Create completion trigger
    completion_trigger = FlowFactory.new_trigger(
        lambda: counter[0] >= 5,
        "CompletionTrigger"
    )
    
    def completion_callback():
        print("🎉 Processing complete!")
    
    completion_trigger.set_triggered_callback(completion_callback)
    
    # Background work
    background_work = FlowFactory.new_async_coroutine(
        demo_task("Background processing", 4.5),
        "BackgroundWork"
    )
    
    kernel3.root.add_child(progress_timer)
    kernel3.root.add_child(completion_trigger)
    kernel3.root.add_child(background_work)
    
    await kernel3.run_until_complete()
    
    print()
    await asyncio.sleep(0.8)

    # Demo 4: Future Communication
    print("📡 Demo 4: Future-based Communication")
    print("--------------------------------------")
    
    kernel4 = AsyncKernel()
    
    # Create futures
    config_future = FlowFactory.new_future("ConfigFuture")
    auth_future = FlowFactory.new_future("AuthFuture")
    
    # Configuration loader
    async def load_config():
        print("⚙️  Loading configuration...")
        await asyncio.sleep(1.4)
        await config_future.set_value("production-config-v2.1")
        print("✅ Configuration loaded")
    
    config_loader = FlowFactory.new_async_coroutine(load_config(), "ConfigLoader")
    
    # Authentication service  
    async def authenticate():
        print("🔐 Authenticating...")
        await asyncio.sleep(1.8)
        await auth_future.set_value(12345678)
        print("✅ Authentication successful")
    
    auth_service = FlowFactory.new_async_coroutine(authenticate(), "AuthService")
    
    # Main service that waits for both
    async def main_service():
        print("⏳ Main service waiting for dependencies...")
        
        config = await config_future.wait()
        print(f"📋 Using config: {config}")
        
        token = await auth_future.wait()
        print(f"🎫 Using auth token: {token}")
        
        print("🚀 Starting main service...")
        await asyncio.sleep(1.0)
        print("✅ Main service running!")
    
    main_task = FlowFactory.new_async_coroutine(main_service(), "MainService")
    
    # Run all in barrier
    startup_barrier = FlowFactory.new_barrier("StartupServices")
    startup_barrier.add_child(config_loader)
    startup_barrier.add_child(auth_service)
    startup_barrier.add_child(main_task)
    
    kernel4.root.add_child(startup_barrier)
    await kernel4.run_until_complete()
    
    print()
    print("🎊 All demos completed successfully!")
    print("====================================")
    print("AsyncFlow Python demonstrated:")
    print("  ✅ Sequential task execution")
    print("  ✅ Parallel processing with barriers")
    print("  ✅ Timer-based progress tracking")
    print("  ✅ Future-based communication")
    print("  ✅ Zero threads - pure asyncio coordination!")


if __name__ == "__main__":
    # Cross-platform asyncio execution
    try:
        # Python 3.7+
        asyncio.run(main())
    except AttributeError:
        # Python 3.6 fallback
        loop = asyncio.get_event_loop()
        loop.run_until_complete(main())
        loop.close()