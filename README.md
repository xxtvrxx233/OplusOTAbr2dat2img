# OplusOTAbr2dat2img
[![latest release](https://img.shields.io/github/v/tag/xxtvrxx233/OplusOTAbr2dat2img?color=blue&include_prereleases&label=release&sort=semver&style=flat-square)](https://github.com/xxtvrxx233/OplusOTAbr2dat2img)
[![Downloads](https://img.shields.io/github/downloads/xxtvrxx233/OplusOtabr2dat2img/total)](https://github.com/xxtvrxx233/OplusOTAbr2dat2img/releases)

Automatically convert the br file from Oplus Ota to img format image for flash in
# Usage:
## 1. Get Software Updates
Settings→About Phone→Software Updates→Download
## 2. Get OTA file
Go to `/data/ota_package/OTA/.otaPackage/`, copy and unzip all the `.br` and `.list` 
files into workspace.

_Note: You have to create workspace folder manually before using it._
## 3. Rename all files with numbers as
```
[partition names].new.dat.br
[partition names].transfer.list
```
> e.g.
> my_manifest.new.dat.br
> my_manifest.transfer.list

## 4. Install dependences
```bash
apt install brotli python3 #For Debian&Termux Users
pacman -S brotli python3 #For Arch Users
```
## 5. Just run the script
Go to Workspace
``` bash
bash br2dat2img_en.sh
```

# 绿厂OTA文件一键转换为img
[![latest release](https://img.shields.io/github/v/tag/xxtvrxx233/OplusOTAbr2dat2img?color=blue&include_prereleases&label=release&sort=semver&style=flat-square)](https://github.com/xxtvrxx233/OplusOTAbr2dat2img)
[![Downloads](https://img.shields.io/github/downloads/xxtvrxx233/OplusOtabr2dat2img/total)](https://shields.io/category/downloads)

自动将br格式的文件转换为可以直接刷入的img镜像

# 使用方法
## 1.下载系统更新
设置→关于本机→ColorOS/RealmeUI的logo
## 2. 获取OTA文件
转到 `/data/ota_package/OTA/.otaPackage/`文件夹, 解压所有`.br`和`.list`文件到工作目录
_注意：在这之前，你需要自行创建工作目录。_

## 3. 将文件重命名为
```
[分区名].new.dat.br
[分区名].transfer.list
```
> 如：
> my_manifest.new.dat.br
> my_manifest.transfer.list
## 4. 安装依赖
```bash
apt install brotli python3 #Debian和Termux用户
pacman -S brotli python3 #Arch用户
```
## 5. 执行脚本
转到工作目录
```bash
bash br2dat2img.sh
```
_注:这可能需要特殊网络环境_
