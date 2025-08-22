"""Time management for AsyncFlow"""

import time
from typing import Optional


class TimeFrame:
    """Manages timing information for the flow system"""
    
    def __init__(self):
        self.now = time.time()
        self.last = self.now
        self.delta = 0.0
    
    def update(self) -> None:
        """Update time frame with current time"""
        now = time.time()
        self.last = self.now
        self.delta = now - self.now
        self.now = now
    
    def update_with_delta(self, delta: float) -> None:
        """Update time frame with specified delta"""
        self.last = self.now
        self.delta = delta
        self.now = self.last + delta