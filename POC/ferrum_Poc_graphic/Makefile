SHELL := /usr/bin/env bash

PROJECT := ferrumos_graphic

.PHONY: help install boot build run run-screen run-debug run-serial clean

help:
	@echo "Targets disponibles:"
	@echo "  make install     - Installe les dépendances système (Garuda/Arch)"
	@echo "  make boot        - Installe la toolchain nightly + cible no_std"
	@echo "  make build       - Construit le runner + kernel + image BIOS"
	@echo "  make run         - Lance QEMU en mode terminal fiable (serial)"
	@echo "  make run-screen  - Lance QEMU en mode ecran interactif"
	@echo "  make run-debug   - Lance QEMU avec trace série"
	@echo "  make run-serial  - Lance QEMU avec fallback entrée série"
	@echo "  make clean       - Nettoie les artefacts"

install:
	sudo pacman -Syu --needed base-devel rustup qemu-base qemu-desktop llvm lld

boot:
	rustup toolchain install nightly
	rustup target add x86_64-unknown-none --toolchain nightly
	rustup component add rust-src llvm-tools-preview --toolchain nightly

build:
	cargo +nightly build

run: build
	FERRUMOS_SERIAL_FALLBACK=1 cargo +nightly run

run-screen: build
	cargo +nightly run

run-debug: build
	FERRUMOS_SERIAL_FALLBACK=1 RUST_BACKTRACE=1 cargo +nightly run

run-serial: build
	FERRUMOS_SERIAL_FALLBACK=1 RUST_BACKTRACE=1 cargo +nightly run

clean:
	cargo clean
