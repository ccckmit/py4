# ==========================
# 測試 1: 預設參數 (Default Arguments)
# ==========================
print("--- 1. Default Arguments ---")
def greet(name, msg="Hello", punct="!"):
    print(msg + ", " + name + punct)
    
greet("Alice")                     # 使用所有預設值
greet("Bob", "Hi")                 # 覆蓋第一個預設值
greet("Charlie", "Welcome", "!!")  # 覆蓋所有預設值

# ==========================
# 測試 2: 關鍵字參數 (Keyword Arguments)
# ==========================
print("\n--- 2. Keyword Arguments ---")
def describe_person(name, age=20, city="Taipei"):
    print(name, "is", age, "years old, lives in", city)

describe_person("David", city="Tainan")           # 位置參數 + 關鍵字參數
describe_person(age=35, name="Eve")               # 全關鍵字參數 (順序可以不同)
describe_person(city="Kaohsiung", name="Frank")   # 混用與預設值填補

# ==========================
# 測試 3: 不定長度參數 (*args)
# ==========================
print("\n--- 3. Variable-Length Arguments (*args) ---")
def sum_all(first, *args):
    total = first
    for n in args:
        total += n
    return total

print("Sum (10):", sum_all(10))                               # 沒有額外的 args
print("Sum (10, 20, 30):", sum_all(10, 20, 30))               # 多個 args
print("Sum (1, 2, 3, 4, 5):", sum_all(1, 2, 3, 4, 5))         # 更多 args

# ==========================
# 測試 4: 綜合測試 (Positional + Default + *args + Kwargs)
# ==========================
print("\n--- 4. Mixed Test ---")
def mixed_test(a, b=2, *args):
    print("a:", a, "| b:", b, "| args:", args)

mixed_test(1)                        # a=1, b=2 (default), args=()
mixed_test(1, 5)                     # a=1, b=5, args=()
mixed_test(1, 5, 10, 11, 12)         # a=1, b=5, args=(10, 11, 12)
mixed_test(a=100, b=200)             # 完全使用 keyword

# ==========================
# 測試 5: 錯誤處理 (Error Handling)
# ==========================
print("\n--- 5. Error Catching ---")
try:
    describe_person() # 缺少 required 的 name
except Exception as e:
    print("Caught:", e)

try:
    describe_person("Alice", unknown="???") # 丟入未知的 keyword
except Exception as e:
    print("Caught:", e)

try:
    describe_person("Alice", name="Alice2") # 給予多重值 (positional 已經佔了 name)
except Exception as e:
    print("Caught:", e)