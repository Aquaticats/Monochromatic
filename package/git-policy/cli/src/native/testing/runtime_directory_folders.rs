//! What:
//!  Which folder a runtime directory stands in,
// and the runtime directory built to a length inside it.
//! Why:
//!  A socket path holds 107 bytes,
//! so a runtime directory of a fixed length leaves whatever is past its folder for its own name,
//! and that room depends on the account and on the process;
//! the folder is chosen from the places this account offers and owned by the test thread that made it.
//! It moved out of `test_support.rs` when that file passed the code-line limit.

/// The folder and runtime directory types.
use std::path::{Path, PathBuf};

/// The cleanup a folder made here is handed to,
/// which runs when the test thread ends however its test ended.
use crate::test_support::track_for_this_thread;

/// The bytes a digest of a tag takes in a runtime directory's name.
const DIGEST_NAME_BYTES: usize = 8;

/// What:
///  The fewest bytes a runtime directory's name may take to hold `tag`:
/// the tag itself where a name that long fits,
/// or the eight bytes a digest of it takes where it does not.
/// Why:
///   A name cannot be shorter than what it spells,
/// so the room a control asks this account for is this number and never a guess:
/// a name whose tag does not fit is the first bytes of that tag's digest.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// roomARuntimeNameNeeds('boundary') // 8
/// ```
pub(crate) fn room_a_runtime_name_needs(tag: &str) -> usize {
    return tag.len().min(DIGEST_NAME_BYTES);
}

/// The hexadecimal digits a folder's own name token takes.
const FOLDER_TOKEN_DIGITS: usize = 6;

/// How many tokens one thread tries before it gives up on a place this account offers.
const FOLDER_TOKEN_ATTEMPTS: usize = 8;

/// What:
///  One test thread's folders:
/// the token every one of them is named with,
/// and the ones it has made.
/// Why:
///   A folder named after the process number is not a name of one's own:
/// about 549 processes start a second on this machine,
/// so a number comes round in about two hours,
/// and inside a container far sooner,
/// and two runs of one process number met in one folder and failed each other at `AlreadyExists`.
/// The token comes from the operating system's random source and every folder is made exclusively,
/// so two runs cannot share one even when they draw the same token;
/// the folders are remembered so a second call of one thread finds the folder it made rather than another.
#[derive(Default)]
struct ThreadFolders {
    /// The six hexadecimal digits this thread's folders are named with,
    /// made on first use.
    token: String,
    /// The folders this thread made,
    /// in the order it made them.
    made: Vec<PathBuf>,
}

thread_local! {
    /// The folders the running test thread owns.
    static THREAD_FOLDERS: std::cell::RefCell<ThreadFolders> = std::cell::RefCell::new(ThreadFolders::default());
}

/// What:
///  A fresh token of `FOLDER_TOKEN_DIGITS` hexadecimal digits,
/// the first of a UUID from the operating system.
/// Why:
///   Six digits are the room a folder's name has in the shortest place this account offers,
/// and a name that cannot be guessed is what keeps two runs out of one folder;
/// the folder is also made exclusively,
/// so a token drawn twice still names two folders.
fn fresh_folder_token() -> String {
    let uuid: String = crate::random_id::random_uuid().expect("the operating system's random source");
    return uuid.get(..FOLDER_TOKEN_DIGITS).expect("a UUID is longer than the token").to_string();
}

/// What:
///  The token naming this thread's folders,
/// made on first use.
/// Why:
///   Every folder of one thread shares the token,
/// so a control that asks for one folder twice is answered with the folder it made.
fn this_threads_folder_token() -> String {
    return THREAD_FOLDERS.with(|thread_folders| {
        let mut folders = thread_folders.borrow_mut();
        if folders.token.is_empty() {
            folders.token = fresh_folder_token();
        }
        return folders.token.clone();
    });
}

/// What:
///  Whether this thread already made `folder`.
/// Why:
///   A folder that exists because this thread made it is used again,
/// where one another run made is passed over.
fn this_thread_made(folder: &Path) -> bool {
    return THREAD_FOLDERS.with(|folders| { return folders.borrow().made.iter().any(|made| { return made == folder; }); });
}

/// What:
///  Take note that this thread made `folder`,
/// and hand the folder to the thread's cleanup.
/// Why:
///   The folder is the one owner of everything the helper puts in it,
/// so the runtime directories and their own files and links go when the thread ends,
/// whether its test passed,
/// failed or was killed.
fn this_thread_made_folder(folder: &Path) {
    THREAD_FOLDERS.with(|folders| { folders.borrow_mut().made.push(folder.to_path_buf()); });
    track_for_this_thread(folder);
}

/// What:
///  Take another token,
/// for a folder another run already made.
/// Why:
///   A folder is never shared:
/// two runs that draw one token take different names rather than write in one place.
fn this_thread_takes_another_token() {
    THREAD_FOLDERS.with(|folders| { folders.borrow_mut().token = fresh_folder_token(); });
}

