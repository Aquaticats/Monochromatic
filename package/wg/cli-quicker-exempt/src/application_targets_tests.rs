//! Verifies named application cgroups plus exact browser and agent process-to-cgroup discovery.

/// The applications a name claims.
use crate::desktop_applications::{
    desktop_application_of_name,
    is_application_executable,
    is_chatgpt_service_name,
    is_firefox_nightly_service_name,
    is_firefox_release_service_name,
    is_ghostty_cgroup_name,
    is_helium_service_name,
    is_interpreter_service_name,
    is_odytty_service_name,
    is_qure_service_name,
    DesktopApplication,
};
/// Discovery functions and injectable roots.
use crate::application_targets::{
    is_pi_service_name,
    is_in_updates_slice,
    is_unconditionally_marked_cgroup,
    is_toolchain_updates_slice_name,
    scan_application_targets,
    ScanRoots,
    TOOLCHAIN_UPDATES_SLICE,
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

/// What:
/// Registers a unit test of release desktop-service identity.
/// Why:
/// A release exception must leave ESR,
/// Nightly,
/// and lookalike service names unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// test('release service identity', () => {
///   expect(isFirefoxReleaseServiceName('app-firefox@fixture.service')).toBe(true);
/// });
/// ```
#[test]
fn firefox_release_service_name_stays_channel_specific() {
    // Assert the release desktop service is accepted.
    assert!(is_firefox_release_service_name("app-firefox@fixture.service"));
    // Assert the ESR desktop service remains tunnel-routed.
    assert!(!is_firefox_release_service_name("app-firefox\\x2desr@fixture.service"));
    // Nightly has its own matcher and must not become a release-service match.
    assert!(!is_firefox_release_service_name("app-firefox\\x2dnightly@fixture.service"));
    // A similarly named scope is selected only by executable discovery.
    assert!(!is_firefox_release_service_name("app-firefox@fixture.scope"));
    // A wrapper service does not identify the release browser.
    assert!(!is_firefox_release_service_name("app-firefox-wrapper@fixture.service"));
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
    assert!(is_odytty_service_name(
        "app-odytty@84473970c0f544048c4719b8475197d8.service"
    ));
    assert!(!is_odytty_service_name("app-odytty-wrapper@abc.service"));
    assert!(!is_odytty_service_name("app-odytty@abc.scope"));
    // OdyTTY's AppImage scope is named after the executable, not the desktop entry.
    assert!(!is_odytty_service_name("app-odytty-10290.scope"));
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
    let scratch = crate::test_scratch::scratch_root().join(format!(
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
    let firefox_release_service = app_slice.join("app-firefox@fixture.service");
    let firefox_release_bin_scope = app_slice.join("app-release-bin.scope");
    let firefox_release_launcher_scope = app_slice.join("app-release-launcher.scope");
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
        // Borrow the release fixtures so the expected-result assertion can reuse them.
        &firefox_release_service,
        &firefox_release_bin_scope,
        &firefox_release_launcher_scope,
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
    // Each named desktop application's own program is in the cgroup its name claims.
    create_process(
        &proc_root,
        "60",
        "/usr/bin/ghostty",
        "0::/users/app.slice/app-com.mitchellh.ghostty@abc.service\n",
    )?;
    create_process(
        &proc_root,
        "61",
        "/usr/bin/ghostty",
        "0::/users/app.slice/app-ghostty-surface-transient-123.scope\n",
    )?;
    create_process(
        &proc_root,
        "62",
        "/var/home/user/.local/share/Steam/ubuntu12_32/steam",
        "0::/users/app.slice/app-steam@abc.service\n",
    )?;
    create_process(
        &proc_root,
        "63",
        "/tmp/.mount_helium/opt/helium/helium",
        "0::/users/app.slice/app-chrome\\x2dcadlkienfkclaiaibeoongdcgmdikeeg\\x2dDefault@abc.service\n",
    )?;
    create_process(
        &proc_root,
        "64",
        "/home/user/.local/opt/firefox-nightly/firefox-bin",
        "0::/users/app.slice/app-firefox\\x2dnightly@abc.service\n",
    )?;
    create_process(
        &proc_root,
        "65",
        "/fixture/opt/firefox/firefox",
        "0::/users/app.slice/app-firefox@fixture.service\n",
    )?;
    // Borrow the fixture procfs and propagate setup errors for both release executable names.
    create_process(
        &proc_root,
        "54",
        "/fixture/opt/firefox/firefox-bin",
        "0::/users/app.slice/app-release-bin.scope\n",
    )?;
    // Borrow the fixture root for the release launcher before it execs firefox-bin.
    create_process(
        &proc_root,
        "55",
        "/fixture/opt/firefox/firefox",
        "0::/users/app.slice/app-release-launcher.scope\n",
    )?;
    // A backup directory must not acquire the release browser's exemption.
    create_process(
        &proc_root,
        "56",
        "/fixture/opt/firefox-backup/firefox-bin",
        "0::/users/app.slice/app-org.example.Other.scope\n",
    )?;
    // Other programs in the release directory are not browser executable identities.
    create_process(
        &proc_root,
        "57",
        "/fixture/opt/firefox/updater",
        "0::/users/app.slice/app-org.example.Other.scope\n",
    )?;
    let scan = scan_application_targets(&ScanRoots {
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
        firefox_release_service,
        firefox_release_bin_scope,
        firefox_release_launcher_scope,
    ];
    expected.sort();
    assert_eq!(scan.targets, expected);
    assert!(scan.without_application.is_empty(), "{:?}", scan.without_application);
    std::fs::remove_dir_all(&scratch)?;
    return Ok(());
}

/// Discovers ChatGPT package-tree executables and the Interpreter executable family.
#[test]
fn scan_discovers_chatgpt_and_interpreter_cgroups() -> io::Result<()> {
    let scratch = crate::test_scratch::scratch_root().join(format!(
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
    // The desktop integration service of each holds that application's own program.
    create_process(
        &proc_root,
        "53",
        "/usr/lib/chatgpt/ChatGPT",
        "0::/users/app.slice/app-chatgpt@abc.service\n",
    )?;
    create_process(
        &proc_root,
        "52",
        "/home/user/AppImages/interpreter.appimage",
        "0::/users/app.slice/app-interpreter@abc.service\n",
    )?;
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
    let scan = scan_application_targets(&ScanRoots {
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
    assert_eq!(scan.targets, expected);
    assert!(scan.without_application.is_empty(), "{:?}", scan.without_application);
    std::fs::remove_dir_all(&scratch)?;
    return Ok(());
}

/// Discovers Qure's desktop service,
///  its AppImage family,
///  and the agents its CLI wrappers exec.
#[test]
fn scan_discovers_qure_cgroups() -> io::Result<()> {
    let scratch = crate::test_scratch::scratch_root().join(format!(
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
    // The desktop integration service holds Qure's own program.
    create_process(
        &proc_root,
        "64",
        "/home/user/AppImages/qure.appimage",
        "0::/users/app.slice/app-qure@abc.service\n",
    )?;
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
    let scan = scan_application_targets(&ScanRoots {
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
    assert_eq!(scan.targets, expected);
    assert!(scan.without_application.is_empty(), "{:?}", scan.without_application);
    std::fs::remove_dir_all(&scratch)?;
    return Ok(());
}

/// Discovers OdyTTY's desktop service and its AppImage family,
///  including a launch outside desktop integration that gets an executable-named scope.
#[test]
fn scan_discovers_odytty_cgroups() -> io::Result<()> {
    let scratch = crate::test_scratch::scratch_root().join(format!(
        "wg-quicker-application-targets-odytty-{}",
        std::process::id()
    ));
    let cgroup_root = scratch.join("cgroup");
    let app_slice = cgroup_root.join("users/app.slice");
    let proc_root = scratch.join("proc");
    std::fs::create_dir_all(&app_slice)?;
    std::fs::create_dir(&proc_root)?;
    let odytty_service = app_slice.join("app-odytty@abc.service");
    let odytty_runtime = app_slice.join("app-odytty-runtime-73.scope");
    let unrelated = app_slice.join("app-org.example.Other.scope");
    for path in [&odytty_service, &odytty_runtime, &unrelated] {
        std::fs::create_dir(path)?;
    }
    // The desktop entry runs the AppImage file itself inside the desktop service cgroup.
    create_process(
        &proc_root,
        "73",
        "/home/user/AppImages/odytty.appimage",
        "0::/users/app.slice/app-odytty@abc.service\n",
    )?;
    // A launch from a shell lands in a scope named after the executable.
    create_process(
        &proc_root,
        "74",
        "/home/user/AppImages/odytty.appimage",
        "0::/users/app.slice/app-odytty-runtime-73.scope\n",
    )?;
    // A shorter stem, a prefixed lookalike, and a capitalized name stay tunnel-routed.
    create_process(
        &proc_root,
        "75",
        "/usr/bin/odytt",
        "0::/users/app.slice/app-org.example.Other.scope\n",
    )?;
    create_process(
        &proc_root,
        "76",
        "/usr/bin/myodytty",
        "0::/users/app.slice/app-org.example.Other.scope\n",
    )?;
    create_process(
        &proc_root,
        "77",
        "/usr/bin/OdyTTY",
        "0::/users/app.slice/app-org.example.Other.scope\n",
    )?;
    let scan = scan_application_targets(&ScanRoots {
        app_slice: &app_slice,
        proc_root: &proc_root,
        cgroup_root: &cgroup_root,
    })?;
    let mut expected = vec![odytty_service, odytty_runtime];
    expected.sort();
    assert_eq!(scan.targets, expected);
    assert!(scan.without_application.is_empty(), "{:?}", scan.without_application);
    std::fs::remove_dir_all(&scratch)?;
    return Ok(());
}

/// Accepts exactly the toolchain updates slice name,
///  never a scope inside it,
///  a lookalike with a suffix,
///  the nested-hierarchy spelling,
///  or the name without the `app-` prefix that would sit outside `app.slice`.
#[test]
fn toolchain_updates_slice_name_is_exact() {
    assert!(is_toolchain_updates_slice_name(
        "app-monochromatic\\x2dupdates.slice"
    ));
    assert_eq!(
        TOOLCHAIN_UPDATES_SLICE,
        "app-monochromatic\\x2dupdates.slice"
    );
    assert!(!is_toolchain_updates_slice_name(
        "app-monochromatic\\x2dupdates-extra.slice"
    ));
    assert!(!is_toolchain_updates_slice_name(
        "app-monochromatic\\x2dupdates.scope"
    ));
    // An unescaped dash names `app.slice/app-monochromatic.slice/app-monochromatic-updates.slice`.
    assert!(!is_toolchain_updates_slice_name(
        "app-monochromatic-updates.slice"
    ));
    assert!(!is_toolchain_updates_slice_name("monochromatic-updates.slice"));
    assert!(!is_toolchain_updates_slice_name(
        "app-Monochromatic\\x2dupdates.slice"
    ));
}

/// Attaches the updates slice itself,
///  not the run scopes below it,
///  and leaves lookalike names alone.
#[test]
fn scan_discovers_only_the_toolchain_updates_slice() -> io::Result<()> {
    let scratch = crate::test_scratch::scratch_root().join(format!(
        "wg-quicker-application-targets-updates-{}",
        std::process::id()
    ));
    let cgroup_root = scratch.join("cgroup");
    let app_slice = cgroup_root.join("users/app.slice");
    let proc_root = scratch.join("proc");
    std::fs::create_dir_all(&app_slice)?;
    std::fs::create_dir(&proc_root)?;
    let updates_slice = app_slice.join("app-monochromatic\\x2dupdates.slice");
    // A scope created by `systemd-run --slice` lives below the slice and inherits its programs.
    let run_scope = updates_slice.join("run-p4242-i4343.scope");
    let suffixed_slice = app_slice.join("app-monochromatic\\x2dupdates-extra.slice");
    let nested_hierarchy_slice = app_slice.join("app-monochromatic-updates.slice");
    let unrelated = app_slice.join("app-org.example.Other.scope");
    std::fs::create_dir(&updates_slice)?;
    std::fs::create_dir(&run_scope)?;
    for path in [&suffixed_slice, &nested_hierarchy_slice, &unrelated] {
        std::fs::create_dir(path)?;
    }
    // A sibling of `app.slice`, the place a name without the `app-` prefix would land,
    // is outside the watched directory.
    let outside = cgroup_root.join("users/monochromatic-updates.slice");
    std::fs::create_dir(&outside)?;
    let scan = scan_application_targets(&ScanRoots {
        app_slice: &app_slice,
        proc_root: &proc_root,
        cgroup_root: &cgroup_root,
    })?;
    assert_eq!(scan.targets, vec![updates_slice]);
    assert!(scan.without_application.is_empty(), "{:?}", scan.without_application);
    std::fs::remove_dir_all(&scratch)?;
    return Ok(());
}

/// Accepts every `pi-<name>.service` agent unit the owner's Pi runs in,
/// and nothing that merely shares its letters.
#[test]
fn pi_agent_service_names_are_marked_by_name() {
    assert!(is_pi_service_name(
        "pi-wg-quicker-model-providers-off-the-tunnel.service"
    ));
    assert!(is_pi_service_name(
        "pi-commit-scratch-removal-off-the-path.service"
    ));
    // The unit must name something after the prefix.
    assert!(!is_pi_service_name("pi-.service"));
    // An interactive Pi is no unit, and no scope carries the service suffix.
    assert!(!is_pi_service_name("pi.service"));
    assert!(!is_pi_service_name("pi-agent.scope"));
    // Another name that starts with the same letters is another program.
    assert!(!is_pi_service_name("piwork-3.slice"));
    assert!(!is_pi_service_name("not-pi-agent.service"));
    // Any other unit whose name starts with the same letters is marked too,
    // because a unit name is a name the user can choose.
    assert!(is_pi_service_name("pi-agent@abc.service"));
}

/// Names the cgroups the watcher marks by name alone:
/// a Pi agent's unit and the updates slice,
/// the two names the owner accepted as names alone.
#[test]
fn only_a_pi_unit_and_the_slice_are_marked_by_name_alone() {
    let prefix = "/user.slice/user-1000.slice/user@1000.service/app.slice/";
    for leaf in ["pi-wg-quicker-model-providers-off-the-tunnel.service", TOOLCHAIN_UPDATES_SLICE] {
        let cgroup = format!("{prefix}{leaf}");
        assert!(is_unconditionally_marked_cgroup(&cgroup, 1000), "{leaf}");
    }
    // A desktop application's name is not enough: the cgroup it names must hold that application's program.
    for leaf in [
        "app-com.mitchellh.ghostty@abc.service",
        "app-ghostty-surface-transient-123.scope",
        "app-steam@abc.service",
        "app-firefox@abc.service",
        "app-chatgpt@abc.service",
    ] {
        let cgroup = format!("{prefix}{leaf}");
        assert!(!is_unconditionally_marked_cgroup(&cgroup, 1000), "{leaf}");
    }
    // A cgroup below a marked one inherits the program but is not the entry itself.
    assert!(!is_unconditionally_marked_cgroup(
        &format!("{prefix}pi-agent.service/nested.scope"),
        1000
    ));
    // A service the watcher does not name is left to the process watch.
    assert!(!is_unconditionally_marked_cgroup(&format!("{prefix}other.service"), 1000));
    assert!(!is_unconditionally_marked_cgroup(&format!("{prefix}pi-agent.scope"), 1000));
    // Only the watched user's app slice is scanned.
    assert!(!is_unconditionally_marked_cgroup(&format!("{prefix}pi-agent.service"), 2000));
    assert!(!is_unconditionally_marked_cgroup(
        "/user.slice/user-1000.slice/user@1000.service/pi-agent.service",
        1000
    ));
    assert!(!is_unconditionally_marked_cgroup(
        "/user.slice/user-1000.slice/session-3.scope/pi-agent.service",
        1000
    ));
    assert!(!is_unconditionally_marked_cgroup("/", 1000));
}

/// Says which desktop application a cgroup name claims,
/// and which program of that application a cgroup must hold.
#[test]
fn a_cgroup_name_claims_one_application_and_that_application_has_one_program() {
    assert_eq!(
        desktop_application_of_name("app-com.mitchellh.ghostty@abc.service"),
        Some(DesktopApplication::Ghostty)
    );
    assert_eq!(
        desktop_application_of_name("app-ghostty-surface-transient-1.scope"),
        Some(DesktopApplication::Ghostty)
    );
    assert_eq!(desktop_application_of_name("app-steam@abc.service"), Some(DesktopApplication::Steam));
    assert_eq!(
        desktop_application_of_name("app-chatgpt@abc.service"),
        Some(DesktopApplication::Chatgpt)
    );
    assert_eq!(desktop_application_of_name("app-org.example.Other.scope"), None);
    // A Pi unit and the slice are names alone, so no application's program is asked for.
    assert_eq!(
        desktop_application_of_name("pi-wg-quicker-model-providers-off-the-tunnel.service"),
        None
    );
    assert!(is_application_executable(DesktopApplication::Ghostty, Path::new("/usr/bin/ghostty")));
    assert!(!is_application_executable(DesktopApplication::Ghostty, Path::new("/usr/bin/ghostty-wrapper")));
    assert!(is_application_executable(
        DesktopApplication::Steam,
        Path::new("/var/home/user/.local/share/Steam/ubuntu12_32/steam")
    ));
    assert!(!is_application_executable(DesktopApplication::Steam, Path::new("/usr/bin/steamcmd")));
    assert!(is_application_executable(
        DesktopApplication::FirefoxRelease,
        Path::new("/fixture/opt/firefox/firefox-bin")
    ));
    assert!(!is_application_executable(
        DesktopApplication::FirefoxRelease,
        Path::new("/usr/bin/firefox")
    ));
    assert!(is_application_executable(
        DesktopApplication::Chatgpt,
        Path::new("/usr/lib/chatgpt/chatgpt")
    ));
    assert!(!is_application_executable(DesktopApplication::Chatgpt, Path::new("/usr/bin/curl")));
}

/// A unit a user named after a desktop application and filled with another program is not marked,
/// and the scan names the cgroup whose name it did not believe.
///
/// The name is a name the user can choose,
/// so it is believed only while the application's own program is in the cgroup it names.
#[test]
fn a_unit_named_after_an_application_without_it_is_not_marked() -> io::Result<()> {
    let scratch = crate::test_scratch::scratch_root().join(format!(
        "wg-quicker-application-targets-unwitnessed-{}",
        std::process::id()
    ));
    let cgroup_root = scratch.join("cgroup");
    let app_slice = cgroup_root.join("users/app.slice");
    let proc_root = scratch.join("proc");
    std::fs::create_dir_all(&app_slice)?;
    std::fs::create_dir(&proc_root)?;
    let fake_firefox = app_slice.join("app-firefox@not-a-firefox-process.service");
    let fake_ghostty = app_slice.join("app-com.mitchellh.ghostty@abc.service");
    let fake_steam = app_slice.join("app-steam@abc.service");
    for path in [&fake_firefox, &fake_ghostty, &fake_steam] {
        std::fs::create_dir(path)?;
    }
    // Each unit runs something that is not the application its name claims.
    create_process(
        &proc_root,
        "80",
        "/usr/bin/curl",
        "0::/users/app.slice/app-firefox@not-a-firefox-process.service\n",
    )?;
    create_process(
        &proc_root,
        "81",
        "/usr/bin/curl",
        "0::/users/app.slice/app-com.mitchellh.ghostty@abc.service\n",
    )?;
    create_process(
        &proc_root,
        "82",
        "/usr/bin/curl",
        "0::/users/app.slice/app-steam@abc.service\n",
    )?;
    let scan = scan_application_targets(&ScanRoots {
        app_slice: &app_slice,
        proc_root: &proc_root,
        cgroup_root: &cgroup_root,
    })?;
    assert!(scan.targets.is_empty(), "{:?}", scan.targets);
    let mut expected = vec![fake_firefox, fake_ghostty, fake_steam];
    expected.sort();
    assert_eq!(scan.without_application, expected);
    std::fs::remove_dir_all(&scratch)?;
    return Ok(());
}

/// Discovers Pi's agent units beside the other named application cgroups.
#[test]
fn scan_discovers_pi_agent_units() -> io::Result<()> {
    let scratch = crate::test_scratch::scratch_root().join(format!(
        "wg-quicker-application-targets-pi-{}",
        std::process::id()
    ));
    let cgroup_root = scratch.join("cgroup");
    let app_slice = cgroup_root.join("users/app.slice");
    let proc_root = scratch.join("proc");
    std::fs::create_dir_all(&app_slice)?;
    std::fs::create_dir(&proc_root)?;
    let agent_unit = app_slice.join("pi-wg-quicker-model-providers-off-the-tunnel.service");
    let other_unit = app_slice.join("piwork-3.service");
    let agent_scope = app_slice.join("pi-wg-quicker-shell-3.scope");
    for path in [&agent_unit, &other_unit, &agent_scope] {
        std::fs::create_dir(path)?;
    }
    let scan = scan_application_targets(&ScanRoots {
        app_slice: &app_slice,
        proc_root: &proc_root,
        cgroup_root: &cgroup_root,
    })?;
    assert_eq!(scan.targets, vec![agent_unit]);
    assert!(scan.without_application.is_empty(), "{:?}", scan.without_application);
    std::fs::remove_dir_all(&scratch)?;
    return Ok(());
}

/// The updates slice is a place in the cgroup tree:
/// the slice itself and what lies below it are inside,
/// and a unit the user named after it is not,
/// because a name is a name the user chooses.
#[test]
fn the_slice_is_matched_by_whole_path_components() {
    let slice = format!("/user.slice/user-1000.slice/user@1000.service/app.slice/{TOOLCHAIN_UPDATES_SLICE}");
    assert!(is_in_updates_slice(&slice, 1000));
    assert!(is_in_updates_slice(&format!("{slice}/x.scope"), 1000));
    assert!(is_in_updates_slice(&format!("{slice}/x.scope/app.slice/y"), 1000));
    for outside in [format!("{slice}-task.scope"), format!("{slice}x"), "/user.slice/user-1000.slice/user@1000.service/app.slice/x.scope".to_string()] {
        assert!(!is_in_updates_slice(&outside, 1000), "{outside}");
    }
    assert!(!is_in_updates_slice(&slice, 2000), "another user's slice is another place");
}
