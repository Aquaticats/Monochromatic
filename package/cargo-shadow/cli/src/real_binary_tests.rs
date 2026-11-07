//! The unit tests of the PATH lookup:
//! the first executable wins,
//! non-executables and the shadow itself are skipped.

/// Imports the function under test.
use crate::real_binary::find_in_path;
/// Imports the scratch directories.
use crate::test_scratch::scratch;

/// Imports the owner-only and executable mode helpers.
use std::os::unix::fs::{PermissionsExt, symlink};
/// Imports the path types.
use std::path::{Path, PathBuf};

/// The program name the tests look for.
const NAME: &str = "cargo";

/// The mode of a file the owner may run.
const EXECUTABLE_MODE: u32 = 0o755;

/// The mode of a file nobody may run.
const PLAIN_MODE: u32 = 0o644;

/// Writes a file named `NAME` in `directory` with the given mode and returns its path.
fn write_program(directory: &Path, mode: u32) -> PathBuf {
    let path: PathBuf = directory.join(NAME);
    std::fs::write(&path, "#!/bin/sh\n").expect("write the program");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode)).expect("set the mode");
    return path;
}

/// Joins directories into a PATH value.
fn path_value(directories: &[&Path]) -> String {
    let mut parts: Vec<String> = Vec::new();
    for directory in directories {
        parts.push(directory.to_string_lossy().to_string());
    }
    return parts.join(":");
}

#[test]
fn the_first_executable_in_path_order_wins() {
    let first = scratch();
    let second = scratch();
    let expected: PathBuf = write_program(first.path(), EXECUTABLE_MODE);
    write_program(second.path(), EXECUTABLE_MODE);

    let found = find_in_path(NAME, &path_value(&[first.path(), second.path()]), &[]);

    assert_eq!(found, Some(expected));
}

#[test]
fn a_file_without_an_execute_bit_is_skipped() {
    let plain = scratch();
    let runnable = scratch();
    write_program(plain.path(), PLAIN_MODE);
    let expected: PathBuf = write_program(runnable.path(), EXECUTABLE_MODE);

    let found = find_in_path(NAME, &path_value(&[plain.path(), runnable.path()]), &[]);

    assert_eq!(found, Some(expected));
}

#[test]
fn a_candidate_that_resolves_to_the_shadow_is_skipped_under_any_name() {
    let shadow_directory = scratch();
    let link_directory = scratch();
    let real_directory = scratch();
    let shadow: PathBuf = shadow_directory.path().join("cargo-shadow");
    std::fs::write(&shadow, "#!/bin/sh\n").expect("write the shadow");
    std::fs::set_permissions(&shadow, std::fs::Permissions::from_mode(EXECUTABLE_MODE)).expect("set the mode");
    symlink(&shadow, link_directory.path().join(NAME)).expect("link the shadow as cargo");
    let expected: PathBuf = write_program(real_directory.path(), EXECUTABLE_MODE);
    let resolved_shadow: PathBuf = std::fs::canonicalize(&shadow).expect("resolve the shadow");

    let found = find_in_path(
        NAME,
        &path_value(&[link_directory.path(), real_directory.path()]),
        &[resolved_shadow],
    );

    assert_eq!(found, Some(expected));
}

#[test]
fn an_empty_path_entry_is_ignored_and_a_missing_program_is_none() {
    let empty = scratch();

    let found = find_in_path(NAME, &format!(":{}:", empty.path().display()), &[]);

    assert_eq!(found, None);
}

#[test]
fn every_resolved_path_in_the_skip_list_is_skipped_not_only_the_first() {
    let launcher_directory = scratch();
    let tried_directory = scratch();
    let real_directory = scratch();
    let launcher: PathBuf = write_program(launcher_directory.path(), EXECUTABLE_MODE);
    let tried: PathBuf = write_program(tried_directory.path(), EXECUTABLE_MODE);
    let expected: PathBuf = write_program(real_directory.path(), EXECUTABLE_MODE);
    let skip: Vec<PathBuf> = vec![
        std::fs::canonicalize(&launcher).expect("resolve the launcher"),
        std::fs::canonicalize(&tried).expect("resolve the tried candidate"),
    ];

    let found = find_in_path(
        NAME,
        &path_value(&[launcher_directory.path(), tried_directory.path(), real_directory.path()]),
        &skip,
    );

    assert_eq!(found, Some(expected));
}

