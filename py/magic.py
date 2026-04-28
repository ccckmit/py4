# 1. 列表推導式 (List Comprehensions)
nums = [x * 2 for x in range(5) if x != 2]
print("List Comprehension:", nums)

# 2. Lambda 匿名函數
add_func = lambda x, y: x + y
print("Lambda function 10 + 5 =", add_func(10, 5))

# 3. 魔術方法 (__str__ 與 __add__)
class Vector:
    def __init__(self, x, y):
        self.x = x
        self.y = y

    def __str__(self):
        # 注意這裡可以用我們剛加上的 str() 內建函數了！
        return "Vector(" + str(self.x) + ", " + str(self.y) + ")"

    def __add__(self, other):
        return Vector(self.x + other.x, self.y + other.y)

v1 = Vector(1, 2)
v2 = Vector(3, 4)
v3 = v1 + v2

print("v1 is:", v1)
print("v2 is:", v2)
print("v1 + v2 is:", v3)