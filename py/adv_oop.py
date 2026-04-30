# ==========================
# 1. 呼叫父類別方法 (Class Method Access)
# ==========================
print("--- 1. Class Method Calling (Explicit Super) ---")
class Animal:
    def __init__(self, name):
        print("Animal initializing...")
        self.name = name

    def speak(self):
        return "Generic Animal Sound"

class Dog(Animal):
    def __init__(self, name, breed):
        # 這是 Python 經典的手動呼叫父類方法寫法！
        Animal.__init__(self, name)
        self.breed = breed

    def speak(self):
        parent_sound = Animal.speak(self)
        return parent_sound + " but mostly Woof!"

dog = Dog("Rex", "Golden Retriever")
print("Dog Name:", dog.name)
print("Dog Breed:", dog.breed)
print("Dog Speak:", dog.speak())


# ==========================
# 2. **kwargs 關鍵字參數字典
# ==========================
print("\n--- 2. **kwargs Variable Keywords ---")
def build_profile(first, last, **user_info):
    profile = {"first_name": first, "last_name": last}
    for key, val in user_info.items():
        profile[key] = val
    return profile

# 呼叫時傳入多餘的關鍵字，會自動被包成 user_info 字典
my_profile = build_profile("Albert", "Einstein", location="Princeton", field="Physics")
print("Profile:", my_profile)


# ==========================
# 3. 內省與型別檢查 (type & isinstance)
# ==========================
print("\n--- 3. Introspection (type & isinstance) ---")
print("type(42):", type(42))
print("type('Hello'):", type("Hello"))
print("type(dog):", type(dog))
print("type(Dog):", type(Dog))

# 繼承檢查
print("isinstance(dog, Dog):", isinstance(dog, Dog))       # True
print("isinstance(dog, Animal):", isinstance(dog, Animal)) # True (遞迴尋找父類成功)

class Cat:
    pass
print("isinstance(dog, Cat):", isinstance(dog, Cat))       # False