/// The text of a launch record (`mpm-shadow`'s format) whose `real` binary is `real`;
/// built field by field because the `json!` macro
/// unwraps a conversion that clippy.toml disallows.
fn launch_record(real: &Path) -> String {
    let mut fields = serde_json::Map::new();
    fields.insert("toolId".to_string(), serde_json::Value::String("x".to_string()));
    fields.insert("lockFile".to_string(), serde_json::Value::String("/x".to_string()));
    fields.insert("real".to_string(), serde_json::Value::String(real.to_string_lossy().to_string()));
    fields.insert("scope".to_string(), serde_json::Value::String("repo".to_string()));
    return serde_json::Value::Object(fields).to_string();
}

/// A launcher in `directory`:
/// an executable named `launcher_name` with a `<launcher_name>.launch` record whose `real` is `target`,
/// and a `cargo` link to it in `links`;
/// returns the link.
fn write_launcher_link(directory: &Path, links: &Path, launcher_name: &str, target: &Path) -> PathBuf {
    let launcher: PathBuf = directory.join(launcher_name);
    std::fs::write(&launcher, "#!/bin/sh\n").expect("write the launcher");
    std::fs::set_permissions(&launcher, std::fs::Permissions::from_mode(EXECUTABLE_MODE)).expect("set the mode");
    let record: String = launch_record(target);
    std::fs::write(directory.join(format!("{launcher_name}.launch")), record).expect("write the record");
    let link: PathBuf = links.join(NAME);
    symlink(&launcher, &link).expect("link the launcher");
    return link;
}

/// A candidate that is a launcher whose record names the shadow (through a `current` link,
/// as the toolchain's tree has it) is skipped
/// before it is run,
/// and so is a launcher that leads to the shadow through another launcher.
#[test]
fn a_launcher_whose_record_names_the_shadow_is_skipped_directly_and_through_another_launcher() {
    let tree = scratch();
    let launchers = scratch();
    let links = scratch();
    let other_links = scratch();
    let real_directory = scratch();
    std::fs::create_dir_all(tree.path().join("0.1.0").join("bin")).expect("the tree");
    let shadow: PathBuf = tree.path().join("0.1.0").join("bin").join("cargo-shadow");
    std::fs::write(&shadow, "shadow").expect("the shadow binary");
    symlink("0.1.0", tree.path().join("current")).expect("the current link");
    let named: PathBuf = tree.path().join("current").join("bin").join("cargo-shadow");
    let skip: Vec<PathBuf> = vec![std::fs::canonicalize(&shadow).expect("resolves")];
    let expected: PathBuf = write_program(real_directory.path(), EXECUTABLE_MODE);

    write_launcher_link(launchers.path(), links.path(), "direct", &named);
    let direct = find_in_path(NAME, &path_value(&[links.path(), real_directory.path()]), &skip);
    assert_eq!(direct, Some(expected.clone()), "a launcher whose record names the shadow was taken for the real cargo");

    // A second launcher whose record names the first one.
    let first_launcher: PathBuf = launchers.path().join("direct");
    write_launcher_link(launchers.path(), other_links.path(), "outer", &first_launcher);
    let through = find_in_path(NAME, &path_value(&[other_links.path(), real_directory.path()]), &skip);
    assert_eq!(through, Some(expected), "a launcher that leads to the shadow through another launcher was taken for the real cargo");
}

