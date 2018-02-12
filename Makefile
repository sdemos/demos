LIB_DIR ?= /usr/lib
EFI_DIR ?= $(LIB_DIR)
CARGO ?= CARGO_INCREMENTAL=0 cargo

all: CARGO_FLAG := --release
all: build/bootx64-release.efi
.PHONY: all

debug: CARGO_FLAG :=
debug: build/bootx64-debug.efi build/bootx64-debug-symbols.efi
.PHONY: debug

# run: CARGO_FLAG := --release
run: CARGO_FLAG :=
run: build/image.vmdk
	vmplayer /home/demos/vmware/demos/demos.vmx
.PHONY: run

build/image.vmdk: build/image.img build/bootx64-debug.efi
	@mkdir -p build/mnt
	sudo mount -o loop,offset=1048576 $< build/mnt
	sudo mkdir -p build/mnt/efi/boot
	sudo cp $(word 2,$^) build/mnt/efi/boot/bootx64.efi
	sudo umount build/mnt
	qemu-img convert $< -O vmdk $@

build/image.img:
	@mkdir -p build
	sudo ./scripts/make-image $@

build/bootx64-%.efi: build/demos-uefi-%.so
	objcopy -j .text -j .sdata -j .data -j .dynamic -j .dynsym -j .rel -j .rela -j .reloc --target=efi-app-x86_64 $< $@

build/bootx64-%-symbols.efi: build/demos-uefi-%.so
	objcopy --target=efi-app-x86_64 $< $@

# Because cargo doesn't remove old build artifacts from deps/ when building, if
# source files or dependencies are removed, they will still be linked into the
# final application unless they are cleaned up with the `clean` target or
# `cargo clean`.
build/demos-uefi-%.so: target/%/deps/demos-uefi.o
	@mkdir -p build
	ld target/$(*F)/deps/*.o $(EFI_DIR)/crt0-efi-x86_64.o -nostdlib -znocombreloc -T $(EFI_DIR)/elf_x86_64_efi.lds -shared -Bsymbolic -lefi -L $(LIB_DIR) -pie -e efi_entry -N -o $@

target/%/deps/demos-uefi.o: src/lib.rs Cargo.toml
	$(CARGO) build $(CARGO_FLAG)
.PRECIOUS: target/%/deps/demos-uefi.o

clean:
	-rm -rf build
	-$(CARGO) clean
.PHONY: clean
