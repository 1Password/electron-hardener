//! A re-implementation of the `electron-evil-feature-patcher` CLI tool that works nearly identically.

use electron_hardener::{
    patcher::{DevToolsMessage, ElectronOption, NodeJsCommandLineFlag},
    ElectronApp, Fuse,
};
use std::{env, fs};

const FUSES: &[Fuse] = &[Fuse::RunAsNode];

const NODEJS_FLAGS: &[NodeJsCommandLineFlag] = &[
    NodeJsCommandLineFlag::Inspect,
    NodeJsCommandLineFlag::InspectBrk,
    NodeJsCommandLineFlag::InspectPort,
    NodeJsCommandLineFlag::Debug,
    NodeJsCommandLineFlag::DebugBrk,
    NodeJsCommandLineFlag::DebugPort,
    NodeJsCommandLineFlag::InspectBrkNode,
    NodeJsCommandLineFlag::InspectPublishUid,
];

const ELECTRON_FLAGS: &[ElectronOption] = &[
    ElectronOption::JsFlags,
    ElectronOption::RemoteDebuggingPipe,
    ElectronOption::RemoteDebuggingPort,
    ElectronOption::WaitForDebuggerChildren,
];

const DEVTOOLS_MESSAGES: &[DevToolsMessage] =
    &[DevToolsMessage::Listening, DevToolsMessage::ListeningWs];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let application_path = env::args()
        .nth(1)
        .ok_or_else(|| "no file path provided".to_string())?;

    let mut application_bytes = fs::read(&application_path)?;

    let mut app = ElectronApp::from_bytes(&mut application_bytes)?;

    for fuse in FUSES.iter().copied() {
        app.set_fuse_status(fuse, false)?;
    }

    for flag in NODEJS_FLAGS.iter().copied() {
        app.patch_option(flag)?;
    }

    for flag in ELECTRON_FLAGS.iter().copied() {
        app.patch_option(flag)?;
    }

    for msg in DEVTOOLS_MESSAGES.iter().copied() {
        app.patch_option(msg)?;
    }

    fs::write(application_path, application_bytes)?;

    Ok(())
}
