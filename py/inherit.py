# 1. 測試類別繼承 (Inheritance)
class Animal:
    def __init__(self, name):
        self.name = name
    
    def speak(self):
        return "I am an animal"

class Dog(Animal): # Dog 繼承自 Animal
    def speak(self):
        return self.name + " says Woof!"

dog = Dog("Rex")
print("Dog Speak:", dog.speak())

# 2. 測試重載陣列索引 (__getitem__ & __setitem__)
class MyDict:
    def __init__(self):
        self.data = {}
    
    def __setitem__(self, key, value):
        print("Setting", key, "to", value)
        self.data[key] = value

    def __getitem__(self, key):
        print("Getting", key)
        return self.data[key]

d = MyDict()
d["score"] = 100    # 觸發 __setitem__
val = d["score"]    # 觸發 __getitem__
print("Final value:", val)

# 3. 測試字串與列表的內建方法
text = "apple,banana,cherry"
fruits = text.split(",")
print("\nSplit Text:", fruits)

fruits.append("orange")
popped = fruits.pop()
print("Popped:", popped)

joined = " & ".join(fruits)
print("Joined Fruits:", joined)

# 4. 測試字典方法
info = {"a": 1, "b": 2}
print("Dict Keys:", info.keys())
print("Dict Values:", info.values())