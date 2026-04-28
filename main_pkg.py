import utils
from mypkg import VERSION
from mypkg.math_ops import add

utils.greet("Bob")
print("Package Version:", VERSION)
print("10 + 20 =", add(10, 20))