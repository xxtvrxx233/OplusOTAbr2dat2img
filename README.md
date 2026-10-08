# OplusOTAbr2dat2img

[![latest release](https://img.shields.io/github/v/tag/xxtvrxx233/OplusOTAbr2dat2img?color=blue&include_prereleases&label=release&sort=semver&style=flat-square)](https://github.com/xxtvrxx233/OplusOTAbr2dat2img)
[![Downloads](https://img.shields.io/github/downloads/xxtvrxx233/OplusOtabr2dat2img/total)](https://github.com/xxtvrxx233/OplusOTAbr2dat2img/releases)

Convert OTA software update files from Oplus devices (OPPO/OnePlus/Realme) into `.img` images for flashing.

_Note: Only applicable to devices with A-only [Dynamic Partitions](https://source.android.com/docs/core/ota/dynamic_partitions/implement)._

## Prebuilt binary

A self-contained Linux & Windows x86_64 binary is published in [Releases](https://github.com/xxtvrxx233/OplusOTAbr2dat2img/releases). It does not require Python or brotli to be installed.

### Linux

```bash
$ chmod +x br2dat2img
$ ./br2dat2img
```

Requires glibc 2.35 or newer (Ubuntu 22.04+, Debian 12+, Arch). RHEL 9 does not meet this requirement.

Termux users should use the script instead. Android uses bionic libc, which the prebuilt binary cannot run with.

### Windows

Run in Windows PowerShell or cmd (administrator):

```powershell
> .\br2dat2img.exe
```

Put the binary in the workspace and run it. Its behaviour and options are the same as the script.

## Usage

### 1. Get software updates

Settings → About Phone → Software Updates → Download

### 2. Get OTA files

After the download is complete, go to:

`/data/ota_package/OTA/.otaPackage/`

Copy and extract all `.br` and `.list` files into the workspace.

_Note: You need to create the workspace folder manually before using the tool._

### 3. Put the tool into the workspace

#### Prebuilt binary

Download the appropriate binary from [Releases](https://github.com/xxtvrxx233/OplusOTAbr2dat2img/releases) and put it into the workspace.

No additional dependencies are required.

#### Python script

Copy `br2dat2img.py` and `sdat2img.py` from this repository into the workspace.

##### Debian & Ubuntu

```bash
# apt install python3 python3-brotli
```

##### Arch

```bash
# pacman -S python python-brotli
```

##### Termux

```bash
$ pkg install python
$ pip install brotli
```

##### Windows

Get Python 3.11 from the [Microsoft Store](https://apps.microsoft.com/store/detail/python-311/9NRWMJP3717K?), then run the following command in PowerShell:

```powershell
> pip install brotli
```

_Note: If you have cloned the whole repository, `pip install -r requirements.txt` installs the same dependency._

### 4. Start conversion

#### Prebuilt binary

Linux:

```bash
$ ./br2dat2img
```

Windows:

```powershell
> .\br2dat2img.exe
```

#### Python script

```bash
$ python3 br2dat2img.py
```

The tool scans the workspace and lists the detected partitions. Press `y` to start the conversion. Images are written to `./out/`.

_Note: Renaming files is not required. NV IDs in filenames are removed automatically. The interface language follows your system locale. To force Chinese, use `--lang zh`._

_Note: To convert only specific partitions, pass `-p` with their names, for example:_

```bash
$ python3 br2dat2img.py -p system vendor
```

The same option can be used with the prebuilt binary:

```bash
$ ./br2dat2img -p system vendor
```

## Options

```text
Usage:
  ./br2dat2img [-h] [-i DIR] [-o DIR] [-p NAME [NAME ...]]
               [--lang {auto,zh,en}] [-j N] [-y] [--version]
```

_All parameters are optional. The program can operate normally without parameters by default._

| Option | Description |
|---|---|
| `-h`, `--help` | Show this help message and exit |
| `-i DIR`, `--indir DIR` | Directory containing the OTA files (default: current directory) |
| `-o DIR`, `--outdir DIR` | Directory to write the images to (default: `./out`) |
| `-p NAME [NAME ...]`, `--partition NAME [NAME ...]` | Convert only the given partitions. Multiple partitions can be specified. Accepts partition names or original filenames (default: all) |
| `--lang {auto,zh,en}` | UI language (default: `auto`: detect from environment and system locale, falling back to `en`) |
| `-j N`, `--jobs N` | Number of parallel decompression workers (default: CPU count) |
| `-y`, `--yes` | Skip the confirmation prompt |
| `--version` | Show version and exit |

---

Open source project used: [@xpirt/sdat2img](https://github.com/xpirt/sdat2img). Its license is preserved in `LICENSES/sdat2img-MIT.txt`.


# Oplus（OPPO/一加/Realme）设备 OTA 文件一键转换为 img

_注意：仅适用于 A-only [动态分区](https://source.android.google.cn/devices/tech/ota/dynamic_partitions/implement?hl=en-us) 的设备。_

[![latest release](https://img.shields.io/github/v/tag/xxtvrxx233/OplusOTAbr2dat2img?color=blue&include_prereleases&label=release&sort=semver&style=flat-square)](https://github.com/xxtvrxx233/OplusOTAbr2dat2img)
[![Downloads](https://img.shields.io/github/downloads/xxtvrxx233/OplusOtabr2dat2img/total)](https://github.com/xxtvrxx233/OplusOTAbr2dat2img/releases)

自动将 Oplus（OPPO/一加/Realme）设备 OTA 更新文件转换为可以直接刷入的 `.img` 镜像。

## 预编译二进制

[Releases](https://github.com/xxtvrxx233/OplusOTAbr2dat2img/releases) 中提供 Linux 与 Windows x86_64 架构的单文件二进制，无需安装 Python 或 brotli。

### Linux

```bash
$ chmod +x br2dat2img
$ ./br2dat2img
```

要求 glibc 2.35 及以上（Ubuntu 22.04+、Debian 12+、Arch 均可。RHEL 9 不满足要求）。

Termux 用户请使用脚本版。Android 使用 bionic libc，预编译二进制无法在 Termux 中运行。

### Windows

通过 Windows PowerShell 或 cmd（均需要管理员身份）运行：

```powershell
> .\br2dat2img.exe
```

将二进制放入工作目录后直接运行，行为和参数与脚本版一致。

## 使用方法

### 1. 下载软件更新

设置 → 关于本机 → 软件更新 → 下载

### 2. 获取 OTA 文件

等待下载完成后，转到：

`/data/ota_package/OTA/.otaPackage/`

将所有 `.br` 和 `.list` 文件复制并解压到工作目录。

_注意：使用工具前需要自行创建工作目录。_

### 3. 准备工具

#### 预编译二进制

从 [Releases](https://github.com/xxtvrxx233/OplusOTAbr2dat2img/releases) 下载对应的二进制并放入工作目录。

无需安装其他依赖。

#### Python 脚本

将本仓库中的 `br2dat2img.py` 和 `sdat2img.py` 复制到工作目录。

##### Debian / Ubuntu

```bash
# apt install python3 python3-brotli
```

##### Arch

```bash
# pacman -S python python-brotli
```

##### Termux

```bash
$ pkg install python
$ pip install brotli
```

##### Windows

从 [Microsoft Store](https://apps.microsoft.com/store/detail/python-311/9NRWMJP3717K?) 获取 Python 3.11，然后在 PowerShell 中执行：

```powershell
> pip install brotli
```

_注：若已 clone 整个仓库，可执行 `pip install -r requirements.txt` 安装相同依赖。_

### 4. 开始转换

#### 预编译二进制

Linux：

```bash
$ ./br2dat2img
```

Windows：

```powershell
> .\br2dat2img.exe
```

#### Python 脚本

```bash
$ python3 br2dat2img.py
```

工具会扫描工作目录并列出识别到的分区。确认后按 `y` 开始转换，输出的镜像位于 `./out/` 目录。

_注意：无需重命名原始文件，文件名中的 NV 号会自动去除。界面语言跟随系统区域设置。若要强制使用中文，请使用 `--lang zh`。_

_注意：若要只转换部分分区，可使用 `-p` 指定分区名称，例如：_

```bash
$ python3 br2dat2img.py -p system vendor
```

预编译二进制也支持相同参数：

```bash
$ ./br2dat2img -p system vendor
```

## 参数

```text
用法：
  ./br2dat2img [-h] [-i DIR] [-o DIR] [-p NAME [NAME ...]]
               [--lang {auto,zh,en}] [-j N] [-y] [--version]
```

_所有参数都是可选的，程序默认情况下无需参数即可正常运行。_

| 参数 | 说明 |
|---|---|
| `-h`, `--help` | 显示帮助信息并退出 |
| `-i DIR`, `--indir DIR` | OTA 文件所在目录（默认：当前目录） |
| `-o DIR`, `--outdir DIR` | 镜像输出目录（默认：`./out`） |
| `-p NAME [NAME ...]`, `--partition NAME [NAME ...]` | 只转换指定的分区，可指定多个。支持分区名或原始文件名（默认：全部） |
| `--lang {auto,zh,en}` | 界面语言（默认：`auto`，根据环境变量与系统区域自动判定，失败时使用 `en`） |
| `-j N`, `--jobs N` | 并行解压的工作进程数（默认：CPU 核心数） |
| `-y`, `--yes` | 跳过确认提示，直接开始转换 |
| `--version` | 显示版本号并退出 |

---

所用到的开源项目：[@xpirt/sdat2img](https://github.com/xpirt/sdat2img)。

其许可证保存在 `LICENSES/sdat2img-MIT.txt`。