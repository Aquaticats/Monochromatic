//! What:
//!  Reading and writing the `UNTR` index extension (`dir.c`:
//! `read_untracked_extension`,
//! `write_untracked_extension`).
//! Why:
//!  The extension is kept across an in-process index update,
//! so it is decoded to directories and encoded again with the same
//!  layout:
//! directories in preorder,
//! then three bitmaps (valid,
//! check-only,
//! has-exclude-object) indexed by that order,
//!  then the stat records and object names of the flagged directories.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const cache = readUntracked(bytes, hashLength); // undefined: not a form this reads
//! const out = writeUntracked(cache);
//! ```

use crate::index_byte_search::{first_index_of_byte, has_nonzero_byte};
use crate::index_ewah::Ewah;
use crate::index_stat::{CACHE_STAT_LENGTH, StatData, be32, decode_varint, encode_varint};
use crate::index_untracked::{Untracked, UntrackedDir};

/// The deepest directory nesting the reader follows:
/// the depth the tree reader follows (`TREE_DEPTH_LIMIT` in `repo_read_tree.rs`),
/// as a directory of the cache cannot nest deeper than a tree.
const DEPTH_LIMIT: usize = 256;

/// The text up to the first NUL and the bytes after it.
fn take_text(bytes: &[u8]) -> Option<(&[u8], &[u8])> {
    let nul: usize = first_index_of_byte(bytes, 0)?;
    return Some((&bytes[..nul], &bytes[nul + 1..]));
}

/// Read one directory and,
/// after it,
/// its subdirectories in preorder;
/// counts the directories read.
fn read_dir<'a>(bytes: &'a [u8], count: &mut usize, depth: usize) -> Option<(UntrackedDir, &'a [u8])> {
    if depth > DEPTH_LIMIT {
        return None;
    }
    let (untracked_count, after_files) = decode_varint(bytes)?;
    let (dirs_count, after_dirs) = decode_varint(after_files)?;
    let (name, mut rest) = take_text(after_dirs)?;
    let mut untracked: Vec<Vec<u8>> = Vec::new();
    for _ in 0..untracked_count {
        let (file, following) = take_text(rest)?;
        untracked.push(file.to_vec());
        rest = following;
    }
    *count += 1;
    let mut dirs: Vec<UntrackedDir> = Vec::new();
    for _ in 0..dirs_count {
        let (child, following) = read_dir(rest, count, depth + 1)?;
        dirs.push(child);
        rest = following;
    }
    let dir: UntrackedDir = UntrackedDir {
        name: name.to_vec(),
        recurse: true,
        valid: false,
        check_only: false,
        stat: StatData::default(),
        exclude_id: Vec::new(),
        untracked,
        dirs,
    };
    return Some((dir, rest));
}

/// Hand the flags,
/// stat records and object names to the directories in preorder.
fn assign(dir: &mut UntrackedDir, position: &mut usize, flags: &Flags<'_>, cursor: &mut (usize, usize)) -> Option<()> {
    let index: usize = *position;
    *position += 1;
    dir.check_only = flags.check_only.binary_search(&index).is_ok();
    dir.valid = flags.valid.binary_search(&index).is_ok();
    if dir.valid {
        dir.stat = StatData::from_cache_bytes(flags.stats.get(cursor.0..cursor.0 + CACHE_STAT_LENGTH)?)?;
        cursor.0 += CACHE_STAT_LENGTH;
    }
    let has_id: bool = flags.has_id.binary_search(&index).is_ok();
    dir.exclude_id = if has_id { flags.ids.get(cursor.1..cursor.1 + flags.hash_length)?.to_vec() } else { vec![0; flags.hash_length] };
    if has_id {
        cursor.1 += flags.hash_length;
    }
    for child in &mut dir.dirs {
        assign(child, position, flags, cursor)?;
    }
    return Some(());
}

/// The bitmaps and record areas the directories are completed from.
struct Flags<'a> {
    /// Preorder positions of valid directories.
    valid: Vec<usize>,
    /// Preorder positions of check-only directories.
    check_only: Vec<usize>,
    /// Preorder positions of directories with an ignore-file object name.
    has_id: Vec<usize>,
    /// The stat records of the valid directories.
    stats: &'a [u8],
    /// The object names of the directories that have one.
    ids: &'a [u8],
    /// The bytes of one object name.
    hash_length: usize,
}

