//! What:
//!  The two lookups the comparison asks of one type id:
//! the entry it finally names,
//!  through every typedef and qualifier the module wrote it under,
//! and whether that
//!  entry is of a kind,
//! or is a base integer.
//! Why:
//!  OpenZFS declares `dmu_objset_stats_t` and `zinject_record_t` as typedefs of
//!  structures,
//! and writes array elements under names,
//! so a comparison that read one
//!  entry without resolving would refuse a module whose declaration is right.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const [id, entry] = resolve(module, typeId) ?? [];
//! ```

/// The decoded entries and the kinds one member may be.
use super::btf_entry::Entry;
/// The indexed type entries.
use super::btf_types::TypeInformation;

/// One type id and its entry,
/// through however many names and qualifiers it is written under.
pub(super) fn resolve<'a>(module: &TypeInformation<'a>, id: u32) -> Option<(u32, Entry<'a>)> {
    let mut current: u32 = id;
    for _hop in 0..super::btf_entry::ALIAS_HOPS {
        let entry: Entry<'a> = module.entry(current)?;
        match entry {
            Entry::Alias { target, .. } => current = target,
            _ => return Some((current, entry)),
        }
    }
    return None;
}

/// Whether one type id is of the given kind.
pub(super) fn is_kind(module: &TypeInformation<'_>, id: u32, kind: u8) -> bool {
    return resolve(module, id).is_some_and(|(_id, entry)| {
        return entry.kind() == kind;
    });
}

/// Whether one type id is a base integer or an array of them,
/// through however many names it is written under.
pub(super) fn is_base_integer(module: &TypeInformation<'_>, id: u32) -> bool {
    return resolve(module, id).is_some_and(|(_id, entry)| {
        return matches!(entry, Entry::Integer { .. } | Entry::Array { .. });
    });
}
