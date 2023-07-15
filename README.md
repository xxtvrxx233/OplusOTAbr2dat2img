# OplusOTAbr2dat2img
[![latest release](https://img.shields.io/github/v/tag/xxtvrxx233/OplusOTAbr2dat2img?color=green&include_prereleases&label=version&sort=semver&style=flat-square)](https://github.com/xxtvrxx233/OplusOTAbr2dat2img)

Automatically convert the br file from Oplus Ota to img format image for flash in
# How to use?
## 1.First download [br2dat2img.sh](https://github.com/xxtvrxx233/OplusOTAbr2dat2img/releases/br2dat2img.sh) and [files.zip](https://github.com/xxtvrxx233/OplusOTAbr2dat2img/releases/files.zip ) then download Software Updates 
(Settings→About Phone→Software Updates→Download)
## 2.Get OTA file from ```/data/ota_package/OTA/.otaPackage/```
## 3.Unzip,and copy .br files and .list files to ```/sdcard/xxtvrxx233/c13/files```
Tips:If the folder does not exist, you can manually create or run the script once.
### The Most Important Step: rename all files with numbers as
```
[partition names].new.dat.br
[partition names].transfer.list
```
(expamle:my_manifest.new.dat.br

my_manifest.transfer.list)
## 4.Install Termux from F-droid/Google Play,open and type 
```
pkg install python3 -y
```
## 5.After completion,use termux-setup-storage to grant storage permission and use cd command to change the folder to where the script is located
## 6.Just execute the script

# 绿厂OTA文件一键转换为img
[![latest release](https://img.shields.io/github/v/tag/xxtvrxx233/OplusOTAbr2dat2img?color=green&include_prereleases&label=version&sort=semver&style=flat-square)](https://github.com/xxtvrxx233/OplusOTAbr2dat2img)

自动将br格式的文件转换为可以直接刷入的img镜像

# 如何使用?
## 1.首先下载[br2dat2img.sh](https://github.com/xxtvrxx233/OplusOTAbr2dat2img/releases/br2dat2img.sh)和[files.zip](https://github.com/xxtvrxx233/OplusOTAbr2dat2img/releases/files.zip )(到任意目录，目录最好不要带中文) 然后下载系统更新
(设置→关于本机→ColorOS/RealmeUI的logo)
## 2.下载软件更新后，使用MT管理器跳转到```/data/ota_package/OTA/.otaPackage/```
## 3.将压缩包里的br格式和list格式的文件复制到```/sdcard/xxtvrxx233/c13/files```内
提示:若文件夹不存在可自行创建，或执行一次脚本
### 最重要的一步:将所有带有数字的文件重命名为
```
[分区名].new.dat.br
[分区名].transfer.list
```
 (例如:my_manifest.new.dat.br
 和my_manifest.transfer.list)
## 4.从酷安或者F-droid下载Termux终端模拟器，打开，然后输入
```pkg install python3 -y```
来安装必要依赖
## 5.安装完成后，输入
```termux-setup-storage```来取得存储权限，然后使用cd命令转到sh脚本所在目录(开头提到过)
例如```cd /sdcard/Download```
## 6.执行脚本即可
