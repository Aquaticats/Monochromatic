//! What:
//!  The physical address of a file's first extent,
//! which two files share when one is a clone of the other.
//! Why:
//!  A clone shares data blocks with its source and Git's own checkout never does,
//! so a shared physical extent is the sign that clones served a command
//!  and that a byte copy did not,
//! on btrfs and on ZFS alike.
//! Two rung 2 control files read it,
//! so the one definition lives here rather than twice.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // firstPhysicalExtent('../topic/big.bin') === firstPhysicalExtent('big.bin')
//! ```

use std::os::unix::io::AsRawFd;
use std::path::Path;

/// `FS_IOC_FIEMAP`,
/// `_IOWR('f', 11, struct fiemap)`.
const FS_IOC_FIEMAP: libc::c_ulong = 0xc020_660b;

/// What:
///  The physical address of the first extent of a file,
/// which two files share when one is a clone of the other.
/// Why:
///   `FIEMAP` answers both files' first extent from the same device,
/// so equal addresses are one set of blocks,
/// and a control that reads it needs no
///  privileged command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function firstPhysicalExtent(path: string): bigint;
/// ```
pub fn first_physical_extent(path: &Path) -> u64 {
    #[repr(C)]
    struct Request {
        start: u64,
        length: u64,
        flags: u32,
        mapped: u32,
        count: u32,
        reserved: u32,
        extent: [u8; 56],
    }
    let file: std::fs::File = std::fs::File::open(path).expect("open");
    let mut request: Request = Request { start: 0, length: u64::MAX, flags: 1, mapped: 0, count: 1, reserved: 0, extent: [0; 56] };
    // SAFETY: `request` is a `struct fiemap` followed by room for the one extent it asks for, and the descriptor is open.
    let result: libc::c_int = unsafe { libc::ioctl(file.as_raw_fd(), FS_IOC_FIEMAP, &mut request) };
    assert_eq!(result, 0, "FIEMAP of {}", path.display());
    assert_eq!(request.mapped, 1, "{} has an extent", path.display());
    return u64::from_ne_bytes(request.extent[8..16].try_into().expect("eight bytes"));
}
