//! What:
//!  Controls of the handler's argument grammar:
//!  the default invocation,
//!  each mode word,
//!  the adoption's three options in any order,
//!  and every refusal with the argument it names.
//! Why:
//!  The grammar is the only door into a root program,
//!  so a word that slips through becomes a mode nobody asked for;
//!  each control shows one answer of the door
//!  (`handler_mode_arguments`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(parse([])).toEqual({ kind: 'sweep' });
//! expect(() => parse(['sweep'])).toThrow(ArgumentRefusal.UnknownMode);
//! ```

use super::{
    ArgumentRefusal, CHECKOUT_OPTION, DATASET_OPTION, DEVICE_OPTION, HELP_WORD, HandlerMode, HolderScanInvocation, INODE_OPTION, MODE_WORDS, NONCE_OPTION, REPLACE_OPTION, VERSION_WORD, parse, usage,
};
use crate::worktree_clone_mount_handler::root_mode_names::ZEDLET_START_NAME;
use crate::worktree_clone_mount_handler::handler_install_paths::version_line;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use crate::worktree_clone_mount_handler::handler_install_paths::{ROOT_MODE_INSTALL_PATH, root_modes_name};
use monochromatic_zfs_request_to_openzfs_kernel_device::DEVICE_PATH;
use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::path::PathBuf;

/// The arguments of one invocation,
/// as the kernel hands them over.
fn arguments(words: &[&str]) -> Vec<OsString> {
    return words.iter().map(|word| return OsString::from(*word)).collect();
}

/// The adoption a complete invocation asks for.
fn adoption() -> HandlerMode {
    return HandlerMode::AdoptWorktreeParent(super::AdoptRequest {
        checkout: PathBuf::from("/home/user/Monochromatic"),
        device: 66306,
        inode: 1_234_567,
    });
}

/// No argument at all is the sweep the installed ZED event runs.
#[test]
fn no_argument_is_the_sweep() {
    assert_eq!(parse(&arguments(&[])), Ok(HandlerMode::AttachingSweep));
}

/// Each mode word is the mode it names,
/// and the help and version words are their own answers.
#[test]
fn every_mode_word_reads_as_its_mode() {
    assert_eq!(parse(&arguments(&["boot-reconcile"])), Ok(HandlerMode::BootReconcile));
    assert_eq!(parse(&arguments(&["start-probe"])), Ok(HandlerMode::StartProbe));
    assert_eq!(parse(&arguments(&["shutdown-unmount"])), Ok(HandlerMode::ShutdownUnmount));
    assert_eq!(parse(&arguments(&[HELP_WORD])), Ok(HandlerMode::Help));
    assert_eq!(parse(&arguments(&[VERSION_WORD])), Ok(HandlerMode::Version));
    assert_eq!(MODE_WORDS, ["boot-reconcile", "start-probe", "shutdown-unmount", "holder-scan", "adopt-worktree-parent", "record-checkout", "mount-path-descendant"]);
}

