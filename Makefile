.PHONY: all bootstrap check clean

all: check

bootstrap:
	@echo "Running Ind OS toolchain bootstrap..."
	@./tools/bootstrap.sh

check:
	@echo "Running Ind OS CI repository & commit checks..."
	@python3 ci/verify_sp00.py
	@python3 ci/verify_sp01.py

clean:
	@rm -rf build/
