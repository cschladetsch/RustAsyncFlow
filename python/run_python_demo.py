"""
Cross-platform Python demo runner
Handles Python version detection and execution
"""

import sys
import os
import subprocess
import platform


def find_python():
    """Find the best Python executable"""
    # Try different Python commands
    python_commands = ['python3', 'python', 'py']
    
    for cmd in python_commands:
        try:
            # Check if command exists and version
            result = subprocess.run(
                [cmd, '--version'], 
                capture_output=True, 
                text=True, 
                timeout=10
            )
            
            if result.returncode == 0:
                version_line = result.stdout.strip() or result.stderr.strip()
                if 'Python 3.' in version_line:
                    # Extract version number
                    version = version_line.split()[1]
                    major, minor = map(int, version.split('.')[:2])
                    
                    if major == 3 and minor >= 7:
                        print(f"✅ Found {cmd}: {version_line}")
                        return cmd
                    else:
                        print(f"⚠️  {cmd} version {version} found, but need 3.7+")
                        
        except (subprocess.TimeoutExpired, subprocess.SubprocessError, FileNotFoundError):
            continue
            
    return None


def main():
    """Main runner function"""
    print("🐍 AsyncFlow Python Demo Runner")
    print("================================")
    print(f"Operating System: {platform.system()}")
    print(f"Platform: {platform.platform()}")
    print()
    
    # Find Python
    python_cmd = find_python()
    
    if not python_cmd:
        print("❌ Error: Could not find Python 3.7+ installation")
        print()
        print("Please install Python 3.7+ from:")
        print("  • Windows: https://python.org/downloads")
        print("  • macOS: brew install python3 or https://python.org/downloads")  
        print("  • Linux: apt install python3 (or equivalent)")
        return 1
    
    # Get demo script path
    demo_script = os.path.join(os.path.dirname(__file__), 'python_demo.py')
    
    if not os.path.exists(demo_script):
        print(f"❌ Error: Demo script not found: {demo_script}")
        return 1
    
    print("🚀 Starting AsyncFlow Python demo...")
    print()
    
    try:
        # Run the demo
        result = subprocess.run([python_cmd, demo_script], check=True)
        return result.returncode
        
    except subprocess.CalledProcessError as e:
        print(f"❌ Demo failed with exit code: {e.returncode}")
        return e.returncode
        
    except KeyboardInterrupt:
        print("\n⏹️  Demo interrupted by user")
        return 130
        
    except Exception as e:
        print(f"❌ Unexpected error: {e}")
        return 1


if __name__ == "__main__":
    sys.exit(main())