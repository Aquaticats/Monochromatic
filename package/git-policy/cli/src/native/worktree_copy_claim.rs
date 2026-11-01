//! What:
//!  The one-time claim on a destination worktree's copy, published by a hard link and held open for the copy's life,
//!  and the kernel evidence that tells a live claim from an abandoned one.
//! Why:
//!  Two copies must never work on one destination, and a recovery pass must never touch a copy that is still running,
//!  but nothing may wait, poll or record an owner.
//!  The creator keeps the claim file open, so "some process has this file open" is true exactly while the copy runs and
//!  stops being true when the process dies, however it dies.
//!  A claim nobody holds open is abandoned, and the first process to open it and find no other holder settles it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const claiming = await claimDestination({ root, key, destinationRoot, container, settle });
//! ```

use super::json_record::quote;
use super::pid_namespace::{current_pid_namespace, foreign_owner_standing, foreign_record_has_expired};
use super::private_storage::sync_directory;
use super::process_use::{PROC_ROOT, UseTarget, processes_using_through};
use super::random_id::random_uuid;
use super::worktree_copy_error::{WorktreeCopyError, failure_with_cause, raw_failure};
use super::worktree_copy_journal::{TEMPORARY_SUFFIX, remove_file_if_present};
use super::worktree_copy_selection::{STAGE_PREFIX, js_order};
use std::fs::{File, Metadata};
use std::io::{Read, Write};
use std::os::fd::AsRawFd;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

/// What:
///  The suffix of a published claim file.
/// Why:
///   Journals end in `.json`; a different suffix keeps a claim from ever being read as a journal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const CLAIM_SUFFIX = '.claim';
/// ```
pub const CLAIM_SUFFIX: &str = ".claim";

/// What:
///  The key prefix of a claim that adopts a journal no claim names.
/// Why:
///   An older release wrote journals without claims; the adopting pass claims them under their file stem.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ADOPTED_KEY_PREFIX = 'journal-';
/// ```
pub const ADOPTED_KEY_PREFIX: &str = "journal-";

/// What:
///  The longest claim key accepted.
/// Why:
///   The claim name is the key plus the suffix, and a file name holds at most 255 bytes once the temporary name's
///  unique part is added.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const LONGEST_KEY = 180;
/// ```
const LONGEST_KEY: usize = 180;

/// What:
///  How many times claiming retries after a vanished or settled claim.
/// Why:
///   Each retry follows a claim that went away, so a handful of rounds only fails when claims keep appearing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const CLAIM_ROUNDS = 3;
/// ```
const CLAIM_ROUNDS: usize = 3;

/// The permission bits of a claim file: private to the account.
const CLAIM_MODE: u32 = 0o600;

/// The permission bits a private file must not have.
const NON_PRIVATE_MODE_BITS: u32 = 0o077;

/// What:
///  What a claim file says: the destination, the stage container its copy uses, the PID namespace it was made in, and
///  whether the creator can be seen in the process table by other processes of the account.
/// Why:
///   Settling an abandoned claim needs the container's name, and the last two facts say whether the process table of
///  the settler can see the creator at all. A process that changed its credentials (a sandbox that enters a user
///  namespace, a program that clears its dump flag) hides its descriptors from its own account, and a settler judges
///  its claims by age instead. No process identifier is recorded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ClaimContent = { destinationRoot: string; observable: boolean; pidNamespace?: string; stageContainer: string; version: 1 };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaimContent {
    /// The worktree the copy fills.
    pub destination_root: String,
    /// Whether other processes of the account can read the creator's descriptors in the process table.
    pub observable: bool,
    /// The PID namespace of the creator, or nothing where the system has none.
    pub pid_namespace: Option<String>,
    /// The private stage container, which does not exist until staging creates it.
    pub stage_container: String,
}

impl ClaimContent {
    /// The container as a path.
    pub fn container(&self) -> PathBuf {
        return PathBuf::from(self.stage_container.as_str());
    }

