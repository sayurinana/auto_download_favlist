import sys
from pathlib import Path

# 确保src目录在导入路径中
SRC_PATH = Path(__file__).resolve().parents[1] / "src"
if SRC_PATH.exists():
    sys.path.insert(0, str(SRC_PATH))
