// Adapted from `src/nvlist.rs` of `Ukko-Ylijumala/zfsdev-rs` at commit
// 4559ffc711; see PROVENANCE.md. Copyright (c) 2026 Mikko Tanner,
// MIT OR Apache-2.0.
//! What:
//!  The pairs a caller appends to a list:
//! `push`,
//! which replaces an earlier pair of the same name,
//! and one `add_*` for each value a request carries.
//! Why:
//!  The request packers build their input lists through these calls,
//! and every one of them goes through `push`,
//! so the `NV_UNIQUE_NAME` rule that a name appears once is stated in one
//!  place.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! list.addU64('guid', 1);
//! list.addStr('name', 'pool/dataset');
//! ```

/// The flag that makes a name appear once.
use super::data_type::NV_UNIQUE_NAME;
/// The list,
/// the pair and the value the builders append.
use super::value::{
    NvData,
    NvList,
    NvPair,
};

/// The builders of `NvList`.
impl NvList {
    /// Append a pair,
    /// replacing an earlier pair of the same name when
    /// `NV_UNIQUE_NAME` is set,
    /// as the kernel's own add does.
    pub fn push(
        &mut self,
        name: impl Into<String>,
        data: NvData,
    ) -> &mut NvList {
        let pair_name: String = name.into();
        if self.nvflag & NV_UNIQUE_NAME != 0 {
            self.pairs.retain(|pair| {
                return pair.name != pair_name;
            });
        }
        self.pairs.push(NvPair {
            name: pair_name,
            data,
        });
        return self;
    }

    /// Append a `u64` pair.
    pub fn add_u64(
        &mut self,
        name: impl Into<String>,
        value: u64,
    ) -> &mut NvList {
        return self.push(
            name,
            NvData::Uint64 {
                value,
            },
        );
    }

    /// Append a string pair.
    pub fn add_str(
        &mut self,
        name: impl Into<String>,
        value: impl Into<String>,
    ) -> &mut NvList {
        return self.push(
            name,
            NvData::Str {
                value: value.into(),
            },
        );
    }

    /// Append a `DATA_TYPE_BOOLEAN_VALUE` pair.
    pub fn add_boolean(
        &mut self,
        name: impl Into<String>,
        value: bool,
    ) -> &mut NvList {
        return self.push(
            name,
            NvData::Boolean {
                value,
            },
        );
    }

    /// Append a valueless `DATA_TYPE_BOOLEAN` flag.
    pub fn add_bool_flag(
        &mut self,
        name: impl Into<String>,
    ) -> &mut NvList {
        return self.push(name, NvData::BooleanFlag);
    }

    /// Append an `i32` pair.
    pub fn add_i32(
        &mut self,
        name: impl Into<String>,
        value: i32,
    ) -> &mut NvList {
        return self.push(
            name,
            NvData::Int32 {
                value,
            },
        );
    }

    /// Append a `u64` array pair.
    pub fn add_u64_array(
        &mut self,
        name: impl Into<String>,
        value: Vec<u64>,
    ) -> &mut NvList {
        return self.push(
            name,
            NvData::Uint64Array {
                value,
            },
        );
    }

    /// Append a string array pair.
    pub fn add_str_array(
        &mut self,
        name: impl Into<String>,
        value: Vec<String>,
    ) -> &mut NvList {
        return self.push(
            name,
            NvData::StrArray {
                value,
            },
        );
    }

    /// Append an embedded list pair.
    pub fn add_nvlist(
        &mut self,
        name: impl Into<String>,
        value: NvList,
    ) -> &mut NvList {
        return self.push(
            name,
            NvData::List {
                value,
            },
        );
    }
}