/// The holder scan reads its dataset and its optional nonce,
/// and refuses a missing,
/// repeated or unreadable one.
#[test]
fn the_holder_scan_reads_its_dataset_and_its_nonce() {
    let wanted = HandlerMode::HolderScan(HolderScanInvocation { dataset: String::from("zpcachyos/nosnap/clones/one"), nonce: None });
    assert_eq!(parse(&arguments(&["holder-scan", DATASET_OPTION, "zpcachyos/nosnap/clones/one"])), Ok(wanted));
    let nonce: &str = "00112233445566778899aabbccddeeff";
    let named = HandlerMode::HolderScan(HolderScanInvocation {
        dataset: String::from("zpcachyos/nosnap/clones/one"),
        nonce: Some(RequestNonce::parse(nonce).expect("the fixture nonce is one")),
    });
    assert_eq!(parse(&arguments(&["holder-scan", NONCE_OPTION, nonce, DATASET_OPTION, "zpcachyos/nosnap/clones/one"])), Ok(named.clone()));
    assert_eq!(parse(&arguments(&["holder-scan", DATASET_OPTION, "zpcachyos/nosnap/clones/one", NONCE_OPTION, nonce])), Ok(named));
    assert_eq!(parse(&arguments(&["holder-scan", NONCE_OPTION, nonce])), Err(ArgumentRefusal::MissingOption { option: DATASET_OPTION }));
    assert_eq!(parse(&arguments(&["holder-scan", NONCE_OPTION])), Err(ArgumentRefusal::MissingValue { option: NONCE_OPTION }));
    assert_eq!(
        parse(&arguments(&["holder-scan", DATASET_OPTION, "a", DATASET_OPTION, "b"])),
        Err(ArgumentRefusal::RepeatedOption { option: DATASET_OPTION })
    );
    assert_eq!(
        parse(&arguments(&["holder-scan", DATASET_OPTION, "pool/one", NONCE_OPTION, nonce, NONCE_OPTION, nonce])),
        Err(ArgumentRefusal::RepeatedOption { option: NONCE_OPTION })
    );
    let refusal: ArgumentRefusal = parse(&arguments(&["holder-scan", DATASET_OPTION, "pool/one@snapshot"])).expect_err("a snapshot is no dataset name");
    assert!(matches!(refusal, ArgumentRefusal::DatasetName { .. }), "{refusal:?}");
    assert!(refusal.message().contains("@"), "{}", refusal.message());
    for typed in ["00112233445566778899AABBCCDDEEFF", "00112233445566778899aabbccddee", "zz112233445566778899aabbccddeeff"] {
        let not_a_nonce: ArgumentRefusal = parse(&arguments(&["holder-scan", DATASET_OPTION, "pool/one", NONCE_OPTION, typed])).expect_err("a nonce is 32 lowercase hex digits");
        assert!(matches!(not_a_nonce, ArgumentRefusal::Nonce { .. }), "{typed}: {not_a_nonce:?}");
    }
    assert_eq!(parse(&arguments(&["holder-scan", "--force", "/a"])), Err(ArgumentRefusal::UnexpectedArgument { word: String::from("--force") }));
}

/// The record mode reads its checkout and its optional replacement token,
/// and the mount mode reads one dataset,
/// each option once and in any order.
#[test]
fn the_two_new_modes_read_their_options() {
    let recorded = HandlerMode::RecordCheckout(super::RecordCheckoutRequest { checkout: PathBuf::from("/home/user/Monochromatic"), replace: None });
    assert_eq!(parse(&arguments(&["record-checkout", CHECKOUT_OPTION, "/home/user/Monochromatic"])), Ok(recorded));
    let replacing = HandlerMode::RecordCheckout(super::RecordCheckoutRequest {
        checkout: PathBuf::from("/home/user/Monochromatic"),
        replace: Some(String::from("9876543210987654321")),
    });
    assert_eq!(parse(&arguments(&["record-checkout", REPLACE_OPTION, "9876543210987654321", CHECKOUT_OPTION, "/home/user/Monochromatic"])), Ok(replacing));
    assert_eq!(
        parse(&arguments(&["record-checkout", REPLACE_OPTION, "1"])),
        Err(ArgumentRefusal::MissingOption { option: CHECKOUT_OPTION })
    );
    assert_eq!(
        parse(&arguments(&["record-checkout", CHECKOUT_OPTION, "/a", CHECKOUT_OPTION, "/b"])),
        Err(ArgumentRefusal::RepeatedOption { option: CHECKOUT_OPTION })
    );
    assert_eq!(parse(&arguments(&["record-checkout", CHECKOUT_OPTION])), Err(ArgumentRefusal::MissingValue { option: CHECKOUT_OPTION }));
    assert_eq!(parse(&arguments(&["record-checkout", "--force", "/a"])), Err(ArgumentRefusal::UnexpectedArgument { word: String::from("--force") }));
    let mounted = HandlerMode::MountPathDescendant(super::MountPathDescendantRequest { dataset: String::from("zpcachyos/nosnap/vm-builder-output") });
    assert_eq!(parse(&arguments(&["mount-path-descendant", DATASET_OPTION, "zpcachyos/nosnap/vm-builder-output"])), Ok(mounted));
    assert_eq!(parse(&arguments(&["mount-path-descendant"])), Err(ArgumentRefusal::MissingOption { option: DATASET_OPTION }));
    assert_eq!(
        parse(&arguments(&["mount-path-descendant", DATASET_OPTION, "a", DATASET_OPTION, "b"])),
        Err(ArgumentRefusal::RepeatedOption { option: DATASET_OPTION })
    );
    assert_eq!(parse(&arguments(&["mount-path-descendant", CHECKOUT_OPTION, "/a"])), Err(ArgumentRefusal::UnexpectedArgument { word: String::from(CHECKOUT_OPTION) }));
    let pool: ArgumentRefusal = parse(&arguments(&["mount-path-descendant", DATASET_OPTION, "pool/one@snapshot"])).expect_err("a snapshot is no dataset name");
    let ArgumentRefusal::DatasetName { mode, .. } = &pool else {
        panic!("a snapshot is no dataset name: {pool:?}");
    };
    assert_eq!(mode, &"mount-path-descendant");
    assert!(pool.message().contains("the nested dataset's full name"), "the sentence is the mode's own: {}", pool.message());
}