/// A launcher whose record names another program is a real cargo for this purpose,
/// a record that cannot be read is not a launcher,
/// and a record chain that never ends is skipped instead of followed without end.
#[test]
fn a_launcher_of_another_program_a_broken_record_and_a_record_loop() {
    let launchers = scratch();
    let links = scratch();
    let elsewhere = scratch();
    let shadow = scratch();
    let shadow_binary: PathBuf = shadow.path().join("cargo-shadow");
    std::fs::write(&shadow_binary, "shadow").expect("the shadow binary");
    let skip: Vec<PathBuf> = vec![std::fs::canonicalize(&shadow_binary).expect("resolves")];
    let other_program: PathBuf = write_program(elsewhere.path(), EXECUTABLE_MODE);

    // Names another program: not skipped.
    let link: PathBuf = write_launcher_link(launchers.path(), links.path(), "other", &other_program);
    assert_eq!(find_in_path(NAME, &path_value(&[links.path()]), &skip), Some(link.clone()), "a launcher of another program was skipped");

    // A record that is not JSON: not a launcher, so not skipped.
    std::fs::write(launchers.path().join("other.launch"), "not json").expect("break the record");
    assert_eq!(find_in_path(NAME, &path_value(&[links.path()]), &skip), Some(link), "a broken record made the candidate skipped");

    // Two launchers whose records name each other: the walk ends and the candidate is skipped.
    let looping = scratch();
    let loop_links = scratch();
    let a: PathBuf = looping.path().join("a");
    let b: PathBuf = looping.path().join("b");
    for (launcher, target) in [(&a, &b), (&b, &a)] {
        std::fs::write(launcher, "#!/bin/sh\n").expect("write a launcher");
        std::fs::set_permissions(launcher, std::fs::Permissions::from_mode(EXECUTABLE_MODE)).expect("set the mode");
        let record: String = launch_record(target);
        let mut name = launcher.as_os_str().to_os_string();
        name.push(".launch");
        std::fs::write(name, record).expect("write the record");
    }
    symlink(&a, loop_links.path().join(NAME)).expect("link");
    assert_eq!(find_in_path(NAME, &path_value(&[loop_links.path()]), &skip), None, "a loop of launcher records was followed or taken for a real cargo");
}

/// Third certification pass (the launcher row):
/// a hard link to the shadow resolves to another pathname than the shadow's own and is
/// still the same executable,
/// so a PATH candidate that is a hard link to it is skipped by file identity,
/// not taken for the real cargo.
/// A symlink-only fixture cannot show this:
/// a symlink canonicalizes to the shadow's own path.
#[test]
fn a_hard_link_to_the_shadow_is_skipped_by_file_identity() {
    let shadow_directory = scratch();
    let alias_directory = scratch();
    let real_directory = scratch();
    let shadow: PathBuf = shadow_directory.path().join(NAME);
    std::fs::write(&shadow, "#!/bin/sh\n").expect("the shadow");
    std::fs::set_permissions(&shadow, std::fs::Permissions::from_mode(EXECUTABLE_MODE)).expect("set the mode");
    let alias: PathBuf = alias_directory.path().join(NAME);
    std::fs::hard_link(&shadow, &alias).expect("a hard link of the shadow");
    assert_ne!(std::fs::canonicalize(&alias).expect("resolves"), std::fs::canonicalize(&shadow).expect("resolves"), "the alias must resolve to another pathname for this test to mean anything");
    let expected: PathBuf = write_program(real_directory.path(), EXECUTABLE_MODE);
    let skip: Vec<PathBuf> = vec![std::fs::canonicalize(&shadow).expect("resolves")];

    let found = find_in_path(NAME, &path_value(&[shadow_directory.path(), alias_directory.path(), real_directory.path()]), &skip);

    assert_eq!(found, Some(expected), "a hard link to the shadow was taken for the real cargo");
}

