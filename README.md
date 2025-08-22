# RustAsyncFlow

A thread-free, async/await-based flow control system for Rust, inspired by my own [CsharpFlow](https://github.com/cschladetsch/CsharpFlow). This library provides coroutine-like functionality using Rust's native async/await for cooperative multitasking.

## Table of Contents

- [Demo](#demo)
- [Features](#features)
- [Core Components](#core-components)
  - [Generators](#generators)  
  - [Kernel](#kernel)
- [Examples](#examples)
  - [Basic Timer Example](#basic-timer-example)
  - [Sequence Example](#sequence-example)
  - [Barrier Example](#barrier-example)
- [Running Examples](#running-examples)
- [Running Tests](#running-tests)
- [Documentation](#documentation)
- [Timed Components](#timed-components)
- [Comparison with CsharpFlow](#comparison-with-csharpflow)
- [Architecture](#architecture)
  - [System Architecture](#system-architecture)
  - [Component Relationships](#component-relationships)
  - [Flow Execution Model](#flow-execution-model)
  - [Thread-Free Coordination](#thread-free-coordination)

## Demo

![Demo](resources/Demo.gif)

## Features

- **Thread-free**: Uses async/await and tokio for concurrency without threads
- **Flow Control**: Sequences, barriers, triggers, timers, and futures
- **Composable**: Build complex flow graphs from simple components  
- **Async Native**: Designed for Rust's async ecosystem
- **Memory Safe**: Leverages Rust's ownership system for safe concurrent programming

## Core Components

### Generators
Base abstraction for all flow components with lifecycle management:
- `AsyncCoroutine` - Wraps async functions
- `SyncCoroutine` - Wraps synchronous step functions  
- `Node` - Container for child generators
- `Sequence` - Executes children sequentially
- `Barrier` - Waits for all children to complete
- `Timer` - One-shot timer with callback
- `PeriodicTimer` - Repeating timer with callback
- `Trigger` - Fires when condition becomes true
- `AsyncFuture` - Thread-safe future value

### Kernel
The `AsyncKernel` manages the execution of the flow graph:
- `run_until_complete()` - Run until all tasks finish
- `run_for(duration)` - Run for specified time
- `break_flow()` - Stop execution
- `wait(duration)` - Pause execution

## Examples

### Basic Timer Example
```rust
use async_flow::*;
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    let kernel = AsyncKernel::new();
    let root = kernel.root();
    
    let timer = Arc::new(PeriodicTimer::new(Duration::from_millis(500)))
        .named("Heartbeat");
    
    timer.set_elapsed_callback(|| {
        println!("Heartbeat!");
    }).await;
    
    root.add_child(timer).await;
    kernel.run_for(Duration::from_secs(3)).await?;
    
    Ok(())
}
```

### Sequence Example
```rust
use async_flow::*;
use std::sync::Arc;
use std::time::Duration;

async fn task(name: &str) -> Result<()> {
    println!("Running task: {}", name);
    tokio::time::sleep(Duration::from_millis(100)).await;
    println!("Completed task: {}", name);
    Ok(())
}

#[tokio::main] 
async fn main() -> Result<()> {
    let kernel = AsyncKernel::new();
    let sequence = Arc::new(Sequence::new()).named("TaskSequence");
    
    sequence.add_child(Arc::new(AsyncCoroutine::new(task("Alpha")))
        .named("Task1")).await;
    
    sequence.add_child(Arc::new(AsyncCoroutine::new(task("Beta")))
        .named("Task2")).await;
    
    kernel.root().add_child(sequence).await;
    kernel.run_until_complete().await?;
    
    Ok(())
}
```

### Barrier Example
```rust  
use async_flow::*;
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    let kernel = AsyncKernel::new();
    let barrier = Arc::new(Barrier::new()).named("ParallelTasks");
    
    // These tasks run concurrently
    for i in 1..=3 {
        let task = Arc::new(AsyncCoroutine::new(async move {
            println!("Starting task {}", i);
            tokio::time::sleep(Duration::from_millis(i * 100)).await;
            println!("Finished task {}", i);
            Ok(())
        })).named(format!("Task{}", i));
        barrier.add_child(task).await;
    }
    
    let after_barrier = Arc::new(AsyncCoroutine::new(async {
        println!("All parallel tasks completed!");
        Ok(())
    })).named("Cleanup");
    
    let sequence = Arc::new(Sequence::new());
    sequence.add_child(barrier).await;
    sequence.add_child(after_barrier).await;
    
    kernel.root().add_child(sequence).await;
    kernel.run_until_complete().await?;
    
    Ok(())
}
```

## Running Examples

```bash
# Quick human-readable demo showing all major features
cargo run --example simple_human_demo

# Individual feature examples
cargo run --example basic_example
cargo run --example game_loop_example  
cargo run --example barrier_example
cargo run --example future_example
cargo run --example human_speed_demo  # Comprehensive demo (slower-paced)

# Timed component demos
cargo run --example timed_trigger_demo      # Timer + Trigger combinations
cargo run --example timed_barrier_demo      # Timed barriers and synchronization
cargo run --example advanced_timing_demo    # Complex timing patterns
```

### Using the `crne` Helper Script

For convenience, you can use the `crne` script to run examples more easily:

```bash
# Show available examples and help
./crne --help

# Run examples using the helper script
./crne simple_human_demo
./crne basic_example
./crne game_loop_example --flag value
```

## Running Tests

```bash
# Run all tests
cargo test

# Run specific test suites
cargo test integration_tests
cargo test reliable_timed_tests
cargo test timed_components_tests
```

## Documentation

For detailed component documentation and advanced usage patterns:

- [Source Code Structure](src/README.md) - Internal code organization and architecture
- [Flow Components Documentation](src/flow/README.md) - Detailed flow component implementation with comprehensive Mermaid diagrams
- [Timed Components Guide](docs/TIMED_COMPONENTS.md) - Comprehensive timing patterns and examples

The [Flow Components Documentation](src/flow/README.md) includes detailed architectural diagrams showing:
- Component inheritance hierarchy and relationships
- Execution patterns (sequential, parallel, timing)
- State management and lifecycle
- Thread-safe coordination patterns

## Timed Components

AsyncFlow provides sophisticated timing capabilities through several components that can be combined to create complex timing behaviors:

### Available Timed Components

- **Timer**: One-shot timer with configurable duration and callback
- **PeriodicTimer**: Repeating timer that fires at regular intervals
- **Trigger**: Condition-based component that activates when criteria are met
- **Barrier**: Synchronization component for coordinating multiple timed tasks

### Quick Example - Timeout Pattern

```rust
let timeout_timer = Arc::new(Timer::new(Duration::from_millis(500)))
    .named("Timeout");

let work_task = Arc::new(AsyncCoroutine::new(async { 
    /* some work that might take too long */ 
    Ok(()) 
})).named("Work");

// Timer and work race - first to complete wins
let completion_trigger = Arc::new(Trigger::new(move || 
    timeout_occurred || work_completed
)).named("FirstComplete");
```

### Timed Component Demos

Run the timed component examples to see sophisticated patterns:

- `timed_trigger_demo` - Basic and advanced timer/trigger combinations
- `timed_barrier_demo` - Parallel timed tasks with barrier synchronization  
- `advanced_timing_demo` - Complex patterns like cascading timers and orchestration

See the [Timed Components Guide](docs/TIMED_COMPONENTS.md) for comprehensive documentation and patterns.

## Comparison with CsharpFlow

| Feature | CsharpFlow | AsyncFlow |
|---------|------------|-----------|
| Language | C# | Rust |
| Memory Model | GC + locking | Ownership + Arc/RwLock |
| Coroutines | IEnumerator | async functions |
| Performance | Managed runtime | Zero-cost async |
| Safety | Runtime errors | Compile-time safety |

## Architecture

AsyncFlow uses a hierarchical structure where:
- `AsyncKernel` manages the root execution loop  
- `Node` containers hold child generators
- Each generator can be stepped independently
- Async coordination happens through Rust's async runtime
- No threads are created - everything runs on the tokio executor

### System Architecture

```mermaid
graph TB
    K[AsyncKernel] --> R[Root Node]
    R --> S1[Sequence]
    R --> B1[Barrier] 
    R --> T1[Timer]
    
    S1 --> C1[AsyncCoroutine]
    S1 --> C2[AsyncCoroutine]
    S1 --> C3[AsyncCoroutine]
    
    B1 --> P1[AsyncCoroutine]
    B1 --> P2[AsyncCoroutine]
    B1 --> P3[AsyncCoroutine]
    
    T1 --> CB1[Callback]
    
    K --> TF[TimeFrame]
    K --> BF[Break Flag]
    K --> WU[Wait Until]
    
    subgraph "Tokio Runtime"
        direction TB
        JH1[JoinHandle]
        JH2[JoinHandle] 
        JH3[JoinHandle]
        JH4[JoinHandle]
        JH5[JoinHandle]
        JH6[JoinHandle]
    end
    
    C1 -.-> JH1
    C2 -.-> JH2
    C3 -.-> JH3
    P1 -.-> JH4
    P2 -.-> JH5
    P3 -.-> JH6
```

### Component Relationships

```mermaid
classDiagram
    class Generator {
        <<trait>>
        +id() Uuid
        +name() Option~String~
        +is_active() bool
        +is_running() bool
        +is_completed() bool
        +step() async Result
    }
    
    class AsyncKernel {
        -base: GeneratorBase
        -root: Arc~Node~
        -time_frame: Arc~RwLock~TimeFrame~~
        -break_flag: Arc~RwLock~bool~~
        +run_until_complete() async Result
        +run_for(Duration) async Result
        +break_flow() async
    }
    
    class Node {
        -base: GeneratorBase
        -children: Arc~RwLock~Vec~Arc~Generator~~~~
        +add_child(Arc~Generator~) async
        +remove_child(Uuid) async bool
        +clear_completed() async
    }
    
    class Sequence {
        -base: GeneratorBase
        -children: Arc~RwLock~Vec~Arc~Generator~~~~
        -current_index: Arc~RwLock~usize~~
        +add_child(Arc~Generator~) async
    }
    
    class Barrier {
        -base: GeneratorBase
        -children: Arc~RwLock~Vec~Arc~Generator~~~~
        +add_child(Arc~Generator~) async
    }
    
    class AsyncCoroutine {
        -base: GeneratorBase
        -handle: Arc~Mutex~JoinHandle~Result~~~~
    }
    
    class Timer {
        -base: GeneratorBase
        -duration: Duration
        -start_time: Arc~RwLock~Instant~~
        -callback: Arc~RwLock~Callback~~
    }
    
    class Trigger {
        -base: GeneratorBase
        -condition: Arc~RwLock~Condition~~
        -callback: Arc~RwLock~Callback~~
    }
    
    class AsyncFuture~T~ {
        -base: GeneratorBase
        -inner: Arc~RwLock~Option~T~~~
        -notify: Arc~Notify~
        +set_value(T) async
        +wait() async T
    }
    
    Generator <|.. AsyncKernel
    Generator <|.. Node
    Generator <|.. Sequence
    Generator <|.. Barrier
    Generator <|.. AsyncCoroutine
    Generator <|.. Timer
    Generator <|.. Trigger
    Generator <|.. AsyncFuture
    
    AsyncKernel --> Node : contains root
    Node --> Generator : contains children
    Sequence --> Generator : executes sequentially
    Barrier --> Generator : waits for all
```

### Flow Execution Model

```mermaid
sequenceDiagram
    participant User
    participant Kernel as AsyncKernel
    participant Root as Root Node
    participant Seq as Sequence
    participant C1 as Coroutine1
    participant C2 as Coroutine2
    participant JH as JoinHandle
    
    User->>Kernel: run_until_complete()
    
    loop Until Complete
        Kernel->>Kernel: update_time()
        Kernel->>Root: step()
        Root->>Seq: step()
        
        alt Sequential Execution
            Seq->>C1: step()
            C1->>JH: check if finished
            alt Finished
                JH-->>C1: completed
                C1->>C1: complete()
                Seq->>C2: step()
            end
        end
        
        Kernel->>Root: clear_completed()
        Root->>Root: remove completed children
        
        alt All Complete
            Kernel-->>User: Ok(())
        else Continue
            Kernel->>Kernel: sleep(1ms)
        end
    end
```

### Thread-Free Coordination

```mermaid
graph LR
    subgraph "Single Tokio Executor"
        direction TB
        
        subgraph "AsyncKernel Loop"
            A[Update Time] --> B[Step Root]
            B --> C[Clear Completed]
            C --> D[Check Break]
            D --> E[Sleep 1ms]
            E --> A
        end
        
        subgraph "Async Tasks"
            T1[Task 1<br/>JoinHandle]
            T2[Task 2<br/>JoinHandle] 
            T3[Task 3<br/>JoinHandle]
        end
        
        subgraph "Coordination Primitives"
            RW1[RwLock<br/>Children]
            RW2[RwLock<br/>State]
            AT1[AtomicBool<br/>Flags]
            N1[Notify<br/>Futures]
        end
    end
    
    B --> RW1
    T1 -.-> RW2
    T2 -.-> AT1
    T3 -.-> N1
    
    classDef kernel fill:#e1f5fe
    classDef task fill:#f3e5f5
    classDef coord fill:#e8f5e8
    
    class A,B,C,D,E kernel
    class T1,T2,T3 task
    class RW1,RW2,AT1,N1 coord
```

This provides the same flow control capabilities as CsharpFlow while leveraging Rust's zero-cost async abstractions and memory safety guarantees.
