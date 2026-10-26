.PHONY: all bootstrap check clean test-harness

all: check

bootstrap:
	@echo "Running Ind OS toolchain bootstrap..."
	@./tools/bootstrap.sh

check:
	@echo "Running Ind OS CI repository & commit checks..."
	@python3 ci/verify_sp00.py
	@python3 ci/verify_sp01.py
	@python3 ci/verify_sp02.py
	@python3 ci/verify_sp03.py

test-harness:
	@echo "Running Ind OS Test Harness Self-Test..."
	@python3 ci/run_qemu_test.py --self-test

clean:
	@rm -rf build/