/// The adoption's three options read as one request,
/// in the order the help gives them and in another order.
#[test]
fn the_adoption_reads_its_three_options_in_any_order() {
    assert_eq!(
        parse(&arguments(&["adopt-worktree-parent", CHECKOUT_OPTION, "/home/user/Monochromatic", DEVICE_OPTION, "66306", INODE_OPTION, "1234567"])),
        Ok(adoption())
    );
    assert_eq!(
        parse(&arguments(&["adopt-worktree-parent", INODE_OPTION, "1234567", DEVICE_OPTION, "66306", CHECKOUT_OPTION, "/home/user/Monochromatic"])),
        Ok(adoption())
    );
}

/// What:
///  The usage's first line is a spelling that works when typed,
///  and its closing sentences say who installs the root-owned file and when.
/// Why:
///   The words follow whatever path starts the program,
///  so the usage naming them alone prints a command Git answers as an unknown word,
///  and a usage claiming the file is installed states paths no release creates
///  (`doc/decision/cli-git-alpha-46-release-pass-four-answers.md`,
///  "How the root modes' usage reads").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(usage().split('\n')[0]).toBe('usage: <file> monochromatic-git-policy-cli zfs-root <mode>');
/// ```
#[test]
fn the_usage_opens_with_the_spelling_that_works_and_says_who_installs_the_file() {
    let text: String = usage();
    let first: &str = text.lines().next().expect("the usage has a first line");
    assert_eq!(first, format!("usage: <file> {} <mode>", root_modes_name()));
    assert!(!text.contains("[mode]"), "no line offers a mode in brackets: {text}");
    assert!(text.contains(ROOT_MODE_INSTALL_PATH), "the usage names the root-owned file's path: {text}");
    assert!(text.contains("Plan step 19's setup installs"), "the usage says who installs the file: {text}");
    assert!(text.contains("no release installs it yet"), "the usage says no release installs the file yet: {text}");
    assert!(text.contains(ZEDLET_START_NAME), "the usage names the ZED event's own file name: {text}");
}

/// The usage names the program,
/// every mode word,
/// both installed paths and the device the request modes open.
#[test]
fn the_usage_names_the_modes_the_paths_and_the_device() {
    let text: String = usage();
    for word in MODE_WORDS {
        assert!(text.contains(word), "the usage names {word}: {text}");
    }
    assert!(text.contains(ROOT_MODE_INSTALL_PATH), "{text}");
    assert!(text.contains(&root_modes_name()), "the usage opens with the words: {text}");
    assert!(text.contains(crate::worktree_clone_mount_handler::root_mode_names::ZEDLET_START_NAME), "the usage names the ZED event's own file name: {text}");
    assert!(text.contains(DEVICE_PATH), "{text}");
}

