build_dir ?= build

arch ?= x86_64
kernel := build/kernel-$(arch).bin
iso := build/demos-$(arch).iso

linker_script := src/arch/$(arch)/linker.ld
grub_cfg := src/arch/$(arch)/grub.cfg
asm_src := $(wildcard src/arch/$(arch)/*.asm)
asm_obj := $(patsubst src/arch/$(arch)/%.asm, \
	build/arch/$(arch)/%.o, $(asm_src))

.PHONY: all clean run iso

all: $(kernel)

clean:
	@rm -r build

run: $(iso)
	@qemu-system-x86_64 -cdrom $(iso)

$(iso): $(kernel) $(grub_cfg)
	@mkdir -p $(build_dir)/iso/boot/grub
	@cp $(kernel) $(build_dir)/iso/boot/kernel.bin
	@cp $(grub_cfg) $(build_dir)/iso/boot/grub
	@grub-mkrescue -o $(iso) $(build_dir)/iso 2> /dev/null
	@rm -r $(build_dir)/iso

$(kernel): $(asm_obj) $(linker_script)
	@ld -n -T $(linker_script) -o $(kernel) $(asm_obj)

build/arch/$(arch)/%.o: src/arch/$(arch)/%.asm
	@mkdir -p $(shell dirname $@)
	@nasm -felf64 $< -o $@
