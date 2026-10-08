#!/usr/bin/env bash
# 构建 Linux x86_64 单文件二进制。
#
# 必须在「带共享库的 CPython」环境里运行。manylinux 镜像用不了：它的 CPython 是
# --disable-shared 编的，没有 libpython3.x.so，PyInstaller 会直接报错退出。
# CI 里跑在 python:3.12-slim-bookworm（--enable-shared，glibc 2.36）里。
#
# 环境变量：
#   PYTHON   指定解释器，默认 python3
#   NAME     产物文件名，默认 br2dat2img
#   OUTDIR   输出目录，默认 <repo>/dist
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
NAME="${NAME:-br2dat2img}"
OUTDIR="${OUTDIR:-$ROOT/dist}"
BUILD="$(mktemp -d)"
trap 'rm -rf "$BUILD"' EXIT

PYTHON="${PYTHON:-python3}"
echo "解释器: $("$PYTHON" -V) ($PYTHON)"

# 提前拦一道：没有共享库的话 PyInstaller 会在最后一步才失败，日志很难看懂
if ! "$PYTHON" -c "import sysconfig, sys; sys.exit(0 if sysconfig.get_config_var('Py_ENABLE_SHARED') else 1)"; then
  echo "错误：该解释器没有共享库（Py_ENABLE_SHARED=0），PyInstaller 无法工作。" >&2
  echo "      需要 --enable-shared 编译的 CPython，例如 python:3.12-slim-bookworm 镜像。" >&2
  exit 1
fi

# 优先用 venv：开发机上可绕过 PEP 668 的 externally-managed 限制。
# 万一解释器没带 ensurepip（--with-ensurepip=no 编的），退回 --target：
# 装到临时目录 + PYTHONPATH，不污染系统环境。
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
