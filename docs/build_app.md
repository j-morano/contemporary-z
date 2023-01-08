# Build app for release

## Dependencies

### Debian-based

* `musl-tools`
* `libsqlite3-dev`

### Arch

* `musl`
* `sqlite3`

### Rust

```sh
rustup target add x86_64-unknown-linux-musl`
```


## Build commands

### Linux

```sh
cargo build --release --target=x86_64-unknown-linux-musl
```