    /// The claim text: sorted keys, one line.
    fn encode(&self) -> String {
        let namespace: String = self
            .pid_namespace
            .as_deref()
            .map_or_else(String::new, |name| { return format!("\"pidNamespace\":{},", quote(name)); });
        return format!(
            "{{\"destinationRoot\":{},\"observable\":{},{namespace}\"stageContainer\":{},\"version\":1}}\n",
            quote(self.destination_root.as_str()),
            self.observable,
            quote(self.stage_container.as_str())
        );
    }
}

/// What:
///  One claim this process holds open: the file, its name, and what it says.
/// Why:
///   Holding the file is the proof of life, so the claim keeps it for as long as the copy runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Claim = { path: string; handle: FileHandle; content: ClaimContent; release(): Promise<void> };
/// ```
#[derive(Debug)]
pub struct Claim {
    /// The claim file's name under the journal root.
    path: PathBuf,
    /// The open claim file; closing it is how the claim stops proving life.
    file: File,
    /// What the claim file says.
    content: ClaimContent,
}

impl Claim {
    /// What the claim file says.
    pub fn content(&self) -> &ClaimContent {
        return &self.content;
    }

    /// The stage container this claim names.
    pub fn container(&self) -> PathBuf {
        return self.content.container();
    }

    /// The claim file's name.
    pub fn path(&self) -> &Path {
        return self.path.as_path();
    }

    /// What:
    ///  Removes the claim file when the name still refers to the file this process holds, and makes the removal durable.
    /// Why:
    ///   The claim is the last thing a finished copy removes, after its journal and stage.
    ///  A name that no longer refers to this file is another claimant's and stays.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await claim.release();
    /// ```
    pub fn release(self) -> Result<(), WorktreeCopyError> {
        let held: Metadata = self.file.metadata().map_err(|error| return raw_failure(&error))?;
        let named: Option<Metadata> = match std::fs::symlink_metadata(self.path.as_path()) {
            Ok(found) => Some(found),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(raw_failure(&error)),
        };
        if named.is_some_and(|found| return same_file(&found, &held)) {
            remove_file_if_present(self.path.as_path())?;
            if let Some(parent) = self.path.parent() {
                sync_directory(parent).map_err(|error| return raw_failure(&error))?;
            }
        }
        return Ok(());
    }
}

/// What:
///  What looking at an existing claim found.
/// Why:
///   A caller yields to a live claim, retries after a vanished one, and settles an abandoned one it now holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Examination = { kind: 'gone' } | { kind: 'held'; reason: string } | { kind: 'abandoned'; claim: Claim };
/// ```
#[derive(Debug)]
pub enum Examination {
    /// The claim was removed or replaced while it was looked at.
    Gone,
    /// A process may still run the copy, or nothing proves otherwise; the text says why.
    Held {
        /// What the claim file says, so a caller can tell which containers claims still name.
        content: ClaimContent,
        /// Why the claim counts as held.
        reason: String,
    },
    /// No process holds the claim; this process now holds it and is the only one that does.
    Abandoned(Claim),
}

/// What:
///  The result of asking for a destination's claim.
/// Why:
///   The caller copies when it holds the claim and yields when another copy does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Claiming = { kind: 'claimed'; claim: Claim } | { kind: 'held'; reason: string };
/// ```
#[derive(Debug)]
pub enum Claiming {
    /// This process holds the destination's claim.
    Claimed(Claim),
    /// Another copy holds it, or it could not be settled; the text says why.
    Held(String),
}

/// What:
///  Whether two metadata records name one file.
/// Why:
///   A claim name can be replaced; only the device and inode say whether it is still the held file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// left.dev === right.dev && left.ino === right.ino
/// ```
fn same_file(left: &Metadata, right: &Metadata) -> bool {
    return left.dev() == right.dev() && left.ino() == right.ino();
}

/// What:
///  The claim path for a key.
/// Why:
///   The key is a worktree administrative name or an adopted journal's stem, so it must be one plain path component.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// join(root, `${key}.claim`)
/// ```
pub fn claim_path(root: &Path, key: &str) -> Result<PathBuf, WorktreeCopyError> {
    let plain: bool = !key.is_empty() && !key.contains(['/', '\0']) && key != "." && key != "..";
    if !plain || key.len() > LONGEST_KEY {
        return Err(failure_with_cause(
            format!(
                "cli-git: the worktree name {} cannot name a copy claim; the ignored-state copy into it is skipped.",
                quote(key)
            )
            .as_str(),
            None,
        ));
    }
    return Ok(root.join(format!("{key}{CLAIM_SUFFIX}")));
}

