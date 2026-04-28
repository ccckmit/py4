# 測試串列與方法
lst = [1, 2, 3]
lst.append(4)
print("List:", lst)

# 測試字典與索引賦值
d = {"name": "Alice"}
d["age"] = 25
print("Dict:", d)
print("Dict name:", d["name"])

# 測試 for 迴圈與 range
sum = 0
for x in range(5):
    if x == 2:
        continue
    sum += x
print("Sum (0+1+3+4) =", sum)

# 測試 elif 與邏輯運算 (短路求值)
val = 10
if val > 20 and print("This should not print"):
    print("Wrong")
elif val == 10 or print("This should not print"):
    print("Correct, short-circuit works!")

# 測試 while 與 break
i = 0
while True:
    i += 1
    if i == 3:
        break
print("While loop broke at i =", i)