//! Verifies named application cgroups plus exact browser and agent process-to-cgroup discovery.

/// Discovery functions and injectable roots.
use crate::application_targets::{
    is_chatgpt_service_name,
    is_firefox_nightly_service_name,
    is_ghostty_cgroup_name,
    is_helium_service_name,
    is_interpreter_service_name,
    is_qure_service_name,
    scan_application_targets,
    ScanRoots,
};
/// Standard fixture errors.
use std::io;
/// Unix executable symlink fixture.
use std::os::unix::fs::symlink;
/// Fixture paths.
use std::path::Path;

/// Accepts Ghostty service and surface names without broad unrelated matches.
#[test]
fn ghostty_names_cover_service_and_surface() {
    assert!(is_ghostty_cgroup_name(
        "app-com.mitchellh.ghostty@abc.service"
    ));
    assert!(is_ghostty_cgroup_name(
        "app-ghostty-surface-transient-123.scope"
    ));
    assert!(!is_ghostty_cgroup_name("app-ghostty-other.scope"));
    assert!(!is_ghostty_cgroup_name(
        "app-com.mitchellh.ghostty@abc.scope"
    ));
}

/// Accepts observed Helium Chrome application ID service only.
#[test]
fn helium_service_name_uses_exact_application_id() {
    assert!(is_helium_service_name(
        "app-chrome\\x2dcadlkienfkclaiaibeoongdcgmdikeeg\\x2dDefault@abc.service"
    ));
    assert!(!is_helium_service_name(
        "app-chrome\\x2dother\\x2dDefault@abc.service"
    ));
}

/// Accepts exact Firefox Nightly service name without matching another channel or scope.
#[test]
fn firefox_nightly_service_name_stays_channel_specific() {
    assert!(is_firefox_nightly_service_name(
        "app-firefox\\x2dnightly@abc.service"
    ));
    assert!(!is_firefox_nightly_service_name(
        "app-firefox\\x2desr@abc.service"
    ));
    assert!(!is_firefox_nightly_service_name(
        "app-firefox\\x2dnightly@abc.scope"
    ));
}

/// Accepts exact ChatGPT,
///  Interpreter,
///  and Qure desktop services without matching lookalikes.
#[test]
fn agent_service_names_use_desktop_entry_identifiers() {
    assert!(is_chatgpt_service_name(
        "app-chatgpt@154672a9c04f47348b46f5514349b059.service"
    ));
    assert!(!is_chatgpt_service_name("app-chatgpt-wrapper@abc.service"));
    assert!(!is_chatgpt_service_name("app-chatgpt@abc.scope"));
    assert!(is_interpreter_service_name(
        "app-interpreter@e71b3b7638784ed28d000c8e59dce501.service"
    ));
    assert!(!is_interpreter_service_name("app-interpreter@abc.scope"));
    // The application's own executable-named scope stays executable discovery's job.
    assert!(!is_interpreter_service_name("app-interpreter-19784.scope"));
    assert!(is_qure_service_name(
        "app-qure@5f0d0f4b6c9d4a4d9d0a83b4b62f2f7c.service"
    ));
    assert!(!is_qure_service_name("app-qure-wrapper@abc.service"));
    assert!(!is_qure_service_name("app-qure@abc.scope"));
    // Qure's AppImage scope is named after the executable, not the desktop entry.
    assert!(!is_qure_service_name("app-qure-3862005.scope"));
}

/// Creates one fake proc process with executable target and unified cgroup path.
fn create_process(
    proc_root: &Path,
    pid: &str,
    executable: &str,
    cgroup: &str,
) -> io::Result<()> {
    let process = proc_root.join(pid);
    std::fs::create_dir(&process)?;
    symlink(executable, process.join("exe"))?;
    return std::fs::write(process.join("cgroup"), cgroup);
}