/// What:
///  The claim contents a stage container name must have: absolute, and named like a stage.
/// Why:
///   The container is removed recursively when its claim is settled, so a claim that names anything else is never acted on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isAbsolute(container) && basename(container).startsWith(STAGE_PREFIX)
/// ```
fn names_a_stage(container: &Path) -> bool {
    return container.is_absolute()
        && container
            .file_name()
            .is_some_and(|name| return name.to_string_lossy().starts_with(STAGE_PREFIX));
}

/// What:
///  Parses a claim file's text, or says it is not a claim of this release.
/// Why:
///   Settling removes whatever the claim names, so every field is checked before it is believed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseClaim(text: string): ClaimContent | undefined;
/// ```
fn parse_content(text: &str) -> Option<ClaimContent> {
    let value: serde_json::Value = serde_json::from_str(text).ok()?;
    let object = value.as_object()?;
    if object.get("version")?.as_f64()? != 1.0 {
        return None;
    }
    let stage_container: &str = object.get("stageContainer")?.as_str()?;
    let destination_root: &str = object.get("destinationRoot")?.as_str()?;
    if destination_root.is_empty() || !names_a_stage(Path::new(stage_container)) {
        return None;
    }
    return Some(ClaimContent {
        destination_root: destination_root.to_string(),
        // A claim that does not say is read as visible, as the claims of this release always say.
        observable: object
            .get("observable")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true),
        pid_namespace: object
            .get("pidNamespace")
            .and_then(serde_json::Value::as_str)
            .filter(|namespace| return !namespace.is_empty())
            .map(String::from),
        stage_container: stage_container.to_string(),
    });
}

