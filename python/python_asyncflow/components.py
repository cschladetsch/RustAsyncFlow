"""Core AsyncFlow components implemented using asyncio"""

import asyncio
import time
from typing import List, Optional, Callable, Any, TypeVar, Generic, Awaitable
from .generator import Generator, GeneratorBase

T = TypeVar('T')


class AsyncCoroutine(GeneratorBase, Generator):
    """Wraps an async coroutine for execution in the flow system"""
    
    def __init__(self, coro: Awaitable[None], name: Optional[str] = None):
        super().__init__(name)
        self._task: Optional[asyncio.Task] = None
        self._coro = coro
    
    async def step(self) -> None:
        """Execute one step of the coroutine"""
        if not self.is_active() or not self.is_running() or self.is_completed():
            return
        
        if self.name:
            self.logger.verbose(4, f"Stepping coroutine: {self.name}")
        
        if self._task is None:
            self._task = asyncio.create_task(self._coro)
        
        if self._task.done():
            try:
                await self._task  # Get any exception
                self.complete()
            except Exception as e:
                self.logger.error(f"Coroutine failed: {e}")
                self.complete()


class Node(GeneratorBase, Generator):
    """Container for child generators"""
    
    def __init__(self, name: Optional[str] = None):
        super().__init__(name)
        self._children: List[Generator] = []
    
    def add_child(self, child: Generator) -> None:
        """Add a child generator"""
        self._children.append(child)
    
    def remove_child(self, child_id: str) -> bool:
        """Remove a child by ID"""
        for i, child in enumerate(self._children):
            if child.id == child_id:
                del self._children[i]
                return True
        return False
    
    def child_count(self) -> int:
        """Get number of children"""
        return len(self._children)
    
    def clear_completed(self) -> None:
        """Remove completed children"""
        self._children = [child for child in self._children if not child.is_completed()]
    
    async def step(self) -> None:
        """Step all active children"""
        if not self.is_active() or not self.is_running() or self.is_completed():
            return
        
        if not self._children:
            return
        
        if len(self._children) > 0:
            self.logger.verbose(4, f"Stepping node with {len(self._children)} children")
        
        for child in self._children:
            if child.is_active() and child.is_running() and not child.is_completed():
                try:
                    await child.step()
                except Exception as e:
                    self.logger.error(f"Child step failed: {e}")


class Sequence(GeneratorBase, Generator):
    """Executes children sequentially"""
    
    def __init__(self, name: Optional[str] = None):
        super().__init__(name)
        self._children: List[Generator] = []
        self._current_index = 0
    
    def add_child(self, child: Generator) -> None:
        """Add a child generator"""
        self._children.append(child)
    
    def current_index(self) -> int:
        """Get current child index"""
        return self._current_index
    
    def child_count(self) -> int:
        """Get number of children"""
        return len(self._children)
    
    async def step(self) -> None:
        """Step current child in sequence"""
        if not self.is_active() or not self.is_running() or self.is_completed():
            return
        
        if not self._children:
            self.complete()
            return
        
        if self._current_index >= len(self._children):
            self.complete()
            return
        
        current_child = self._children[self._current_index]
        
        if current_child.is_completed():
            self._current_index += 1
            if self._current_index >= len(self._children):
                self.complete()
        elif current_child.is_active() and current_child.is_running():
            try:
                await current_child.step()
            except Exception as e:
                self.logger.error(f"Child step failed in sequence: {e}")


class Barrier(GeneratorBase, Generator):
    """Waits for all children to complete"""
    
    def __init__(self, name: Optional[str] = None):
        super().__init__(name)
        self._children: List[Generator] = []
    
    def add_child(self, child: Generator) -> None:
        """Add a child generator"""
        self._children.append(child)
    
    def child_count(self) -> int:
        """Get number of children"""
        return len(self._children)
    
    def all_children_completed(self) -> bool:
        """Check if all children are completed"""
        return all(child.is_completed() for child in self._children)
    
    async def step(self) -> None:
        """Step all children and check for completion"""
        if not self.is_active() or not self.is_running() or self.is_completed():
            return
        
        if not self._children:
            self.complete()
            return
        
        for child in self._children:
            if child.is_active() and child.is_running() and not child.is_completed():
                try:
                    await child.step()
                except Exception as e:
                    self.logger.error(f"Child step failed in barrier: {e}")
        
        if self.all_children_completed():
            self.complete()


