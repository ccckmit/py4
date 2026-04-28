# 1. 測試元組 (Tuples) 及其不可變性與解構
tup = (10, 20, "Hello")
print("Tuple:", tup)
print("Tuple index 1:", tup[1])

# 2. 測試物件導向 OOP (類別、實例化、__init__、self、綁定方法)
class Dog:
    def __init__(self, name, age):
        self.name = name
        self.age = age

    def bark(self):
        print(self.name, "says Woof! I am", self.age, "years old.")

    def birthday(self):
        self.age += 1

dog1 = Dog("Buddy", 3)
dog1.bark()
dog1.birthday()
dog1.bark()

# 3. 測試例外處理 (Try/Except/Raise)
def divide(a, b):
    if b == 0:
        raise Exception("Division by zero!")
    return a / b

print("Attempting to divide...")
try:
    print("10 / 2 =", divide(10, 2))
    print("10 / 0 =", divide(10, 0)) # 這會觸發 raise
    print("This will not print")
except Exception as e:
    print("Caught an error:", e)

print("Program finished successfully.")