.PHONY: all bootstrap check clean test-harness kernel qemu-boot

KERNEL_ELF = kernel/target/aarch64-unknown-none/debug/indk
KERNEL_BIN = $(KERNEL_ELF).bin
LLVM_OBJCOPY = $(shell find $(shell rustc --print sysroot 2>/dev/null) -name "llvm-objcopy" 2>/dev/null | head -n 1)

all: check

bootstrap:
	@echo "Running Ind OS toolchain bootstrap..."
	@./tools/bootstrap.sh

kernel:
	@echo "Building IndK ARM64 microkernel..."
	@cd kernel && cargo build --target aarch64-unknown-none
	@if [ -n "$(LLVM_OBJCOPY)" ]; then \
		$(LLVM_OBJCOPY) -O binary $(KERNEL_ELF) $(KERNEL_BIN); \
	fi

qemu-boot: kernel
	@echo "Booting IndK kernel in headless QEMU..."
	@./tools/run_qemu.sh --kernel $(KERNEL_ELF) --timeout 5

check:
	@echo "Running Ind OS CI repository & commit checks..."
	@python3 ci/verify_sp00.py
	@python3 ci/verify_sp01.py
	@python3 ci/verify_sp02.py
	@python3 ci/verify_sp03.py
	@python3 ci/verify_sp04.py
	@python3 ci/verify_sp05.py
	@python3 ci/verify_sp06.py
	@python3 ci/verify_sp07.py
	@python3 ci/verify_sp08.py

test-harness:
	@echo "Running Ind OS Test Harness Self-Test..."
	@python3 ci/run_qemu_test.py --self-test

clean:
	@rm -rf build/
	@cd kernel 2>/dev/null && cargo clean || true
