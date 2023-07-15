#!/bin/bash

#by xxtvrxx233
echo "Make sure that you execute this script in Termux,And python3 has been installed.(Use command pkg install python3 -y)"

#prepare for the workspace
mkdir -p /sdcard/xxtvrxx233/c13/files
cd /sdcard/xxtvrxx233/c13/files
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

echo -e "\e[36m Make sure /sdcard/xxtvrxx233/c13/files included list and dat files,Press any key to continue... \e[0m"
char=`get_char`

if [ -f "/sdcard/xxtvrxx233/c13/files/system.new.dat.br" ];then
    echo "File exists, start immediately."
else
    echo -e "\e[36m File does not exist.，Please decompress br and dat files to /sdcard/xxtvrxx233/c13/files \e[0m"
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
else
    echo -e "\e[36m Decompression failed. Please check the source file and try again. \e[0m"
    exit
fi

sleep 3s

echo -e "\e[36m Make sure /sdcard/xxtvrxx233/c13/files included list and dat files，Press any key to continue... \e[0m"
char=`get_char`

echo "Start sdat2img"


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
if [ -f "/sdcard/xxtvrxx233/c13/files/system.img" ];then
    rm -f /sdcard/xxtvrxx233/c13/files/*.dat
    rm -f /sdcard/xxtvrxx233/c13/files/*.list
    echo "Successful execution, exit immediately."
    exit
else
    echo "Unable to execute sdat2img,Please check whether python3 are installed or source files "
    exit
fi
