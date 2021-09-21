# electron-hardener changelog

Notable changes to this project will be documented in the [keep a changelog](https://keepachangelog.com/en/1.0.0/) format.

## [Unreleased]

### Changed
* Updated minimum supported Electron version to 15.
* Deprecated patching with `NodeJsCommandLineFlag`. This has been superseded by the `NodeCliInspect` fuse.
* Deprecated patching with `DevToolsMessage`. It is no longer needed due to the functionality provided by the `NodeCliInspect` fuse.

### New
* Added support for Electron's experimental cookie encryption fuse added in version 13.
* Added suport for Electron's new fuses to disable NodeJS debugging flags and environment variables.

## [0.2.1] - 2021-06-02

### Fixed
* Fixed NodeJS flag patching on macOS and Linux Electron apps.

## [0.2.0] - 2021-06-01

### Changed

* Updated minimum supported Electron version to 13.

## [0.1.0] - 2021-04-23

Inital release

[Unreleased]: https://github.com/1Password/electron-hardener/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/1Password/electron-hardener/releases/tag/v0.1.0
[0.2.0]: https://github.com/1Password/electron-hardener/releases/tag/v0.2.0
[0.2.1]: https://github.com/1Password/electron-hardener/releases/tag/v0.2.1