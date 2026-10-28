#!/bin/bash
# Preprocess the firmware C files (same flags as the reference build) for source analysis.
set -e
SRC=/home/user/lxr-ref/lxr-desktop-app
OUT=$(dirname "$(readlink -f "$0")")/pp
GCC=/home/user/lxr-ref/toolchains/arm-gnu-toolchain-11.3.rel1-x86_64-arm-none-eabi/bin/arm-none-eabi-gcc
mkdir -p $OUT
INC="-ICore/Inc -IDrivers/CMSIS/Include -IDrivers/CMSIS/Device/ST/STM32F7xx/Include -IDrivers/STM32F7xx_HAL_Driver/Inc -IDrivers/STM32F7xx_HAL_Driver/Inc/Legacy -IMiddlewares/Third_Party/FatFs/src -IFATFS/App -IFATFS/Target -IMiddlewares/JS/STM32_USB_Device_Library/Core/Inc -IMiddlewares/JS/STM32_USB_Device_Library/Class/CDC/Inc -IUSB_DEVICE_MIDI/App -IUSB_DEVICE_MIDI/Target"
MCU="-mcpu=cortex-m7 -mthumb -mfpu=fpv5-d16 -mfloat-abi=hard"
CFLAGS="$MCU --specs=nano.specs -DSTM32F765xx -DUSE_HAL_DRIVER -O3 -ffast-math -freciprocal-math -mfp16-format=ieee -w"
cd $SRC
for f in $(cat $(dirname "$(readlink -f "$0")")/files.txt); do
  o=$OUT/$(echo $f | tr '/' '_')
  $GCC $CFLAGS $INC -E $f -o $o.i
  $GCC $CFLAGS $INC -E -dD $f -o $o.dd
done
echo done