/// Finds named groups plus exact Helium,
///  Pale Moon,
///  and Firefox Nightly executable groups.
#[test]
fn scan_combines_named_and_process_targets() -> io::Result<()> {
    let scratch = std::env::temp_dir().join(format!(
        "wg-quicker-application-targets-{}",
        std::process::id()
    ));
    let cgroup_root = scratch.join("cgroup");
    let app_slice = cgroup_root.join("users/app.slice");
    let proc_root = scratch.join("proc");
    std::fs::create_dir_all(&app_slice)?;
    std::fs::create_dir(&proc_root)?;
    let ghostty_service = app_slice.join("app-com.mitchellh.ghostty@abc.service");
    let ghostty_surface = app_slice.join("app-ghostty-surface-transient-123.scope");
    let steam_service = app_slice.join("app-steam@abc.service");
    let steam_scope = app_slice.join("app-steam@abc.scope");
    let steam_helper = app_slice.join("app-steam-helper.service");
    let helium_service = app_slice.join(
        "app-chrome\\x2dcadlkienfkclaiaibeoongdcgmdikeeg\\x2dDefault@abc.service",
    );
    let helium_scope = app_slice.join("app-org.chromium.Chromium-42.scope");
    let pale_moon_scope = app_slice.join("app-palemoon-44.scope");
    let pale_moon_bin_scope = app_slice.join("app-palemoon-bin-45.scope");
    let firefox_nightly_service =
        app_slice.join("app-firefox\\x2dnightly@abc.service");
    let firefox_nightly_bin_scope = app_slice.join("app-firefox-nightly-bin-49.scope");
    let firefox_nightly_launcher_scope =
        app_slice.join("app-firefox-nightly-launcher-50.scope");
    let unrelated = app_slice.join("app-org.example.Other.scope");
    for path in [
        &ghostty_service,
        &ghostty_surface,
        &steam_service,
        &steam_scope,
        &steam_helper,
        &helium_service,
        &helium_scope,
        &pale_moon_scope,
        &pale_moon_bin_scope,
        &firefox_nightly_service,
        &firefox_nightly_bin_scope,
        &firefox_nightly_launcher_scope,
        &unrelated,
    ] {
        std::fs::create_dir(path)?;
    }
    create_process(
        &proc_root,
        "42",
        "/tmp/.mount_helium/opt/helium/helium",
        "1:net_cls:/\n0::/users/app.slice/app-org.chromium.Chromium-42.scope\n",
    )?;
    create_process(
        &proc_root,
        "43",
        "/usr/bin/firefox",
        "0::/users/app.slice/app-org.example.Other.scope\n",
    )?;
    // Both installed Pale Moon executable names must map to their current cgroups.
    create_process(
        &proc_root,
        "44",
        "/home/user/.local/opt/palemoon/palemoon",
        "0::/users/app.slice/app-palemoon-44.scope\n",
    )?;
    // `palemoon-bin` is byte-identical in current installation but remains valid launch name.
    create_process(
        &proc_root,
        "45",
        "/home/user/.local/opt/palemoon/palemoon-bin",
        "0::/users/app.slice/app-palemoon-bin-45.scope\n",
    )?;
    // Case and suffix near misses must not exempt unrelated executable names.
    create_process(
        &proc_root,
        "46",
        "/home/user/.local/opt/palemoon/PALEMOON",
        "0::/users/app.slice/app-org.example.Other.scope\n",
    )?;
    // Prefix lookalike verifies Pale Moon matching stays exact rather than family-wide.
    create_process(
        &proc_root,
        "47",
        "/home/user/.local/opt/palemoon/palemoon2",
        "0::/users/app.slice/app-org.example.Other.scope\n",
    )?;
    // Backup suffix verifies installed binary name cannot carry arbitrary trailing text.
    create_process(
        &proc_root,
        "48",
        "/home/user/.local/opt/palemoon/palemoon-bin.bak",
        "0::/users/app.slice/app-org.example.Other.scope\n",
    )?;
    // Firefox Nightly's actual process image must map back to its current cgroup.
    create_process(
        &proc_root,
        "49",
        "/home/user/.local/opt/firefox-nightly/firefox-bin",
        "0::/users/app.slice/app-firefox-nightly-bin-49.scope\n",
    )?;
    // Launcher image is also accepted if procfs exposes it before it replaces itself.
    create_process(
        &proc_root,
        "50",
        "/home/user/.local/opt/firefox-nightly/firefox",
        "0::/users/app.slice/app-firefox-nightly-launcher-50.scope\n",
    )?;
    // Matching Firefox ESR binary name must remain routed through the tunnel.
    create_process(
        &proc_root,
        "51",
        "/home/user/.local/opt/firefox-esr/firefox-bin",
        "0::/users/app.slice/app-org.example.Other.scope\n",
    )?;
    // Other executables inside Firefox Nightly's install directory must not widen exemption.
    create_process(
        &proc_root,
        "52",
        "/home/user/.local/opt/firefox-nightly/updater",
        "0::/users/app.slice/app-org.example.Other.scope\n",
    )?;
    // Similar parent directory must not impersonate exact Nightly installation name.
    create_process(
        &proc_root,
        "53",
        "/home/user/.local/opt/firefox-nightly-backup/firefox-bin",
        "0::/users/app.slice/app-org.example.Other.scope\n",
    )?;
    let targets = scan_application_targets(&ScanRoots {
        app_slice: &app_slice,
        proc_root: &proc_root,
        cgroup_root: &cgroup_root,
    })?;
    let mut expected = vec![
        ghostty_service,
        ghostty_surface,
        steam_service,
        helium_service,
        helium_scope,
        pale_moon_scope,
        pale_moon_bin_scope,
        firefox_nightly_service,
        firefox_nightly_bin_scope,
        firefox_nightly_launcher_scope,
    ];
    expected.sort();
    assert_eq!(targets, expected);
    std::fs::remove_dir_all(&scratch)?;
    return Ok(());
}

