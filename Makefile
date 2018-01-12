build_dir ?= build

arch ?= x86_64
target ?= $(arch)-demos
rust_os := target/$(target)/debug/libdemos.a
kernel := build/kernel-$(arch).bin
iso := build/demos-$(arch).iso

linker_script := src/arch/$(arch)/linker.ld
grub_cfg := src/arch/$(arch)/grub.cfg
asm_src := $(wildcard src/arch/$(arch)/*.asm)
asm_obj := $(patsubst src/arch/$(arch)/%.asm, \
	build/arch/$(arch)/%.o, $(asm_src))

.PHONY: all clean run iso kernel

all: $(kernel)

clean:
	@rm -r build
	@xargo clean

run: $(iso)
	@qemu-system-x86_64 -cdrom $(iso)

$(iso): $(kernel) $(grub_cfg)
	@mkdir -p $(build_dir)/iso/boot/grub
	@cp $(kernel) $(build_dir)/iso/boot/kernel.bin
	@cp $(grub_cfg) $(build_dir)/iso/boot/grub
	@grub-mkrescue -o $(iso) $(build_dir)/iso 2> /dev/null
	@rm -r $(build_dir)/iso

$(kernel): kernel $(rust_os) $(asm_obj) $(linker_script)
	@ld -n --gc-sections -T $(linker_script) -o $(kernel) $(asm_obj) $(rust_os)

kernel: export RUST_TARGET_PATH=$(shell pwd)
kernel:
	@xargo build --target $(target)

build/arch/$(arch)/%.o: src/arch/$(arch)/%.asm
	@mkdir -p $(shell dirname $@)
	@nasm -felf64 $< -o $@
