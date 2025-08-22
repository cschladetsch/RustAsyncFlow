"""Base Generator classes for AsyncFlow components"""

import asyncio
import uuid
from abc import ABC, abstractmethod
from typing import Optional
from .logger import Logger


class Generator(ABC):
    """Base interface for all flow components"""
    
    @property
    @abstractmethod
    def id(self) -> str:
        """Unique identifier for this generator"""
        pass
    
    @property
    @abstractmethod
    def name(self) -> Optional[str]:
        """Optional name for debugging"""
        pass
    
    @name.setter
    @abstractmethod
    def name(self, name: str) -> None:
        """Set the name of this generator"""
        pass
    
    @abstractmethod
    def is_active(self) -> bool:
        """Check if generator is active"""
        pass
    
    @abstractmethod
    def is_running(self) -> bool:
        """Check if generator is running"""
        pass
    
    @abstractmethod
    def is_completed(self) -> bool:
        """Check if generator has completed"""
        pass
    
    @abstractmethod
    def activate(self) -> None:
        """Activate this generator"""
        pass
    
    @abstractmethod
    def deactivate(self) -> None:
        """Deactivate this generator"""
        pass
    
    @abstractmethod
    def complete(self) -> None:
        """Mark this generator as completed"""
        pass
    
    @abstractmethod
    async def step(self) -> None:
        """Execute one step of this generator"""
        pass
    
    @property
    @abstractmethod
    def logger(self) -> Logger:
        """Get the logger for this generator"""
        pass


class GeneratorBase:
    """Base implementation for Generator interface"""
    
    def __init__(self, name: Optional[str] = None):
        self._id = str(uuid.uuid4())
        self._name = name
        self._active = True
        self._running = True
        self._completed = False
        self._logger = Logger("AsyncFlow")
    
    @property
    def id(self) -> str:
        return self._id
    
    @property
    def name(self) -> Optional[str]:
        return self._name
    
    @name.setter
    def name(self, name: str) -> None:
        self._name = name
    
    def is_active(self) -> bool:
        return self._active
    
    def is_running(self) -> bool:
        return self._running
    
    def is_completed(self) -> bool:
        return self._completed
    
    def activate(self) -> None:
        self._active = True
    
    def deactivate(self) -> None:
        self._active = False
    
    def complete(self) -> None:
        self._completed = True
        self._running = False
    
    @property
    def logger(self) -> Logger:
        return self._logger