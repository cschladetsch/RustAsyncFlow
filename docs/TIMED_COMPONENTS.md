# AsyncFlow Timed Components Guide

This guide demonstrates the powerful timing capabilities of AsyncFlow through comprehensive examples and patterns.

## Overview

AsyncFlow provides several timing components that can be combined to create sophisticated timing behaviors:

- **Timer**: One-shot timer that completes after a specified duration
- **PeriodicTimer**: Repeating timer that fires at regular intervals
- **Trigger**: Condition-based component that fires when criteria are met
- **Barrier**: Synchronization component that waits for all children to complete

## Core Timing Components

### Timer

A Timer executes once after a specified duration and then completes.

```rust
let timer = FlowFactory::new_timer_with_name(
    "MyTimer",
    Duration::from_secs(2)
);

timer.set_elapsed_callback(|| {
    println!("Timer elapsed!");
}).await;
```

**Key Features:**
- One-shot execution
- Configurable duration
- Optional callback on completion
- Integrates with AsyncFlow lifecycle

### PeriodicTimer

A PeriodicTimer repeatedly fires at regular intervals until the flow completes.

```rust
let periodic = FlowFactory::new_periodic_timer_with_name(
    "Heartbeat",
    Duration::from_millis(500)
);

periodic.set_elapsed_callback(|| {
    println!("Heartbeat tick");
}).await;
```

**Key Features:**
- Continuous execution at intervals
- Precise timing control
- Useful for heartbeats, monitoring, and periodic tasks

### Trigger

A Trigger monitors a condition and fires once when the condition becomes true.

```rust
let counter = Arc::new(AtomicU32::new(0));
let trigger = FlowFactory::new_trigger_with_name(
    "CounterTrigger",
    {
        let counter = counter.clone();
        move || counter.load(Ordering::Relaxed) >= 10
    }
);

trigger.set_triggered_callback(|| {
    println!("Counter reached 10!");
}).await;
```

**Key Features:**
- Condition-based activation
- One-time firing per condition
- Flexible condition functions

## Timing Patterns

### 1. Basic Timer with Trigger

Combine timers with triggers for simple timing logic:

```rust
// Timer sets a flag when it completes
let timer_done = Arc::new(AtomicBool::new(false));
let timer = FlowFactory::new_timer_with_name("Timer", Duration::from_secs(1));

let timer_done_clone = timer_done.clone();
timer.set_elapsed_callback(move || {
    timer_done_clone.store(true, Ordering::Relaxed);
}).await;

// Trigger waits for the timer flag
let completion_trigger = FlowFactory::new_trigger_with_name(
    "CompletionTrigger",
    {
        let timer_done = timer_done.clone();
        move || timer_done.load(Ordering::Relaxed)
    }
);
```

### 2. Timeout Pattern

Implement timeout behavior by racing work against a timer:

```rust
let work_completed = Arc::new(AtomicBool::new(false));
let timeout_occurred = Arc::new(AtomicBool::new(false));

// Work task
let work_task = FlowFactory::new_async_coroutine_with_name(
    "WorkTask",
    {
        let work_completed = work_completed.clone();
        async move {
            // Simulate work
            sleep(Duration::from_millis(700)).await;
            work_completed.store(true, Ordering::Relaxed);
            Ok(())
        }
    }
);

// Timeout timer
let timeout_timer = FlowFactory::new_timer_with_name(
    "TimeoutTimer",
    Duration::from_millis(500)
);

let timeout_occurred_clone = timeout_occurred.clone();
timeout_timer.set_elapsed_callback(move || {
    timeout_occurred_clone.store(true, Ordering::Relaxed);
}).await;

// Success/timeout triggers determine outcome
```

### 3. Heartbeat Monitoring

Use periodic timers with health checks:

```rust
let heartbeat_count = Arc::new(AtomicU32::new(0));
let system_healthy = Arc::new(AtomicBool::new(true));

// Heartbeat every 200ms
let heartbeat_timer = FlowFactory::new_periodic_timer_with_name(
    "Heartbeat",
    Duration::from_millis(200)
);

let heartbeat_count_clone = heartbeat_count.clone();
heartbeat_timer.set_elapsed_callback(move || {
    let count = heartbeat_count_clone.fetch_add(1, Ordering::Relaxed) + 1;
    println!("Heartbeat #{}", count);
}).await;

// Health monitoring with separate timer
let health_timer = FlowFactory::new_periodic_timer_with_name(
    "HealthCheck",
    Duration::from_millis(600)
);

// Stop on unhealthy condition or max heartbeats
```

### 4. Cascading Timer Chain

Create sequential timing where each timer activates the next:

```rust
let stage1_complete = Arc::new(AtomicBool::new(false));
let stage2_complete = Arc::new(AtomicBool::new(false));

// Stage 1 timer
let stage1_timer = FlowFactory::new_timer_with_name(
    "Stage1",
    Duration::from_millis(300)
);

// Stage 2 starts when Stage 1 completes
let stage2_trigger = FlowFactory::new_trigger_with_name(
    "Stage2Start",
    {
        let stage1_complete = stage1_complete.clone();
        move || stage1_complete.load(Ordering::Relaxed)
    }
);

// Continue the chain...
```