/// What:
///  Reads and parses a claim file without keeping it open.
/// Why:
///   The read is short, so it hardly shows up as a holder to anyone scanning at the same moment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parseClaim(await readFile(path, 'utf8'))
/// ```
fn read_content(path: &Path) -> Result<Option<ClaimContent>, WorktreeCopyError> {
    let mut text: String = String::new();
    // `match` on the open: a claim removed in the meantime is simply gone.
    match File::open(path) {
        Ok(mut file) => {
            file.read_to_string(&mut text)
                .map_err(|error| return raw_failure(&error))?;
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(raw_failure(&error)),
    }
    return match parse_content(text.as_str()) {
        Some(content) => Ok(Some(content)),
        None => Err(failure_with_cause(
            format!(
                "cli-git: worktree-copy claim {} does not name a stage container; remove the file after checking it.",
                quote(path.to_string_lossy().as_ref())
            )
            .as_str(),
            None,
        )),
    };
}

/// What:
///  Whether a file is a plain file of this account that nobody else can read or write.
/// Why:
///   A claim others could write could name any directory for removal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// stat.isFile() && stat.uid === process.geteuid() && (stat.mode & 0o077) === 0
/// ```
fn is_private_file(metadata: &Metadata) -> bool {
    // SAFETY: `geteuid` has no preconditions and cannot fail.
    let effective_user: u32 = unsafe { libc::geteuid() };
    return metadata.is_file()
        && metadata.uid() == effective_user
        && (metadata.mode() & NON_PRIVATE_MODE_BITS) == 0;
}

/// What:
///  Publishes a claim for a key, unless one is already published.
/// Why:
///   The content is written to a temporary file this process keeps open and then linked to the claim name, so the
///  claim name never exists without a holder and never shows partial content; a link never replaces, so one
///  creator wins and the others learn it from `AlreadyExists` without waiting.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const claim = await publishClaim({ root, key, content }); // undefined when the name is taken
/// ```
pub fn publish_claim(
    root: &Path,
    key: &str,
    destination_root: &Path,
    container: &Path,
) -> Result<Option<Claim>, WorktreeCopyError> {
    let path: PathBuf = claim_path(root, key)?;
    let content: ClaimContent = ClaimContent {
        destination_root: destination_root.to_string_lossy().into_owned(),
        observable: this_process_is_observable(),
        pid_namespace: current_pid_namespace(),
        stage_container: container.to_string_lossy().into_owned(),
    };
    let unique: String = random_uuid().map_err(|error| return raw_failure(&error.0))?;
    let temporary: PathBuf = root.join(format!("{key}{CLAIM_SUFFIX}.{unique}{TEMPORARY_SUFFIX}"));
    let mut file: File = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(CLAIM_MODE)
        .open(temporary.as_path())
        .map_err(|error| return raw_failure(&error))?;
    let written: std::io::Result<()> = file
        .write_all(content.encode().as_bytes())
        .and_then(|()| return file.sync_all());
    // `match` on the link: the name is ours, taken by another claimant, or refused for a reason to report.
    let linked: std::io::Result<()> = written.and_then(|()| return std::fs::hard_link(temporary.as_path(), path.as_path()));
    let removal: Result<(), WorktreeCopyError> = remove_file_if_present(temporary.as_path());
    match linked {
        Ok(()) => {
            removal?;
            sync_directory(root).map_err(|error| return raw_failure(&error))?;
            return Ok(Some(Claim { path, file, content }));
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            removal?;
            return Ok(None);
        }
        Err(error) => {
            // The refusal is the failure to report; a leftover temporary is settled like any abandoned one.
            let _ignored_removal: Result<(), WorktreeCopyError> = removal;
            return Err(failure_with_cause(
                format!(
                    "cli-git: could not publish worktree-copy claim {}.",
                    quote(path.to_string_lossy().as_ref())
                )
                .as_str(),
                Some(&error),
            ));
        }
    }
}

/// What:
///  Whether another descriptor of this very process is open on a file.
/// Why:
///   The process scan skips this process, which cannot tell a claim of its own thread from the descriptor it opened
///  to look; the descriptor numbers can.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// openDescriptors().some((descriptor) => descriptor !== own && sameFile(descriptor, file))
/// ```
fn other_descriptor_here(identity: &Metadata, own: Option<i32>) -> Result<bool, String> {
    let listing = std::fs::read_dir("/proc/self/fd").map_err(|error| return format!("/proc/self/fd: {error}"))?;
    for item in listing.flatten() {
        let number: Option<i32> = item.file_name().to_string_lossy().parse::<i32>().ok();
        if number.is_none() || number == own {
            continue;
        }
        // `match` on the stat: a descriptor closed meanwhile, or one that is no file, is not the claim.
        if let Ok(found) = std::fs::metadata(item.path())
            && same_file(&found, identity)
        {
            return Ok(true);
        }
    }
    return Ok(false);
}

/// What:
///  Whether other processes of this account can read this process's descriptors in the process table.
/// Why:
///   A process that entered a user namespace, gained privileges or cleared its dump flag cannot be seen holding a claim,
///  so it must not take claims on the strength of the process table, and a claim it makes is judged by age.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// prctl(PR_GET_DUMPABLE) === 1
/// ```
fn this_process_is_observable() -> bool {
    #[cfg(target_os = "linux")]
    {
        // SAFETY: `PR_GET_DUMPABLE` takes no pointer arguments and only reads a flag of the calling process.
        return unsafe { libc::prctl(libc::PR_GET_DUMPABLE, 0, 0, 0, 0) } == 1;
    }
    // Without a process table nothing is observable, and the scan has already reported it unavailable.
    #[cfg(not(target_os = "linux"))]
    {
        return false;
    }
}

/// What:
///  Whether a claim is held or abandoned, from the process table.
/// Why:
///   An open descriptor on the claim is the one proof of life that dies with the process; everything unprovable counts as held.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reason = await heldReason({ path, identity, content, ownDescriptor }); // undefined when abandoned
/// ```
fn held_reason(
    path: &Path,
    identity: &Metadata,
    content: Option<&ClaimContent>,
    own: Option<i32>,
) -> Option<String> {
    let recorded: Option<&str> = content.and_then(|found| return found.pid_namespace.as_deref());
    // `match` on the namespace rule: a creator in another namespace is judged by the age of its claim, not by process.
    match foreign_owner_standing(recorded, path) {
        Some(true) => {
            return Some(String::from("it was made in another PID namespace and is not yet old enough to call abandoned"));
        }
        Some(false) => return None,
        None => {}
    }
    // A creator that hides its descriptors from its own account cannot be found in the process table either way.
    if content.is_some_and(|found| return !found.observable) {
        if foreign_record_has_expired(path) {
            return None;
        }
        return Some(String::from(
            "its creator hides its descriptors from other processes and the claim is not yet old enough to call abandoned",
        ));
    }
    let scan = processes_using_through(
        Path::new(PROC_ROOT),
        &UseTarget::File {
            device: identity.dev(),
            inode: identity.ino(),
        },
        std::process::id(),
    );
    if let Some(why) = scan.unavailable {
        return Some(format!("the process table cannot be read ({why})"));
    }
    if let Some(user) = scan.users.first() {
        return Some(format!("PID {} has it open", user.pid));
    }
    // Processes of this account whose descriptors cannot be read are no holders: every holder of a claim was observable
    // when it took the claim, because a creator records that it is and an adopter refuses to adopt unless it is.
    return match other_descriptor_here(identity, own) {
        Ok(true) => Some(String::from("another descriptor of this process has it open")),
        Ok(false) => None,
        Err(why) => Some(format!("the process table cannot be read ({why})")),
    };
}

/// What:
///  Looks at an existing claim, and takes it over when no process holds it.
/// Why:
///   The first look only stats, so a live claim is never disturbed; an abandoned one is opened and scanned again, and
///  only a process that finds no other holder after opening acts. Two processes that open together each see the other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const examination = await examineClaim(path);
/// ```
pub fn examine_claim(path: &Path) -> Result<Examination, WorktreeCopyError> {
    return examine_claim_with(path, &mut || {});
}

/// What:
///  `examine_claim` with a step that runs between the first look and the open.
/// Why:
///   A holder that appears in that gap is the case the second scan exists for, and a test cannot otherwise place it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await examineClaim(path, { betweenLooks });
/// ```
pub fn examine_claim_with(
    path: &Path,
    between_looks: &mut dyn FnMut(),
) -> Result<Examination, WorktreeCopyError> {
    let named: Metadata = match std::fs::symlink_metadata(path) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Examination::Gone),
        Err(error) => return Err(raw_failure(&error)),
    };
    if !is_private_file(&named) {
        return Err(failure_with_cause(
            format!(
                "cli-git: worktree-copy claim file is unsafe: {}.",
                quote(path.to_string_lossy().as_ref())
            )
            .as_str(),
            None,
        ));
    }
    let Some(content) = read_content(path)? else {
        return Ok(Examination::Gone);
    };
    if let Some(reason) = held_reason(path, &named, Some(&content), None) {
        return Ok(Examination::Held { content, reason });
    }
    between_looks();
    if !this_process_is_observable() {
        return Ok(Examination::Held {
            content,
            reason: String::from("this process hides its descriptors from other processes, so it does not adopt claims"),
        });
    }
    let file: File = match File::open(path) {
        Ok(opened) => opened,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Examination::Gone),
        Err(error) => return Err(raw_failure(&error)),
    };
    let opened: Metadata = file.metadata().map_err(|error| return raw_failure(&error))?;
    if !same_file(&opened, &named) {
        return Ok(Examination::Gone);
    }
    if let Some(reason) = held_reason(path, &opened, Some(&content), Some(file.as_raw_fd())) {
        return Ok(Examination::Held { content, reason });
    }
    return Ok(Examination::Abandoned(Claim {
        path: path.to_path_buf(),
        file,
        content,
    }));
}

