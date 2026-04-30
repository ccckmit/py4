set -x
rm py4
rustc py4.rs
./py4 py/basic.py
./py4 py/oop.py
./py4 py/magic.py
# ./py4 main_import.py
# ./py4 main_pkg.py
./py4 py/io.py
./py4 py/inherit.py
#./py4 py/decorator.py
./py4 py/args.py
./py4 py/unpack.py
./py4 py/adv_oop.py
./py4 py/modern.py
./py4 py/test_stdlib.py
./py4 py/test_path.py

PYTHONPATH=./py/ ./py4 py/main_import.py
PYTHONPATH=./py/ ./py4 py/main_pkg.py