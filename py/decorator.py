# 1. 測試多重賦值 (Tuple Unpacking) 與交換變數
a, b = 10, 20
print(f"Before Swap: a={a}, b={b}")
a, b = b, a
print(f"After Swap: a={a}, b={b}")

# 2. 測試 for 迴圈內的解構
pairs = [(1, "apple"), (2, "banana")]
for num, fruit in pairs:
    print(f"Item #{num} is {fruit}")

# 3. 測試裝飾器 (Decorators)
def bold(func):
    def wrapper():
        return "**" + func() + "**"
    return wrapper

def italic(func):
    def wrapper():
        return "*" + func() + "*"
    return wrapper

@bold
@italic
def greet():
    return "Hello World"

print("Decorated Greeting:", greet())