/// What:
///  Claims a destination: publishes the claim, or settles the abandoned claim in the way and tries again.
/// Why:
///   A live claim means another copy owns the destination and this one yields at once; nothing waits for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const claiming = await claimDestination({ root, key, destinationRoot, container, settle });
/// ```
pub fn claim_destination(
    root: &Path,
    key: &str,
    destination_root: &Path,
    container: &Path,
    settle: &mut dyn FnMut(Claim) -> Result<(), WorktreeCopyError>,
) -> Result<Claiming, WorktreeCopyError> {
    let path: PathBuf = claim_path(root, key)?;
    for _round in 0..CLAIM_ROUNDS {
        if let Some(claim) = publish_claim(root, key, destination_root, container)? {
            return Ok(Claiming::Claimed(claim));
        }
        // `match` on what stands in the way: a vanished claim is retried, a live one is yielded to, and an abandoned one settled.
        match examine_claim(path.as_path())? {
            Examination::Gone => {}
            Examination::Held { reason, .. } => return Ok(Claiming::Held(reason)),
            Examination::Abandoned(abandoned) => settle(abandoned)?,
        }
    }
    return Ok(Claiming::Held(String::from("its claim kept changing")));
}

/// What:
///  The file names under a journal root that a test accepts, in journal-name order.
/// Why:
///   The pending check, the recovery pass and the temporary cleanup read the same listing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await readdir(root)).filter(matches).sort(jsOrder)
/// ```
fn names_matching(root: &Path, matches: &dyn Fn(&str) -> bool) -> Result<Vec<String>, WorktreeCopyError> {
    // `match` treats an absent root as having nothing, and every other failure as fatal.
    let listing = match std::fs::read_dir(root) {
        Ok(listing) => listing,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(raw_failure(&error)),
    };
    let mut names: Vec<String> = Vec::new();
    for listed in listing {
        let name: String = listed
            .map_err(|error| return raw_failure(&error))?
            .file_name()
            .to_string_lossy()
            .into_owned();
        if matches(name.as_str()) {
            names.push(name);
        }
    }
    names.sort_by(|left, right| return js_order(left, right));
    return Ok(names);
}

