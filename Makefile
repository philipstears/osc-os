.PHONY: \
	run

OVMF_CODE_IMAGE_PATH := _assets/ovmf/code.fd
OVMF_VARS_IMAGE_PATH := _assets/ovmf/vars.fd

run:
	qemu-system-x86_64 -cpu qemu64 \
		-drive if=pflash,format=raw,unit=0,file=$(OVMF_CODE_IMAGE_PATH),readonly=on \
		-drive if=pflash,format=raw,unit=1,file=$(OVMF_VARS_IMAGE_PATH) \
		-net none

