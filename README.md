# electron-hardener

[![crates.io version](https://img.shields.io/crates/v/electron-hardener.svg)](https://crates.io/crates/electron-hardener)
[![crate documentation](https://docs.rs/electron-hardener/badge.svg)](https://docs.rs/electron-hardener)
![MSRV](https://img.shields.io/badge/rustc-1.46+-blue.svg)
[![crates.io downloads](https://img.shields.io/crates/d/electron-hardener.svg)](https://crates.io/crates/electron-hardener)
![CI](https://github.com/1Password/electron-hardener/workflows/CI/badge.svg)


A Rust library and command line tool to harden Electron binaries against runtime behavior modifications.

This provides a way to harden Electron applications against a specific class of runtime behavior
modification. Specifically, if an unprivileged process can't write to the application's binary file or process
address space, it should not be able to change what an app does at runtime.

The library provides two sets of functionality:
 - An interface to view and modify the status of fuses in an application, similar to the [official fuses package](https://github.com/electron/fuses).
 - A fast and configurable alternative implementation of the [electron-evil-feature-patcher](https://github.com/antelle/electron-evil-feature-patcher) tool created by [Dimitri Witkowski].
All patches it can perform are also exposed in this crate. See its README for more details on how it works.

## Usage

### Library
The library exposes a simple and configurable interface:
```rust
use electron_hardener::{ElectronApp, Fuse, NodeJsCommandLineFlag};

let mut app = ElectronApp::from_bytes(&mut application_bytes)?;

app.set_fuse_status(Fuse::RunAsNode, false)?;

app.patch_option(NodeJsCommandLineFlag::Inspect)?;
```

Check out the [command line tool](./src/bin/main.rs)'s source or the [example](./examples/usage.rs) to see more ways to use it.

### Command line tool

The command line tool exposes the same functionality and interface as `electron-evil-feature-patcher`:
```bash
electron-hardener ./path/to/packaged/electron/app
```

## Install
### Library
In your project's `Cargo.toml` file:
```toml
electron_hardener = "0.2.2"
```

### Command line tool
Make sure you have a [Rust compiler](https://rustup.rs/) installed and then run
```bash
cargo install electron-hardener
```

## Electron compatibility
`electron-harder` tracks the latest stable version of Electron. Functionality is currently tested on a minimum version of Electron 15. Older versions may partially work but this is not guaranteed.

## MSRV

The Minimum Supported Rust Version is currently 1.46.0. This will be bumped to the latest stable version of Rust when needed.

## Credits
Made with ❤️ by the [1Password](https://1password.com/) team, with full credits to [Dimitri Witkowski] for taking the time and effort to discover the command line flags that can be disabled, and finally creating the original tool which served as inspiration for this project.

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>

[Dimitri Witkowski]: https://github.com/antelle
