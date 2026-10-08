#!/usr/bin/env bash
# 冒烟测试：用合成的 OTA 数据跑一遍完整流程，验证二进制真的能工作。
#
# 合成数据需要 brotli 模块来压缩。脚本自己准备解释器：传入的解释器缺 brotli 时，
# 自动建一个临时 venv 装上（容器里 pip 有 externally-managed 限制，装不进系统环境）。
#
# 用法: smoke-test.sh <二进制路径> [python解释器]
set -euo pipefail

BIN="${1:?用法: smoke-test.sh <二进制路径> [python解释器]}"
PYTHON="${2:-python3}"

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

if ! "$PYTHON" -c "import brotli" 2>/dev/null; then
  echo "$PYTHON 缺少 brotli，改用临时 venv"
  "$PYTHON" -m venv "$WORK/venv"
  "$WORK/venv/bin/python" -m pip install --quiet --disable-pip-version-check brotli
  PYTHON="$WORK/venv/bin/python"
fi
"$PYTHON" -c "import brotli" || { echo "错误：无法准备带 brotli 的解释器" >&2; exit 1; }

cd "$WORK"

# 造 3 个正常分区（带 NV 号）、1 个损坏的、1 个缺配对的文件
"$PYTHON" - <<'PY'
import os, subprocess, sys
import brotli

for part, blocks in [("system", 3), ("vendor", 2), ("my_bigball", 4)]:
    stem = "%s.10010111" % part
    flat = []
    for i in range(blocks):
        flat += [i * 2, i * 2 + 1]
    with open(stem + ".transfer.list", "w") as fh:
        fh.write("4\n%d\n0\n0\nnew %d,%s\n" % (blocks, len(flat), ",".join(map(str, flat))))
    raw = bytes([65 + ord(part[0]) % 26]) * (4096 * blocks * 2)
    with open(stem + ".new.dat.br", "wb") as fh:
        fh.write(brotli.compress(raw, quality=5))

# transfer.list 首数字与元素个数不符 -> sdat2img 会拒绝
with open("my_broken.transfer.list", "w") as fh:
    fh.write("4\n2\n0\n0\nnew 2,0,1,2,3\n")
with open("my_broken.new.dat.br", "wb") as fh:
    fh.write(brotli.compress(b"X" * 8192, quality=5))

# 只有 .br，没有 .transfer.list
with open("my_orphan.new.dat.br", "wb") as fh:
    fh.write(brotli.compress(b"O" * 100, quality=5))
PY

echo "--- 运行: $BIN -y ---"
set +e
"$BIN" -y 2>/dev/null
code=$?
set -e

# 1 个坏分区 -> 预期退出码 1
[ "$code" -eq 1 ] || { echo "失败：退出码 $code，预期 1"; exit 1; }

# 三个好分区必须都产出，且布局正确（数据块按 transfer.list 落位，空洞补零）
"$PYTHON" - <<'PY'
import os, sys

expected = {"system": 3, "vendor": 2, "my_bigball": 4}
for part, blocks in expected.items():
    path = os.path.join("out", part + ".img")
    if not os.path.isfile(path):
        sys.exit("失败：缺少产物 %s" % path)
    data = open(path, "rb").read()
    want = "X." * (blocks - 1) + "X"
    got = "".join(
        "X" if data[i * 4096:(i + 1) * 4096].count(data[i * 4096]) == 4096 and data[i * 4096] else "."
        for i in range(2 * blocks - 1)
    )
    if got != want:
        sys.exit("失败：%s 布局 %s，预期 %s" % (part, got, want))

# 输出目录顶层只应有 .img
extra = [n for n in os.listdir("out") if not n.endswith(".img") and n != ".tmp"]
if extra:
    sys.exit("失败：out/ 顶层混入了 %s" % extra)

# 失败分区的中间文件应保留
if not os.path.isfile(os.path.join("out", ".tmp", "my_broken.new.dat")):
    sys.exit("失败：失败分区的中间文件未保留")

print("冒烟测试通过：3 个分区转换正确，坏分区被隔离，未配对文件被跳过")
PY
