//! Functionality for viewing and modifying [fuses] in an Electron application.
//!
//! [fuses]: https://www.electronjs.org/docs/tutorial/fuses

use crate::{BinaryError, ElectronApp, PatcherError};
use std::ops::Range;

#[cfg(test)]
use enum_iterator::IntoEnumIterator;

/// A representation of a [fuse] that Electron has
/// built in. They are used to disable specific functionality in the application in a way that can be enforced
/// via signature checks and codesigning at the OS level.
///
/// In the binary, fuses look like this:
/// ```text
/// | ...binary | sentinel_bytes | fuse_version | fuse_wire_length | fuse_wire | ...binary |
/// ```
///
/// Refer to the Electron project's [fuse documentation] for more details.
///
/// [fuse]: https://www.electronjs.org/docs/tutorial/fuses#the-hard-way
/// [fuse documentation]: https://www.electronjs.org/docs/tutorial/fuses#what-are-fuses
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(test, derive(IntoEnumIterator))]
#[non_exhaustive]
pub enum Fuse {
    /// Disables `ELECTRON_RUN_AS_NODE` functionality in the application.
    RunAsNode,
    /// Enables [experimental cookie encryption](https://github.com/electron/electron/pull/27524) support
    /// in the application.
    EncryptedCookies,
    /// Disbles the ability to use the [NODE_OPTIONS] environment variable on the application.
    ///
    /// [NODE_OPTIONS]: (https://nodejs.org/api/cli.html#cli_node_options_options)
    NodeOptions,
    /// Disables the ability to use the [debugging command-line flags] on the application.
    ///
    /// [debugging command-line flags](https://nodejs.org/en/docs/guides/debugging-getting-started/#command-line-options)
    NodeCliInspect,
}

#[derive(Debug, PartialEq)]
#[non_exhaustive]
/// The result of an [operation](ElectronApp::set_fuse_status) on a fuse.
pub enum FuseStatus {
    /// The fuse existed in the binary.
    Present(bool),
    /// The fuse existed in the binary and was updated with the supplied value.
    Modified,
    /// The fuse existed in the binary, but was marked as removed.
    ///
    /// The binary contents will not be modified.
    Removed,
}

impl Fuse {
    /// Marker bytes that signal where the fuse wires start inside an Electron app's bytes.
    const SENTINEL: &'static [u8] = b"dL7pKGdnNz796PbbjQWNKmHXBZaB9tsX";

    /// Marked as disabled and the feature it controls can't be used.
    const DISABLED: u8 = b'0';
    /// Marked as enabled and the feature can be used.
    const ENABLED: u8 = b'1';
    /// The fuse was removed from the [Electron schema] and marked as such.
    ///
    /// Disabling or enabling a fuse that has been removed will have no effect.
    ///
    /// [Electron schema]: https://github.com/electron/electron/blob/master/build/fuses/fuses.json
    const REMOVED: u8 = b'r';

    /// The version of the fuse schema this tool can work with.
    const EXPECTED_VERSION: u8 = 1;

    /// Returns where in the fuse wire this fuse is located.
    fn schema_pos(&self) -> usize {
        let wire_pos = match self {
            Self::RunAsNode => 1,
            Self::EncryptedCookies => 2,
            Self::NodeOptions => 3,
            Self::NodeCliInspect => 4,
        };

        wire_pos - 1
    }

    /// Locates the start of the fuses binary section.
    ///
    /// Returns the position of the fuse wire.
    pub(crate) fn find_wire(binary: &[u8]) -> Result<Range<usize>, PatcherError> {
        let sentinel_len = Self::SENTINEL.len();

        let pos = binary
            .windows(sentinel_len)
            .position(|slice| slice == Self::SENTINEL)
            .ok_or(BinaryError::NoSentinel)?;

        let start = pos + sentinel_len;

        let version = binary.get(start).ok_or(BinaryError::NoFuseVersion)?;

        if *version != Self::EXPECTED_VERSION {
            return Err(PatcherError::FuseVersion {
                expected: Self::EXPECTED_VERSION,
                found: *version,
            });
        }

        let len_pos = start + 1;
        let wire_len = binary.get(len_pos).ok_or(BinaryError::NoFuseLength)?;

        let wire_start = len_pos + 1;
        let fuse_bytes = (wire_start)..(wire_start + usize::from(*wire_len));

        Ok(fuse_bytes)
    }

