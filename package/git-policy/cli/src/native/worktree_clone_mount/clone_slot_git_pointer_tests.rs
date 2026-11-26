//! What:
//!  Controls of the first observation's Git pointer read:
//!  the registration a slot must name,
//!  and every shape that is not one.
//! Why:
//!  The slot's `.git` is account-controlled text the route reads as root,
//!  so the read is bounded,
//!  regular and no-follow,
//!  and a control proves each refusal on a fixture folder.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const registration = readTheSlotGitPointer(slot, '/fixture/checkout', 'one');
//! ```

use super::{
    GIT_POINTER_NAME,
    GitPointerRefusal,
    POINTER_BOUND,
    read_the_slot_git_pointer,
    the_registration_of,
};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

/// One fixture checkout with a slot folder in it.
struct Fixture {
    /// The checkout.
    checkout: PathBuf,
    /// The slot folder.
    slot: PathBuf,
}

impl Fixture {
    /// The fixture's own folders under this process's temporary root.
    fn new(name: &str) -> Self {
        let checkout: PathBuf =
            crate::test_support::fixture(format!("git-pointer-{name}").as_str()).join("checkout");
        let slot: PathBuf = checkout.join(".claude").join("worktrees").join("one");
        std::fs::create_dir_all(slot.as_path()).expect("the slot folder is made");
        return Fixture {
            checkout,
            slot,
        };
    }

    /// Write the slot's `.git` with exactly the given bytes.
    fn with_pointer(
        self,
        text: &[u8],
    ) -> Self {
        let pointer: PathBuf = self.slot.join(GIT_POINTER_NAME);
        std::fs::write(pointer.as_path(), text).expect("the pointer is written");
        std::fs::set_permissions(pointer.as_path(), std::fs::Permissions::from_mode(0o644))
            .expect("the pointer holds its mode");
        return self;
    }

    /// The slot's own registration,
    /// which a correct pointer names.
    fn registration(&self) -> PathBuf {
        return the_registration_of(self.checkout.as_path(), "one");
    }

    /// Read the pointer as the route does.
    fn read(&self) -> Result<PathBuf, GitPointerRefusal> {
        let folder: std::fs::File =
            std::fs::File::open(self.slot.as_path()).expect("the slot opens");
        return read_the_slot_git_pointer(&folder, self.checkout.as_path(), "one");
    }
}

/// The registration path is the checkout's own Git directory below
/// `worktrees`.
#[test]
fn the_registration_is_the_checkouts_own() {
    let fixture: Fixture = Fixture::new("registration");
    assert_eq!(
        fixture.registration(),
        fixture.checkout.join(".git").join("worktrees").join("one")
    );
}

/// A pointer naming this checkout's registration reads back as it is.
#[test]
fn a_pointer_that_names_the_registration_is_accepted() {
    let fixture: Fixture = Fixture::new("accepted");
    let wanted: PathBuf = fixture.registration();
    let text: String = format!("gitdir: {}\n", wanted.display());
    let read: PathBuf = fixture
        .with_pointer(text.as_bytes())
        .read()
        .expect("the pointer is this checkout's registration");
    assert_eq!(read, wanted);
}

/// A slot with no `.git` is no registered worktree.
#[test]
fn a_slot_with_no_pointer_is_refused() {
    let fixture: Fixture = Fixture::new("absent");
    let refusal: GitPointerRefusal = fixture.read().expect_err("no `.git` is no pointer");
    assert!(
        matches!(refusal, GitPointerRefusal::Absent { .. }),
        "{refusal:?}"
    );
    assert!(
        refusal.message().contains("`one`"),
        "the refusal names the slot: {}",
        refusal.message()
    );
}

/// A `.git` that is a link is never followed.
#[test]
fn a_pointer_that_is_a_link_is_refused() {
    use std::os::unix::fs::symlink;
    let fixture: Fixture = Fixture::new("link");
    let target: PathBuf = fixture.checkout.join("elsewhere");
    std::fs::write(target.as_path(), b"gitdir: /somewhere/else\n").expect("the target is written");
    symlink(
        target.as_path(),
        fixture.slot.join(GIT_POINTER_NAME).as_path(),
    )
    .expect("the link is made");
    let refusal: GitPointerRefusal = fixture.read().expect_err("a link is not one regular file");
    assert!(
        matches!(refusal, GitPointerRefusal::Unreadable { .. }),
        "{refusal:?}"
    );
}

