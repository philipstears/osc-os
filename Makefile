.PHONY: \
	all \
	clean \
	run \
	disk \
	part \
	stub \
	kernel

# ------------------------------------------------------------------------------
# Run Vars
# ------------------------------------------------------------------------------
OVMF_CODE_IMAGE_PATH := _assets/ovmf/code.fd
OVMF_VARS_IMAGE_PATH := _assets/ovmf/vars.fd
PART_NAME := osc-os.part
DISK_NAME := osc-os.disk

# ------------------------------------------------------------------------------
# Shared Vars
# ------------------------------------------------------------------------------
BUILD_TYPE := release
MAIN_BUILD_DIR := _build

ifeq ($(BUILD_TYPE),release)
	CARGO_PROFILE_ARG := "--release"
else
	CARGO_PROFILE_ARG :=
endif

# ------------------------------------------------------------------------------
# Boot Stub Vars
# ------------------------------------------------------------------------------
STUB_NAME := osc-os-boot-stub.efi
STUB_SOURCE_DIR := boot-stub
STUB_PLATFORM := x86_64-unknown-uefi
STUB_BUILD_DIR := $(STUB_SOURCE_DIR)/target/$(STUB_PLATFORM)/$(BUILD_TYPE)

STUB_SOURCE_FILES := \
	Makefile \
	$(STUB_SOURCE_DIR)/Cargo.lock \
	$(STUB_SOURCE_DIR)/Cargo.toml \
	$(shell find $(STUB_SOURCE_DIR)/src/ -type f -name "*.rs")

# ------------------------------------------------------------------------------
# Kernel Vars
# ------------------------------------------------------------------------------
KERNEL_NAME := osc-os-kernel.elf
KERNEL_SOURCE_DIR := kernel
KERNEL_PLATFORM := x86_64-unknown-none-gnuabi
KERNEL_BUILD_DIR := $(KERNEL_SOURCE_DIR)/target/$(KERNEL_PLATFORM)/$(BUILD_TYPE)

# ------------------------------------------------------------------------------
# Shared Build
# ------------------------------------------------------------------------------
all: $(STUB_BUILD_DIR)/$(STUB_NAME)

clean:
	rm -rf $(STUB_BUILD_DIR)
	rm -rf $(KERNEL_BUILD_DIR)
	rm -rf $(MAIN_BUILD_DIR)

stub: $(STUB_BUILD_DIR)/$(STUB_NAME)
disk: $(MAIN_BUILD_DIR)/$(DISK_NAME)
part: $(MAIN_BUILD_DIR)/$(PART_NAME)
kernel: $(KERNEL_BUILD_DIR)/$(KERNEL_NAME)

# ------------------------------------------------------------------------------
# Run Build
# ------------------------------------------------------------------------------
run: $(MAIN_BUILD_DIR)/$(DISK_NAME)
	qemu-system-x86_64 -cpu qemu64 \
		-serial stdio \
		-net none \
		-m 1024M \
		-drive if=pflash,format=raw,unit=0,file=$(OVMF_CODE_IMAGE_PATH),readonly=on \
		-drive if=pflash,format=raw,unit=1,file=$(OVMF_VARS_IMAGE_PATH) \
		-drive if=ide,format=raw,file=$<

run-with-monitor: $(MAIN_BUILD_DIR)/$(DISK_NAME)
	qemu-system-x86_64 -cpu qemu64 \
		-monitor stdio \
		-net none \
		-m 1024M \
		-drive if=pflash,format=raw,unit=0,file=$(OVMF_CODE_IMAGE_PATH),readonly=on \
		-drive if=pflash,format=raw,unit=1,file=$(OVMF_VARS_IMAGE_PATH) \
		-drive if=ide,format=raw,file=$<

$(MAIN_BUILD_DIR)/$(PART_NAME): $(STUB_BUILD_DIR)/$(STUB_NAME) $(KERNEL_BUILD_DIR)/$(KERNEL_NAME)
	mkdir -p $(MAIN_BUILD_DIR)
	dd if=/dev/zero of=$@ bs=512 count=91669
	mformat -i $@ -h 32 -t 32 -n 64 -c 1
	mmd -i $@ ::EFI
	mmd -i $@ ::EFI/BOOT
	mmd -i $@ ::OSCOS
	mcopy -i $@ $(STUB_BUILD_DIR)/$(STUB_NAME) ::EFI/BOOT/BOOTx64.EFI
	mcopy -i $@ $(KERNEL_BUILD_DIR)/$(KERNEL_NAME) ::OSCOS/KERNEL.BIN

$(MAIN_BUILD_DIR)/$(DISK_NAME): $(MAIN_BUILD_DIR)/$(PART_NAME)
	dd if=/dev/zero of=$@ bs=512 count=93750
	parted $@ -s -a minimal mklabel gpt
	parted $@ -s -a minimal mkpart EFI FAT16 2048s 93716s
	parted $@ -s -a minimal toggle 1 boot
	dd if=$< of=$@ bs=512 count=91669 seek=2048 conv=notrunc

# ------------------------------------------------------------------------------
# Kernel Build
# ------------------------------------------------------------------------------
$(KERNEL_BUILD_DIR)/$(KERNEL_NAME): Makefile
	mkdir -p $(KERNEL_BUILD_DIR)
	printf "Hello, World\n" > $@

# ------------------------------------------------------------------------------
# Boot Stub Build
# ------------------------------------------------------------------------------
$(STUB_BUILD_DIR)/$(STUB_NAME): $(STUB_SOURCE_FILES)
	cd $(STUB_SOURCE_DIR) && cargo build -Z build-std=core,alloc --target $(STUB_PLATFORM) $(CARGO_PROFILE_ARG)
