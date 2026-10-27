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
	@python3 ci/verify_sp48.py
	@python3 ci/verify_sp49.py
	@python3 ci/verify_sp50.py

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

gate-g6:
	@echo "Running Ind OS Gate G6 Audit & Verification Suite..."
	@python3 ci/verify_g6.py

gate-g7:
	@echo "Running Ind OS Gate G7 Audit & Verification Suite..."
	@python3 ci/verify_g7.py

gate-g8:
	@echo "Running Ind OS Gate G8 Audit & Verification Suite..."
	@python3 ci/verify_g8.py


sp-51:
	@echo "Running Ind OS Sprint SP-51 Verification Suite..."
	@python3 ci/verify_sp51.py

sp-52:
	@echo "Running Ind OS Sprint SP-52 Verification Suite..."
	@python3 ci/verify_sp52.py

sp-53:
	@echo "Running Ind OS Sprint SP-53 Verification Suite..."
	@python3 ci/verify_sp53.py

sp-54:
	@echo "Running Ind OS Sprint SP-54 Verification Suite..."
	@python3 ci/verify_sp54.py

sp-55:
	@echo "Running Ind OS Sprint SP-55 Verification Suite..."
	@python3 ci/verify_sp55.py

sp-56:
	@echo "Running Ind OS Sprint SP-56 Verification Suite..."
	@python3 ci/verify_sp56.py

sp-57:
	@echo "Running Ind OS Sprint SP-57 Verification Suite..."
	@python3 ci/verify_sp57.py

sp-58:
	@echo "Running Ind OS Sprint SP-58 Verification Suite..."
	@python3 ci/verify_sp58.py

sp-59:
	@echo "Running Ind OS Sprint SP-59 Verification Suite..."
	@python3 ci/verify_sp59.py

sp-60:
	@echo "Running Ind OS Sprint SP-60 Verification Suite..."
	@python3 ci/verify_sp60.py

sp-61:
	@echo "Running Ind OS Sprint SP-61 Verification Suite..."
	@python3 ci/verify_sp61.py

sp-62:
	@echo "Running Ind OS Sprint SP-62 Verification Suite..."
	@python3 ci/verify_sp62.py

sp-63:
	@echo "Running Ind OS Sprint SP-63 Verification Suite..."
	@python3 ci/verify_sp63.py

sp-64:
	@echo "Running Ind OS Sprint SP-64 Verification Suite..."
	@python3 ci/verify_sp64.py

sp-65:
	@echo "Running Ind OS Sprint SP-65 Verification Suite..."
	@python3 ci/verify_sp65.py

sp-66:
	@echo "Running Ind OS Sprint SP-66 Verification Suite..."
	@python3 ci/verify_sp66.py

sp-67:
	@echo "Running Ind OS Sprint SP-67 Verification Suite..."
	@python3 ci/verify_sp67.py

sp-68:
	@echo "Running Ind OS Sprint SP-68 Verification Suite..."
	@python3 ci/verify_sp68.py

sp-69:
	@echo "Running Ind OS Sprint SP-69 Verification Suite..."
	@python3 ci/verify_sp69.py

sp-70:
	@echo "Running Ind OS Sprint SP-70 Verification Suite..."
	@python3 ci/verify_sp70.py

sp-71:
	@echo "Running Ind OS Sprint SP-71 Verification Suite..."
	@python3 ci/verify_sp71.py

sp-72:
	@echo "Running Ind OS Sprint SP-72 Verification Suite..."
	@python3 ci/verify_sp72.py

sp-73:
	@echo "Running Ind OS Sprint SP-73 Verification Suite..."
	@python3 ci/verify_sp73.py

sp-74:
	@echo "Running Ind OS Sprint SP-74 Verification Suite..."
	@python3 ci/verify_sp74.py

sp-75:
	@echo "Running Ind OS Sprint SP-75 Verification Suite..."
	@python3 ci/verify_sp75.py

sp-76:
	@echo "Running Ind OS Sprint SP-76 Verification Suite..."
	@python3 ci/verify_sp76.py

sp-77:
	@echo "Running Ind OS Sprint SP-77 Verification Suite..."
	@python3 ci/verify_sp77.py

sp-78:
	@echo "Running Ind OS Sprint SP-78 Verification Suite..."
	@python3 ci/verify_sp78.py

sp-79:
	@echo "Running Ind OS Sprint SP-79 Verification Suite..."
	@python3 ci/verify_sp79.py

