// Adapted from `src/nvlist.rs` of `Ukko-Ylijumala/zfsdev-rs` at commit
// 4559ffc711; see PROVENANCE.md. Copyright (c) 2026 Mikko Tanner,
// MIT OR Apache-2.0.
//! What:
//!  A decoded name-value list:
//! its pairs,
//! the typed value of each pair,
//! and
//!  the lookups and builders the request packers use.
//! Why:
//!  The kernel's replies are lists of pairs with no declared schema,
//! so the
//!  crate keeps every value it understands and the raw bytes of a type it does
//!  not,
//! rather than dropping what it cannot name.
//!  Building and reading go through one pair type so the encoder and the
//!  decoder cannot disagree about what a pair holds.
//!  The list is split across siblings because this file is over the line limit:
//!  this one holds the three types and the lookups that read a list,
//!  `value_data_type.rs` answers the wire type number of each value,
//!  and `list_builders.rs` holds the calls that append a pair.
//!
//! `NV_UNIQUE_NAME` is the flag every list this crate builds carries,
//! matching
//! what the kernel expects on input:
//! adding a name twice replaces the earlier
//! pair instead of producing a list the kernel would refuse.

/// The flag every list this crate builds carries.
use super::data_type::NV_UNIQUE_NAME;

/// The value of one pair,
/// one variant per `data_type_t` this crate names.
#[derive(Debug, Clone, PartialEq)]
pub enum NvData {
    /// `DATA_TYPE_BOOLEAN`:
    /// a valueless flag whose presence is the information.
    BooleanFlag,
    /// `DATA_TYPE_BOOLEAN_VALUE`:
    /// a 32-bit boolean.
    Boolean {
        /// The boolean the kernel stored.
        value: bool,
    },
    /// `DATA_TYPE_BYTE`.
    Byte {
        /// The byte the kernel stored.
        value: u8,
    },
    /// `DATA_TYPE_INT8`.
    Int8 {
        /// The signed byte the kernel stored.
        value: i8,
    },
    /// `DATA_TYPE_UINT8`.
    Uint8 {
        /// The unsigned byte the kernel stored.
        value: u8,
    },
    /// `DATA_TYPE_INT16`.
    Int16 {
        /// The 16-bit signed integer the kernel stored.
        value: i16,
    },
    /// `DATA_TYPE_UINT16`.
    Uint16 {
        /// The 16-bit unsigned integer the kernel stored.
        value: u16,
    },
    /// `DATA_TYPE_INT32`.
    Int32 {
        /// The 32-bit signed integer the kernel stored.
        value: i32,
    },
    /// `DATA_TYPE_UINT32`.
    Uint32 {
        /// The 32-bit unsigned integer the kernel stored.
        value: u32,
    },
    /// `DATA_TYPE_INT64`.
    Int64 {
        /// The 64-bit signed integer the kernel stored.
        value: i64,
    },
    /// `DATA_TYPE_UINT64`.
    Uint64 {
        /// The 64-bit unsigned integer the kernel stored.
        value: u64,
    },
    /// `DATA_TYPE_HRTIME`:
    /// a 64-bit nanosecond count.
    HrTime {
        /// The nanosecond count the kernel stored.
        value: i64,
    },
    /// `DATA_TYPE_DOUBLE`.
    Double {
        /// The double the kernel stored.
        value: f64,
    },
    /// `DATA_TYPE_STRING`.
    Str {
        /// The text the kernel stored.
        value: String,
    },
    /// `DATA_TYPE_BYTE_ARRAY`.
    ByteArray {
        /// The bytes the kernel stored.
        value: Vec<u8>,
    },
    /// `DATA_TYPE_INT8_ARRAY`.
    Int8Array {
        /// The signed bytes the kernel stored.
        value: Vec<i8>,
    },
    /// `DATA_TYPE_UINT8_ARRAY`.
    Uint8Array {
        /// The unsigned bytes the kernel stored.
        value: Vec<u8>,
    },
    /// `DATA_TYPE_INT16_ARRAY`.
    Int16Array {
        /// The 16-bit signed integers the kernel stored.
        value: Vec<i16>,
    },
    /// `DATA_TYPE_UINT16_ARRAY`.
    Uint16Array {
        /// The 16-bit unsigned integers the kernel stored.
        value: Vec<u16>,
    },
    /// `DATA_TYPE_INT32_ARRAY`.
    Int32Array {
        /// The 32-bit signed integers the kernel stored.
        value: Vec<i32>,
    },
    /// `DATA_TYPE_UINT32_ARRAY`.
    Uint32Array {
        /// The 32-bit unsigned integers the kernel stored.
        value: Vec<u32>,
    },
    /// `DATA_TYPE_INT64_ARRAY`.
    Int64Array {
        /// The 64-bit signed integers the kernel stored.
        value: Vec<i64>,
    },
    /// `DATA_TYPE_UINT64_ARRAY`.
    Uint64Array {
        /// The 64-bit unsigned integers the kernel stored.
        value: Vec<u64>,
    },
    /// `DATA_TYPE_BOOLEAN_ARRAY`:
    /// 4-byte booleans on the wire.
    BooleanArray {
        /// The booleans the kernel stored.
        value: Vec<bool>,
    },
    /// `DATA_TYPE_STRING_ARRAY`.
    StrArray {
        /// The strings the kernel stored.
        value: Vec<String>,
    },
    /// `DATA_TYPE_NVLIST`.
    List {
        /// The embedded list the kernel stored.
        value: NvList,
    },
    /// `DATA_TYPE_NVLIST_ARRAY`.
    ListArray {
        /// The embedded lists the kernel stored.
        value: Vec<NvList>,
    },
    /// A `data_type_t` this crate does not name;
    /// its element count and bytes are kept as read,
    /// so a re-pack is faithful.
    Unknown {
        /// The type number the pair declared.
        dtype: i32,
        /// The pair's `nvp_value_elem`,
        /// the count the type's encoder wrote.
        elements: usize,
        /// The pair's value bytes,
        /// unpadded.
        raw: Vec<u8>,
    },
}

