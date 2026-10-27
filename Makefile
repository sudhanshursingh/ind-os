.PHONY: all bootstrap check clean test-harness kernel qemu-boot gate-g1 gate-g2 gate-g3 gate-g4 gate-g5

KERNEL_ELF = kernel/target/aarch64-unknown-none/debug/indk
KERNEL_BIN = $(KERNEL_ELF).bin
LLVM_OBJCOPY = $(shell find $(shell rustc --print sysroot 2>/dev/null) -name "llvm-objcopy" 2>/dev/null | head -n 1)

all: check

bootstrap:
	@echo "Running Ind OS toolchain bootstrap..."
	@./tools/bootstrap.sh

kernel:
	@echo "Building IndK ARM64 microkernel..."
	@cd kernel && cargo build --target aarch64-unknown-none --target-dir target
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
	@python3 ci/verify_sp09.py
	@python3 ci/verify_sp10.py
	@python3 ci/verify_sp11.py
	@python3 ci/verify_sp12.py
	@python3 ci/verify_sp13.py
	@python3 ci/verify_sp14.py
	@python3 ci/verify_sp15.py
	@python3 ci/verify_sp16.py
	@python3 ci/verify_sp17.py
	@python3 ci/verify_sp18.py
	@python3 ci/verify_sp19.py
	@python3 ci/verify_sp20.py
	@python3 ci/verify_sp21.py
	@python3 ci/verify_sp22.py
	@python3 ci/verify_sp23.py
	@python3 ci/verify_sp24.py
	@python3 ci/verify_sp25.py
	@python3 ci/verify_sp26.py
	@python3 ci/verify_sp27.py
	@python3 ci/verify_sp28.py
	@python3 ci/verify_sp29.py
	@python3 ci/verify_sp30.py
	@python3 ci/verify_sp31.py
	@python3 ci/verify_sp32.py
	@python3 ci/verify_sp33.py
	@python3 ci/verify_sp34.py
	@python3 ci/verify_sp35.py
	@python3 ci/verify_sp36.py
	@python3 ci/verify_sp37.py
	@python3 ci/verify_sp38.py
	@python3 ci/verify_sp39.py
	@python3 ci/verify_sp40.py
	@python3 ci/verify_sp41.py
	@python3 ci/verify_sp42.py
	@python3 ci/verify_sp43.py
	@python3 ci/verify_sp44.py
	@python3 ci/verify_sp45.py
	@python3 ci/verify_sp46.py
	@python3 ci/verify_sp47.py

gate-g1:
	@echo "Running Ind OS Gate G1 Audit & Verification Suite..."
	@python3 ci/verify_sp09.py

gate-g2:
	@echo "Running Ind OS Gate G2 Audit & Verification Suite..."
	@python3 ci/verify_g2.py

gate-g3:
	@echo "Running Ind OS Gate G3 Audit & Verification Suite..."
	@python3 ci/verify_sp23.py

gate-g4:
	@echo "Running Ind OS Gate G4 Audit & Verification Suite..."
	@python3 ci/verify_g4.py

gate-g5:
	@echo "Running Ind OS Gate G5 Audit & Verification Suite..."
	@python3 ci/verify_g5.py

test-harness:
	@echo "Running Ind OS Test Harness Self-Test..."
	@python3 ci/run_qemu_test.py --self-test

clean:
	@rm -rf build/
	@cd kernel 2>/dev/null && cargo clean || true

