import os
import sys
import time
import json
import math

print("========== 1. sys ==========")
print("sys.argv:", sys.argv)
# 如果你有傳參數，它會印出 ['test_stdlib.py', 'Hello', 'World']

print("\n========== 2. os ==========")
# 讀取環境變數 (預設 fallback 測試)
user = os.getenv("USER", "Unknown")
print("Current User:", user)
print("Running a simple shell command (ls -l):")
os.system("ls -l | head -n 3")

print("\n========== 3. json ==========")
# 測試 JSON Dump
data = {
    "name": "Interpreter",
    "version": 4.0,
    "is_fast": True,
    "features": ["F-Strings", "Standard Library", None]
}
json_str = json.dumps(data)
print("Dumped JSON:", json_str)

# 測試 JSON Load (注意 true/false/null 的完美解析)
loaded_data = json.loads('{"hello": "world", "status": true, "error": null}')
print("Loaded Status is True:", loaded_data["status"] == True)
print("Loaded Data:", loaded_data)

print("\n========== 4. time & math ==========")
start_time = time.time()
print(f"Start time: {start_time}")

# 假裝做一些運算
print("Calculating sqrt(256)...")
time.sleep(0.5) # 暫停 0.5 秒
print("Result:", math.sqrt(256))

end_time = time.time()
print(f"End time: {end_time}")
print(f"Elapsed: {end_time - start_time} seconds")