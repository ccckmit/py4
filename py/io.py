# 1. 寫入檔案
filename = "out/hello.txt"
print("Writing to file:", filename)
f = open(filename, "w")
f.write("Hello, Rust and Python!\n")
f.write("This is a mini interpreter doing real I/O.")
f.close()

# 測試對已關閉的檔案操作是否會報錯 (被 try 捕獲)
try:
    f.write("Error!")
except Exception as e:
    print("Expected error caught:", e)

# 2. 讀取檔案
print("\nReading from file:", filename)
f_in = open(filename, "r")
content = f_in.read()
f_in.close()

print("\n--- File Content Start ---")
print(content)
print("--- File Content End ---")