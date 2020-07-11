.PHONY: \
	all \
	clean \
	run \
	disk \
	part \
	stub \
	stub-combined

OVMF_CODE_IMAGE_PATH := _assets/ovmf/code.fd
OVMF_VARS_IMAGE_PATH := _assets/ovmf/vars.fd

SOURCE_DIR := boot-stub

PLATFORM := x86_64-unknown-uefi
BUILD_TYPE := release
BUILD_DIR := $(SOURCE_DIR)/target/$(PLATFORM)/$(BUILD_TYPE)

KERNEL_NAME := osc-os-kernel.elf

STUB_NAME := osc-os-boot-stub.efi
STUB_COMBINED_NAME := osc-os-boot-stub-combined.efi
PART_NAME := osc-os.part
DISK_NAME := osc-os.disk

STUB_SOURCE_FILES := \
	Makefile \
	$(SOURCE_DIR)/Cargo.lock \
	$(SOURCE_DIR)/Cargo.toml \
	$(shell find $(SOURCE_DIR)/src/ -type f -name "*.rs")

ifeq ($(BUILD_TYPE),release)
	CARGO_PROFILE_ARG := "--release"
else
	CARGO_PROFILE_ARG :=
endif

all: $(BUILD_DIR)/$(STUB_NAME)

clean:
	rm -rf $(BUILD_DIR)

run: $(BUILD_DIR)/$(DISK_NAME)
	qemu-system-x86_64 -cpu qemu64 \
		-serial stdio \
		-net none \
		-m 1024M \
		-drive if=pflash,format=raw,unit=0,file=$(OVMF_CODE_IMAGE_PATH),readonly=on \
		-drive if=pflash,format=raw,unit=1,file=$(OVMF_VARS_IMAGE_PATH) \
		-drive if=ide,format=raw,file=$<

# ------------------------------------------------------------------------------
# Kernel Build
# ------------------------------------------------------------------------------
kernel: $(BUILD_DIR)/$(KERNEL_NAME)

$(BUILD_DIR)/$(KERNEL_NAME): Makefile
	printf "Hello, World\n" > $@

# ------------------------------------------------------------------------------
# Boot Stub Build
# ------------------------------------------------------------------------------
disk: $(BUILD_DIR)/$(DISK_NAME)
part: $(BUILD_DIR)/$(PART_NAME)
stub: $(BUILD_DIR)/$(STUB_NAME)
stub-combined: $(BUILD_DIR)/$(STUB_COMBINED_NAME)

$(BUILD_DIR)/$(STUB_NAME): $(STUB_SOURCE_FILES)
	cd $(SOURCE_DIR) && cargo build -Z build-std=core,alloc --target $(PLATFORM) $(CARGO_PROFILE_ARG)

$(BUILD_DIR)/$(STUB_COMBINED_NAME): $(BUILD_DIR)/$(STUB_NAME) $(BUILD_DIR)/$(KERNEL_NAME)
	cat $^ > $@

$(BUILD_DIR)/$(PART_NAME): $(BUILD_DIR)/$(STUB_COMBINED_NAME)
	dd if=/dev/zero of=$@ bs=512 count=91669
	mformat -i $@ -h 32 -t 32 -n 64 -c 1
	mmd -i $@ ::EFI
	mmd -i $@ ::EFI/BOOT
	mcopy -i $@ $< ::EFI/BOOT/BOOTx64.EFI

$(BUILD_DIR)/$(DISK_NAME): $(BUILD_DIR)/$(PART_NAME)
	dd if=/dev/zero of=$@ bs=512 count=93750
	parted $@ -s -a minimal mklabel gpt
	parted $@ -s -a minimal mkpart EFI FAT16 2048s 93716s
	parted $@ -s -a minimal toggle 1 boot
	dd if=$< of=$@ bs=512 count=91669 seek=2048 conv=notrunc


