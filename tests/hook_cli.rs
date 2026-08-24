#[cfg(unix)]
#[test]
fn desktop_hook_does_not_start_codex_app_server() {
    use std::{
        fs,
        io::Write,
        os::unix::fs::PermissionsExt,
        process::{Command, Stdio},
    };

    let directory = std::env::temp_dir().join(format!("gearbox-hook-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let marker = directory.join("codex-started");
    let fake_codex = directory.join("codex");
    fs::write(
        &fake_codex,
        format!("#!/bin/sh\ntouch '{}'\n", marker.display()),
    )
    .unwrap();
    fs::set_permissions(&fake_codex, fs::Permissions::from_mode(0o755)).unwrap();

    let mut child = Command::new(env!("CARGO_BIN_EXE_shift"))
        .arg("hook")
        .env("CODEX_GEARBOX_DISABLE_UPDATE", "1")
        .env("PATH", &directory)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(br#"{"prompt":"Rename the variable","model":"gpt-5.6-luna"}"#)
        .unwrap();

    assert!(child.wait().unwrap().success());
    assert!(!marker.exists());
    fs::remove_dir_all(directory).unwrap();
}
