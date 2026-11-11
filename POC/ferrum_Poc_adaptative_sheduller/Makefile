SHELL := /usr/bin/env bash

PROJECT := ferrumos
TARGET_JSON := x86_64-ferrumos.json
TARGET_DIR := target/x86_64-ferrumos/debug
BOOTIMAGE := $(TARGET_DIR)/bootimage-$(PROJECT).bin
QEMU ?= qemu-system-x86_64

.PHONY: help install boot build run run-serial test clean

help:
	@echo "Targets disponibles:"
	@echo "  make install     - Installe les dépendances système (Garuda/Arch)"
	@echo "  make boot        - Installe la toolchain Rust nécessaire"
	@echo "  make build       - Construit l'image bootable"
	@echo "  make run         - Lance QEMU (fenêtre VGA, clavier actif)"
	@echo "  make run-debug   - Lance QEMU avec logs cpu_reset dans /tmp/qemu_ferrumos.log"
	@echo "  make run-serial  - Lance QEMU sans fenêtre (sortie terminal série)"
	@echo "  make test        - Tests noyau sous QEMU + résultats + comparaison baseline"
	@echo "  make clean       - Nettoie les artefacts"

install:
	sudo pacman -Syu --needed base-devel rustup qemu-base qemu-desktop llvm lld

boot:
	rustup toolchain install nightly
	rustup default nightly
	rustup component add rust-src llvm-tools-preview
	cargo install bootimage

build:
	cargo +nightly bootimage --target $(TARGET_JSON) -Zbuild-std=core,alloc,compiler_builtins -Zbuild-std-features=compiler-builtins-mem -Zjson-target-spec

run: build
	$(QEMU) \
		-drive format=raw,file=$(BOOTIMAGE) \
		-m 256M \
		-no-reboot \
		-no-shutdown

run-debug: build
	$(QEMU) \
		-drive format=raw,file=$(BOOTIMAGE) \
		-m 256M \
		-no-reboot \
		-no-shutdown \
		-d cpu_reset,int \
		-D /tmp/qemu_ferrumos.log \
		-display gtk

run-serial: build
	$(QEMU) \
		-drive format=raw,file=$(BOOTIMAGE) \
		-m 256M \
		-no-reboot \
		-no-shutdown \
		-nographic \
		-serial mon:stdio

test:
	bash ../../scripts/kernel_tests/run.sh

clean:
	cargo clean
