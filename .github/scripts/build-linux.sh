#!/usr/bin/env bash
# 构建 Linux x86_64 单文件二进制。
#
# 在 manylinux2014 容器里运行可得到兼容性最广的产物（glibc >= 2.17，覆盖 2014 年
# 之后的绝大多数发行版）；在普通开发机上运行则用于本地验证，产物只保证本机能跑。
#
# 环境变量：
#   PYTHON   指定解释器，默认在容器里挑 /opt/python 下最新的，否则用 python3
#   NAME     产物文件名，默认 br2dat2img
#   OUTDIR   输出目录，默认 <repo>/dist
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
NAME="${NAME:-br2dat2img}"
OUTDIR="${OUTDIR:-$ROOT/dist}"
BUILD="$(mktemp -d)"
trap 'rm -rf "$BUILD"' EXIT

# 选解释器：manylinux 镜像里装了多个 Python，挑一个 PyInstaller 支持得上的。
PYTHON="${PYTHON:-}"
if [ -z "$PYTHON" ] && [ -d /opt/python ]; then
  for ver in 314 313 312 311; do
    candidate="/opt/python/cp${ver}-cp${ver}/bin/python3"
    if [ -x "$candidate" ]; then
      PYTHON="$candidate"
      break
    fi
  done
fi
PYTHON="${PYTHON:-python3}"
echo "解释器: $("$PYTHON" -V) ($PYTHON)"

# 优先用 venv：开发机上可绕过 PEP 668 的 externally-managed 限制。
# manylinux 镜像里的 CPython 是用 --with-ensurepip=no 编的，venv 建不出带 pip 的
# 环境，所以那里会走 --target 分支：装到临时目录 + PYTHONPATH，不污染镜像自带的
# /opt/python（容器以 root 运行，写进去虽然可以，但没必要）。
if "$PYTHON" -m venv "$BUILD/venv" >/dev/null 2>&1; then
  PYBIN="$BUILD/venv/bin/python"
  "$PYBIN" -m pip install --quiet --upgrade pip
  "$PYBIN" -m pip install --quiet pyinstaller brotli
else
  echo "venv 不可用，改用 pip --target 安装"
  "$PYTHON" -m pip install --quiet --upgrade pip
  "$PYTHON" -m pip install --quiet --target "$BUILD/libs" pyinstaller brotli
  PYBIN="$PYTHON"
  export PYTHONPATH="$BUILD/libs"
fi
echo "PyInstaller: $("$PYBIN" -m PyInstaller --version)"

rm -rf "$OUTDIR"
mkdir -p "$OUTDIR"
"$PYBIN" -m PyInstaller \
  --onefile \
  --name "$NAME" \
  --paths "$ROOT" \
  --distpath "$OUTDIR" \
  --workpath "$BUILD/work" \
  --specpath "$BUILD" \
  --log-level WARN \
  "$ROOT/br2dat2img.py"

echo
echo "产物: $OUTDIR/$NAME"
ls -l "$OUTDIR/$NAME"
