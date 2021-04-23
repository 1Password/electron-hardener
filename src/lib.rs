//! A library for working with Electron [fuses] and preventing runtime behavior modifications.
//!
//! `electron-hardener` provides a way to harden Electron applications against a specific class of runtime behavior
//! modification. Specifically, if an unprivileged process can't write to the application's binary file or process
//! address space, it should not be able to change what an app does at runtime.
//!
//! This library provides two sets of functionality:
//! - An interface to view and modify the status of fuses in an application, similar to the [official fuses package].
//! - A fast and configurable alternative implementation of the [electron-evil-feature-patcher] tool created by [Dimitri Witkowski].
//!     All patches it can perform are also exposed in this crate. See its README for more details on how it works.
//!
//! Functionality is tested on a minimum version of Electron 12. Older versions may work but this is not guaranteed.
//!
//! ### A Note on Effectiveness
//!
//! Any patching this tool does is considered a strong "best effort" as the Chromium, Electron, and Node.JS teams are free to potentially
//! make changes to the argument parser, set of flags, etc.
//!
//! For a stronger assurance, consider disabling the [dev tools messages](patcher::DevToolsMessage).
//!
//! [fuses]: https://www.electronjs.org/docs/tutorial/fuses
//! [official fuses package]: https://github.com/electron/fuses
//! [electron-evil-feature-patcher]: https://github.com/antelle/electron-evil-feature-patcher
//! [Dimitri Witkowski]: https://github.com/antelle
#![warn(missing_docs)]

mod error;
pub use error::{BinaryError, PatcherError};

pub mod fuses;
pub use fuses::Fuse;

pub mod patcher;

/// An Electron application binary.
pub struct ElectronApp<'a> {
    contents: &'a mut [u8],
    wire_start: usize,
    wire_end: usize,
}