/// The version line names the program and this build's release.
#[test]
fn the_version_line_names_the_words_and_the_release() {
    assert_eq!(version_line(), format!("{} {}", root_modes_name(), env!("CARGO_PKG_VERSION")));
}

/// A leading word that is no mode is refused as that word,
/// and the answer lists the modes this build has.
#[test]
fn an_unknown_mode_is_refused_by_its_word() {
    let refusal: ArgumentRefusal = parse(&arguments(&["sweep"])).expect_err("`sweep` is not the word for the default invocation");
    assert_eq!(refusal, ArgumentRefusal::UnknownMode { word: String::from("sweep") });
    let message: String = refusal.message();
    assert!(message.contains("`sweep`"), "{message}");
    for word in MODE_WORDS {
        assert!(message.contains(word), "the refusal says which modes exist, and it lacks {word}: {message}");
    }
}

/// A mode takes no extra word.
#[test]
fn a_word_a_mode_does_not_take_is_refused() {
    assert_eq!(parse(&arguments(&["boot-reconcile", "--force"])), Err(ArgumentRefusal::UnexpectedArgument { word: String::from("--force") }));
    assert_eq!(parse(&arguments(&[HELP_WORD, "boot-reconcile"])), Err(ArgumentRefusal::UnexpectedArgument { word: String::from("boot-reconcile") }));
}

/// An option at the end has no value,
/// and the refusal names the option.
#[test]
fn an_option_without_a_value_is_refused() {
    let refusal: ArgumentRefusal = parse(&arguments(&["adopt-worktree-parent", CHECKOUT_OPTION])).expect_err("the option has no value");
    assert_eq!(refusal, ArgumentRefusal::MissingValue { option: CHECKOUT_OPTION });
    assert!(refusal.message().contains(CHECKOUT_OPTION), "{refusal:?}");
}

/// An option given twice,
/// or an option of the adoption that is missing,
/// is refused rather than guessed.
#[test]
fn a_repeated_or_missing_option_is_refused() {
    assert_eq!(
        parse(&arguments(&["adopt-worktree-parent", CHECKOUT_OPTION, "/a", CHECKOUT_OPTION, "/b", DEVICE_OPTION, "1", INODE_OPTION, "2"])),
        Err(ArgumentRefusal::RepeatedOption { option: CHECKOUT_OPTION })
    );
    assert_eq!(
        parse(&arguments(&["adopt-worktree-parent", CHECKOUT_OPTION, "/a", DEVICE_OPTION, "1"])),
        Err(ArgumentRefusal::MissingOption { option: INODE_OPTION })
    );
    assert_eq!(parse(&arguments(&["adopt-worktree-parent", DEVICE_OPTION, "1", INODE_OPTION, "2"])), Err(ArgumentRefusal::MissingOption { option: CHECKOUT_OPTION }));
}

/// A device or inode that is not decimal digits is refused with what was typed.
#[test]
fn a_value_that_is_no_decimal_count_is_refused() {
    for typed in ["0x10", "-1", "+1", "1_000", "", "6.5"] {
        let refusal: ArgumentRefusal = parse(&arguments(&["adopt-worktree-parent", CHECKOUT_OPTION, "/a", DEVICE_OPTION, typed, INODE_OPTION, "2"]))
            .expect_err("a value that is no decimal count");
        assert_eq!(refusal, ArgumentRefusal::OptionNotACount { option: DEVICE_OPTION, text: String::from(typed) });
        assert!(refusal.message().contains(typed), "the refusal quotes what was typed: {refusal:?}");
    }
}

/// An argument that is not text is refused by its place,
/// which is how a name the shell produced but the kernel did not keep as text is answered.
#[test]
fn an_argument_that_is_not_text_is_refused_by_its_place() {
    let mut words: Vec<OsString> = arguments(&["adopt-worktree-parent", CHECKOUT_OPTION]);
    words.push(OsString::from_vec(vec![0x2f, 0xff, 0xfe]));
    assert_eq!(parse(words.as_slice()), Err(ArgumentRefusal::NotText { place: 3 }));
}
