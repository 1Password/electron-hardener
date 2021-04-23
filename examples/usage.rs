//! A small and basic sample of how to use the library's functionality, without needing an Electron app present.

use electron_hardener::{
    patcher::NodeJsCommandLineFlag, BinaryError, ElectronApp, Fuse, PatcherError,
};

fn main() {
    let mut application_bytes = {
        let mut bytes = include_bytes!("./fake_electron_fuses.bin").to_vec();
        bytes.extend_from_slice(include_bytes!("./fake_electron_flags.bin"));
        bytes
    };

    let mut app = ElectronApp::from_bytes(&mut application_bytes).unwrap();

    let fuse = Fuse::RunAsNode;

    let original_status = app.get_fuse_status(fuse).unwrap();
    println!("The unmodified fuse status is {:?}", original_status);

    println!("Removing RUN_AS_NODE functionality");
    app.set_fuse_status(fuse, false).unwrap();

    let new_status = app.get_fuse_status(fuse).unwrap();
    println!("The new fuse status is now {:?}", new_status);

    let flag = NodeJsCommandLineFlag::Inspect;

    println!("Removing {:?} functionality from the app", flag);
    app.patch_option(flag).unwrap();

    match app.patch_option(flag) {
        Err(PatcherError::Binary(BinaryError::NodeJsFlagNotPresent(_))) => {
            println!("Removed the Node.JS flag!")
        }
        _ => println!("Didn't remove the flag!"),
    }
}
