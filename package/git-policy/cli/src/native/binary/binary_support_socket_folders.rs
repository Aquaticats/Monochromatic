//! What:
//!  The folder a binary control's sockets stand in:
// its name token,
// the places this account offers,
// and its owner.
//! Why:
//!  A Unix socket's name must fit `SUN_LEN`,
//! so the folder has to be short,
// and it must be this control's own:
//! a name built from a process number comes round,
// and two runs that meet in one folder fail each other.
//! The folder belongs to the fixture that made it,
// which removes it when the control ends.
//! It lives in a file of its own because `binary_support_fixture_service.rs` is at the code-line limit.

/// The path types the folders are named with.
use std::path::PathBuf;

/// What:
///  The folders one fixture's sockets stand in:
/// the token they are named with,
/// and the ones it has made.
/// Why:
///   A folder named after the process number is not a name of one's own,
/// and a folder is never shared:
/// the token comes from the operating system's random source and every folder is made exclusively,
/// so two runs cannot meet in one even when they draw the same token.
/// The folders are remembered so a second call finds the folder this fixture made rather than another,
/// and they are what the fixture removes when the control ends.
#[derive(Default)]
pub struct SocketFolders {
    /// The six hexadecimal digits this fixture's folders are named with,
    /// made on first use.
    token: String,
    /// The folders this fixture made,
    /// in the order it made them.
    made: Vec<PathBuf>,
}

/// The hexadecimal digits the name token of a folder a control's sockets stand in takes.
const FOLDER_TOKEN_DIGITS: usize = 6;

/// How many tokens one fixture tries before it gives up on a place this account offers.
const FOLDER_TOKEN_ATTEMPTS: usize = 8;

/// What:
///  A fresh token of `FOLDER_TOKEN_DIGITS` hexadecimal digits,
/// the first of a UUID from the operating system.
/// Why:
///   A folder named after the process number is not a name of one's own:
/// about 549 processes start a second on this machine,
/// so a number comes round in about two hours,
/// and inside a container far sooner,
/// and two runs of one process number met in one folder and failed each other at `AlreadyExists`.
/// Six digits are the room a folder's name has in the shortest place this account offers,
/// and the folder is made exclusively,
/// so a token drawn twice still names two folders.
fn fresh_folder_token() -> String {
    let uuid: String = git_policy_cli::random_id::random_uuid().expect("the operating system's random source");
    return uuid.get(..FOLDER_TOKEN_DIGITS).expect("a UUID is longer than the token").to_string();
}

/// What:
///  The places this account offers for a folder a control's sockets stand in,
/// the longest first.
/// Why:
///   The place names no part of its own:
/// the folder under it is this fixture's,
/// and its caller adds that.
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
impl SocketFolders {
    /// What:
    ///  The folders this fixture made,
    /// in the order it made them,
    /// which it removes when the control ends.
    /// Why:
    ///   The owner removes what it made,
    /// so a passing control leaves nothing in a place it shares.
    pub fn made_folders(&self) -> Vec<PathBuf> {
        return self.made.clone();
    }

/// What:
///  The folder this fixture's sockets stand in:
///  the longest place this account offers that holds its own name and stays within `longest` bytes.
/// Why:
///   A Unix socket's name must fit `SUN_LEN`,
/// which is 108 bytes,
/// and a fixture root under a deep checkout does not,
/// so a control that bound its socket beside the fixture failed on a long checkout and passed on a short one
/// (2026-11-08),
/// and the same control then failed by the length of the home directory it was given (2026-11-15).
/// The places are tried from the package's own folder to the shortest this account has,
/// and a place that leaves too little is named rather than assumed away.
/// The folder is this fixture's alone,
/// named with a token and made exclusively,
/// and it goes with the fixture,
/// so a passing control leaves nothing in a place it shares.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// socketFolders.folder('rt', 27) // '/home/tester/temp/agent/rt3f9a1c'
/// ```
pub fn folder(&mut self, prefix: &str, longest: usize) -> PathBuf {
    let name_length: usize = prefix.len() + FOLDER_TOKEN_DIGITS;
    let offered: Vec<PathBuf> = places_this_account_offers();
    if !offered.iter().any(|place| { return place.as_os_str().len() + 1 + name_length <= longest; }) {
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
    for _attempt in 0..FOLDER_TOKEN_ATTEMPTS {
        if self.token.is_empty() {
            self.token = fresh_folder_token();
        }
        let name: String = format!("{prefix}{}", self.token);
        for place in &offered {
            if place.as_os_str().len() + 1 + name_length > longest {
                continue;
            }
            let folder: PathBuf = place.join(name.as_str());
            if self.made.iter().any(|made| { return made == &folder; }) {
                return folder;
            }
            std::fs::create_dir_all(place.as_path()).expect("a place this account offers");
            match std::fs::create_dir(folder.as_path()) {
                Ok(()) => {
                    self.made.push(folder.clone());
                    return folder;
                }
                // A folder another run made is not shared, so this fixture takes another token.
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => break,
                Err(error) => panic!("the folder {} cannot be made: {error}", folder.display()),
            }
        }
        self.token = fresh_folder_token();
    }
    panic!("no folder was left for this fixture: {FOLDER_TOKEN_ATTEMPTS} tokens were tried and every one of them named a folder another run had made");
}
}
