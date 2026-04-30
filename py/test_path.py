import sys
import os

print("1. Default sys.path:")
print(sys.path)

# 動態加入路徑
custom_dir = "/tmp/my_py4_modules"
print(f"\n2. Appending {custom_dir} to sys.path...")
sys.path.append(custom_dir)
print(sys.path)

# 測試機制: 嘗試建立一個假模組並 Import 它
os.system(f"mkdir -p {custom_dir}")
os.system(f"echo 'def greet(): return \"Hello from /tmp!\"' > {custom_dir}/dummy_pkg.py")

print("\n3. Importing dummy_pkg...")
import dummy_pkg
print(dummy_pkg.greet())