sp-80:
	@echo "Running Ind OS Sprint SP-80 Verification Suite..."
	@python3 ci/verify_sp80.py

sp-81:
	@echo "Running Ind OS Sprint SP-81 Verification Suite..."
	@python3 ci/verify_sp81.py

sp-82:
	@echo "Running Ind OS Sprint SP-82 Verification Suite..."
	@python3 ci/verify_sp82.py

sp-83:
	@echo "Running Ind OS Sprint SP-83 Verification Suite..."
	@python3 ci/verify_sp83.py

sp-84:
	@echo "Running Ind OS Sprint SP-84 Verification Suite..."
	@python3 ci/verify_sp84.py

sp-85:
	@echo "Running Ind OS Sprint SP-85 Verification Suite..."
	@python3 ci/verify_sp85.py

sp-86:
	@echo "Running Ind OS Sprint SP-86 Verification Suite..."
	@python3 ci/verify_sp86.py

sp-87:
	@echo "Running Ind OS Sprint SP-87 Verification Suite..."
	@python3 ci/verify_sp87.py

sp-88:
	@echo "Running Ind OS Sprint SP-88 Verification Suite..."
	@python3 ci/verify_sp88.py

sp-89:
	@echo "Running Ind OS Sprint SP-89 Verification Suite..."
	@python3 ci/verify_sp89.py

sp-90:
	@echo "Running Ind OS Sprint SP-90 Verification Suite..."
	@python3 ci/verify_sp90.py

sp-91:
	@echo "Running Ind OS Sprint SP-91 Verification Suite..."
	@python3 ci/verify_sp91.py

sp-92:
	@echo "Running Ind OS Sprint SP-92 Verification Suite..."
	@python3 ci/verify_sp92.py

sp-93:
	@echo "Running Ind OS Sprint SP-93 Verification Suite..."
	@python3 ci/verify_sp93.py

sp-94:
	@echo "Running Ind OS Sprint SP-94 Verification Suite..."
	@python3 ci/verify_sp94.py

sp-95:
	@echo "Running Ind OS Sprint SP-95 Verification Suite..."
	@python3 ci/verify_sp95.py

sp-96:
	@echo "Running Ind OS Sprint SP-96 Verification Suite..."
	@python3 ci/verify_sp96.py

sp-97:
	@echo "Running Ind OS Sprint SP-97 Verification Suite..."
	@python3 ci/verify_sp97.py

sp-98:
	@echo "Running Ind OS Sprint SP-98 Verification Suite..."
	@python3 ci/verify_sp98.py

sp-99:
	@echo "Running Ind OS Sprint SP-99 Verification Suite..."
	@python3 ci/verify_sp99.py

sp-100:
	@echo "Running Ind OS Sprint SP-100 Verification Suite..."
	@python3 ci/verify_sp100.py

sp-101:
	@echo "Running Ind OS Sprint SP-101 Verification Suite..."
	@python3 ci/verify_sp101.py

sp-102:
	@echo "Running Ind OS Sprint SP-102 Verification Suite..."
	@python3 ci/verify_sp102.py

sp-103:
	@echo "Running Ind OS Sprint SP-103 Verification Suite..."
	@python3 ci/verify_sp103.py

sp-104:
	@echo "Running Ind OS Sprint SP-104 Verification Suite..."
	@python3 ci/verify_sp104.py

sp-105:
	@echo "Running Ind OS Sprint SP-105 Verification Suite..."
	@python3 ci/verify_sp105.py

sp-106:
	@echo "Running Ind OS Sprint SP-106 Verification Suite..."
	@python3 ci/verify_sp106.py

sp-107:
	@echo "Running Ind OS Sprint SP-107 Verification Suite..."
	@python3 ci/verify_sp107.py

sp-108:
	@echo "Running Ind OS Sprint SP-108 Verification Suite..."
	@python3 ci/verify_sp108.py

sp-109:
	@echo "Running Ind OS Sprint SP-109 Verification Suite..."
	@python3 ci/verify_sp109.py

gate-g9:
	@echo "Running Ind OS Gate G9 Hard-Stop Audit & Ratification..."
	@python3 ci/verify_g9.py



















test-harness:
	@echo "Running Ind OS Test Harness Self-Test..."
	@python3 ci/run_qemu_test.py --self-test

clean:
	@rm -rf build/
	@cd kernel 2>/dev/null && cargo clean || true

