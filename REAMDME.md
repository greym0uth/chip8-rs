# Chip8 in Rust
> An emulator implementation of chip8 written in rust.

## Getting Started
```sh
cargo run
```
To load a rom just add the `.ch8` file to the root of the project then change the `read_file` to the newly added file. This is just temporary until I add a cli flag for selecting a rom.

```sh
cargo test
```
I attempted to get as much coverage as I could (all the opcodes I have written tests for), but I still missed a decent amount.

## Notes
This is my first attempt at writing an emulator of any sorts, but was alot of fun. I would love feedback on some of my architecture design choices, as I want to make a NES emulator next and want to definately make some improvements.