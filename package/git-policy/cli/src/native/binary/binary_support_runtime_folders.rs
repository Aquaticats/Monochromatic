//! What:
//!  The runtime directories the commit-service controls listen in:
//!  who makes one,
//!  who names it,
//!  and who removes it.
//! Why:
//!  A socket name must fit one socket address (107 bytes),
//! and a fixture root under the build directory is far deeper than that,
//! so a control's service keeps its socket,
//! its unit files and its notes in a folder at the top of the agent scratch directory,
//! which every run of every session shares.
//!  A folder there belongs to the test threads that named it,
//! and the last of them removes it when it ends,
//! however its test ended.
//!  The folder is named for the process number,
//! and a process number comes round (about 549 processes start a second on this machine),
//! so each thread keeps the device and inode its folder was made with and removes only the folder those still name:
//! a folder another run made at the same name is that run's and is left alone.
//!  A folder is removed only once its service has ended:
//! the making threads are all gone,
//! and nothing answers at any socket file the folder still holds.
//! Nothing removes a folder whose service still listens;
//! it is kept,
//! and a line on standard error says so.
//!  A control that fails removes its folder like one that passes,
//! and `CLI_GIT_KEEP_FIXTURES` keeps it where it stands instead,
//! saying so on standard error,
//! until a person removes it.
//! A test binary that is killed runs no cleanup at all,
//! so its folder stays where it is,
//! and nothing removes it later:
//! a person does,
//! or the sweep the owner runs over folders older than a day.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const runtime = runtimeFolderFor(name); // `${HOME}/temp/agent/rn-${process.pid}-${n}`
//! ```

/// The path types the folders are named with.
use std::path::{Path, PathBuf};

/// The patient removal a fixture tree is already removed with,
/// so a folder that is still gaining an entry is tried again rather than left.
use crate::support_fixture_directories::{REMOVAL_PAUSE, REMOVAL_TRIES, remove_patiently};

/// The variable that keeps a control's folders,
/// and the ending of a process still working in one.
use crate::support_fixture_service::{KEEP_FIXTURES_VARIABLE, end_processes_working_in};

/// The device and inode an entry had when a control made it,
/// and what stands at that path now.
#[path = "../testing/folder_identity.rs"]
mod folder_identity;

/// The identity a folder is made and removed by,
/// and the answer to what stands at its path now.
use folder_identity::{FolderIdentity, FolderStanding};

/// What:
///  One name this process has given,
///  the folder it names now,
///  the identity that folder had when it was made,
///  and how many test threads have named it and not yet ended.
/// Why:
///   Two controls of one process can give one name:
/// the folder is then one directory both listen in,
/// and it goes when the last of them ends.
/// The identity is what the removal is held to,
/// so the name alone never decides what is removed.
struct Given {
    /// The name the control asked with.
    name: String,
    /// The folder that name stands for now.
    folder: PathBuf,
    /// The device and the inode that folder had when it was made.
    identity: FolderIdentity,
    /// The test threads that have named this folder and have not yet ended.
    threads: usize,
}

/// What:
///  The names this process has given,
///  and the number the next folder takes.
/// Why:
///   A name asked twice is answered with the folder it named,
/// which is the one folder both controls listen in;
/// a folder that is gone or that another run took is replaced by a fresh number,
/// so a folder is never shared with another run.
struct Names {
    /// The names given so far,
    /// in the order they were first given.
    given: Vec<Given>,
    /// The number the next folder of this process carries.
    next: usize,
}

/// The names this process has given.
static NAMES: std::sync::Mutex<Names> = std::sync::Mutex::new(Names { given: Vec::new(), next: 0 });

/// The runtime folders the running test thread named,
/// removed when the thread ends however its test ended.
struct MadeHere {
    /// The folders this thread named,
    /// in the order it named them.
    folders: std::cell::RefCell<Vec<PathBuf>>,
}

/// A panic unwinds the test thread,
/// which runs this;
/// a thread whose test passed runs it too.
impl Drop for MadeHere {
    fn drop(&mut self) {
        for folder in self.folders.borrow().iter() {
            release(folder.as_path());
        }
    }
}

thread_local! {
    /// The runtime folders the running test thread owns.
    static MADE_HERE: MadeHere = const { MadeHere { folders: std::cell::RefCell::new(Vec::new()) } };
}

/// What:
///  The folder this machine's agent scratch directory holds these runtime directories in.
/// Why:
///   A socket name must fit one socket address,
/// and this is the shortest place an account offers that a control may write in;
/// each folder under it is one process's own.
fn scratch_root() -> PathBuf {
    return PathBuf::from(std::env::var_os("HOME").expect("a home directory")).join("temp/agent");
}

/// What:
///  Make one runtime folder of this process and identify it.
/// Why:
///   The name a control asks with is not a folder name:
/// the socket path is the runtime directory,
/// the folder of the sockets and a 32-digit instance name,
/// and the whole must fit one socket address,
/// so the folder is a number,
/// `rn-<process number>-<number>`,
/// and not the test's name.
fn make_folder(names: &mut Names) -> (PathBuf, FolderIdentity) {
    let folder: PathBuf = scratch_root().join(format!("rn-{}-{}", std::process::id(), names.next));
    names.next += 1;
    if let Err(error) = std::fs::create_dir_all(folder.as_path()) {
        panic!("the runtime directory {} cannot be made: {error}", folder.display());
    }
    let identity: FolderIdentity = folder_identity::identity_of(folder.as_path()).expect("the runtime directory this call made is there to be identified");
    return (folder, identity);
}

