import sys
import os
import shutil

os.system("cd leda/; cargo build --release --features py_bindings")

filename = None
not_windows = True

if sys.platform.startswith("linux"):
    filename = "libleda.so"
elif sys.platform.startswith("darwin"):
    filename = "libleda.dylib"
else:
    filename = "leda.dll"
    is_windows = False

filename = "leda/target/release/" + filename

if not_windows:
    shutil.copy(filename, "src/leda.so")
else:
    shutil.copy(filename, "src/leda.pyd")