    fn fuse_status(&self, wire: &[u8]) -> Result<FuseStatus, PatcherError> {
        let status = wire
            .get(self.schema_pos())
            .ok_or(BinaryError::FuseDoesNotExist(*self))?;

        let status = match *status {
            Self::ENABLED => FuseStatus::Present(true),
            Self::DISABLED => FuseStatus::Present(false),
            Self::REMOVED => FuseStatus::Removed,
            s => {
                return Err(BinaryError::UnknownFuse {
                    fuse: *self,
                    value: s,
                }
                .into())
            }
        };

        Ok(status)
    }

    fn disable(&self, wire: &mut [u8]) -> Result<FuseStatus, PatcherError> {
        let mut enabled = self.fuse_status(wire)?;

        match enabled {
            FuseStatus::Present(e) if e => {
                wire[self.schema_pos()] = Self::DISABLED;
                enabled = FuseStatus::Modified
            }
            FuseStatus::Removed => return Err(PatcherError::RemovedFuse(*self)),
            _ => {}
        }

        Ok(enabled)
    }

    fn enable(&self, wire: &mut [u8]) -> Result<FuseStatus, PatcherError> {
        let mut enabled = self.fuse_status(wire)?;

        match enabled {
            FuseStatus::Present(e) if !e => {
                wire[self.schema_pos()] = Self::ENABLED;
                enabled = FuseStatus::Modified
            }
            FuseStatus::Removed => return Err(PatcherError::RemovedFuse(*self)),
            _ => {}
        }

        Ok(enabled)
    }
}

