//! What:
//!  The loaded module's identity:
//! its release string,
//! its source version,
//! its build ID
//!  note,
//! and each of the two type-information files' device,
//! inode,
//! size and
//!  modification time.
//! Why:
//!  The check reads the identity,
//! reads the two files and reads the identity again,
//! so a
//!  module that unloaded or was replaced between the reads is noticed rather
//! than
//!  compared half from one module and half from another;
//! and the per-process cache
//!  compares the same facts before it reuses a verdict.
//!  None of the three values is a rule:
//! no step compares the release string with
//!  an approved one,
//! and an attribute a module does not carry weakens the reload check
//!  instead of refusing,
//! because the project's answer leaves only the layout and the
//!  request numbers to decide.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const identity = readIdentity(paths); // { version?, srcversion?, buildId?, base, module }
//! ```

use super::gate_paths::GatePaths;
use monochromatic_zfs_request_to_openzfs_kernel_device::{CompatibilityRefusal as Refusal, CompatibilityStep as Step};
use std::path::Path;
use std::time::SystemTime;

/// The most bytes one identity attribute is read as:
/// the sysfs page it is created as,
/// less the newline some of them end with.
pub const IDENTITY_BOUND: usize = 4_096;

/// What:
///  One file's own facts,
/// which a reload moves.
/// Why:
///  The inode and size move when a file is replaced,
/// the modification time is
///  the kernel's own reading of when it was last written,
/// and the device says
///  which filesystem answered.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileFacts {
    /// The filesystem the file sits on.
    pub device: u64,
    /// The file's inode number.
    pub inode: u64,
    /// The file's size in bytes.
    pub size: u64,
    /// When the file was last modified,
    /// when the filesystem reports it.
    pub modified: Option<SystemTime>,
}

impl FileFacts {
    /// One file's facts,
    /// or nothing when it cannot be read.
    pub fn of(path: &Path) -> Option<Self> {
        let metadata = std::fs::metadata(path).ok()?;
        use std::os::unix::fs::MetadataExt;
        return Some(FileFacts {
            device: metadata.dev(),
            inode: metadata.ino(),
            size: metadata.size(),
            modified: metadata.modified().ok(),
        });
    }
}

/// What:
///  The identity one check compares before and after the two reads.
/// Why:
///  An absent attribute is `None` and never a refusal,
/// and the two files' facts
///  stand in its place only for the cache's recheck,
/// never for the reload check.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModuleIdentity {
    /// The module's release string,
    /// stripped of one trailing newline.
    pub version: Option<String>,
    /// The module's source version.
    pub srcversion: Option<String>,
    /// The module's build ID note,
    /// compared as bytes.
    pub build_id: Option<Vec<u8>>,
    /// The kernel's type information.
    pub base: Option<FileFacts>,
    /// The loaded module's type information.
    pub module: Option<FileFacts>,
}

impl ModuleIdentity {
    /// Read the module's three values and the two files' facts.
    pub fn read(paths: &GatePaths) -> Self {
        return ModuleIdentity {
            version: read_text(paths.module_version.as_path()),
            srcversion: read_text(paths.module_srcversion.as_path()),
            build_id: read_bytes(paths.module_build_id.as_path()),
            base: FileFacts::of(paths.base_btf.as_path()),
            module: FileFacts::of(paths.module_btf.as_path()),
        };
    }

    /// Whether the module that answered now is the one that answered before.
    ///
    /// The three values decide where the module carries any of them;
    /// where it
    ///  carries none the two files' facts decide,
    /// which is the weaker check the design
    ///  states and `doctor` reports.
    pub fn same_as(&self, other: &Self) -> bool {
        let carried: bool = self.version.is_some() || self.srcversion.is_some() || self.build_id.is_some();
        if carried {
            return self.version == other.version
                && self.srcversion == other.srcversion
                && self.build_id == other.build_id;
        }
        return self.base == other.base && self.module == other.module;
    }

    /// Which identity attributes this module does not carry,
    /// so a reader can tell a weaker reload check from a refused one.
    pub fn absent_attributes(&self) -> Vec<&'static str> {
        let mut absent: Vec<&'static str> = Vec::<&'static str>::new();
        if self.version.is_none() {
            absent.push("version");
        }
        if self.srcversion.is_none() {
            absent.push("srcversion");
        }
        if self.build_id.is_none() {
            absent.push("build ID");
        }
        return absent;
    }

    /// The refusal one module's own identity gets when the module changed under
    ///  the check.
    pub fn changed_under_the_check() -> Refusal {
        return Refusal {
            step: Step::Identity,
            file: None,
            detail: String::from(
                "the loaded module's release string, source version or build ID differed between the two identity reads, and the retry's own two reads differed as well, so the layout that was compared may be another module's",
            ),
        };
    }
}

/// One text attribute,
/// read to a page and stripped of one trailing newline.
pub(super) fn read_text(path: &Path) -> Option<String> {
    let bytes: Vec<u8> = read_bytes(path)?;
    let mut text: String = String::from_utf8_lossy(bytes.as_slice()).into_owned();
    if text.ends_with('\n') {
        text.pop();
    }
    return Some(text);
}

/// One attribute's bytes,
/// read to a page.
fn read_bytes(path: &Path) -> Option<Vec<u8>> {
    use std::io::Read;
    let file = std::fs::File::open(path).ok()?;
    let mut bytes: Vec<u8> = Vec::<u8>::new();
    file.take(IDENTITY_BOUND as u64).read_to_end(&mut bytes).ok()?;
    return Some(bytes);
}
