# OplusOTAbr2dat2img
Automatically convert the br file from Oplus Ota to img format image for flash in
# How to use?
## 1.First download br2dat2img.sh and files.zip then download Software Updates 
(Settings→About Phone→Software Updates→Download)
## 2.Get OTA file from /data/ota_package/OTA/.otaPackage/
## 3.Unzip,and copy .br files and .list files to /sdcard/xxtvrxx233/c13/files
### The Most Important Step: rename all files with numbers as [partition names] .new.dat.br(expamle:my_manifest.new.dat.br)
## 4.Install Termux from F-droid/Google Play,open and type pkg install python3 -y
## 5.After completion,use termux-setup-storage to grant storage permission and use cd command to change the folder to where the script is located
## 6.Just execute the script

# 绿厂OTA文件一键转换为img
自动将br格式的文件转换为可以直接刷入的img镜像

# 如何使用?
## 1.首先下载br2dat2img.sh和files.zip(到任意目录，目录最好不要带中文) 然后下载系统更新(设置→关于本机→ColorOS的logo)
## 2.下载软件更新后，使用MT管理器跳转到/data/ota_package/OTA/.otaPackage/
## 3.将压缩包里的br格式和list格式的文件复制到/sdcard/xxtvrxx233/c13/files内
(没有文件夹就自己创建)
### 最重要的一步:将带有数字的文件重命名为标准的 [分区名].new.dat.br 形式(例如:my_manifest.new.dat.br)
## 4.从酷安或者F-droid下载Termux终端模拟器，打开然后输入pkg install python3来安装必要依赖
## 5.安装完成后，输入termux-setup-storage来取得存储权限，然后使用cd命令转到sh脚本所在目录(开头提到过)
## 6.执行脚本即可
