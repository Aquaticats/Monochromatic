//! What:
//!  Whether a new index file holds the bytes of the live one.
//! Why:
//!  The release of the index lock installs a copy only when it differs from the live index, so an unchanged index keeps its inode
//!  and its access list and costs no flush or rename. The module is split from `service_index_hold.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (!sameIndex(built, live)) { flush(built); rename(built, live); }
//! ```

use std::path::Path;

/// What:
///  Whether the file `built` holds the same bytes as the live index file `live`.
/// Why:
///   An index ends with a checksum of all it holds, so files of one size with the same tail are the same file, and the common
///  case is settled by reading the tails only. A tail of zeros is what a skipped checksum looks like and proves nothing, so then
///  the files are compared in full. A file that cannot be read is taken as different, which installs the copy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// sameIndex(built, live) // true: nothing to install
/// ```
pub(super) fn same_index(built: &Path, live: &Path) -> bool {
    use std::io::{Read as _, Seek as _, SeekFrom};
    const TAIL: u64 = 32;
    let (Ok(mut first), Ok(mut second)) = (std::fs::File::open(built), std::fs::File::open(live)) else {
        return false;
    };
    let (Ok(first_size), Ok(second_size)) = (first.metadata().map(|found| { return found.len(); }), second.metadata().map(|found| { return found.len(); })) else {
        return false;
    };
    if first_size != second_size {
        return false;
    }
    let kept: u64 = first_size.min(TAIL);
    let mut tails: [Vec<u8>; 2] = [vec![0; kept as usize], vec![0; kept as usize]];
    let read_tail = |file: &mut std::fs::File, tail: &mut Vec<u8>| -> std::io::Result<()> {
        file.seek(SeekFrom::Start(first_size - kept))?;
        return file.read_exact(tail.as_mut_slice());
    };
    let [first_tail, second_tail] = &mut tails;
    if read_tail(&mut first, first_tail).is_err() || read_tail(&mut second, second_tail).is_err() || first_tail != second_tail {
        return false;
    }
    if first_tail.iter().any(|byte| { return *byte != 0; }) {
        return true;
    }
    // A skipped checksum: compare every byte.
    return match (std::fs::read(built), std::fs::read(live)) {
        (Ok(left), Ok(right)) => left == right,
        _ => false,
    };
}
