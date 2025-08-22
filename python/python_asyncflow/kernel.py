"""AsyncKernel - Main execution engine for AsyncFlow"""

import asyncio
import time
from typing import Optional
from .generator import Generator, GeneratorBase
from .components import Node
from .time_frame import TimeFrame


class AsyncKernel(GeneratorBase, Generator):
    """Main execution kernel for the flow system"""
    
    def __init__(self, name: Optional[str] = None):
        super().__init__(name or "AsyncKernel")
        self._root = Node("Root")
        self._time_frame = TimeFrame()
        self._break_flag = False
        self._wait_until: Optional[float] = None
    
    @property
    def root(self) -> Node:
        """Get the root node"""
        return self._root
    
    @property
    def time_frame(self) -> TimeFrame:
        """Get current time frame"""
        return self._time_frame
    
    def break_flow(self) -> None:
        """Stop execution"""
        self._break_flag = True
    
    def is_breaking(self) -> bool:
        """Check if break was requested"""
        return self._break_flag
    
    def wait(self, duration: float) -> None:
        """Pause execution for duration"""
        self._wait_until = time.time() + duration
    
    def is_waiting(self) -> bool:
        """Check if currently waiting"""
        if self._wait_until is None:
            return False
        return time.time() < self._wait_until
    
    def clear_wait(self) -> None:
        """Clear wait state"""
        self._wait_until = None
    
    async def update(self, delta_time: float) -> None:
        """Update kernel with time delta"""
        self._time_frame.update_with_delta(delta_time)
        await self.step()
    
    async def update_real_time(self) -> None:
        """Update kernel with real time"""
        self._time_frame.update()
        await self.step()
    
    async def run_until_complete(self) -> None:
        """Run until all tasks complete"""
        while self.is_running() and not self.is_breaking():
            if self.is_waiting():
                await asyncio.sleep(0.001)
                continue
            
            await self.update_real_time()
            
            if self._root.child_count() == 0:
                break
            
            await asyncio.sleep(0.001)
    
    async def run_for(self, duration: float) -> None:
        """Run for specified duration"""
        start_time = time.time()
        
        while self.is_running() and not self.is_breaking():
            if time.time() - start_time >= duration:
                break
            
            if self.is_waiting():
                await asyncio.sleep(0.001)
                continue
            
            await self.update_real_time()
            await asyncio.sleep(0.001)
    
    async def step(self) -> None:
        """Execute one step of the kernel"""
        if not self.is_active() or not self.is_running() or self.is_completed():
            return
        
        if self.is_breaking():
            return
        
        if self.is_waiting():
            return
        
        child_count = self._root.child_count()
        if child_count > 0:
            self.logger.verbose(4, f"Stepping kernel with {child_count} root children")
        
        await self._root.step()
        self._root.clear_completed()