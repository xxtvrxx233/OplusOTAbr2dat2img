# OplusOTAbr2dat2img
[![latest release](https://img.shields.io/github/v/tag/xxtvrxx233/OplusOTAbr2dat2img?color=blue&include_prereleases&label=release&sort=semver&style=flat-square)](https://github.com/xxtvrxx233/OplusOTAbr2dat2img)
[![Downloads](https://img.shields.io/github/downloads/xxtvrxx233/OplusOtabr2dat2img/total)](https://github.com/xxtvrxx233/OplusOTAbr2dat2img/releases)

Convert OTA software update files from Oplus devices (OPPO/OnePlus/Realme) into `.img` images for flashing

_Note: Only applicable to devices with A-only [Dynamic Partitons](https://source.android.com/docs/core/ota/dynamic_partitions/implement)_

## Prebuilt binary

A self-contained Linux & Windows x86_64 binary is published in [Releases](https://github.com/xxtvrxx233/OplusOTAbr2dat2img/releases). It needs no Python and no brotli installed.

### Linux
```bash
$ chmod +x br2dat2img
$ ./br2dat2img
```
Requires glibc 2.35 or newer (Ubuntu 22.04+, Debian 12+, Arch, RHEL 9 is too old).

Termux users should use the script instead - Android uses bionic libc, which this binary cannot run on.

### Windows
Run in Windows PowerShell or cmd (administrator)
```bash
> .\br2dat2img.exe
```

Put it in the workspace and run it - same behaviour as the script, same options.


# Usage
## 1. Get Software Updates

Settings → About Phone → Software Updates → Download

 ## 2. Get OTA file
After finishing
 
Go to `/data/ota_package/OTA/.otaPackage/`, copy and unzip all the `.br` and `.list` 
files into workspace.

_Note: You have to create workspace folder manually before using it._

## 3. Put scripts into workspace
Copy `br2dat2img.py` and `sdat2img.py` from this repository into the workspace.
Skip this step and step 4 if you use the prebuilt binary - just put `br2dat2img` in the workspace.

## 4. Install dependences
### Debian & Ubuntu Users
```
# apt install python3 python3-brotli
```

### Arch Users
```
# pacman -S python python-brotli
```

### Termux Users
```bash
$ pkg install python
$ pip install brotli
```

### Windows Users
Get Python 3.11 from [Microsoft Store](https://apps.microsoft.com/store/detail/python-311/9NRWMJP3717K?) 
then run the following command in PowerShell
```bash
> pip install brotli
```

_Note: If you have cloned the whole repository, `pip install -r requirements.txt` installs the same dependency._

## 5. Run the script
Go to Workspace
``` bash
$ python3 br2dat2img.py
```
The script scans the workspace and lists the partitions found. Press `y` to start. Images are written to `./out/`.

_Note: Renaming files is not needed - nv id in filenames are removed automatically. The interface language follows your system locale._

_Note: To convert only some partitions, pass `-p` with their names, e.g. `python3 br2dat2img.py -p system vendor`._

---
Open source projects used [@xpirt/sdat2img](https://github.com/xpirt/sdat2img). Its license is kept in `LICENSES/sdat2img-MIT.txt`.


# Oplus（OPPO/一加/Realme） 设备 OTA 文件一键转换为 img
_注意:仅适用于A-only[动态分区](https://source.android.google.cn/devices/tech/ota/dynamic_partitions/implement?hl=en-us)的设备_

[![latest release](https://img.shields.io/github/v/tag/xxtvrxx233/OplusOTAbr2dat2img?color=blue&include_prereleases&label=release&sort=semver&style=flat-square)](https://github.com/xxtvrxx233/OplusOTAbr2dat2img)
[![Downloads](https://img.shields.io/github/downloads/xxtvrxx233/OplusOtabr2dat2img/total)](https://shields.io/category/downloads)

自动将 br 格式的文件转换为可以直接刷入的 img 镜像

## 预编译二进制

[Releases](https://github.com/xxtvrxx233/OplusOTAbr2dat2img/releases) 中提供 Linux 与 Windows 的 x86_64 架构单文件二进制，无需安装外部依赖

### Linux
```bash
$ chmod +x br2dat2img
$ ./br2dat2img
```
要求 glibc 2.35 及以上（Ubuntu 22.04+、Debian 12+、Arch 均可。HEL 9 偏低）。

Termux 用户请使用脚本版：Android 使用 bionic libc，Releases 中提供的二进制无法运行。

### Windows
通过 Windows PowerShell (管理员) 或 cmd (管理员) 运行

```bash
> .\br2dat2img.exe
```

将二进制放入工作目录直接运行，行为与脚本版完全一致，参数也相同。

# 使用方法
## 1.下载软件更新
设置 → 关于本机 → 查看更新 → 立即下载

## 2. 获取OTA文件
等待下载完成后

转到 `/data/ota_package/OTA/.otaPackage/`文件夹, 解压所有`.br`和`.list`文件到工作目录
_注意：在这之前，你需要自行创建工作目录。_

## 3. 准备转换
### 若要使用预编译二进制:
参见 [预编译二进制](##预编译二进制)

### 若要使用 Python 脚本

将本仓库中的 `br2dat2img.py` 和 `sdat2img.py` 复制到工作目录。
若使用预编译二进制，本步与第 4 步均可跳过 

#### 安装依赖
### Debian / Ubuntu 用户
```
# apt install python3 python3-brotli
```
 ### Arch 用户
```
# pacman -S python python-brotli
```
### Termux 用户
```bash
$ pkg install python
$ pip install brotli
```

### Windows 用户
在 [Microsoft Store](https://apps.microsoft.com/store/detail/python-311/9NRWMJP3717K?)  上获取 Python 3.11  
然后在 Powershell 中执行
```bash
> pip install brotli
```

_注: 若已 clone 整个仓库，可执行 `pip install -r requirements.txt` 安装依赖。_

## 5. 执行脚本
转到工作目录
```bash
$ python3 br2dat2img.py
```
脚本会扫描工作目录，列出识别到的分区。待您确认后，按 `y` 开始转换。输出的镜像在 `./out/` 目录。

_注: 无需重命名原始文件，文件名中的 nv 号会自动去除。界面语言跟随系统区域设置。若要强制使用中文，请使用 --lang zh 参数_

_注: 若要只转换部分分区，可用 `-p` 指定，如 `python3 br2dat2img.py -p system vendor`。_

---

所用到的开源项目 [@xpirt/sdat2img](https://github.com/xpirt/sdat2img)，其许可证保存在 `LICENSES/sdat2img-MIT.txt`。
