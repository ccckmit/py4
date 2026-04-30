# ==========================
# 1. 原生模組擴充 (Native Modules)
# ==========================
print("--- 1. Native module: math ---")
import math
print("math.pi =", math.pi)
print("math.sqrt(16) =", math.sqrt(16))
print("math.sqrt(2) =", math.sqrt(2))

# ==========================
# 2. F-Strings 字串插值
# ==========================
print("\n--- 2. F-Strings ---")
name = "Rust"
version = 4
# 測試變數替換
message = f"Hello {name}, welcome to py{version}!"
print(message)

# 測試在 F-String 內直接執行表達式（超強能力！）
radius = 5
print(f"Area of circle (r={radius}): {math.pi * radius * radius}")
print(f"List length test: {[1, 2, 3]} has {len([1, 2, 3])} items.")

# ==========================
# 3. 多重異常捕獲 (Multiple Exception Catching)
# ==========================
print("\n--- 3. Multiple Exception Catching ---")

def test_errors(val):
    try:
        if val == 1:
            raise TypeError("This is a TypeError!")
        elif val == 2:
            raise ValueError("This is a ValueError!")
        else:
            raise Exception("Some other error!")
    except (TypeError, ValueError) as e:
        print("Caught a specific error:", e)
    except Exception as e:
        print("Caught a generic error:", e)

test_errors(1)  # 應該觸發第一個 except
test_errors(2)  # 應該觸發第一個 except
test_errors(3)  # 應該觸發第二個 except