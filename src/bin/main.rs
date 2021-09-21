//! A re-implementation of the `electron-evil-feature-patcher` CLI tool that works nearly identically.

use electron_hardener::{patcher::ElectronOption, ElectronApp, Fuse};
use std::{env, fs};

const FUSES: &[Fuse] = &[Fuse::RunAsNode, Fuse::NodeOptions, Fuse::NodeCliInspect];

const ELECTRON_FLAGS: &[ElectronOption] = &[
    ElectronOption::JsFlags,
    ElectronOption::RemoteDebuggingPipe,
    ElectronOption::RemoteDebuggingPort,
    ElectronOption::WaitForDebuggerChildren,
];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let application_path = env::args()
        .nth(1)
        .ok_or_else(|| "no file path provided".to_string())?;

    let mut application_bytes = fs::read(&application_path)?;

    let mut app = ElectronApp::from_bytes(&mut application_bytes)?;

    for fuse in FUSES.iter().copied() {
        app.set_fuse_status(fuse, false)?;
    }

    for flag in ELECTRON_FLAGS.iter().copied() {
        app.patch_option(flag)?;
    }

    fs::write(application_path, application_bytes)?;

    Ok(())
}
