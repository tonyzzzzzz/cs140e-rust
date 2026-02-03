#!/bin/bash

nice_path() {
  perl -le "use File::Spec;print File::Spec->abs2rel(@ARGV)" "$1" "$(pwd)"
}

elf_path=$1
target_dir=$(nice_path "$(dirname "$elf_path")")
base_name=$(basename "$elf_path")
bin_path="${target_dir}/${base_name/.elf/}.bin"
list_path="${target_dir}/${base_name/.elf/}.list"
img_path="${target_dir}/kernel.img"

# load_addr=0x8000

arm-none-eabi-objcopy "$elf_path" -O binary "$bin_path"
arm-none-eabi-objdump -D "$elf_path" > "$list_path"
cp $bin_path $img_path

echo "Created $bin_path from $elf_path"