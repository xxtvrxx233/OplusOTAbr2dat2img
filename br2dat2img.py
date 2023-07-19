#!/usr/bin/env python
# -*- coding: utf-8 -*-

import os
import subprocess
import sdat2img
import glob

current_path = os.getcwd().replace('\\', '/')
filelist = os.listdir(current_path)
required_file = [
    'vendor.transfer.list',
    'system.transfer.list',
    'system_ext.transfer.list',
    'odm.transfer.list',
    'product.transfer.list',
    'my_product.transfer.list',
    'my_bigball.transfer.list',
    'my_carrier.transfer.list',
    'my_engineering.transfer.list',
    'my_heytap.transfer.list',
    'my_stock.transfer.list',
    'my_manifest.transfer.list',
    'my_region.transfer.list',
    'vendor.new.dat.br',
    'system.new.dat.br',
    'odm.new.dat.br',
    'system_ext.new.dat.br',
    'product.new.dat.br',
    'my_product.new.dat.br',
    'my_bigball.new.dat.br',
    'my_carrier.new.dat.br',
    'my_engineering.new.dat.br',
    'my_heytap.new.dat.br',
    'my_manifest.new.dat.br',
    'my_stock.new.dat.br',
    'my_region.new.dat.br'
]
contained_files = all(file in filelist for file in required_file)


def run_decompress():
    commands = [
        "brotli --decompress --output=vendor.new.dat vendor.new.dat.br",
        "brotli --decompress --output=system.new.dat system.new.dat.br",
        "brotli --decompress --output=odm.new.dat odm.new.dat.br",
        "brotli --decompress --output=system_ext.new.dat system_ext.new.dat.br",
        "brotli --decompress --output=product.new.dat product.new.dat.br",
        "brotli --decompress --output=my_product.new.dat my_product.new.dat.br",
        "brotli --decompress --output=my_bigball.new.dat my_bigball.new.dat.br",
        "brotli --decompress --output=my_carrier.new.dat my_carrier.new.dat.br",
        "brotli --decompress --output=my_engineering.new.dat my_engineering.new.dat.br",
        "brotli --decompress --output=my_heytap.new.dat my_heytap.new.dat.br",
        "brotli --decompress --output=my_manifest.new.dat my_manifest.new.dat.br",
        "brotli --decompress --output=my_stock.new.dat my_stock.new.dat.br",
        "brotli --decompress --output=my_region.new.dat my_region.new.dat.br"
    ]

    for command in commands:
        try:
            subprocess.run(command, shell=True, check=True)
            print(f"{command} 执行成功")
        except subprocess.CalledProcessError as e:
            print(f"{command} 执行失败：{str(e)}")


def convert():
    files = [
        ("vendor.transfer.list", "vendor.new.dat", "vendor.img"),
        ("system.transfer.list", "system.new.dat", "system.img"),
        ("system_ext.transfer.list", "system_ext.new.dat", "system_ext.img"),
        ("odm.transfer.list", "odm.new.dat", "odm.img"),
        ("product.transfer.list", "product.new.dat", "product.img"),
        ("my_product.transfer.list", "my_product.new.dat", "my_product.img"),
        ("my_bigball.transfer.list", "my_bigball.new.dat", "my_bigball.img"),
        ("my_carrier.transfer.list", "my_carrier.new.dat", "my_carrier.img"),
        ("my_engineering.transfer.list",
         "my_engineering.new.dat", "my_engineering.img"),
        ("my_heytap.transfer.list", "my_heytap.new.dat", "my_heytap.img"),
        ("my_stock.transfer.list", "my_stock.new.dat", "my_stock.img"),
        ("my_manifest.transfer.list", "my_manifest.new.dat", "my_manifest.img"),
        ("my_region.transfer.list", "my_region.new.dat", "my_region.img")
    ]

    for file in files:
        try:
            sdat2img.main(file[0], file[1], file[2])
            print(f"转换 {file[0]} 成功")
        except Exception as e:
            print(f"转换 {file[0]} 失败：{str(e)}")

    print("完成.")


def delete_files():
    new_dat_files = glob.glob("*.new.dat")
    for file in new_dat_files:
        try:
            os.remove(file)
            print(f"删除临时文件 {file}")
        except Exception as e:
            print(f"删除临时文件 {file} 失败：{str(e)}")

    print("完成.")


if contained_files:
    print("文件存在,开始解压...")
    run_decompress()
    convert()
    delete_files()
else:
    print("错误! 缺少以下文件:")
    missing_files = [file for file in required_file if file not in filelist]
    for filelist in missing_files:
        print(filelist)
