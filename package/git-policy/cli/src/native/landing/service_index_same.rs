//! What:
//!  Whether a new index file holds the bytes of the live one.
//! Why:
//!  The release of the index lock installs a copy only when it differs from the live index,
//! so an unchanged index keeps its inode
//!  and its access list and costs no flush or rename.
//! The module is split from `service_index_hold.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (!sameIndex(built, live)) { flush(built); rename(built, live); }
//! ```

use std::path::Path;

/// What:
///  The length of a file from its metadata.
/// Why:
///  `Result::map` hands the metadata to a function.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lengthOf = (stats: Stats) => stats.size;
/// ```
fn length_of(found: std::fs::Metadata) -> u64 {
    return found.len();
}

/// What:
///  Reads the last bytes of a file into `tail`,
/// from `start`.
/// Why:
///  Two index files are compared by their tails first,
/// where the checksum sits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await file.read(tail, 0, tail.length, start)
/// ```
fn read_tail(file: &mut std::fs::File, tail: &mut Vec<u8>, start: u64) -> std::io::Result<()> {
    use std::io::{Read as _, Seek as _, SeekFrom};
    file.seek(SeekFrom::Start(start))?;
    return file.read_exact(tail.as_mut_slice());
}

/// What:
///  Whether a byte is not zero.
/// Why:
///  A checksum of all zero bytes means the index was written without one,
/// and then every byte is compared.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isNonzero = (byte: number) => byte !== 0;
/// ```
fn is_nonzero(byte: &u8) -> bool {
    return *byte != 0;
}

/// What:
///  Whether the file `built` holds the same bytes as the live index file `live`.
/// Why:
///   An index ends with a checksum of all it holds,
/// so files of one size with the same tail are the same file,
/// and the common
///  case is settled by reading the tails only.
/// A tail of zeros is what a skipped checksum looks like and proves nothing,
/// so then
///  the files are compared in full.
/// A file that cannot be read is taken as different,
/// which installs the copy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// sameIndex(built, live) // true: nothing to install
/// ```
pub(crate) fn same_index(built: &Path, live: &Path) -> bool {
    const TAIL: u64 = 32;
    let (Ok(mut first), Ok(mut second)) = (std::fs::File::open(built), std::fs::File::open(live)) else {
        return false;
    };
    let (Ok(first_size), Ok(second_size)) = (first.metadata().map(length_of), second.metadata().map(length_of)) else {
        return false;
    };
    if first_size != second_size {
        return false;
    }
    let kept: u64 = first_size.min(TAIL);
    let mut tails: [Vec<u8>; 2] = [vec![0; kept as usize], vec![0; kept as usize]];
    let [first_tail, second_tail] = &mut tails;
    let tail_start: u64 = first_size - kept;
    if read_tail(&mut first, first_tail, tail_start).is_err() || read_tail(&mut second, second_tail, tail_start).is_err() || first_tail != second_tail {
        return false;
    }
    if first_tail.iter().any(is_nonzero) {
        return true;
    }
    // A skipped checksum: compare every byte.
    return match (std::fs::read(built), std::fs::read(live)) {
        (Ok(left), Ok(right)) => left == right,
        _ => false,
    };
}
