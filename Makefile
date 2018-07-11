# basic information
TARGET ?= x86_64-uefi

# important locations
# LIB_DIR ?= /usr/lib
# EFI_DIR ?= $(LIB_DIR)
BUILD_DIR = target/$(TARGET)/debug
EFI_EXE ?= demos.efi
BUILD_OUT = $(BUILD_DIR)/$(EFI_EXE)
ESP_DIR = $(BUILD_DIR)/esp
BOOT_DIR = $(ESP_DIR)/efi/boot
BOOT_OUT = $(BOOT_DIR)/bootx64.efi

# binary names
QEMU = qemu-system-x86_64
# force the rust target path, it seems to not pick up the one in the working
# directory correctly
XARGO = RUST_TARGET_PATH=$(shell pwd) xargo

# ovmf firmware information
FIRMWARE = firmware
OVMF_CODE = $(FIRMWARE)/ovmf_code.fd
OVMF_VARS = $(FIRMWARE)/ovmf_vars.fd

# a massive declaration of all the qemu flags we need
QEMU_FLAGS ?=
# disable machine defaults, we are in control
QFLAGS = -nodefaults
# use standard vga for graphics output
QFLAGS += -vga std
# use a modern machine, preferably with acceleration
QFLAGS += -machine q35,accel=kvm:tcg
# give us plenty of memory to work with (relatively...)
QFLAGS += -m 128M
# hook in ovmf
QFLAGS += -drive if=pflash,format=raw,file=$(OVMF_CODE),readonly=on
QFLAGS += -drive if=pflash,format=raw,file=$(OVMF_VARS),readonly=on
# create an ahci controller (what is this?)
QFLAGS += -device ahci,id=ahci,multifunction=on
# mount our local esp directory in as a FAT partition
QFLAGS += -drive if=none,format=raw,file=fat:rw:$(ESP_DIR),id=esp
QFLAGS += -device ide-drive,bus=ahci.0,drive=esp
# some debugging stuff I don't understand
QFLAGS += -debugcon file:demos.log -global isa-debugcon.iobase=0xE9
# QFLAGS += -debugcon file:debug.log -global isa-debugcon.iobase=0x402
# allow arbitrary flags to get plugged in
QFLAGS += $(QEMU_FLAGS)

all: $(BUILD_OUT)
.PHONY: all

$(BUILD_OUT):
	$(XARGO) build --target $(TARGET)
# get around the fact that we depend on a lot of source files
# xargo is smart enough to deal with it, don't let make in the mix
.PHONY: $(BUILD_OUT)

run: $(BUILD_OUT)
# copy the build artifact
	mkdir -p $(BOOT_DIR)
	cp $(BUILD_OUT) $(BOOT_OUT)
# run qemu
	$(QEMU) $(QFLAGS)
.PHONY: run

clean:
	$(XARGO) clean
.PHONY: clean