/// The fixed part of the extension:
/// what comes before the directories.
struct Header {
    /// The machine identification.
    ident: Vec<u8>,
    /// Stat data and object name of `.git/info/exclude`.
    info_exclude: (StatData, Vec<u8>),
    /// Stat data and object name of the excludes file.
    excludes_file: (StatData, Vec<u8>),
    /// The directory flags.
    dir_flags: u32,
    /// The per-directory ignore file name.
    exclude_per_dir: Vec<u8>,
}

impl Header {
    /// The cache these fields start,
    /// with `root` as its directories.
    fn with_root(self, root: Option<UntrackedDir>) -> Untracked {
        return Untracked {
            ident: self.ident,
            info_exclude: self.info_exclude,
            excludes_file: self.excludes_file,
            dir_flags: self.dir_flags,
            exclude_per_dir: self.exclude_per_dir,
            root,
        };
    }
}

/// The fixed part of `body` and the bytes after it.
fn read_header(body: &[u8], hash_length: usize) -> Option<(Header, &[u8])> {
    let (ident_length, after_length) = decode_varint(body)?;
    let ident: &[u8] = after_length.get(..usize::try_from(ident_length).ok()?)?;
    let fixed: &[u8] = &after_length[ident.len()..];
    let ids_at: usize = 2 * CACHE_STAT_LENGTH + 4;
    let header_end: usize = ids_at + 2 * hash_length;
    let (exclude_per_dir, after_name) = take_text(fixed.get(header_end..)?)?;
    let header: Header = Header {
        ident: ident.to_vec(),
        info_exclude: (StatData::from_cache_bytes(fixed.get(..CACHE_STAT_LENGTH)?)?, fixed.get(ids_at..ids_at + hash_length)?.to_vec()),
        excludes_file: (StatData::from_cache_bytes(fixed.get(CACHE_STAT_LENGTH..2 * CACHE_STAT_LENGTH)?)?, fixed.get(ids_at + hash_length..header_end)?.to_vec()),
        dir_flags: be32(fixed, 2 * CACHE_STAT_LENGTH)?,
        exclude_per_dir: exclude_per_dir.to_vec(),
    };
    return Some((header, after_name));
}

/// The extension from its bytes,
/// or `None` for anything outside the layout Git writes (Git would drop such a cache;
/// the
/// caller lets Git answer instead).
pub fn read_untracked(data: &[u8], hash_length: usize) -> Option<Untracked> {
    let (last, body) = data.split_last()?;
    if *last != 0 {
        return None;
    }
    let (header, mut rest) = read_header(body, hash_length)?;
    // A cache no status run has filled ends after the ignore-file name (Git then reads it without directories).
    if rest.is_empty() {
        return Some(header.with_root(None));
    }
    let (directory_count, after_count) = decode_varint(rest)?;
    rest = after_count;
    if directory_count == 0 {
        return if rest.is_empty() { Some(header.with_root(None)) } else { None };
    }
    let mut count: usize = 0;
    let (mut root, after_dirs) = read_dir(rest, &mut count, 0)?;
    rest = after_dirs;
    if u64::try_from(count).ok()? != directory_count {
        return None;
    }
    let mut bitmaps: Vec<Ewah> = Vec::new();
    for _ in 0..3 {
        let (bitmap, used) = Ewah::read(rest, count)?;
        bitmaps.push(bitmap);
        rest = &rest[used..];
    }
    let valid_bits: Vec<usize> = bitmaps[0].bits();
    let has_id_bits: Vec<usize> = bitmaps[2].bits();
    let stats_length: usize = valid_bits.len().checked_mul(CACHE_STAT_LENGTH)?;
    // The three areas must fill the rest exactly, as Git's final `next != end` test demands.
    if rest.len() != stats_length.checked_add(has_id_bits.len().checked_mul(hash_length)?)? {
        return None;
    }
    let flags: Flags<'_> = Flags { valid: valid_bits, check_only: bitmaps[1].bits(), has_id: has_id_bits, stats: &rest[..stats_length], ids: &rest[stats_length..], hash_length };
    assign(&mut root, &mut 0, &flags, &mut (0, 0))?;
    return Some(header.with_root(Some(root)));
}