class Timer(GeneratorBase, Generator):
    """One-shot timer with callback"""
    
    def __init__(self, duration: float, name: Optional[str] = None):
        super().__init__(name)
        self.duration = duration
        self._start_time: Optional[float] = None
        self._elapsed_callback: Optional[Callable[[], None]] = None
    
    def set_elapsed_callback(self, callback: Callable[[], None]) -> None:
        """Set callback to call when timer elapses"""
        self._elapsed_callback = callback
    
    def is_elapsed(self) -> bool:
        """Check if timer has elapsed"""
        if self._start_time is None:
            return False
        return time.time() - self._start_time >= self.duration
    
    def _start_if_needed(self) -> None:
        """Start timer if not already started"""
        if self._start_time is None:
            self._start_time = time.time()
    
    async def step(self) -> None:
        """Check timer and fire callback if elapsed"""
        if not self.is_active() or not self.is_running() or self.is_completed():
            return
        
        self._start_if_needed()
        
        if self.is_elapsed():
            if self._elapsed_callback:
                self._elapsed_callback()
            self.complete()


class PeriodicTimer(GeneratorBase, Generator):
    """Repeating timer with callback"""
    
    def __init__(self, interval: float, name: Optional[str] = None):
        super().__init__(name)
        self.interval = interval
        self._last_trigger: Optional[float] = None
        self._elapsed_callback: Optional[Callable[[], None]] = None
    
    def set_elapsed_callback(self, callback: Callable[[], None]) -> None:
        """Set callback to call on each interval"""
        self._elapsed_callback = callback
    
    def should_trigger(self) -> bool:
        """Check if timer should trigger"""
        if self._last_trigger is None:
            return True
        return time.time() - self._last_trigger >= self.interval
    
    def trigger(self) -> None:
        """Mark timer as triggered"""
        self._last_trigger = time.time()
    
    async def step(self) -> None:
        """Check timer and fire callback if interval elapsed"""
        if not self.is_active() or not self.is_running() or self.is_completed():
            return
        
        if self.should_trigger():
            if self._elapsed_callback:
                self._elapsed_callback()
            self.trigger()


class Trigger(GeneratorBase, Generator):
    """Fires when condition becomes true"""
    
    def __init__(self, condition: Callable[[], bool], name: Optional[str] = None):
        super().__init__(name)
        self._condition = condition
        self._triggered_callback: Optional[Callable[[], None]] = None
        self._triggered = False
    
    def set_triggered_callback(self, callback: Callable[[], None]) -> None:
        """Set callback to call when triggered"""
        self._triggered_callback = callback
    
    def is_triggered(self) -> bool:
        """Check if trigger has fired"""
        return self._triggered
    
    def check_condition(self) -> bool:
        """Check the trigger condition"""
        return self._condition()
    
    def trigger(self) -> None:
        """Mark trigger as fired"""
        self._triggered = True
    
    async def step(self) -> None:
        """Check condition and fire if true"""
        if not self.is_active() or not self.is_running() or self.is_completed():
            return
        
        if self.check_condition():
            if not self.is_triggered():
                if self._triggered_callback:
                    self._triggered_callback()
                self.trigger()
            self.complete()


class AsyncFuture(GeneratorBase, Generator, Generic[T]):
    """Thread-safe future value"""
    
    def __init__(self, name: Optional[str] = None):
        super().__init__(name)
        self._value: Optional[T] = None
        self._event = asyncio.Event()
        self._has_value = False
    
    async def set_value(self, value: T) -> None:
        """Set the future value"""
        self._value = value
        self._has_value = True
        self._event.set()
        self.complete()
    
    async def get_value(self) -> Optional[T]:
        """Get the current value (may be None)"""
        return self._value
    
    async def wait(self) -> T:
        """Wait for the value to be set"""
        await self._event.wait()
        return self._value
    
    def is_ready(self) -> bool:
        """Check if value is available"""
        return self.is_completed()
    
    async def step(self) -> None:
        """Check if value has been set"""
        if not self.is_active() or not self.is_running() or self.is_completed():
            return
        
        if self._has_value:
            self.complete()