/// What:
///  The published claim files of a journal root.
/// Why:
///   Recovery examines each one; the pending check only asks whether any exists.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await listClaims(root)
/// ```
pub fn list_claims(root: &Path) -> Result<Vec<PathBuf>, WorktreeCopyError> {
    let names: Vec<String> = names_matching(root, &|name| return name.ends_with(CLAIM_SUFFIX))?;
    return Ok(names.into_iter().map(|name| return root.join(name)).collect());
}

/// Whether a file name is a claim's temporary: `<key>.claim.<unique>.tmp`.
fn is_claim_temporary(name: &str) -> bool {
    return name.contains(format!("{CLAIM_SUFFIX}.").as_str()) && name.ends_with(TEMPORARY_SUFFIX);
}

/// What:
///  Whether anything of a copy in flight stands under a journal root: a claim or a claim being published.
/// Why:
///   A forwarded command looks at recovery only when something is there, so the common case costs one directory read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await listClaimNames(root)).length > 0
/// ```
pub fn claims_exist(root: &Path) -> Result<bool, WorktreeCopyError> {
    let names: Vec<String> =
        names_matching(root, &|name| return name.ends_with(CLAIM_SUFFIX) || is_claim_temporary(name))?;
    return Ok(!names.is_empty());
}

/// What:
///  Removes the temporaries of claims whose publisher died before the link, and keeps the ones a live publisher holds.
/// Why:
///   A publisher keeps its temporary open until it is linked, so a temporary nobody holds belongs to a dead process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await removeAbandonedClaimTemporaries(root);
/// ```
pub fn remove_abandoned_temporaries(root: &Path) -> Result<(), WorktreeCopyError> {
    let names: Vec<String> = names_matching(root, &is_claim_temporary)?;
    for name in names {
        let path: PathBuf = root.join(name);
        let identity: Metadata = match std::fs::symlink_metadata(path.as_path()) {
            Ok(found) => found,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(raw_failure(&error)),
        };
        if held_reason(path.as_path(), &identity, None, None).is_none() {
            remove_file_if_present(path.as_path())?;
        }
    }
    return Ok(());
}

/// Claim controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_claim_tests.rs"]
mod tests;