/// Third certification pass:
/// a launcher whose record names a hard-linked alias of the shadow leads back to the shadow,
/// so it is
/// skipped,
/// directly and through a second launcher.
#[test]
fn a_launcher_record_that_names_a_hard_linked_alias_of_the_shadow_is_skipped() {
    let tree = scratch();
    let alias_directory = scratch();
    let launchers = scratch();
    let links = scratch();
    let real_directory = scratch();
    let shadow: PathBuf = tree.path().join("cargo-shadow");
    std::fs::write(&shadow, "shadow").expect("the shadow binary");
    let alias: PathBuf = alias_directory.path().join("alias-of-the-shadow");
    std::fs::hard_link(&shadow, &alias).expect("a hard link of the shadow");
    let skip: Vec<PathBuf> = vec![std::fs::canonicalize(&shadow).expect("resolves")];
    let expected: PathBuf = write_program(real_directory.path(), EXECUTABLE_MODE);
    write_launcher_link(launchers.path(), links.path(), "direct", &alias);

    let found = find_in_path(NAME, &path_value(&[links.path(), real_directory.path()]), &skip);

    assert_eq!(found, Some(expected), "a launcher whose record names a hard link of the shadow was taken for the real cargo");
}

/// Fourth certification pass (the launcher row):
/// the identity of the running image is taken from the open image,
/// so an installer that
/// atomically replaces the installed path while the shadow runs does not hide a hard link of that image on PATH.
/// The image is opened first (its identity is what `/proc/self/exe` gives),
/// the installed path is then replaced by another file with a
/// rename,
/// the skip set built from the installed path now names the new file,
/// and the hard-linked alias of the old image must still be
/// skipped.
#[test]
fn a_hard_link_of_the_running_image_is_skipped_after_the_installed_path_was_replaced() {
    use std::os::unix::fs::MetadataExt;
    use crate::real_binary::find_in_path_with_image;
    let installed_directory = scratch();
    let alias_directory = scratch();
    let real_directory = scratch();
    let installed: PathBuf = installed_directory.path().join(NAME);
    std::fs::write(&installed, "#!/bin/sh\n# image A\n").expect("image A");
    std::fs::set_permissions(&installed, std::fs::Permissions::from_mode(EXECUTABLE_MODE)).expect("set the mode");
    let alias: PathBuf = alias_directory.path().join(NAME);
    std::fs::hard_link(&installed, &alias).expect("a hard link of image A");
    let running = std::fs::File::open(&installed).expect("the running image stays open");
    let image = running.metadata().map(|found| { return (found.dev(), found.ino()); }).expect("the image identity");
    // The installer replaces the installed path with image B atomically.
    let replacement: PathBuf = installed_directory.path().join("new-image");
    std::fs::write(&replacement, "#!/bin/sh\n# image B\n").expect("image B");
    std::fs::set_permissions(&replacement, std::fs::Permissions::from_mode(EXECUTABLE_MODE)).expect("set the mode");
    std::fs::rename(&replacement, &installed).expect("replace the installed path");
    assert_ne!(std::fs::metadata(&installed).expect("examines").ino(), image.1, "the installed path must now name another file");
    let expected: PathBuf = write_program(real_directory.path(), EXECUTABLE_MODE);
    // What the pathname-based skip list holds after the replacement: the installed path, which names image B.
    let skip: Vec<PathBuf> = vec![installed.clone()];

    let by_path_only = find_in_path_with_image(NAME, &path_value(&[installed_directory.path(), alias_directory.path(), real_directory.path()]), &skip, None);
    let with_the_image = find_in_path_with_image(NAME, &path_value(&[installed_directory.path(), alias_directory.path(), real_directory.path()]), &skip, Some(image));

    assert_eq!(by_path_only, Some(alias), "the control: without the image identity the alias is the candidate, so the schedule is real");
    assert_eq!(with_the_image, Some(expected), "a hard link of the running image was taken for the real cargo after the installed path was replaced");
    drop(running);
}

