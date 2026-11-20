//! What:
//!  The request numbers this crate can send,
//! and the range's two endpoints,
//! as the
//!  table the comparison asks for.
//! Why:
//!   The numbers come from the crate's own operation list,
//! so a request added to it
//!  cannot go uncompared,
//! and the two endpoints come from the header the design
//!  pins.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const wanted = expectedEnumerators(); // [{ name, value }]
//! ```

/// The crate's request list.
use crate::requests::Operation;

/// `ZFS_IOC_FIRST`,
/// `('Z' << 8)` in `include/sys/fs/zfs.h`,
/// where the request range starts.
pub const FIRST_REQUEST: i64 = 0x5a00;
/// `ZFS_IOC_LAST`,
/// where the request range ends.
pub const LAST_REQUEST: i64 = 0x5a89;
/// The `ZFS_IOC_` prefix every enumerator of `enum zfs_ioc` carries.
pub const REQUEST_PREFIX: &str = "ZFS_IOC_";
/// The name of the enumeration the request numbers live in.
pub const REQUEST_ENUM: &str = "zfs_ioc";
/// The byte size one enumerator of `enum zfs_ioc` is declared in.
pub const REQUEST_ENUM_BYTES: u32 = 4;

/// What:
///  One enumerator the module must declare with the value this crate sends.
/// Why:
///   A request number sent under another value arrives at another handler,
/// so every
///  operation this crate can send is compared,
/// and the range's two endpoints with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ExpectedEnumerator = { name: string; value: number };
/// ```
pub struct ExpectedEnumerator {
    /// The enumerator's name.
    pub name: String,
    /// The value it must carry.
    pub value: i64,
}

/// Every request number this crate can send,
/// the range's two endpoints last.
pub fn expected_enumerators() -> Vec<ExpectedEnumerator> {
    let mut wanted: Vec<ExpectedEnumerator> = Vec::<ExpectedEnumerator>::with_capacity(
        Operation::ALL.len().saturating_add(2),
    );
    for operation in Operation::ALL {
        wanted.push(ExpectedEnumerator {
            name: format!("{REQUEST_PREFIX}{}", operation.as_str()),
            value: operation.code() as i64,
        });
    }
    wanted.push(ExpectedEnumerator {
        name: format!("{REQUEST_PREFIX}FIRST"),
        value: FIRST_REQUEST,
    });
    wanted.push(ExpectedEnumerator {
        name: format!("{REQUEST_PREFIX}LAST"),
        value: LAST_REQUEST,
    });
    return wanted;
}
