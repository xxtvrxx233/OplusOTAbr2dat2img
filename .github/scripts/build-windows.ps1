# 构建 Windows x86_64 单文件二进制（br2dat2img.exe）。
#
# 用 python.org 官方构建的 Python：它有 python3xx.dll（sys.dllhandle），
# PyInstaller 需要它。自行编译的静态链接 Python 会报
# "Python was built without a shared library"。
#
# 环境变量：
#   NAME     产物文件名，默认 br2dat2img
#   OUTDIR   输出目录，默认 <repo>\dist
$ErrorActionPreference = "Stop"

$Root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
if (-not $env:NAME)   { $env:NAME = "br2dat2img" }
if (-not $env:OUTDIR) { $env:OUTDIR = Join-Path $Root "dist" }

$Build = Join-Path ([System.IO.Path]::GetTempPath()) ("pyi-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $Build | Out-Null

try {
    Write-Host "解释器: $(python -V) ($(Get-Command python).Source)"

    # 建 venv 隔离依赖，同时避开系统 Python 的 externally-managed 限制
    python -m venv (Join-Path $Build "venv")
    $Py = Join-Path $Build "venv\Scripts\python.exe"

    & $Py -m pip install --quiet --upgrade pip
    & $Py -m pip install --quiet pyinstaller brotli
    Write-Host "PyInstaller: $(& $Py -m PyInstaller --version)"

    if (Test-Path $env:OUTDIR) { Remove-Item -Recurse -Force $env:OUTDIR }
    New-Item -ItemType Directory -Path $env:OUTDIR | Out-Null

    & $Py -m PyInstaller `
        --onefile `
        --name $env:NAME `
        --paths $Root `
        --distpath $env:OUTDIR `
        --workpath (Join-Path $Build "work") `
        --specpath $Build `
        --log-level WARN `
        (Join-Path $Root "br2dat2img.py")

    Write-Host ""
    $exe = Join-Path $env:OUTDIR "$env:NAME.exe"
    Write-Host "产物: $exe"
    Get-Item $exe | Select-Object Name, Length | Format-List
}
finally {
    Remove-Item -Recurse -Force $Build -ErrorAction SilentlyContinue
}