/// What:
///  The runtime directory the control named `name` listens in,
///  made on the first call and the same folder on every call after it.
/// Why:
///   The maker is also the owner:
/// the thread that asks takes the folder into its own cleanup,
/// so a folder no service handle ever names is still removed when that thread ends.
///  A folder an earlier control of this process made and removed is made again under this thread's name,
/// and a folder another run made at that path is passed over for a fresh number,
/// so no two runs ever write in one folder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// runtimeFolderFor('ceiling-4242') // '/home/tester/temp/agent/rn-4242-0'
/// ```
pub fn runtime_folder_for(name: &str) -> PathBuf {
    let mut names: std::sync::MutexGuard<'_, Names> = NAMES.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let known: Option<usize> = names.given.iter().position(|one| { return one.name == name; });
    if let Some(index) = known
        && names.given.get(index).is_some_and(|one| { return one.identity.standing(one.folder.as_path()) == FolderStanding::StillTheOne; })
    {
        let Some(one) = names.given.get_mut(index) else { unreachable!("the index came from this list") };
        one.threads += 1;
        let folder: PathBuf = one.folder.clone();
        drop(names);
        MADE_HERE.with(|made| { made.folders.borrow_mut().push(folder.clone()); });
        return folder;
    }
    let (folder, identity): (PathBuf, FolderIdentity) = make_folder(&mut names);
    match known.and_then(|index| { return names.given.get_mut(index); }) {
        Some(one) => {
            one.folder = folder.clone();
            one.identity = identity;
            one.threads = 1;
        }
        None => names.given.push(Given { name: name.to_owned(), folder: folder.clone(), identity, threads: 1 }),
    }
    drop(names);
    MADE_HERE.with(|made| { made.folders.borrow_mut().push(folder.clone()); });
    return folder;
}

/// What:
///  Let go of one runtime folder this test thread named,
///  and remove it when this thread was the last one that named it,
///  if it is still the folder this process made and nothing listens in it.
/// Why:
///   A thread that ends after a passing test,
/// one whose test panicked,
/// and a handle whose service has ended all come here,
/// so the question is asked in one place.
///  A folder another run made at that path is not this process's and is left as it is;
/// a folder whose service still answers is kept,
/// because removing the socket folder under a live service leaves it listening at nothing.
pub fn release(folder: &Path) {
    let mut names: std::sync::MutexGuard<'_, Names> = NAMES.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let Some(index) = names.given.iter().position(|one| { return one.folder == folder; }) else {
        drop(names);
        eprintln!("runtime folders: {} was not made by this run, so it is left as it is", folder.display());
        return;
    };
    let Some(one) = names.given.get_mut(index) else { unreachable!("the index came from this list") };
    if one.threads > 1 {
        one.threads -= 1;
        return;
    }
    one.threads = 0;
    let identity: FolderIdentity = one.identity;
    drop(names);
    match identity.standing(folder) {
        FolderStanding::Gone => {}
        FolderStanding::AnotherEntryNow => {
            eprintln!("runtime folders: {} is another run's folder now, so it is left as it is", folder.display());
        }
        FolderStanding::StillTheOne => {
            if std::env::var_os(KEEP_FIXTURES_VARIABLE).is_some() {
                eprintln!("runtime folders: {KEEP_FIXTURES_VARIABLE} is set, so {} is kept where it is until a person removes it", folder.display());
                return;
            }
            if let Some(address) = something_answers(folder) {
                eprintln!(
                    "runtime folders: {} is kept because a service still answers at {}",
                    folder.display(),
                    address.display()
                );
                return;
            }
            // Anything a stopped service left working in the folder goes before the folder does.
            end_processes_working_in(folder);
            let removed: std::io::Result<()> = remove_patiently(folder, &|root| { return std::fs::remove_dir_all(root); }, REMOVAL_TRIES, REMOVAL_PAUSE);
            if let Err(error) = removed {
                eprintln!("runtime folders: {} could not be removed: {error}", folder.display());
            }
        }
    }
}

/// What:
///  One socket file the folder holds that something answers on,
///  or nothing where no service listens in the folder.
/// Why:
///   The folder is removed only once its service has ended,
/// and a socket a live service holds is that service still being there.
///  Only a socket file is asked:
/// a service at an abstract address has no file,
/// and such a service is not found this way (the folder is long past what a socket path carries,
/// and no control of this suite starts one there).
fn something_answers(folder: &Path) -> Option<PathBuf> {
    let sockets: std::fs::ReadDir = std::fs::read_dir(folder.join(git_policy_cli::service_instance::SOCKET_DIRECTORY)).ok()?;
    for entry in sockets.flatten() {
        let socket: PathBuf = entry.path();
        if socket.extension().is_none_or(|extension| { return extension != "sock"; }) {
            continue;
        }
        // A socket file a service has let go refuses a connection, and one a service holds accepts it.
        if std::os::unix::net::UnixStream::connect(&socket).is_ok() {
            return Some(socket);
        }
    }
    return None;
}
