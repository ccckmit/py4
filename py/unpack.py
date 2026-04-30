# ==========================
# 1. 基礎解包與變數交換
# ==========================
print("--- 1. Basic Unpacking ---")
a, b = 10, 20
print("Initial: a =", a, ", b =", b)

# 最經典的 Python 變數交換！
a, b = b, a
print("Swapped: a =", a, ", b =", b)

# ==========================
# 2. 嵌套清單 / 元組解包
# ==========================
print("\n--- 2. Nested Unpacking ---")
info = ["Alice", (25, "Engineer")]
name, (age, job) = info
print("Name:", name, "| Age:", age, "| Job:", job)

# 解包字串 (將字串轉為字元列表)
first, second, third = "CAT"
print("String unpack:", first, second, third)

# ==========================
# 3. 迴圈中的解包與 Dict.items()
# ==========================
print("\n--- 3. For Loop Unpacking & items() ---")
scores = {"Math": 90, "English": 85, "Science": 95}

for subject, score in scores.items():
    print(subject, "->", score)

# ==========================
# 4. 列表推導式 (List Comprehension) 結合解包
# ==========================
print("\n--- 4. Unpacking in Comprehensions ---")
points = [(1, 2), (3, 4), (5, 6)]

# 把 (x, y) 的 x 與 y 相加
sums = [x + y for x, y in points]
print("Sums of points:", sums)

# 篩選並解包
high_scores = [subj for subj, scr in scores.items() if scr >= 90]
print("High score subjects:", high_scores)

# ==========================
# 5. 錯誤捕捉 (數量不匹配)
# ==========================
print("\n--- 5. Error Catching ---")
try:
    x, y, z = [1, 2] # 少給一個
except Exception as e:
    print("Caught:", e)

try:
    x, y = 100, 200, 300 # 多給一個
except Exception as e:
    print("Caught:", e)