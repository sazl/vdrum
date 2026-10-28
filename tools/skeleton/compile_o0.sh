#!/bin/bash
# Compile selected firmware C files at -O0 with all static/inline functions kept, for DWARF extraction only.
set -e
SRC=/home/user/lxr-ref/lxr-desktop-app
OUT=$(dirname "$(readlink -f "$0")")/o0
GCC=/home/user/lxr-ref/toolchains/arm-gnu-toolchain-11.3.rel1-x86_64-arm-none-eabi/bin/arm-none-eabi-gcc
mkdir -p $OUT
INC="-I$SRC/Core/Inc -I$SRC/Drivers/CMSIS/Include -I$SRC/Drivers/CMSIS/Device/ST/STM32F7xx/Include -I$SRC/Drivers/STM32F7xx_HAL_Driver/Inc -I$SRC/Drivers/STM32F7xx_HAL_Driver/Inc/Legacy -I$SRC/Middlewares/Third_Party/FatFs/src -I$SRC/FATFS/App -I$SRC/FATFS/Target -I$SRC/Middlewares/JS/STM32_USB_Device_Library/Core/Inc -I$SRC/Middlewares/JS/STM32_USB_Device_Library/Class/CDC/Inc -I$SRC/USB_DEVICE_MIDI/App -I$SRC/USB_DEVICE_MIDI/Target"
MCU="-mcpu=cortex-m7 -mthumb -mfpu=fpv5-d16 -mfloat-abi=hard"
CFLAGS="$MCU --specs=nano.specs -ffunction-sections -fdata-sections -DSTM32F765xx -DUSE_HAL_DRIVER -O0 -g -gdwarf-4 -ffast-math -freciprocal-math -mfp16-format=ieee -fomit-frame-pointer -fkeep-static-functions -fkeep-inline-functions -fno-eliminate-unused-debug-types -w"
for f in $(cat $(dirname "$(readlink -f "$0")")/files.txt); do
  o=$OUT/$(echo $f | tr '/' '_').o
  $GCC $CFLAGS $INC -c $SRC/$f -o $o
done
echo done