/// The identity of the running image can be read in a test process:
/// it is the test executable's own file.
#[test]
fn the_running_image_identity_is_the_identity_of_the_test_executable() {
    use std::os::unix::fs::MetadataExt;
    let found = crate::real_binary::running_image_identity().expect("the image can be opened");
    let executable = std::env::current_exe().expect("this test executable");
    let by_path = std::fs::metadata(executable).map(|metadata| { return (metadata.dev(), metadata.ino()); }).expect("examines");
    assert_eq!(found, by_path);
}

/// Fourth certification pass:
/// an outer launcher whose record names an inner launcher whose record names a hard-linked alias of the
/// shadow is skipped;
/// identity is applied at every hop,
/// not only at the first.
#[test]
fn a_launcher_chain_that_ends_at_a_hard_linked_alias_of_the_shadow_is_skipped() {
    let tree = scratch();
    let alias_directory = scratch();
    let inner_directory = scratch();
    let outer_directory = scratch();
    let links = scratch();
    let real_directory = scratch();
    let shadow: PathBuf = tree.path().join("cargo-shadow");
    std::fs::write(&shadow, "shadow").expect("the shadow binary");
    let alias: PathBuf = alias_directory.path().join("alias-of-the-shadow");
    std::fs::hard_link(&shadow, &alias).expect("a hard link of the shadow");
    let skip: Vec<PathBuf> = vec![std::fs::canonicalize(&shadow).expect("resolves")];
    let expected: PathBuf = write_program(real_directory.path(), EXECUTABLE_MODE);
    let unused_links = scratch();
    // The inner launcher names the alias; the outer launcher names the inner launcher.
    write_launcher_link(inner_directory.path(), unused_links.path(), "inner", &alias);
    write_launcher_link(outer_directory.path(), links.path(), "outer", &inner_directory.path().join("inner"));

    let found = find_in_path(NAME, &path_value(&[links.path(), real_directory.path()]), &skip);

    assert_eq!(found, Some(expected), "a launcher chain that ends at a hard link of the shadow was taken for the real cargo");
}

/// The environment variable that gives the running-image helper its directory.
const FIXTURE_IMAGE_ROOT: &str = "CARGO_SHADOW_IMAGE_FIXTURE_ROOT";

