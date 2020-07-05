.PHONY: \
	all \
	run

OVMF_CODE_IMAGE_PATH := _assets/ovmf/code.fd
OVMF_VARS_IMAGE_PATH := _assets/ovmf/vars.fd

SOURCE_DIR := kernel

PLATFORM := x86_64-unknown-uefi
BUIlD_TYPE := release
BUILD_DIR := $(SOURCE_DIR)/target/$(PLATFORM)/$(BUILD_TYPE)

EFI_NAME=osc-os-kernel.efi

SOURCE_FILES := \
	Makefile \
	$(SOURCE_DIR)/Cargo.toml \
	$(SOURCE_DIR)/src/main.rs

ifeq ($(BUILD_TYPE),release)
	CARGO_PROFILE_ARG := "--release"
else
	CARGO_PROFILE_ARG :=
endif

all: $(BUILD_DIR)/$(EFI_NAME)

run:
	qemu-system-x86_64 -cpu qemu64 \
		-drive if=pflash,format=raw,unit=0,file=$(OVMF_CODE_IMAGE_PATH),readonly=on \
		-drive if=pflash,format=raw,unit=1,file=$(OVMF_VARS_IMAGE_PATH) \
		-net none

$(BUILD_DIR)/$(EFI_NAME): $(SOURCE_FILES)
	cd $(SOURCE_DIR) && cargo build -Z build-std=core --target $(PLATFORM) $(CARGO_PROFILE_ARG)

