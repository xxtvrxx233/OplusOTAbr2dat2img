#by xxtvrxx233
#Create a working floder

echo "确保您在Termux中执行此脚本，并已安装了python3(使用命令pkg install python3 -y)"
get_char()
{
       SAVEDSTTY=`stty -g`
       stty -echo
       stty raw
       dd if=/dev/tty bs=1 count=1 2> /dev/null
       stty -raw
       stty echo
       stty $SAVEDSTTY
}

echo "确保 内置存储/xxtvrxx233/c13/files 内包含list和dat文件，按任意键继续..."
char=`get_char`




mkdir /sdcard/xxtvrxx233

mkdir /sdcard/xxtvrxx233/c13


unzip files.zip -od /sdcard/xxtvrxx233/c13/files 

cd /sdcard/xxtvrxx233/c13/files

if [ -f "/sdcard/xxtvrxx233/c13/files/system.new.dat.br" ];then
     echo "文件存在，即将开始"
else
     echo "文件不存在，请将官方OTA包中br,list格式的文件解压到 内置存储/xxtvrxx233/c13/files中"
     exit
fi

#start decompressing
sleep 2s
brotli --decompress --output=vendor.new.dat vendor.new.dat.br

brotli --decompress --output=system.new.dat system.new.dat.br

brotli --decompress --output=odm.new.dat odm.new.dat.br

brotli --decompress --output=system_ext.new.dat system_ext.new.dat.br

brotli --decompress --output=product.new.dat product.new.dat.br

brotli --decompress --output=my_product.new.dat my_product.new.dat.br

brotli --decompress --output=my_bigball.new.dat my_bigball.new.dat.br

brotli --decompress --output=my_carrier.new.dat my_carrier.new.dat.br

brotli --decompress --output=my_engineering.new.dat my_engineering.new.dat.br

brotli --decompress --output=my_heytap.new.dat my_heytap.new.dat.br

brotli --decompress --output=my_manifest.new.dat my_manifest.new.dat.br

brotli --decompress --output=my_stock.new.dat my_stock.new.dat.br

brotli --decompress --output=my_region.new.dat my_region.new.dat.br

#clear br files
if [ -f "/sdcard/xxtvrxx233/c13/files/system.new.dat" ];then

rm -f /sdcard/xxtvrxx233/c13/files/*.br
    else echo "解压缩失败，请检查源文件后重试！"
    exit
fi
sleep 3s
#!/bin/sh
get_char()
{
       SAVEDSTTY=`stty -g`
       stty -echo
       stty raw
       dd if=/dev/tty bs=1 count=1 2> /dev/null
       stty -raw
       stty echo
       stty $SAVEDSTTY
}

echo "确保 内置存储/xxtvrxx233/c13/files 内包含list和dat文件，按任意键继续..."
char=`get_char`

echo 开始执行sdat2img


#start transfering

python sdat2img.py vendor.transfer.list vendor.new.dat vendor.img

python sdat2img.py system.transfer.list system.new.dat system.img

python sdat2img.py system_ext.transfer.list system_ext.new.dat system_ext.img

python sdat2img.py odm.transfer.list odm.new.dat odm.img

python sdat2img.py product.transfer.list product.new.dat product.img

python sdat2img.py my_product.transfer.list my_product.new.dat my_product.img

python sdat2img.py my_bigball.transfer.list my_bigball.new.dat my_bigball.img

python sdat2img.py my_carrier.1001011.transfer.list my_carrier.new.dat my_carrier.img

python sdat2img.py my_engineering.transfer.list my_engineering.new.dat my_engineering.img

python sdat2img.py my_heytap.transfer.list my_heytap.new.dat my_heytap.img

python sdat2img.py my_stock.transfer.list my_stock.new.dat my_stock.img

python sdat2img.py my_manifest.transfer.list my_manifest.new.dat my_manifest.img

python sdat2img.py my_region.transfer.list my_region.new.dat my_region.img

#clear dat files

if [ -f "/sdcard/xxtvrxx233/c13/files/system.img" ];then

rm -f /sdcard/xxtvrxx233/c13/files/*.dat
rm -f /sdcard/xxtvrxx233/c13/files/*.list
     echo "执行成功，即将退出"
     exit
else echo "无法执行sdat2img,请检查是否安装python3，或源文件完整性！"
exit
fi