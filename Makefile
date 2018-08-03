# basic information
BOOT_TARGET ?= x86_64-uefi
KERNEL_TARGET ?= x86_64-demos

# build artifact locations
BUILD_BOOT_DIR = target/$(BOOT_TARGET)/debug
EFI_EXE ?= demos-bootloader.efi
EFI_IN = $(BUILD_BOOT_DIR)/$(EFI_EXE)
BUILD_KERNEL_DIR = target/$(KERNEL_TARGET)/debug
KERNEL_IN = $(BUILD_KERNEL_DIR)/demos-kernel

# output structure
ESP_DIR = target/esp
BOOT_DIR = $(ESP_DIR)/efi/boot
EFI_OUT = $(BOOT_DIR)/bootx64.efi
KERNEL_OUT = $(ESP_DIR)/kernel

# binary names
QEMU = qemu-system-x86_64
CARGO = cargo

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

all: boot kernel
.PHONY: all

boot:
	$(CARGO) xbuild --target $(BOOT_TARGET).json --package demos-bootloader
.PHONY: boot

kernel:
	$(CARGO) xbuild --target $(KERNEL_TARGET).json --package demos-kernel
.PHONY: kernel

esp: boot kernel
# copy the build artifact
	mkdir -p $(BOOT_DIR)
	cp $(EFI_IN) $(EFI_OUT)
	cp $(KERNEL_IN) $(KERNEL_OUT)
.PHONY: esp

debug: esp
	$(QEMU) -s -S $(QFLAGS)
.PHONY: debug

run: esp
# run qemu
	$(QEMU) $(QFLAGS)
.PHONY: run

clean:
	$(CARGO) clean
.PHONY: clean