/// What the writer collects while it walks the directories in preorder.
struct Writer {
    /// The number of directories written.
    count: usize,
    /// The directory records.
    records: Vec<u8>,
    /// Positions of valid directories.
    valid: Vec<usize>,
    /// Positions of check-only directories.
    check_only: Vec<usize>,
    /// Positions of directories with an object name.
    has_id: Vec<usize>,
    /// The stat records.
    stats: Vec<u8>,
    /// The object names.
    ids: Vec<u8>,
}

/// The subdirectories of `dir` that the cache descends into,
/// in order.
fn recursing_children(dir: &UntrackedDir) -> Vec<&UntrackedDir> {
    let mut recursing: Vec<&UntrackedDir> = Vec::new();
    for child in &dir.dirs {
        if child.recurse {
            recursing.push(child);
        }
    }
    return recursing;
}

/// Write one directory and its recursing subdirectories (`write_one_dir`).
fn write_dir(dir: &UntrackedDir, writer: &mut Writer) {
    let index: usize = writer.count;
    writer.count += 1;
    // A directory that is not valid keeps no listing and no check-only mark, as Git clears them while writing.
    let listing: &[Vec<u8>] = if dir.valid { dir.untracked.as_slice() } else { &[] };
    if dir.valid && dir.check_only {
        writer.check_only.push(index);
    }
    if dir.valid {
        writer.valid.push(index);
        dir.stat.to_cache_bytes(&mut writer.stats);
    }
    if has_nonzero_byte(dir.exclude_id.as_slice()) {
        writer.has_id.push(index);
        writer.ids.extend_from_slice(dir.exclude_id.as_slice());
    }
    encode_varint(listing.len() as u64, &mut writer.records);
    let recursing: Vec<&UntrackedDir> = recursing_children(dir);
    encode_varint(recursing.len() as u64, &mut writer.records);
    writer.records.extend_from_slice(dir.name.as_slice());
    writer.records.push(0);
    for file in listing {
        writer.records.extend_from_slice(file.as_slice());
        writer.records.push(0);
    }
    for child in recursing {
        write_dir(child, writer);
    }
}

/// The extension's bytes.
pub fn write_untracked(cache: &Untracked) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    encode_varint(cache.ident.len() as u64, &mut out);
    out.extend_from_slice(cache.ident.as_slice());
    cache.info_exclude.0.to_cache_bytes(&mut out);
    cache.excludes_file.0.to_cache_bytes(&mut out);
    out.extend_from_slice(&cache.dir_flags.to_be_bytes());
    out.extend_from_slice(cache.info_exclude.1.as_slice());
    out.extend_from_slice(cache.excludes_file.1.as_slice());
    out.extend_from_slice(cache.exclude_per_dir.as_slice());
    out.push(0);
    let Some(root) = cache.root.as_ref() else {
        // No directories: Git writes a zero count and nothing after it.
        encode_varint(0, &mut out);
        return out;
    };
    let mut writer: Writer = Writer { count: 0, records: Vec::new(), valid: Vec::new(), check_only: Vec::new(), has_id: Vec::new(), stats: Vec::new(), ids: Vec::new() };
    write_dir(root, &mut writer);
    encode_varint(writer.count as u64, &mut out);
    out.extend_from_slice(writer.records.as_slice());
    Ewah::from_bits(writer.valid.as_slice()).serialize(&mut out);
    Ewah::from_bits(writer.check_only.as_slice()).serialize(&mut out);
    Ewah::from_bits(writer.has_id.as_slice()).serialize(&mut out);
    out.extend_from_slice(writer.stats.as_slice());
    out.extend_from_slice(writer.ids.as_slice());
    out.push(0);
    return out;
}

#[cfg(test)]
#[path = "index_untracked_io_tests.rs"]
mod tests;
