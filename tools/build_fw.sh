#!/bin/bash
# usage: build_fw.sh <src_dir> <out_dir> <toolchain_bin_prefix e.g. /usr/bin/arm-none-eabi-> [itcm_opt]
set -e
SRC=$1; OUT=$2; TC=$3; ITCM_OPT=${4:-}
mkdir -p $OUT/obj
cd $SRC
SOURCES=$(grep -hE '^\s+[^#]*\.(c|s)\s*$' Drivers/CMakeLists.txt Middlewares/CMakeLists.txt Core/CMakeLists.txt | tr -d ' ' | sort -u)
INC="-ICore/Inc -IDrivers/CMSIS/Include -IDrivers/CMSIS/Device/ST/STM32F7xx/Include -IDrivers/STM32F7xx_HAL_Driver/Inc -IDrivers/STM32F7xx_HAL_Driver/Inc/Legacy -IMiddlewares/Third_Party/FatFs/src -IFATFS/App -IFATFS/Target -IMiddlewares/JS/STM32_USB_Device_Library/Core/Inc -IMiddlewares/JS/STM32_USB_Device_Library/Class/CDC/Inc -IUSB_DEVICE_MIDI/App -IUSB_DEVICE_MIDI/Target"
MCU="-mcpu=cortex-m7 -mthumb -mfpu=fpv5-d16 -mfloat-abi=hard"
CFLAGS="$MCU --specs=nano.specs -ffunction-sections -fdata-sections -DSTM32F765xx -DUSE_HAL_DRIVER -O3 -g -ffast-math -freciprocal-math -mfp16-format=ieee -fomit-frame-pointer"
ITCM_LIST="Src/Preset/presetManager.c Src/Soundengine/cs4344_cs5343.c Src/Soundengine/FxBus.c Src/Soundengine/DSPAudio/1PoleLp.c Src/Soundengine/DSPAudio/CymbalVoice.c Src/Soundengine/DSPAudio/Decay.c Src/Soundengine/DSPAudio/distortion.c Src/Soundengine/DSPAudio/DrumVoice.c Src/Soundengine/DSPAudio/HiHat.c Src/Soundengine/DSPAudio/lfo.c Src/Soundengine/DSPAudio/mixer.c Src/Soundengine/DSPAudio/modulationNode.c Src/Soundengine/DSPAudio/Oscillator.c Src/Soundengine/DSPAudio/ResonantFilter.c Src/Soundengine/DSPAudio/SlopeEg2.c Src/Soundengine/DSPAudio/Snare.c Src/Soundengine/DSPAudio/transientGenerator.c Src/Soundengine/DSPAudio/FX/Distortion.c Src/Soundengine/DSPAudio/FX/Reverb.c Src/Soundengine/MIDI/ParameterArray.c Src/Soundengine/Sequencer/sequencer.c"
OBJS=""
for s in $SOURCES; do
  f=${s/\$\{PROJECT_SOURCE_DIR\}\//}
  [ -f "$f" ] || f="Drivers/$f"; [ -f "$f" ] || f="Middlewares/${s}"; [ -f "$f" ] || f="Core/${s}"
  [ -f "$f" ] || { echo "missing $s"; exit 1; }
  o=$OUT/obj/$(echo $f | tr '/' '_').o
  extra=""
  case "$f" in Drivers/*|Middlewares/*|FATFS/*|USB_DEVICE_MIDI/*) extra="-w";; Core/*) extra="-Wall";; esac
  rel=${f#Core/}
  if [ -n "$ITCM_OPT" ] && echo " $ITCM_LIST " | grep -q " $rel "; then extra="$extra $ITCM_OPT"; fi
  if [ "$rel" = "Src/Soundengine/DSPAudio/ResonantFilter.c" ] && [ -n "$NO_THREAD_JUMPS" ]; then extra="$extra -fno-thread-jumps"; fi
  if [ "$rel" = "Src/main.c" ]; then extra="$extra -DFIRMWARE_VERSION=\"${FWVER:-1.71}\""; fi
  case "$f" in *.s) ${TC}gcc $MCU -x assembler-with-cpp -c $f -o $o;; *) ${TC}gcc $CFLAGS $extra $INC -c $f -o $o 2>>$OUT/warn.log;; esac
  OBJS="$OBJS $o"
done
${TC}gcc $MCU --specs=nosys.specs --specs=nano.specs -static -Wl,--gc-sections -TSTM32F765VGTX_FLASH.ld -Wl,-Map=$OUT/fw.map -Wl,--print-memory-usage -Wl,--emit-relocs $OBJS -Wl,--start-group -lc -lm -Wl,--end-group -o $OUT/fw.elf
${TC}objcopy -O binary $OUT/fw.elf $OUT/fw.bin
ls -l $OUT/fw.bin
