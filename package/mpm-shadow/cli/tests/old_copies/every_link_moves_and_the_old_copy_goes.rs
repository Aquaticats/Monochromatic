//! What:
//! The first end-to-end run of the rule:
//! `list` names what `install` would do and changes nothing,
//! `install` re-points every link of the old copy to the managed copy and then removes the copy,
//! and a second run finds nothing to do.
//! Why:
//! The unit tests prove each judgement;
//! only a real process shows that the links a person already has move before the copy they run goes,
//! and that a link to a foreign file and a regular file beside them are left alone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // old_copies/every_link_moves_and_the_old_copy_goes.test.ts
//! test('every link moves and the old copy goes', () => { const fixture = Fixture.new(); ... });
//! ```

use std::fs;
use std::process::Command;

use crate::{
    Fixture,
    make_link,
    text,
    write_script,
};

#[test]
fn install_moves_every_link_to_the_managed_copy_then_removes_the_old_one_and_a_second_run_does_nothing()
 {
    let fixture = Fixture::new();
    let local_git = fixture.home().join(".local/bin/git");
    let second_checkout_git = fixture.work_two().join(".cache/cli-git-bin/git");
    let foreign_target = fixture.home().join(".local/share/other/tool");
    write_script(&foreign_target, "echo foreign");
    let foreign_link = fixture.home().join(".local/bin/foreign");
    write_script(
        &fixture.home().join(".local/bin/not-a-link"),
        "echo regular",
    );
    make_link(&local_git, &fixture.old_copy());
    make_link(&second_checkout_git, &fixture.old_copy());
    make_link(&foreign_link, &foreign_target);

    let listed = fixture.run(&["list"]);
    assert_eq!(
        listed.status.code(),
        Some(0),
        "an old copy is not drift: {}",
        text(&listed.stdout)
    );
    assert!(
        text(&listed.stdout)
            .contains("would be removed by `install` after it did: re-point 2 link(s)"),
        "{}",
        text(&listed.stdout)
    );
    assert!(fixture.old_copy().exists(), "list changes nothing");

    let installed = fixture.run(&["install", "--frozen-lockfile"]);
    assert_eq!(
        installed.status.code(),
        Some(0),
        "{}{}",
        text(&installed.stdout),
        text(&installed.stderr)
    );
    let managed = fixture.managed_root().join("current/bin/cli-git-native");
    for link in [&local_git, &second_checkout_git] {
        assert_eq!(
            fs::read_link(link).expect("still a link"),
            managed,
            "{}",
            link.display()
        );
    }
    assert!(
        fs::symlink_metadata(fixture.old_copy()).is_err(),
        "the old copy is gone: {}",
        text(&installed.stdout)
    );
    assert_eq!(
        fs::read_link(&foreign_link).expect("a link"),
        foreign_target,
        "a foreign link is untouched"
    );
    assert!(
        fixture.home().join(".local/bin/not-a-link").is_file(),
        "a regular file is untouched"
    );
    let output = text(&installed.stdout);
    assert_eq!(
        output.matches("toolchain install: re-pointed ").count(),
        2,
        "{output}"
    );
    assert!(
        output.contains(&format!(
            "toolchain install: removed {}",
            fixture.old_copy().display()
        )),
        "{output}"
    );
    let through_the_link = Command::new(&local_git)
        .env_clear()
        .output()
        .expect("run git through the re-pointed link");
    assert_eq!(
        text(&through_the_link.stdout).trim(),
        "managed",
        "git now runs the managed copy"
    );

    let again = fixture.run(&["install", "--frozen-lockfile"]);
    assert_eq!(again.status.code(), Some(0), "{}", text(&again.stderr));
    assert!(
        !text(&again.stdout).contains("old copy") && !text(&again.stdout).contains("re-pointed"),
        "{}",
        text(&again.stdout)
    );
    assert!(!text(&fixture.run(&["list"]).stdout).contains("old copy"));
}
