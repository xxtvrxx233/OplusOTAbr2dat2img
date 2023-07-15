#!/usr/bin/env bash

#by xxtvrxx233
echo "确保您已安装了python和brotli"
echo "请确保此脚本在工作目录下运行"

echo "下载所需文件..."
curl https://raw.githubusercontent.com/xpirt/sdat2img/master/sdat2img.py -o sdat2img.py

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

echo -e "\e[36m  请确保工作目录中内包含list和dat文件，按任意键继续...\e[0m"
char=`get_char`

if [ -f "system.new.dat.br" ];then
    echo "文件存在，即将开始"
else
    echo -e "\e[36m  文件不存在，请将官方OTA包中br,list格式的文件解压到工作目录中\e[0m"
    exit
fi


#start decompressing
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
if [ -f "system.new.dat" ];then
    rm -f ./*.br
else
    echo -e "\e[36m  解压缩失败，请检查源文件后重试！\e[0m"
    exit
fi

sleep 3s

echo -e "\e[36m  请确保工作目录中内包含list和dat文件，按任意键继续...\e[0m"
char=`get_char`

echo "开始执行sdat2img"


#start transfering
python sdat2img.py vendor.transfer.list vendor.new.dat vendor.img
python sdat2img.py system.transfer.list system.new.dat system.img
python sdat2img.py system_ext.transfer.list system_ext.new.dat system_ext.img
python sdat2img.py odm.transfer.list odm.new.dat odm.img
python sdat2img.py product.transfer.list product.new.dat product.img
python sdat2img.py my_product.transfer.list my_product.new.dat my_product.img
python sdat2img.py my_bigball.transfer.list my_bigball.new.dat my_bigball.img
python sdat2img.py my_carrier.transfer.list my_carrier.new.dat my_carrier.img
python sdat2img.py my_engineering.transfer.list my_engineering.new.dat my_engineering.img
python sdat2img.py my_heytap.transfer.list my_heytap.new.dat my_heytap.img
python sdat2img.py my_stock.transfer.list my_stock.new.dat my_stock.img
python sdat2img.py my_manifest.transfer.list my_manifest.new.dat my_manifest.img
python sdat2img.py my_region.transfer.list my_region.new.dat my_region.img


#clear dat files
if [ -f "system.img" ];then
    rm -f ./*.dat
    rm -f ./*.list
    echo "执行成功，即将退出"
    exit
else
    echo "无法执行sdat2img,请检查是否安装python3，或源文件完整性！"
    exit
fi