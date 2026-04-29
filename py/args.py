print("=== 1. 測試字串與陣列切片 (Slicing) ===")
text = "Hello, Python and Rust!"
print(f"Original text: '{text}'")
print(f"text[7:13]      -> '{text[7:13]}'")      # 擷取 'Python'
print(f"text[:5]        -> '{text[:5]}'")        # 擷取 'Hello'
print(f"text[-5:-1]     -> '{text[-5:-1]}'")     # 擷取 'Rust' (支援負數索引)
print(f"text[::-1]      -> '{text[::-1]}'")      # 字串反轉！(step=-1)

arr = [0, 10, 20, 30, 40, 50, 60]
print(f"\nOriginal array: {arr}")
print(f"arr[1:5:2]      -> {arr[1:5:2]}")        # 擷取 [10, 30] (step=2)
print(f"arr[::-1]       -> {arr[::-1]}")         # 陣列反轉！


print("\n=== 2. 測試預設參數 (Default Arguments) ===")
def greet(name, greeting="Hello", punctuation="!"):
    return f"{greeting}, {name}{punctuation}"

print("greet('Alice')                     ->", greet("Alice"))
# 注意：目前我們的架構還不支援呼叫時使用 Keyword arguments (greet(name="Alice"))
# 但支援依照順序覆蓋預設值
print("greet('Bob', 'Hi')                 ->", greet("Bob", "Hi"))
print("greet('Charlie', 'Welcome', '...') ->", greet("Charlie", "Welcome", "..."))


print("\n=== 3. 測試不定長度參數 (*args) ===")
def sum_all(first, *rest):
    total = first
    for num in rest:
        total += num
    return f"First: {first}, Rest: {rest}, Total Sum: {total}"

print("sum_all(10, 20, 30, 40) ->")
print("  " + sum_all(10, 20, 30, 40))

print("\nsum_all(100) ->")
print("  " + sum_all(100)) # rest 應該要是空 Tuple ()


print("\n=== 4. 測試多重賦值與交換變數 (Tuple Unpacking) ===")
a, b, c = 1, 2, 3
print(f"Before: a={a}, b={b}, c={c}")

# 經典的 Python 變數交換
a, b, c = c, a, b
print(f"After swap (c, a, b): a={a}, b={b}, c={c}")


print("\n=== 5. 測試裝飾器與 *args 組合 (Decorator) ===")
# 寫一個可以印出參數的日誌裝飾器
def logger(func):
    def wrapper(*args):
        print(f"[LOG] Calling function with args: {args}")
        return func(*args) # 解構 *args 呼叫尚未支援，目前只能傳遞 Tuple，這是一個妥協寫法
    return wrapper

@logger
def multiply(nums):
    # 因為目前不支援解構呼叫 func(*args)，我們接收一個 tuple
    a, b = nums[0], nums[1]
    return a * b

result = multiply( (5, 6) )
print(f"Result: {result}")