impl<'a> ElectronApp<'a> {
    /// Constructs a new [electron app](Self) and verifies that the bytes came from
    /// a packaged Electron app binary file.
    ///
    /// # Errors
    ///
    /// This function returns an error if the bytes couldn't be validated to contain an Electron application.
    pub fn from_bytes(application_bytes: &'a mut [u8]) -> Result<ElectronApp<'a>, PatcherError> {
        let wire_pos = Fuse::find_wire(application_bytes)?;

        Ok(Self {
            contents: application_bytes,
            wire_start: wire_pos.start,
            wire_end: wire_pos.end,
        })
    }

    /// Parses and returns this fuse type's status in the provided binary.
    ///
    /// # Return
    ///
    /// Returns the current fuse status. This will not return a [modification result](FuseResult::Modified).
    ///
    /// # Errors
    ///
    /// This function will return an error if an invalid binary is provided or one that is not an Electron application.
    pub fn get_fuse_status(&self, fuse: Fuse) -> Result<FuseStatus, PatcherError> {
        let wire = &self.contents[self.wire_start..self.wire_end];
        fuse.fuse_status(wire)
    }

    /// Toggles a fuse in the application binary based off the provided value.
    ///
    /// # Return
    ///
    /// Returns the [result](FuseResult) of the operation if it succeeded.
    ///
    /// # Errors
    ///
    /// This function will return an error if a fuse wire couldn't be found in the provided binary or
    /// if a modification of a removed fuse was attempted.
    pub fn set_fuse_status(
        &mut self,
        fuse: Fuse,
        enabled: bool,
    ) -> Result<FuseStatus, PatcherError> {
        let wire = &mut self.contents[self.wire_start..self.wire_end];

        if enabled {
            fuse.enable(wire)
        } else {
            fuse.disable(wire)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_BYTES: &[u8] = include_bytes!("../examples/fake_electron_fuses.bin");
    const FUSE: Fuse = Fuse::RunAsNode;

    fn get_wire() -> &'static [u8] {
        let wire_pos = Fuse::find_wire(TEST_BYTES).unwrap();
        &TEST_BYTES[wire_pos]
    }

    #[test]
    fn sentinal_is_found() {
        assert!(Fuse::find_wire(TEST_BYTES).is_ok());
    }

    #[test]
    fn enabled_fuse_is_correct() {
        assert_eq!(
            FUSE.fuse_status(get_wire()).unwrap(),
            FuseStatus::Present(true)
        );
    }

    #[test]
    fn disabled_fuse_is_correct() {
        let mut wire = get_wire().to_vec();
        assert_eq!(FUSE.disable(&mut wire).unwrap(), FuseStatus::Modified);
        assert_eq!(FUSE.fuse_status(&wire).unwrap(), FuseStatus::Present(false));
    }

    #[test]
    fn removed_fuse_is_correct() {
        let mut wire = get_wire().to_vec();
        wire[FUSE.schema_pos()] = Fuse::REMOVED;

        assert_eq!(FUSE.fuse_status(&wire).unwrap(), FuseStatus::Removed);
    }

    #[test]
    fn unknown_fuse_value_is_correct() {
        let value = 9;
        let mut wire = get_wire().to_vec();
        wire[FUSE.schema_pos()] = value;

        assert_eq!(
            FUSE.fuse_status(&wire),
            Err(PatcherError::Binary(BinaryError::UnknownFuse {
                fuse: FUSE,
                value,
            }))
        );
    }

    #[test]
    fn modfying_removed_fuse_errors() {
        let mut wire = get_wire().to_vec();
        wire[FUSE.schema_pos()] = Fuse::REMOVED;

        assert_eq!(
            FUSE.disable(&mut wire),
            Err(PatcherError::RemovedFuse(FUSE))
        );
        assert_eq!(FUSE.enable(&mut wire), Err(PatcherError::RemovedFuse(FUSE)));
    }

    #[test]
    fn test_app_fuse_actions() {
        let mut application_bytes = TEST_BYTES.to_vec();
        let mut app = ElectronApp::from_bytes(&mut application_bytes).unwrap();

        assert_eq!(
            app.get_fuse_status(FUSE).unwrap(),
            FuseStatus::Present(true)
        );

        // Setting the fuse to what it already is shouldn't modify anything.
        assert_eq!(
            app.set_fuse_status(FUSE, true).unwrap(),
            FuseStatus::Present(true)
        );

        assert_eq!(
            app.set_fuse_status(FUSE, false).unwrap(),
            FuseStatus::Modified
        );
        assert_eq!(
            app.get_fuse_status(FUSE).unwrap(),
            FuseStatus::Present(false)
        );
    }

    #[test]
    fn can_read_all_fuses() {
        let wire = get_wire();

        for fuse in Fuse::into_enum_iter() {
            assert!(matches!(
                fuse.fuse_status(wire).unwrap(),
                FuseStatus::Present(_)
            ));
        }
    }

    #[test]
    fn fuse_modifies_correct_position() {
        let mut wire = get_wire().to_vec();

        let fuse1 = Fuse::RunAsNode;
        let fuse2 = Fuse::EncryptedCookies;
        let fuse3 = Fuse::NodeOptions;

        let fuse_2_original_status = fuse2.fuse_status(&wire).unwrap();

        fuse1.disable(&mut wire).unwrap();

        // Check that modifying one fuse doesn't affect others.
        assert_eq!(fuse2.fuse_status(&wire).unwrap(), fuse_2_original_status);

        let fuse_1_original_status = fuse1.fuse_status(&wire).unwrap();

        fuse2.disable(&mut wire).unwrap();

        assert_eq!(fuse1.fuse_status(&wire).unwrap(), fuse_1_original_status);

        let left_fuse_original_status = fuse1.fuse_status(&wire).unwrap();
        let right_fuse_original_status = fuse3.fuse_status(&wire).unwrap();

        fuse2.enable(&mut wire).unwrap();

        assert_eq!(fuse1.fuse_status(&wire).unwrap(), left_fuse_original_status);
        assert_eq!(
            fuse3.fuse_status(&wire).unwrap(),
            right_fuse_original_status
        );
    }
}
