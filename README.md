# OplusOTAbr2dat2img
[![latest release](https://img.shields.io/github/v/tag/xxtvrxx233/OplusOTAbr2dat2img?color=blue&include_prereleases&label=release&sort=semver&style=flat-square)](https://github.com/xxtvrxx233/OplusOTAbr2dat2img)
[![Downloads](https://img.shields.io/github/downloads/xxtvrxx233/OplusOtabr2dat2img/total)](https://github.com/xxtvrxx233/OplusOTAbr2dat2img/releases)

Convert the software updates files (OTA) from Oplus Deivices(OPPO/OnePlus/Realme) to img format image for flash in

_Note:Only applicable to devices with [Dynamic Partitons](https://source.android.com/docs/core/ota/dynamic_partitions/implement)_
# Usage
## 1. Get Software Updates

<div>Open 
     <a href="https://github.com/xxtvrxx233/OplusOTAbr2dat2img"><img src="ColorOS_Settings_Icon.png" width="20" height="20"                                              alt="ColorOS_Settings_Icon.png" style="border: 2px solid cyan; border-radius: 50%"></a> Settings App<div>Tap 
     <a href="https://github.com/xxtvrxx233/OplusOTAbr2dat2img"><img src="ColorOS_About_Icon.png" width="20" height="22"                                              alt="ColorOS_About_Icon.png" style="border: 2px solid cyan; border-radius: 50%"></a>About Phone→Software Updates→Download 
     </div>

 ## 2. Get OTA file
 While finish
 
Go to `/data/ota_package/OTA/.otaPackage/`, copy and unzip all the `.br` and `.list` 
files into workspace.

_Note: You have to create workspace folder manually before using it._
## 3. Rename files
Rename all files with numbers as
```
[partition names].new.dat.br
[partition names].transfer.list
```
> e.g.
> my_manifest.new.dat.br
> my_manifest.transfer.list

## 4. Install dependences
### Debian & Termux Users
```bash
apt install brotli python3
```
### Arch Users
```bash
pacman -S brotli python3 
```
### Windows Users
Get Python 3.11 from [Microsoft Store](https://apps.microsoft.com/store/detail/python-311/9NRWMJP3717K?) 

Get brotli from [google/brotli](https://github.com/google/brotli/releases)
## 5. Run the script
Go to Workspace
``` bash
python br2dat2img_en.py
```
---
Open source projects used [@xpirt/sdat2img](https://github.com/xpirt/sdat2img)
# Oplus（OPPO/一加/Realme） 设备OTA文件一键转换为img
_注意:仅适用于[动态分区](https://source.android.google.cn/devices/tech/ota/dynamic_partitions/implement?hl=en-us)的设备_

[![latest release](https://img.shields.io/github/v/tag/xxtvrxx233/OplusOTAbr2dat2img?color=blue&include_prereleases&label=release&sort=semver&style=flat-square)](https://github.com/xxtvrxx233/OplusOTAbr2dat2img)
[![Downloads](https://img.shields.io/github/downloads/xxtvrxx233/OplusOtabr2dat2img/total)](https://shields.io/category/downloads)

自动将br格式的文件转换为可以直接刷入的img镜像

# 使用方法
## 1.下载软件更新
<div>打开 
     <a href="https://github.com/xxtvrxx233/OplusOTAbr2dat2img"><img src="ColorOS_Settings_Icon.png" width="20" height="20"                                              alt="ColorOS_Settings_Icon.png" style="border: 2px solid cyan; border-radius: 50%"></a> 设置 应用<div>点按
     <a href="https://github.com/xxtvrxx233/OplusOTAbr2dat2img"><img src="ColorOS_About_Icon.png" width="20" height="21"                                              alt="ColorOS_About_Icon.png" style="border: 2px solid cyan; border-radius: 50%"></a>关于本机→查看更新→立即下载
      
## 2. 获取OTA文件
等待下载完成后

转到 `/data/ota_package/OTA/.otaPackage/`文件夹, 解压所有`.br`和`.list`文件到工作目录
_注意：在这之前，你需要自行创建工作目录。_

## 3.重命名文件
将部分带有数字的文件重命名为
```
[分区名].new.dat.br
[分区名].transfer.list
```
> 如：
> my_manifest.new.dat.br
> my_manifest.transfer.list
## 4. 安装依赖
###  Debian系Linux和Termux用户
```bash
apt install brotli python3
```
 ### Arch用户
```
pacman -S brotli python3
```
### Windows用户
在 [Microsoft Store](https://apps.microsoft.com/store/detail/python-311/9NRWMJP3717K?)  上获取 Python 3.11  

从 [google/brotli](https://github.com/google/brotli/releases) 下载 Brotli 的最新发行版
## 5. 执行脚本
转到工作目录
```bash
python br2dat2img.py
```
_注:这可能需要特殊网络环境_

---

所用到的开源项目 [@xpirt/sdat2img](https://github.com/xpirt/sdat2img)
