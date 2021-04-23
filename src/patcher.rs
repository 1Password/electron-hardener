//! A configurable and fast implementation of [electron-evil-feature-patcher]'s binary patching capabilites.
//!
//! [electron-evil-feature-patcher]: https://github.com/antelle/electron-evil-feature-patcher

use crate::{BinaryError, ElectronApp, PatcherError};
use regex::bytes::Regex;

#[cfg(test)]
use enum_iterator::IntoEnumIterator;

/// A flag inside an Electron application binary that can be patched to disable it.
pub trait Patchable: private::Sealed {
    #[doc(hidden)]
    /// Disables the option.
    ///
    /// You are probably looking for [patch_option](ElectronApp::patch_option).
    fn disable(&self, binary: &mut [u8]) -> Result<(), PatcherError>;
}

mod private {
    use super::{DevToolsMessage, ElectronOption, NodeJsCommandLineFlag};

    pub trait Sealed {}

    impl Sealed for NodeJsCommandLineFlag {}
    impl Sealed for ElectronOption {}
    impl Sealed for DevToolsMessage {}
}

/// List of known command line debugging flags that can be disabled
///
/// See the [Node.JS documentation] for details on what each flag does.
///
/// [Node.JS documentation]: https://nodejs.org/en/docs/guides/debugging-getting-started/#command-line-options
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(test, derive(IntoEnumIterator))]
#[non_exhaustive]
pub enum NodeJsCommandLineFlag {
    Inspect,
    InspectBrk,
    InspectPort,
    Debug,
    DebugBrk,
    DebugPort,
    InspectBrkNode,
    InspectPublishUid,
}

impl NodeJsCommandLineFlag {
    const fn search_string(&self) -> &'static str {
        match self {
            Self::Inspect => "\0--inspect\0",
            Self::InspectBrk => "\0--inspect-brk\0",
            Self::InspectPort => "\0--inspect-port\0",
            Self::Debug => "\0--debug\0",
            Self::DebugBrk => "\0--debug-brk\0",
            Self::DebugPort => "\0--debug-port\0",
            Self::InspectBrkNode => "\0--inspect-brk-node\0",
            Self::InspectPublishUid => "\0--inspect-publish-uid\0",
        }
    }
}

impl Patchable for NodeJsCommandLineFlag {
    fn disable(&self, binary: &mut [u8]) -> Result<(), PatcherError> {
        let search = Regex::new(self.search_string()).expect("all regex patterns should be valid");
        let found = search
            .find(binary)
            .ok_or(BinaryError::NodeJsFlagNotPresent(*self))?
            .range();

        for b in &mut binary[found] {
            if *b == b'-' {
                *b = b' '
            }
        }

        Ok(())
    }
}

/// List of known Electron command line flags that can be disabled.
///
/// See the [Electron documentation] for details on what each flag does.
///
/// [Electron documentation]: https://www.electronjs.org/docs/api/command-line-switches
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(test, derive(IntoEnumIterator))]
#[non_exhaustive]
pub enum ElectronOption {
    JsFlags,
    RemoteDebuggingPipe,
    RemoteDebuggingPort,
    WaitForDebuggerChildren,
}

impl ElectronOption {
    const fn search_string(&self) -> &'static str {
        match self {
            Self::JsFlags => "\0js-flags\0",
            Self::RemoteDebuggingPipe => "\0remote-debugging-pipe\0",
            Self::RemoteDebuggingPort => "\0remote-debugging-port\0",
            Self::WaitForDebuggerChildren => "\0wait-for-debugger-children\0",
        }
    }
}

impl Patchable for ElectronOption {
    fn disable(&self, binary: &mut [u8]) -> Result<(), PatcherError> {
        let search = Regex::new(self.search_string()).expect("all regex patterns should be valid");
        let found = search
            .find(binary)
            .ok_or(BinaryError::ElectronOptionNotPresent(*self))?
            .range();

        let replacement = b"\0xx\r\n"
            .iter()
            .copied()
            .chain(std::iter::repeat(0))
            .take(found.len());

        for (old, new) in binary[found].iter_mut().zip(replacement) {
            *old = new;
        }

        Ok(())
    }
}

