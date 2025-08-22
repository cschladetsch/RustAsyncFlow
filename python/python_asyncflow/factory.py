"""Factory for creating AsyncFlow components"""

from typing import Callable, Optional, Awaitable, TypeVar
from .components import (
    AsyncCoroutine, Node, Sequence, Barrier,
    Timer, PeriodicTimer, Trigger, AsyncFuture
)

T = TypeVar('T')


class FlowFactory:
    """Factory for creating flow components"""
    
    @staticmethod
    def new_node(name: Optional[str] = None) -> Node:
        """Create a new Node"""
        return Node(name)
    
    @staticmethod
    def new_sequence(name: Optional[str] = None) -> Sequence:
        """Create a new Sequence"""
        return Sequence(name)
    
    @staticmethod
    def new_barrier(name: Optional[str] = None) -> Barrier:
        """Create a new Barrier"""
        return Barrier(name)
    
    @staticmethod
    def new_timer(duration: float, name: Optional[str] = None) -> Timer:
        """Create a new Timer"""
        return Timer(duration, name)
    
    @staticmethod
    def new_periodic_timer(interval: float, name: Optional[str] = None) -> PeriodicTimer:
        """Create a new PeriodicTimer"""
        return PeriodicTimer(interval, name)
    
    @staticmethod
    def new_trigger(condition: Callable[[], bool], name: Optional[str] = None) -> Trigger:
        """Create a new Trigger"""
        return Trigger(condition, name)
    
    @staticmethod
    def new_async_coroutine(coro: Awaitable[None], name: Optional[str] = None) -> AsyncCoroutine:
        """Create a new AsyncCoroutine"""
        return AsyncCoroutine(coro, name)
    
    @staticmethod
    def new_future(name: Optional[str] = None) -> AsyncFuture[T]:
        """Create a new AsyncFuture"""
        return AsyncFuture(name)