## Barrier Synchronization Patterns

### 1. Parallel Timer Execution

Use barriers to wait for multiple timers to complete:

```rust
let barrier = FlowFactory::new_barrier_with_name("TimedBarrier");

// Create multiple timers with different durations
let timers_data = vec![
    ("Timer1", 500),
    ("Timer2", 300),
    ("Timer3", 800),
];

for (name, duration_ms) in timers_data {
    let timer = FlowFactory::new_timer_with_name(
        name,
        Duration::from_millis(duration_ms)
    );
    barrier.add_child(timer).await;
}

// Barrier completes when ALL timers finish (after 800ms)
```

### 2. Staged Barriers

Create multi-stage execution with barriers:

```rust
// Stage 1: Fast timers (100-300ms)
let stage1_barrier = FlowFactory::new_barrier_with_name("Stage1");
// Add fast timers...

// Stage 2: Medium timers (400-600ms) 
let stage2_barrier = FlowFactory::new_barrier_with_name("Stage2");
// Add medium timers...

// Sequence the stages
let sequence = FlowFactory::new_sequence_with_name("StagedSequence");
sequence.add_child(stage1_barrier).await;
sequence.add_child(stage2_barrier).await;
```

### 3. Mixed Component Barriers

Combine different component types in barriers:

```rust
let mixed_barrier = FlowFactory::new_barrier_with_name("MixedBarrier");

// Regular timer
let timer = FlowFactory::new_timer_with_name("Timer", Duration::from_millis(600));

// Periodic timer (controlled by external logic)
let periodic = FlowFactory::new_periodic_timer_with_name("Periodic", Duration::from_millis(150));

// Trigger waiting for condition
let trigger = FlowFactory::new_trigger_with_name("Trigger", || condition_check());

// Async task
let task = FlowFactory::new_async_coroutine_with_name("Task", async { /* work */ Ok(()) });

// All must complete for barrier to finish
mixed_barrier.add_child(timer).await;
mixed_barrier.add_child(periodic).await;
mixed_barrier.add_child(trigger).await;
mixed_barrier.add_child(task).await;
```

## Advanced Patterns

### Timer Orchestration

Create complex timing orchestrations with dynamic barriers:

```rust
// Wave 1: Initial timers
let wave1_barrier = FlowFactory::new_barrier_with_name("Wave1");
// Add wave1 timers...

// Wave 2: Triggered by Wave 1 completion
let wave2_trigger = FlowFactory::new_trigger_with_name(
    "Wave2Start",
    move || wave1_complete.load(Ordering::Relaxed)
);

let wave2_barrier = FlowFactory::new_barrier_with_name("Wave2");
// Add wave2 timers...

// Sequence waves for orchestrated execution
```

### Conditional Timing

Use triggers to create conditional timing behaviors:

```rust
let condition = Arc::new(AtomicBool::new(false));

// Timer only starts when condition is met
let conditional_trigger = FlowFactory::new_trigger_with_name(
    "ConditionalStart",
    {
        let condition = condition.clone();
        move || condition.load(Ordering::Relaxed)
    }
);

// Timer begins after trigger fires
```

## Running the Demos

Several comprehensive demos showcase these patterns:

### Basic Timed Triggers
```bash
cargo run --example timed_trigger_demo
```

Demonstrates:
- Timer with completion trigger
- Periodic timer with counter trigger  
- Multiple timers with conditional triggers

### Timed Barriers
```bash
cargo run --example timed_barrier_demo
```

Demonstrates:
- Barrier with multiple timed tasks
- Staged timed barriers
- Mixed component barriers

### Advanced Timing Patterns
```bash
cargo run --example advanced_timing_demo
```

Demonstrates:
- Timeout patterns with race conditions
- Heartbeat monitoring with health checks
- Cascading timer chains
- Dynamic timer orchestration

## Testing

Comprehensive tests verify timing behavior:

```bash
cargo test timed_components_tests
```

Tests include:
- Basic timer functionality and timing accuracy
- Periodic timer tick counting and intervals
- Trigger activation with timer conditions
- Barrier synchronization with multiple timers
- Race conditions and timeout patterns
- Cascading timer sequences
- Mixed component barriers

## Best Practices

1. **Timing Accuracy**: Allow timing tolerances in tests due to system scheduling
2. **Resource Management**: Use triggers to properly terminate periodic timers
3. **State Management**: Use atomic types for thread-safe condition sharing
4. **Error Handling**: Handle potential timing edge cases gracefully
5. **Performance**: Consider timer granularity vs. system overhead

## Common Pitfalls

- **Timing Assumptions**: Don't assume exact timing in concurrent systems
- **Memory Leaks**: Ensure periodic timers have completion conditions
- **Race Conditions**: Use proper synchronization for shared state
- **Callback Lifetimes**: Ensure captured variables live long enough

The timed components in AsyncFlow provide a powerful foundation for building complex, time-aware asynchronous workflows with precise control over timing behavior and synchronization.