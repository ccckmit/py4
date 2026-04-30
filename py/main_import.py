import math_lib
from math_lib import add, pi

print("Imported pi via math_lib:", math_lib.pi)
print("Imported add via from:", add(10, 20))

calc = math_lib.Calculator()
print("Calculator 5 * 6 =", calc.multiply(5, 6))