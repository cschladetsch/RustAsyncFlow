"""Logging utilities for AsyncFlow"""

import logging
from typing import Optional


class Logger:
    """Simple logger wrapper for AsyncFlow components"""
    
    def __init__(self, prefix: str = "AsyncFlow", verbosity: int = 3):
        self.prefix = prefix
        self.verbosity = verbosity
        
        # Setup Python logging
        logging.basicConfig(
            level=logging.DEBUG,
            format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
        )
        self._logger = logging.getLogger(prefix)
    
    def error(self, message: str) -> None:
        """Log error message"""
        self._logger.error(f"[{self.prefix}] {message}")
    
    def warn(self, message: str) -> None:
        """Log warning message"""
        self._logger.warning(f"[{self.prefix}] {message}")
    
    def info(self, message: str) -> None:
        """Log info message"""
        self._logger.info(f"[{self.prefix}] {message}")
    
    def debug(self, message: str) -> None:
        """Log debug message"""
        self._logger.debug(f"[{self.prefix}] {message}")
    
    def verbose(self, level: int, message: str) -> None:
        """Log message at specific verbosity level"""
        if level <= self.verbosity:
            if level == 0:
                self.error(message)
            elif level == 1:
                self.warn(message)
            elif level == 2:
                self.info(message)
            else:
                self.debug(message)