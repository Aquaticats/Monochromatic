//! What:
//! What `search-shadow link` does with programs that are shadows:
//! it makes no link for a tool whose only candidate is a shadow,
//! and it takes over a link an older copy made,
//! because that link is a shadow link and not a real tool.
//! Why:
//! `link` deletes or replaces only its own links,
//! so it must recognize the link of another shadow version for what it is
//! and must never mistake it for a real tool a user put there.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! assert(!existsSync(join(links, 'grep'))); // no real grep on PATH
//! symlinkSync(olderShadow, join(links, 'grep'));
//! assert(canonicalize(join(links, 'grep')) === canonicalize(SHADOW));
//! ```

/// Imports the file-system module that makes and resolves the links.
use std::fs;

/// Imports the command type that runs the `link` subcommand.
use std::process::Command;

/// Imports the fixtures this part of the rule runs on.
use super::{
    SHADOW_EXECUTABLE,
    copy_of_another_version,
    link_as,
    scratch,
    write_real_script,
};

#[test]
fn link_does_not_take_the_link_of_another_shadow_version_for_a_real_tool() {
    let root = scratch();
    let (second, links) = (root.path().join("second"), root.path().join("links"));
    copy_of_another_version(&second.join("search-shadow"));
    link_as(&second, "grep");

    // The only `grep` on PATH is the link of another shadow: there is no real tool, so no link is made.
    let output = Command::new(SHADOW_EXECUTABLE)
        .arg("link")
        .arg(&links)
        .env_clear()
        .env("PATH", &second)
        .output()
        .expect("run link");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        links.join("grep").symlink_metadata().is_err(),
        "a link was made for a tool whose only candidate is a shadow"
    );
}

/// An upgrade takes over the links an older copy made:
/// a link to another shadow version is a shadow link,
/// so `link` points it at this program instead of leaving it behind.
#[test]
fn link_takes_over_a_link_that_points_at_another_shadow_version() {
    let root = scratch();
    let (older, links, tools) = (
        root.path().join("older"),
        root.path().join("links"),
        root.path().join("tools"),
    );
    copy_of_another_version(&older.join("search-shadow"));
    fs::create_dir_all(&links).expect("make the links folder");
    // The link an older copy made into the very directory this run is asked to link into.
    std::os::unix::fs::symlink(older.join("search-shadow"), links.join("grep"))
        .expect("make the older link");
    write_real_script(&tools, "grep");

    let path_variable: String = format!("{}:{}", tools.display(), older.display());
    let output = Command::new(SHADOW_EXECUTABLE)
        .arg("link")
        .arg(&links)
        .env_clear()
        .env("PATH", &path_variable)
        .output()
        .expect("run link");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::canonicalize(links.join("grep")).expect("resolve the link"),
        fs::canonicalize(SHADOW_EXECUTABLE).expect("resolve this program"),
    );
    assert!(
        !String::from_utf8_lossy(&output.stderr).contains("alone"),
        "an upgrade must not leave its own older link behind: {}",
        String::from_utf8_lossy(&output.stderr),
    );
}