/// Discovers ChatGPT package-tree executables and the Interpreter executable family.
#[test]
fn scan_discovers_chatgpt_and_interpreter_cgroups() -> io::Result<()> {
    let scratch = std::env::temp_dir().join(format!(
        "wg-quicker-application-targets-agents-{}",
        std::process::id()
    ));
    let cgroup_root = scratch.join("cgroup");
    let app_slice = cgroup_root.join("users/app.slice");
    let proc_root = scratch.join("proc");
    std::fs::create_dir_all(&app_slice)?;
    std::fs::create_dir(&proc_root)?;
    let chatgpt_service = app_slice.join("app-chatgpt@abc.service");
    let chatgpt_application = app_slice.join("app-chatgpt-54.scope");
    let chatgpt_agent = app_slice.join("app-chatgpt-codex-55.scope");
    let chatgpt_crashpad = app_slice.join("app-chatgpt-crashpad-56.scope");
    let interpreter_service = app_slice.join("app-interpreter@abc.service");
    let interpreter_runtime = app_slice.join("app-interpreter-runtime-58.scope");
    let interpreter_application = app_slice.join("app-interpreter-59.scope");
    let interpreter_agent = app_slice.join("app-interpreter-exec-60.scope");
    let interpreter_cli = app_slice.join("app-interpreter-cli-61.scope");
    let unrelated = app_slice.join("app-org.example.Other.scope");
    for path in [
        &chatgpt_service,
        &chatgpt_application,
        &chatgpt_agent,
        &chatgpt_crashpad,
        &interpreter_service,
        &interpreter_runtime,
        &interpreter_application,
        &interpreter_agent,
        &interpreter_cli,
        &unrelated,
    ] {
        std::fs::create_dir(path)?;
    }
    // ChatGPT's Electron image is the process the desktop launcher leaves behind.
    create_process(
        &proc_root,
        "54",
        "/usr/lib/chatgpt/ChatGPT",
        "0::/users/app.slice/app-chatgpt-54.scope\n",
    )?;
    // A bundled ChatGPT agent keeps its exemption when the app moves it to its own cgroup.
    create_process(
        &proc_root,
        "55",
        "/usr/lib/chatgpt/resources/codex",
        "0::/users/app.slice/app-chatgpt-codex-55.scope\n",
    )?;
    // ChatGPT's crash reporter carries a Chromium-generic name inside the same package tree.
    create_process(
        &proc_root,
        "56",
        "/usr/lib/chatgpt/browser_crashpad_handler",
        "0::/users/app.slice/app-chatgpt-crashpad-56.scope\n",
    )?;
    // A sibling directory merely starting with the install name must stay tunnel-routed.
    create_process(
        &proc_root,
        "57",
        "/usr/lib/chatgpt-backup/ChatGPT",
        "0::/users/app.slice/app-org.example.Other.scope\n",
    )?;
    // Interpreter's AppImage runtime file is itself a live executable image.
    create_process(
        &proc_root,
        "58",
        "/home/user/AppImages/interpreter.appimage",
        "0::/users/app.slice/app-interpreter-runtime-58.scope\n",
    )?;
    // The mounted Electron image and its renderer helpers share this exact name.
    create_process(
        &proc_root,
        "59",
        "/tmp/.mount_interpAb12Cd/interpreter",
        "0::/users/app.slice/app-interpreter-59.scope\n",
    )?;
    // Interpreter's bundled agents extend the family prefix without a rename.
    create_process(
        &proc_root,
        "60",
        "/tmp/.mount_interpAb12Cd/resources/interpreter-exec",
        "0::/users/app.slice/app-interpreter-exec-60.scope\n",
    )?;
    // The same vendor's terminal agent shares the installed executable name.
    create_process(
        &proc_root,
        "61",
        "/home/user/.local/share/mise/installs/github-openinterpreter-openinterpreter/rust-v0.0.55/bin/interpreter",
        "0::/users/app.slice/app-interpreter-cli-61.scope\n",
    )?;
    // A shorter stem, a prefixed lookalike, and a capitalized name stay tunnel-routed,
    // while the family prefix deliberately accepts suffixes such as `interpreter-exec`.
    create_process(
        &proc_root,
        "62",
        "/usr/bin/interpret",
        "0::/users/app.slice/app-org.example.Other.scope\n",
    )?;
    create_process(
        &proc_root,
        "63",
        "/usr/bin/myinterpreter",
        "0::/users/app.slice/app-org.example.Other.scope\n",
    )?;
    create_process(
        &proc_root,
        "64",
        "/usr/bin/Interpreter",
        "0::/users/app.slice/app-org.example.Other.scope\n",
    )?;
    let targets = scan_application_targets(&ScanRoots {
        app_slice: &app_slice,
        proc_root: &proc_root,
        cgroup_root: &cgroup_root,
    })?;
    let mut expected = vec![
        chatgpt_service,
        chatgpt_application,
        chatgpt_agent,
        chatgpt_crashpad,
        interpreter_service,
        interpreter_runtime,
        interpreter_application,
        interpreter_agent,
        interpreter_cli,
    ];
    expected.sort();
    assert_eq!(targets, expected);
    std::fs::remove_dir_all(&scratch)?;
    return Ok(());
}

