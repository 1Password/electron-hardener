use std::fmt;

/// An error that the provided binary didn't contain the required information for
/// an operation on it.
#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub enum BinaryError {
    /// No [sentinel byte marker]() could be found in the binary.
    ///
    /// [sentinel byte marker]: https://www.electronjs.org/docs/tutorial/fuses#quick-glossary
    NoSentinel,
    /// No fuse version was found in the binary.
    NoFuseVersion,
    /// The length of the fuse was not found in the binary.
    NoFuseLength,
    /// The requested fuse to be modifed wasn't present in the fuse wire.
    FuseDoesNotExist(crate::Fuse),
    /// An unknown fuse status was encountered.
    ///
    /// The Electron project may have made a breaking change to the fuse format if
    /// this is returned.
    UnknownFuse {
        /// The fuse that returned an unknown value.
        fuse: crate::Fuse,
        /// The value found querying the fuse.
        value: u8,
    },
    #[allow(deprecated)]
    /// The Node.JS command line flag attempted to be disabled wasn't present.
    NodeJsFlagNotPresent(crate::patcher::NodeJsCommandLineFlag),
    /// The Electron command line flag attempted to be disabled wasn't present.
    ElectronOptionNotPresent(crate::patcher::ElectronOption),
    #[allow(deprecated)]
    /// The Node.JS debugging message attempted to be disabled wasn't present.
    MessageNotPresent(crate::patcher::DevToolsMessage),
}

impl fmt::Display for BinaryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryError::NoSentinel => f.write_str("No fuse sentinel found"),
            BinaryError::NoFuseVersion => f.write_str("Fuse had no version present"),
            BinaryError::NoFuseLength => f.write_str("Fuse had no length specified"),
            BinaryError::FuseDoesNotExist(fuse) => write!(f, "The {:?} fuse wasn't present", fuse),
            BinaryError::UnknownFuse { fuse, value } => write!(
                f,
                "The {:?} fuse returned an unknown value of '{}'",
                fuse, value
            ),
            BinaryError::NodeJsFlagNotPresent(flag) => {
                write!(f, "The {:?} debugging flag wasn't present", flag)
            }
            BinaryError::ElectronOptionNotPresent(opt) => {
                write!(f, "The Electron option for {:?} wasn't present", opt)
            }
            BinaryError::MessageNotPresent(msg) => {
                write!(f, "The DevTools message {:?} wasn't present", msg)
            }
        }
    }
}

impl std::error::Error for BinaryError {}

/// An error that can result from parsing an Electron binary and attempting to modify it.
#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub enum PatcherError {
    /// A part of the provided binary's contents was invalid.
    Binary(BinaryError),
    /// A different fuse schema version was found then what the library supports.
    FuseVersion {
        /// The supported version of the Electron fuse schema by this library.
        expected: u8,
        /// The Electron fuse schema version found in the provided application binary.
        found: u8,
    },
    /// An attempt was made to modify a fuse which has been removed from the Electron schema.
    ///
    /// This is an error because modifying a removed fuse has no effect, so this may lead to unexpected behavior.
    RemovedFuse(crate::Fuse),
}

impl From<BinaryError> for PatcherError {
    fn from(e: BinaryError) -> Self {
        PatcherError::Binary(e)
    }
}

impl fmt::Display for PatcherError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PatcherError::Binary(e) => write!(f, "{}", e),
            PatcherError::FuseVersion { expected, found } => write!(
                f,
                "Unknown fuse version found. Expected {}, but found {}",
                expected, found
            ),
            PatcherError::RemovedFuse(fuse) => write!(
                f,
                "Failed to modify the {:?} fuse because it is marked as removed",
                fuse
            ),
        }
    }
}

impl std::error::Error for PatcherError {}