/// One name and its value.
#[derive(Debug, Clone, PartialEq)]
pub struct NvPair {
    /// The pair's name,
    /// unique within its list when the flag says so.
    pub name: String,
    /// The pair's value.
    pub data: NvData,
}

/// A name-value list,
/// decoded or being built.
#[derive(Debug, Clone, PartialEq)]
pub struct NvList {
    /// The wire version,
    /// always zero for a list this crate reads or writes.
    pub version: i32,
    /// `NV_UNIQUE_NAME` and future flags.
    pub nvflag: u32,
    /// The pairs,
    /// in wire order.
    pub pairs: Vec<NvPair>,
}

/// The value a caller gets without choosing one.
impl Default for NvList {
    /// An empty list carrying `NV_UNIQUE_NAME`,
    /// like [`NvList::new`].
    fn default() -> NvList {
        return NvList::new();
    }
}

/// The lookups of `NvList`:
/// the pairs in wire order and the typed value of one name.
impl NvList {
    /// An empty list carrying `NV_UNIQUE_NAME`.
    pub fn new() -> NvList {
        return NvList {
            version: 0,
            nvflag: NV_UNIQUE_NAME,
            pairs: Vec::new(),
        };
    }

    /// The pairs in wire order.
    pub fn iter(&self) -> impl Iterator<Item = &NvPair> {
        return self.pairs.iter();
    }

    /// The value of the last pair named `name`,
    /// which is the one the kernel's
    /// `NV_UNIQUE_NAME` decode keeps when a list carries the name twice.
    pub fn get(
        &self,
        name: &str,
    ) -> Option<&NvData> {
        return self
            .pairs
            .iter()
            .rfind(|pair| {
                return pair.name == name;
            })
            .map(|pair| {
                return &pair.data;
            });
    }

    /// The `u64` value of a pair named `name`,
    /// if that is what it holds.
    pub fn get_u64(
        &self,
        name: &str,
    ) -> Option<u64> {
        return match self.get(name)? {
            NvData::Uint64 {
                value,
            } => Some(*value),
            _ => None,
        };
    }

    /// The string value of a pair named `name`,
    /// if that is what it holds.
    pub fn get_str(
        &self,
        name: &str,
    ) -> Option<&str> {
        return match self.get(name)? {
            NvData::Str {
                value,
            } => Some(value),
            _ => None,
        };
    }

    /// The embedded list of a pair named `name`,
    /// if that is what it holds.
    pub fn get_list(
        &self,
        name: &str,
    ) -> Option<&NvList> {
        return match self.get(name)? {
            NvData::List {
                value,
            } => Some(value),
            _ => None,
        };
    }

    /// The embedded list array of a pair named `name`,
    /// if that is what it holds.
    pub fn get_list_array(
        &self,
        name: &str,
    ) -> Option<&[NvList]> {
        return match self.get(name)? {
            NvData::ListArray {
                value,
            } => Some(value),
            _ => None,
        };
    }
}

/// The device-free tests of this module.
#[cfg(test)]
#[path = "value_tests.rs"]
mod tests;