/// A `.git` that is a folder is refused by its type,
/// which is what a clone of the checkout's copied administration holds.
#[test]
fn a_pointer_that_is_a_folder_is_refused() {
    let fixture: Fixture = Fixture::new("folder");
    std::fs::create_dir_all(fixture.slot.join(GIT_POINTER_NAME).as_path())
        .expect("the folder is made");
    let refusal: GitPointerRefusal = fixture
        .read()
        .expect_err("a folder is not one regular file");
    assert!(
        matches!(
            refusal,
            GitPointerRefusal::NotARegularFile {
                found: "a folder",
                ..
            }
        ),
        "{refusal:?}"
    );
}

/// Two lines,
/// no prefix and a relative path are each no pointer.
#[test]
fn a_file_that_is_no_pointer_is_refused() {
    for (name, text) in [
        (
            "two-lines",
            format!(
                "gitdir: {}\ngitdir: /elsewhere\n",
                "/fixture/checkout/.git/worktrees/one"
            ),
        ),
        (
            "no-prefix",
            String::from("/fixture/checkout/.git/worktrees/one\n"),
        ),
        ("relative", String::from("gitdir: .git/worktrees/one\n")),
        (
            "no-newline",
            String::from("gitdir: /fixture/checkout/.git/worktrees/one"),
        ),
    ] {
        let fixture: Fixture = Fixture::new(name);
        let refusal: GitPointerRefusal = fixture
            .with_pointer(text.as_bytes())
            .read()
            .expect_err("this is no pointer");
        assert!(
            matches!(refusal, GitPointerRefusal::NotAPointer { .. }),
            "{name}: {refusal:?}"
        );
    }
}

/// A path with a `..` component is refused rather than resolved.
#[test]
fn a_pointer_with_a_parent_component_is_refused() {
    let fixture: Fixture = Fixture::new("escape");
    let text: String = format!("gitdir: {}/../../../etc\n", fixture.checkout.display());
    let refusal: GitPointerRefusal = fixture
        .with_pointer(text.as_bytes())
        .read()
        .expect_err("a parent component is not resolved");
    assert!(
        matches!(refusal, GitPointerRefusal::NotAPointer { .. }),
        "{refusal:?}"
    );
}

/// A pointer naming another checkout or another slot is another registration.
#[test]
fn a_pointer_naming_another_registration_is_refused() {
    let fixture: Fixture = Fixture::new("another");
    let text: String = String::from("gitdir: /somewhere/else/.git/worktrees/two\n");
    let refusal: GitPointerRefusal = fixture
        .with_pointer(text.as_bytes())
        .read()
        .expect_err("another registration is not this slot's");
    assert!(
        matches!(refusal, GitPointerRefusal::AnotherRegistration { .. }),
        "{refusal:?}"
    );
    assert!(
        refusal
            .message()
            .contains("/somewhere/else/.git/worktrees/two"),
        "the refusal names the pointer: {}",
        refusal.message()
    );
}

/// A pointer longer than the bound is refused rather than read.
#[test]
fn a_pointer_longer_than_the_bound_is_refused() {
    let fixture: Fixture = Fixture::new("long");
    let long: String = format!("gitdir: /{}\n", "a".repeat(POINTER_BOUND));
    let refusal: GitPointerRefusal = fixture
        .with_pointer(long.as_bytes())
        .read()
        .expect_err("a pointer over the bound is refused");
    assert!(
        matches!(refusal, GitPointerRefusal::TooLong { .. }),
        "{refusal:?}"
    );
}

/// The read resolves nothing:
/// the path it answers is the text the file held,
/// which the caller compares.
#[test]
fn the_read_answers_the_text_and_resolves_nothing() {
    let fixture: Fixture = Fixture::new("text");
    let wanted: PathBuf = fixture.registration();
    let text: String = format!("gitdir: {}\n", wanted.display());
    let read: PathBuf = fixture
        .with_pointer(text.as_bytes())
        .read()
        .expect("the pointer reads");
    assert_eq!(read, wanted);
}