/// The helper process of the running-image test:
/// it is started from the installed path of an image,
/// tells the test it runs,
/// waits for
/// the test to replace that installed path,
/// and then asks the production lookup (`find_in_path`,
/// which takes the identity of the
/// image it runs from `/proc/self/exe`) for the real cargo twice:
/// with a hard-linked alias of its own image on PATH,
/// and with a
/// recorded launcher chain that ends at that alias.
/// It writes both answers to files.
/// Run on its own it does nothing.
#[test]
#[ignore = "the helper process of a_running_image_whose_installed_path_was_replaced_still_skips_its_hard_links_in_the_production_lookup"]
fn running_image_lookup_fixture() {
    let Ok(root_text) = std::env::var(FIXTURE_IMAGE_ROOT) else {
        return;
    };
    let root: PathBuf = PathBuf::from(root_text);
    std::fs::write(root.join("started"), "x").expect("announce the start");
    let waited = std::time::Instant::now();
    while !root.join("go").exists() {
        assert!(waited.elapsed() < std::time::Duration::from_secs(120), "the test never replaced the installed path");
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let own: PathBuf = crate::real_binary::own_real_path();
    let direct = find_in_path(NAME, &path_value(&[&root.join("alias"), &root.join("real")]), std::slice::from_ref(&own));
    let chain = find_in_path(NAME, &path_value(&[&root.join("links"), &root.join("real")]), std::slice::from_ref(&own));
    std::fs::write(root.join("direct.answer"), direct.map(|found| { return found.display().to_string(); }).unwrap_or_default()).expect("write the direct answer");
    std::fs::write(root.join("chain.answer"), chain.map(|found| { return found.display().to_string(); }).unwrap_or_default()).expect("write the chain answer");
}

/// Fifth certification pass (the launcher row),
/// a real process:
/// an image is started from its installed path,
/// an installer replaces
/// that path (a rename of another file over it) while a hard link of the image stays on PATH,
/// and the production lookup of the running
/// process still skips the alias,
/// directly and at the end of a recorded launcher chain.
/// The test injects no identity:
/// it is the
/// helper's own `/proc/self/exe`,
/// so a lookup that took the identity from the installed pathname,
/// or passed none,
/// returns the alias.
#[test]
fn a_running_image_whose_installed_path_was_replaced_still_skips_its_hard_links_in_the_production_lookup() {
    let root = scratch();
    let installed: PathBuf = root.path().join("installed");
    let alias: PathBuf = root.path().join("alias");
    let real: PathBuf = root.path().join("real");
    let launchers: PathBuf = root.path().join("launchers");
    let links: PathBuf = root.path().join("links");
    for directory in [&installed, &alias, &real, &launchers, &links] {
        std::fs::create_dir_all(directory).expect("a directory");
    }
    let image: PathBuf = std::env::current_exe().expect("this test executable");
    std::fs::hard_link(&image, installed.join(NAME)).expect("the installed image");
    std::fs::hard_link(&image, alias.join(NAME)).expect("a hard link of the image");
    let expected: PathBuf = write_program(&real, EXECUTABLE_MODE);
    // A recorded chain: the outer launcher names the inner one, which names the alias.
    let inner: PathBuf = launchers.join("inner");
    std::fs::write(&inner, "#!/bin/sh\n").expect("the inner launcher");
    std::fs::set_permissions(&inner, std::fs::Permissions::from_mode(EXECUTABLE_MODE)).expect("set the mode");
    std::fs::write(launchers.join("inner.launch"), launch_record(&alias.join(NAME))).expect("the inner record");
    write_launcher_link(&launchers, &links, "outer", &inner);

    let mut child = std::process::Command::new(installed.join(NAME))
        .args(["--ignored", "--exact", "real_binary_tests::running_image_lookup_fixture", "--nocapture"])
        .env(FIXTURE_IMAGE_ROOT, root.path())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("start the image from its installed path");
    let waited = std::time::Instant::now();
    while !root.path().join("started").exists() {
        assert!(child.try_wait().expect("polls").is_none() && waited.elapsed() < std::time::Duration::from_secs(120), "the helper did not start");
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    // The installer replaces the installed path atomically while the image runs; the alias keeps the old image.
    let replacement: PathBuf = root.path().join("replacement");
    std::fs::write(&replacement, "#!/bin/sh\n# another image\n").expect("another image");
    std::fs::set_permissions(&replacement, std::fs::Permissions::from_mode(EXECUTABLE_MODE)).expect("set the mode");
    std::fs::rename(&replacement, installed.join(NAME)).expect("replace the installed path");
    std::fs::write(root.path().join("go"), "x").expect("let the helper go on");
    let status = child.wait().expect("the helper ends");

    assert!(status.success(), "the helper failed: {status:?}");
    let direct: String = std::fs::read_to_string(root.path().join("direct.answer")).expect("the direct answer");
    let chain: String = std::fs::read_to_string(root.path().join("chain.answer")).expect("the chain answer");
    assert_eq!(direct, expected.display().to_string(), "the lookup returned a hard link of the running image as the real cargo");
    assert_eq!(chain, expected.display().to_string(), "the lookup followed a recorded chain to a hard link of the running image");
}

/// Fifth certification pass:
/// a path chosen as the real cargo that names the shadow at the moment of the start is refused,
/// and a path that
/// names another program is not.
/// The path is a hard link of the running image (swapped in after the choice,
/// in the schedule),
/// checked
/// by the same check the three start sites make.
#[test]
fn a_chosen_path_that_names_the_running_image_at_the_start_is_refused() {
    let directory = scratch();
    let chosen: PathBuf = directory.path().join(NAME);
    write_program(directory.path(), EXECUTABLE_MODE);
    assert!(crate::real_binary::refuse_if_ourselves(&chosen).is_ok(), "a program that is not the shadow was refused");

    // The installer swaps a hard link of the running image over the chosen path after it was chosen.
    std::fs::remove_file(&chosen).expect("remove the chosen program");
    std::fs::hard_link(std::env::current_exe().expect("this test executable"), &chosen).expect("swap in a hard link of the image");

    let refused = crate::real_binary::refuse_if_ourselves(&chosen);
    assert!(refused.is_err_and(|reason| { return reason.contains("names this program itself"); }), "a path that names the running image was started");
}

/// The one argument the stand-in real program is run with.
/// It is also a listing request that the test image itself answers at once,
/// so a
/// regression that starts the swapped-in shadow lists tests and exits instead of running tests recursively.
const REAL_PROGRAM_ARGUMENT: &str = "--list";

/// Round 21 (group 15,
/// b):
/// a chosen real cargo that is a compiled program is started through the file that was checked,
/// so a path made to
/// name the shadow between the check and the start does not reach the start:
/// the real program runs,
/// not the shadow.
/// The program here is a copy of `echo` (a compiled program);
/// between the check and the start the path is replaced by a hard link of the
/// running image (the seam of `after_check_for_test`).
/// Started by path it would be the test image;
/// started through the checked file it
/// prints its arguments.
#[test]
fn a_compiled_real_cargo_swapped_to_the_shadow_after_the_check_still_runs_the_real_one() {
    let directory = scratch();
    let chosen: PathBuf = directory.path().join(NAME);
    std::fs::copy("/usr/bin/echo", &chosen).expect("a compiled stand-in for the real cargo");
    std::fs::set_permissions(&chosen, std::fs::Permissions::from_mode(EXECUTABLE_MODE)).expect("set the mode");
    let vetted = crate::real_binary::vet(&chosen).expect("a program that is not the shadow passes the check");
    let swapped: PathBuf = chosen.clone();
    crate::real_binary::after_check_for_test(Some(std::rc::Rc::new(move || {
        let link: PathBuf = swapped.with_extension("swap");
        std::fs::hard_link(std::env::current_exe().expect("this test executable"), &link).expect("a hard link of the shadow");
        std::fs::rename(&link, &swapped).expect("name the shadow at the chosen path");
    })));

    let mut command = vetted.command().expect("a compiled program is started through its checked file");
    crate::real_binary::after_check_for_test(None);
    let output = command.arg(REAL_PROGRAM_ARGUMENT).output().expect("starts");

    assert_eq!(String::from_utf8_lossy(&output.stdout), format!("{REAL_PROGRAM_ARGUMENT}\n"), "the shadow was started in place of the real cargo: {output:?}");
    assert!(crate::real_binary::refuse_if_ourselves(&chosen).is_err(), "the swap did not name the shadow, so the test showed nothing");
}

/// Round 21:
/// a script cannot be started through a descriptor,
/// so it keeps its pathname and is checked again at the start;
/// a path swapped
/// to the shadow after the first check is refused there (the residual window of scripts is the instants after that second check,
/// which
/// ends at the arrival stop of the second cargo-shadow).
#[test]
fn a_script_real_cargo_swapped_to_the_shadow_after_the_check_is_refused_at_the_start() {
    let directory = scratch();
    let chosen: PathBuf = directory.path().join(NAME);
    write_program(directory.path(), EXECUTABLE_MODE);
    let vetted = crate::real_binary::vet(&chosen).expect("a script that is not the shadow passes the check");
    let swapped: PathBuf = chosen.clone();
    crate::real_binary::after_check_for_test(Some(std::rc::Rc::new(move || {
        let link: PathBuf = swapped.with_extension("swap");
        std::fs::hard_link(std::env::current_exe().expect("this test executable"), &link).expect("a hard link of the shadow");
        std::fs::rename(&link, &swapped).expect("name the shadow at the chosen path");
    })));

    let started = vetted.command();
    crate::real_binary::after_check_for_test(None);

    assert!(started.is_err_and(|reason| { return reason.contains("names this program itself"); }), "a script swapped to the shadow was started");
}