/// List of known developer tool command line messages that can be
/// written to stdout by Node.JS during debugging.
///
/// ### Warning
///
/// Disabling these is a worst-case fallback protection against internal changes to the way
/// that Chromium/Electron/Node.JS handle parsing command line arguments. If something is changed
/// and a debugging flag slips through, modifying one of these will cause the application to trigger a segemntation fault
/// and be terminated by the OS, exiting immediately.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(test, derive(IntoEnumIterator))]
#[non_exhaustive]
pub enum DevToolsMessage {
    /// The message printed to standard out when Node.JS listens on TCP port.
    ///
    /// Ex: `Debugger listening on 127.0.0.1:9229/uuid`
    Listening,
    /// The message printed to standard out when Node.JS listens on a websocket.
    ///
    /// Ex: `Debugger listening on ws://127.0.0.1:9229/uuid`
    ListeningWs,
}

impl DevToolsMessage {
    const fn search_string(&self) -> &'static str {
        match self {
            Self::Listening => "\0Debugger listening on %s\n\0",
            Self::ListeningWs => "\0\nDevTools listening on ws://%s%s\n\0",
        }
    }
}

impl Patchable for DevToolsMessage {
    fn disable(&self, binary: &mut [u8]) -> Result<(), PatcherError> {
        let search = Regex::new(self.search_string()).expect("all regex patterns should be valid");
        let found = search
            .find(binary)
            .ok_or(BinaryError::MessageNotPresent(*self))?
            .range();

        let mut replacement = Vec::with_capacity(found.len());
        replacement.push(b'\0');
        let str_len = found.len() - 3;
        for _ in (0..str_len).step_by(2) {
            replacement.push(b'%');
            replacement.push(b's');
        }
        replacement.extend_from_slice(b"\n\0");

        for (old, new) in binary[found].iter_mut().zip(replacement) {
            *old = new;
        }

        Ok(())
    }
}

impl ElectronApp<'_> {
    /// Disables the ability to use this command line flag in the application.
    ///
    /// After being disabled, the flag will no longer be processed by the application. The removal
    /// is a best-effort attempt. See the [crate documentation on effectiveness](crate).
    pub fn patch_option<P: Patchable>(&mut self, to_disable: P) -> Result<(), PatcherError> {
        to_disable.disable(self.contents)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_DATA: &[u8] = include_bytes!("../examples/fake_electron_flags.bin");

    #[test]
    fn disabling_nodejs_flags_works() {
        let mut data = TEST_DATA.to_vec();

        // Remove all the flags supported.
        for flag in NodeJsCommandLineFlag::into_enum_iter() {
            flag.disable(&mut data).unwrap();
        }

        // Ensure they no longer exist
        for flag in NodeJsCommandLineFlag::into_enum_iter() {
            assert_eq!(
                flag.disable(&mut data),
                Err(PatcherError::Binary(BinaryError::NodeJsFlagNotPresent(
                    flag
                )))
            );
        }
    }

    #[test]
    fn disabling_electron_options_works() {
        let mut data = TEST_DATA.to_vec();

        // Remove all the options supported.
        for opt in ElectronOption::into_enum_iter() {
            opt.disable(&mut data).unwrap();
        }

        // Ensure they no longer exist
        for opt in ElectronOption::into_enum_iter() {
            assert_eq!(
                opt.disable(&mut data),
                Err(PatcherError::Binary(BinaryError::ElectronOptionNotPresent(
                    opt
                )))
            );
        }
    }

    #[test]
    fn disabling_debugging_messages_works() {
        let mut data = TEST_DATA.to_vec();

        // Remove all the options supported.
        for msg in DevToolsMessage::into_enum_iter() {
            msg.disable(&mut data).unwrap();
        }

        // Ensure they no longer exist
        for msg in DevToolsMessage::into_enum_iter() {
            assert_eq!(
                msg.disable(&mut data),
                Err(PatcherError::Binary(BinaryError::MessageNotPresent(msg)))
            );
        }
    }
}
