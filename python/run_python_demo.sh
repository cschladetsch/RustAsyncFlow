#!/bin/bash
# Cross-platform shell script to run Python AsyncFlow demo
# Works on Linux, macOS, and Windows (with WSL/Git Bash)

echo "🐍 AsyncFlow Python Demo (Unix/Linux/macOS)"
echo "============================================="

# Function to check Python version
check_python() {
    local cmd=$1
    if command -v "$cmd" >/dev/null 2>&1; then
        local version=$($cmd --version 2>&1 | grep -o 'Python [0-9]\+\.[0-9]\+' | cut -d' ' -f2)
        if [[ ! -z "$version" ]]; then
            local major=$(echo $version | cut -d. -f1)
            local minor=$(echo $version | cut -d. -f2)
            if [[ $major -eq 3 && $minor -ge 7 ]]; then
                echo "✅ Using $cmd (Python $version)"
                return 0
            else
                echo "⚠️  $cmd found (Python $version), but need 3.7+"
            fi
        fi
    fi
    return 1
}

# Try different Python commands in order of preference
if check_python python3; then
    python3 run_python_demo.py
elif check_python python; then
    python run_python_demo.py  
elif check_python py; then
    py run_python_demo.py
else
    echo "❌ Error: Python 3.7+ not found"
    echo ""
    echo "Please install Python 3.7+ using:"
    echo "  • Ubuntu/Debian: apt install python3"
    echo "  • CentOS/RHEL: yum install python3"
    echo "  • macOS: brew install python3"
    echo "  • Or download from: https://python.org/downloads"
    exit 1
fi