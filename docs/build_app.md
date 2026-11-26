# Build app for release

## Dependencies

### Debian-based

* `musl-tools`

### Arch

* `musl`

### Rust

Rust 1.89 or newer.

```sh
rustup target add x86_64-unknown-linux-musl
```


## Build commands

### Linux

```sh
cargo build --release --target=x86_64-unknown-linux-musl
```
