//! What:
//!  What a caller of the index stat rewrite supplies and reads back:
//!  the stat fields a file reports now,
//!  the stat fields an entry stored,
//!  the two checks the rewrite asks for,
//!  what the caller knows about the index beyond its bytes,
//!  and how one rewrite ended.
//! Why:
//!  Keeping the file system out of the rewrite makes every branch testable with built bytes,
//!  so the lookup and the trust check are traits a type that holds the state implements,
//!  and the context carries the two facts only the caller has:
//!  the repository's hash length and the index file's own modification time.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! interface StatLookup { statNow(path: Uint8Array): StatFields | undefined }
//! interface StatTrust { isTrusted(path: Uint8Array, stored: StoredStat): boolean }
//! type RewriteContext = { hashLength?: number; indexMtime?: [number, number] };
//! type IndexOutcome = { bytes: Uint8Array; updated: number; total: number } | { skipped: string };
//! ```

/// What:
///  What a file on disk reports now,
/// in the units an index entry stores.
///  `u32` is an unsigned 32-bit integer (siblings `u64` and `i64`);
/// an index stores each stat field as 32 bits.
/// Why:
///   The caller fills it from `lstat`;
/// the rewrite compares size and modification time and copies the rest.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StatFields = { ctimeSeconds: number; ctimeNanos: number; mtimeSeconds: number; mtimeNanos: number; dev: number; ino: number; uid: number; gid: number; size: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StatFields {
    /// Change time,
    /// seconds.
    pub ctime_seconds: u32,
    /// Change time,
    /// nanoseconds.
    pub ctime_nanos: u32,
    /// Modification time,
    /// seconds.
    pub mtime_seconds: u32,
    /// Modification time,
    /// nanoseconds.
    pub mtime_nanos: u32,
    /// Device,
    /// low 32 bits.
    pub dev: u32,
    /// Inode,
    /// low 32 bits.
    pub ino: u32,
    /// Owner.
    pub uid: u32,
    /// Group.
    pub gid: u32,
    /// Size,
    /// low 32 bits.
    pub size: u32,
}

/// What:
///  The result of one rewrite.
///  `Vec<u8>` is the owned bytes of the new file;
/// `String` is the owned reason a rewrite was declined.
/// Why:
///   A declined index is left as copied,
/// and the reason goes into the report,
/// so the cost (a slow first status) is
///  explained.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type IndexOutcome = { bytes: Uint8Array; updated: number; total: number } | { skipped: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IndexOutcome {
    /// The new file;
    /// `updated` of `total` entries got fresh stat fields.
    Rewritten {
        /// The whole new index file,
        /// checksum included.
        bytes: Vec<u8>,
        /// Entries whose stat fields were replaced.
        updated: usize,
        /// Entries in the index.
        total: usize,
    },
    /// The index was not changed,
    /// for this reason.
    Skipped(String),
}

/// What:
///  Where the rewrite of an index asks what `lstat` reports now for an entry's path.
///  A `trait` is a named set of methods that different types can each supply,
/// like a TypeScript `interface`.
/// Why:
///   The lookup may keep state (a table the clone filled,
/// a cursor that keeps its directory open),
/// and the rewrite stays free of the
///  file system;
/// a type that holds the state implements this.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface StatLookup { statNow(path: Uint8Array): StatFields | undefined }
/// ```
pub trait StatLookup {
    /// What `lstat` reports now for `path`,
    /// or nothing for a file that is missing.
    fn stat_now(
        &mut self,
        path: &[u8],
    ) -> Option<StatFields>;
}

/// What:
///  Where the rewrite of an index asks whether the file an entry was recorded for is still the one the copy was made from.
/// Why:
///   The check keeps the reference trees it compares with;
/// a type that holds them implements this.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface StatTrust { isTrusted(path: Uint8Array, stored: StoredStat): boolean }
/// ```
pub trait StatTrust {
    /// Whether the entry for `path`,
    /// with the fields it stored,
    /// is still trusted.
    fn is_trusted(
        &mut self,
        path: &[u8],
        stored: &StoredStat,
    ) -> bool;
}

/// The check that accepts every entry:
/// no reference index needs to agree.
pub struct AcceptEveryStat;

impl StatTrust for AcceptEveryStat {
    fn is_trusted(
        &mut self,
        _path: &[u8],
        _stored: &StoredStat,
    ) -> bool {
        return true;
    }
}

/// What:
///  What the caller knows about the index beyond its bytes.
///  `Option<usize>` is a value that may be absent.
/// Why:
///   The repository's hash length decides how an all-zero checksum is read,
/// and the index file's own modification time
///  decides which entries are racy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RewriteContext = { hashLength?: number; indexMtime?: [number, number] };
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct RewriteContext {
    /// The repository's hash length in bytes (20 or 32) when it was read from its configuration.
    pub hash_length: Option<usize>,
    /// The modification time of the index file the bytes came from:
    /// seconds and nanoseconds.
    pub index_mtime: Option<(u32, u32)>,
}

/// What:
///  The stat fields an entry holds,
/// as stored.
/// Why:
///   The caller compares them with the file the index was written for,
/// so an entry is refreshed only when that file is
///  the one Git recorded and not one that changed since.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StoredStat = { ctimeSeconds: number; ctimeNanos: number; mtimeSeconds: number; mtimeNanos: number; ino: number; uid: number; gid: number; size: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StoredStat {
    /// Change time,
    /// seconds.
    pub ctime_seconds: u32,
    /// Change time,
    /// nanoseconds (0 when Git was built without nanosecond support).
    pub ctime_nanos: u32,
    /// Modification time,
    /// seconds.
    pub mtime_seconds: u32,
    /// Modification time,
    /// nanoseconds (0 when Git was built without nanosecond support).
    pub mtime_nanos: u32,
    /// Inode,
    /// low 32 bits.
    pub ino: u32,
    /// Owner.
    pub uid: u32,
    /// Group.
    pub gid: u32,
    /// Size,
    /// low 32 bits.
    pub size: u32,
}
