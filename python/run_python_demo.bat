@echo off
REM Windows batch file to run Python AsyncFlow demo
REM Works with any Python 3.7+ installation

echo 🐍 AsyncFlow Python Demo (Windows)
echo ===================================

REM Try different Python commands
python --version >nul 2>&1
if %errorlevel% == 0 (
    echo Using: python
    python run_python_demo.py
    goto :end
)

python3 --version >nul 2>&1
if %errorlevel% == 0 (
    echo Using: python3
    python3 run_python_demo.py
    goto :end
)

py --version >nul 2>&1
if %errorlevel% == 0 (
    echo Using: py
    py run_python_demo.py
    goto :end
)

echo ❌ Error: Python 3.7+ not found
echo Please install Python from https://python.org/downloads
echo Make sure to check "Add Python to PATH" during installation
pause

:end
pause