/// Discovers Qure's desktop service,
///  its AppImage family,
///  and the agents its CLI wrappers exec.
#[test]
fn scan_discovers_qure_cgroups() -> io::Result<()> {
    let scratch = std::env::temp_dir().join(format!(
        "wg-quicker-application-targets-qure-{}",
        std::process::id()
    ));
    let cgroup_root = scratch.join("cgroup");
    let app_slice = cgroup_root.join("users/app.slice");
    let proc_root = scratch.join("proc");
    std::fs::create_dir_all(&app_slice)?;
    std::fs::create_dir(&proc_root)?;
    let qure_service = app_slice.join("app-qure@abc.service");
    let qure_runtime = app_slice.join("app-qure-runtime-65.scope");
    let qure_chromium = app_slice.join("app-org.chromium.Chromium-66.scope");
    let qure_wrapper_agent = app_slice.join("app-qure-browser-use-67.scope");
    let qure_pytest = app_slice.join("app-qure-pytest-68.scope");
    let unrelated = app_slice.join("app-org.example.Other.scope");
    for path in [
        &qure_service,
        &qure_runtime,
        &qure_chromium,
        &qure_wrapper_agent,
        &qure_pytest,
        &unrelated,
    ] {
        std::fs::create_dir(path)?;
    }
    // Qure's AppImage runtime file is itself a live executable image.
    create_process(
        &proc_root,
        "65",
        "/home/user/AppImages/qure.appimage",
        "0::/users/app.slice/app-qure-runtime-65.scope\n",
    )?;
    // The mounted Electron image keeps its exemption after Electron moves itself to its own scope.
    create_process(
        &proc_root,
        "66",
        "/tmp/.mount_qure.anGojH1/qure-ai-assistant",
        "0::/users/app.slice/app-org.chromium.Chromium-66.scope\n",
    )?;
    // Qure's bundled CLI wrappers exec that same mounted image,
    // so a browser agent in a cgroup of its own still matches by executable name.
    create_process(
        &proc_root,
        "67",
        "/tmp/.mount_qure.anGojH1/qure-ai-assistant",
        "0::/users/app.slice/app-qure-browser-use-67.scope\n",
    )?;
    // Qure's bundled pytest runner extends the family prefix without a rename.
    create_process(
        &proc_root,
        "68",
        "/tmp/.mount_qure.anGojH1/resources/qure_pytest/qure_pytest",
        "0::/users/app.slice/app-qure-pytest-68.scope\n",
    )?;
    // Qure's crash reporter carries a Chromium-generic name,
    // so alone it stays tunnel-routed and bypasses only while sharing a matched cgroup.
    create_process(
        &proc_root,
        "69",
        "/tmp/.mount_qure.anGojH1/chrome_crashpad_handler",
        "0::/users/app.slice/app-org.example.Other.scope\n",
    )?;
    // A shorter stem, a prefixed lookalike, and a capitalized name stay tunnel-routed,
    // while the family prefix deliberately accepts suffixes such as `qure-ai-assistant`.
    create_process(
        &proc_root,
        "70",
        "/usr/bin/quer",
        "0::/users/app.slice/app-org.example.Other.scope\n",
    )?;
    create_process(
        &proc_root,
        "71",
        "/usr/bin/myqure",
        "0::/users/app.slice/app-org.example.Other.scope\n",
    )?;
    create_process(
        &proc_root,
        "72",
        "/usr/bin/Qure",
        "0::/users/app.slice/app-org.example.Other.scope\n",
    )?;
    let targets = scan_application_targets(&ScanRoots {
        app_slice: &app_slice,
        proc_root: &proc_root,
        cgroup_root: &cgroup_root,
    })?;
    let mut expected = vec![
        qure_service,
        qure_runtime,
        qure_chromium,
        qure_wrapper_agent,
        qure_pytest,
    ];
    expected.sort();
    assert_eq!(targets, expected);
    std::fs::remove_dir_all(&scratch)?;
    return Ok(());
}
