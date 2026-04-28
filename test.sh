set -x
rm py4
rustc py4.rs
./py4 py/basic.py
./py4 py/oop.py
./py4 py/magic.py
./py4 main_import.py
./py4 main_pkg.py