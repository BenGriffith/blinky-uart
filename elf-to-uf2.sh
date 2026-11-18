#!/bin/sh
cp target/thumbv8m.main-none-eabihf/release/blinky-uart target/thumbv8m.main-none-eabihf/release/blinky-uart.elf
picotool uf2 convert target/thumbv8m.main-none-eabihf/release/blinky-uart.elf ~/Desktop/blinky-uart.uf2