/// What:
///  The places this account offers for a folder a runtime directory stands in,
/// the longest first.
/// Why:
///   The place names no part of its own:
/// the folder under it is this thread's,
/// and the caller adds that.
fn places_this_account_offers() -> Vec<PathBuf> {
    let mut places: Vec<PathBuf> = Vec::new();
    if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
        // The folder this package's fixtures keep their sockets in.
        places.push(home.join("temp/agent"));
        // The same place without the two segments the package adds, which a short home directory cannot spare.
        places.push(home.join("temp"));
    }
    if let Some(runtime) = std::env::var_os("XDG_RUNTIME_DIR").map(PathBuf::from) {
        // The directory the product itself serves a session's sockets in, which is the shortest place an account has.
        places.push(runtime);
    }
    return places;
}

/// What:
///  The folder this thread's runtime directories are made in:
///  the longest place this account offers that holds its own name and stays within `longest` bytes.
/// Why:
///   A socket address carries 107 bytes of path,
/// and a runtime directory of a fixed length leaves whatever is past the folder it stands in for its own name,
/// so the room a name has depends on this account and on this process:
/// the gate's container has a home directory of twelve bytes and process numbers of six digits,
/// which leaves seven bytes under the package's usual `$HOME/temp/agent`,
/// so a control whose name needs eight panicked there while the same control passed where the home was shorter and the
/// number had fewer digits.
/// The places are tried from the package's own folder to the shortest this account has,
/// and a place that leaves too little is named rather than assumed away.
/// The folder is this thread's alone,
/// named with a token and made exclusively,
/// and it owns everything the helper puts in it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// runtimeDirectoryParent('pr', 32) // '/home/tester/temp/agent/pr3f9a1c'
/// ```
pub(crate) fn runtime_directory_parent(prefix: &str, longest: usize) -> PathBuf {
    let name_length: usize = prefix.len() + FOLDER_TOKEN_DIGITS;
    let offered: Vec<PathBuf> = places_this_account_offers();
    let fitting: Vec<&PathBuf> = offered.iter().filter(|place| { return place.as_os_str().len() + 1 + name_length <= longest; }).collect();
    if fitting.is_empty() {
        let tried: String = if offered.is_empty() {
            String::from("this account names no home directory and no runtime directory")
        } else {
            let described: Vec<String> = offered.iter().map(|place| {
                let folder: PathBuf = place.join(format!("{prefix}{}", "x".repeat(FOLDER_TOKEN_DIGITS)));
                return format!("{} ({} bytes)", folder.display(), folder.as_os_str().len());
            }).collect();
            described.join(", ")
        };
        panic!("no folder this account offers leaves room for a runtime directory's name: the control needs a folder of at most {longest} bytes, and the folders are {tried}");
    }
    // The longest place that fits is the folder's place; a name another run took there moves this thread to another
    // token rather than to a shorter place, so the place a control asks for is the place it gets.
    let place: &Path = fitting.first().expect("a place that fits was found above").as_path();
    std::fs::create_dir_all(place).expect("a place this account offers");
    for _attempt in 0..FOLDER_TOKEN_ATTEMPTS {
        let folder: PathBuf = place.join(format!("{prefix}{}", this_threads_folder_token()));
        if this_thread_made(folder.as_path()) {
            return folder;
        }
        match std::fs::create_dir(folder.as_path()) {
            Ok(()) => {
                this_thread_made_folder(folder.as_path());
                return folder;
            }
            // A folder another run made is not shared, so this thread takes another token.
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => this_thread_takes_another_token(),
            Err(error) => panic!("the folder {} cannot be made: {error}", folder.display()),
        }
    }
    panic!("no folder was left for this thread: {FOLDER_TOKEN_ATTEMPTS} tokens were tried and every one of them named a folder another run had made");
}

/// What:
///  A runtime directory of exactly `length` bytes under `parent`,
///  named for `tag` where the room below `parent` holds the tag and for a digest of it where it does not.
/// Why:
///   The boundary these controls move past is a count of bytes,
/// so the directory is built to the byte and its length asserted rather than assumed.
/// The folder is the caller's,
/// because a control that needs room for another spelling of the same directory decides how long the name may be.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// runtimeDirectoryUnder(parent, 40, 'longest') // `${parent}/longest`
/// ```
pub(crate) fn runtime_directory_under(parent: &Path, length: usize, tag: &str) -> PathBuf {
    let room: usize = length.checked_sub(parent.as_os_str().len() + 1).expect("a runtime directory longer than the folder it stands in");
    assert!(
        room >= room_a_runtime_name_needs(tag),
        "a runtime directory of {length} bytes leaves {room} bytes below {}, too little to name a control {tag:?} in",
        parent.display()
    );
    let name: String = if room >= tag.len() {
        format!("{tag}{}", "x".repeat(room - tag.len()))
    } else {
        let mut digest: String = crate::service_instance::digest_of_bytes(tag.as_bytes());
        digest.truncate(room);
        digest
    };
    let runtime: PathBuf = parent.join(name);
    std::fs::create_dir_all(&runtime).expect("the runtime directory");
    assert_eq!(runtime.as_os_str().len(), length, "the directory is exactly as long as the control asks");
    track_for_this_thread(runtime.as_path());
    return runtime;
}
