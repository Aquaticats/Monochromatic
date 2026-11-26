//! What:
//!  The one adapter that maps `btf-rs` 2.0.1 onto the reader-neutral
//! declaration:
//!  the two files the gate read,
//! parsed by the crate the project adopted,
//! and the walk
//!  that compares one module's declaration through them.
//! Why:
//!  The project adopted `btf-rs` 2.0.1 on 2026-11-20
//! (`doc/decision/cli-git-zfs-runtime-layout-checker.md`,
//!  "The reader adopted:
//!  `btf-rs` 2.0.1"),
//! so this file is the reader behind
//!  `declaration.rs` and the only file a later reader replaces.
//!  `btf-rs` resolves a named type,
//! a chained type and a record's members and
//!  enumerators,
//! and it leaves one piece of arithmetic to this crate:
//! an array's byte
//!  size,
//! which is the element's own size times the count this file multiplies out.
//!  The crate has no section validation,
//! so `btf_header.rs` keeps the pre-scan that
//!  refuses a header or a section list before `btf-rs` sees the bytes
//! (the vet report's integration note,
//!  `doc/audit/tech-btf-reader-for-the-zfs-layout-checker-vet-2026-11-20.md`).
//!  The release's own masks lag its main branch on the widened fields:
//! 2.0.1 reads a
//!  type entry's kind in five bits and its member count in sixteen,
//! and the pre-scan
//!  reads the header and the section list and no type entry,
//! so a kernel that
//!  widens either field is misread where the high bits only extend a kind,
//! and refused
//!  only when that reader's own parse fails.
//!  No kernel on record writes one,
//! and the
//!  adoption record names the lag as a revisit trigger
//! (`doc/decision/cli-git-zfs-runtime-layout-checker.md`,
//!  "Exit,
//!  rollback and revisit triggers").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const base = Btf.from_bytes(baseBytes); const module = Btf.from_split_bytes(moduleBytes, base);
//! compare(declarationOf(module)); compareRequests(declarationOf(module));
//! ```

/// The walk that compares the declared layout.
use super::comparison::compare;
/// The walk that compares the request numbers.
use super::comparison_requests::compare_requests;
/// The reader-neutral vocabulary this adapter fills.
use super::declaration::{
    ContainerKind,
    Declaration,
    DeclaredMember,
    DeclaredType,
};
/// The one difference value either walk returns.
use super::mismatch::Mismatch;
/// The refusal value and the step one difference is reported as.
use super::refusal::{
    Refusal,
    Step,
    refusal_at,
};
/// The adopted reader.
use btf_rs::{
    Btf,
    BtfType,
    Member,
    Struct,
    Type,
};
/// The borrowed path type a refusal names.
use std::path::Path;

/// The most aliases one handle resolves through before this adapter answers
/// nothing,
/// so a typedef that reaches itself ends in a refusal rather than a loop.
/// The same budget bounds the array elements one handle descends through,
/// so an array whose element is itself ends the same way.
const ALIAS_HOPS: usize = 32;
/// The width of a pointer on this target,
/// which the format states for no pointer.
const POINTER_BYTES: u64 = 8;

/// Read one file pair through `btf-rs` and compare the module's declaration.
///
/// # Errors
/// A refusal naming the step and the file of the read that failed,
/// of the file the reader refused,
/// or of the first difference.
pub fn compare_declared_layout(
    base_bytes: &[u8],
    module_bytes: &[u8],
    base_path: &Path,
    module_path: &Path,
) -> Result<(), Refusal> {
    let base: Btf = Btf::from_bytes(base_bytes).map_err(|error| {
        return refusal_at(
            Step::BaseBtf,
            base_path,
            format!("btf-rs refused it: {error}"),
        );
    })?;
    let module: Btf = Btf::from_split_bytes(module_bytes, &base).map_err(|error| {
        return refusal_at(
            Step::ModuleBtf,
            module_path,
            format!("btf-rs refused it: {error}"),
        );
    })?;
    let declaration: BtfRsDeclaration = BtfRsDeclaration {
        btf: module,
    };
    compare(&declaration).map_err(|mismatch: Mismatch| {
        return refusal_at(mismatch.step, module_path, mismatch.detail);
    })?;
    compare_requests(&declaration).map_err(|mismatch: Mismatch| {
        return refusal_at(mismatch.step, module_path, mismatch.detail);
    })?;
    return Ok(());
}

/// The module's file seen as the reader-neutral declaration.
struct BtfRsDeclaration {
    /// The module's type information,
    /// split over the kernel's own,
    /// which answers the ids and names it does not carry.
    btf: Btf,
}

/// What:
///  The reading one module's file owes the declaration:
/// the type entries it owns,
/// one record's
///  members,
/// one enumeration's values and the array size the reader leaves here.
/// Why:
///   A name or an id the kernel's base file alone carries is not the module's
/// declaration,
/// and the gate compares the module's,
/// so every lookup is answered
///  from the module's own section.
impl BtfRsDeclaration {
    /// One type entry the module's own section holds,
    /// and nothing for a handle only the base file answers.
    fn own_type(
        &self,
        id: u32,
    ) -> Option<Type> {
        let split = self.btf.split()?;
        return split.resolve_type_by_id(id).ok();
    }

    /// One record,
    /// or nothing when any of its members cannot be read.
    fn record(
        &self,
        kind: ContainerKind,
        container: &Struct,
    ) -> Option<DeclaredType> {
        let mut members: Vec<DeclaredMember> =
            Vec::<DeclaredMember>::with_capacity(container.members.len());
        for member in &container.members {
            members.push(self.member(member)?);
        }
        return Some(DeclaredType::Record {
            kind,
            byte_size: container.size() as u64,
            members,
        });
    }

    /// One record member,
    /// or nothing when its name or its type handle cannot be read.
    ///
    /// A member the string section holds no name at makes the whole record
    /// unreadable and never a gap:
    /// a shortened list would compare a record that gained a member as the
    /// record it was,
    /// and the comparison's member count is the only control that would
    /// notice.
    fn member(
        &self,
        member: &Member,
    ) -> Option<DeclaredMember> {
        let name: String = self.btf.resolve_name(member).ok()?;
        let width: Option<u32> = member.bitfield_size();
        let bitfield_bits: Option<u8> = match width {
            Some(bits) if bits > 0 => u8::try_from(bits).ok(),
            _ => None,
        };
        return Some(DeclaredMember {
            name: if name.is_empty() { None } else { Some(name) },
            type_id: member.get_type_id()?,
            bit_offset: member.bit_offset(),
            bitfield_bits,
        });
    }
}

/// The declaration read through the adopted reader.
impl BtfRsDeclaration {
    /// What:
    ///  One entry,
    /// through every alias over it and every array element under it,
    /// inside one hop budget the two chains share.
    /// Why:
    ///   A typedef that reaches itself and an array whose element is itself are
    /// the same walk,
    /// so one budget bounds both and neither ends in a loop:
    /// the answer is nothing,
    /// and the comparison refuses the size it cannot compute.
    fn declared_within(
        &self,
        id: u32,
        budget: usize,
    ) -> Option<DeclaredType> {
        let mut current: u32 = id;
        for hop in 0..budget {
            let left: usize = budget - hop - 1;
            match self.btf.resolve_type_by_id(current).ok()? {
                Type::Typedef(alias) => current = alias.get_type_id()?,
                Type::Volatile(alias) => current = alias.get_type_id()?,
                Type::Const(alias) => current = alias.get_type_id()?,
                Type::Restrict(alias) => current = alias.get_type_id()?,
                Type::TypeTag(alias) => current = alias.get_type_id()?,
                Type::Int(integer) => {
                    return Some(DeclaredType::Integer {
                        byte_size: integer.size() as u64,
                    });
                }
                Type::Ptr(_) => {
                    return Some(DeclaredType::Pointer {
                        byte_size: POINTER_BYTES,
                    });
                }
                Type::Array(array) => {
                    let element: u32 = array.get_type_id()?;
                    return Some(DeclaredType::Array {
                        element,
                        byte_size: self
                            .size_within(element, left)?
                            .saturating_mul(array.len() as u64),
                    });
                }
                Type::Struct(container) => {
                    return self.record(ContainerKind::Struct, &container);
                }
                Type::Union(container) => {
                    return self.record(ContainerKind::Union, &container);
                }
                Type::Enum(enumeration) => {
                    let mut values: Vec<(String, i64)> =
                        Vec::<(String, i64)>::with_capacity(enumeration.members.len());
                    for value in &enumeration.members {
                        values.push((
                            self.btf.resolve_name(value).ok()?,
                            i64::from(value.val() as i32),
                        ));
                    }
                    return Some(DeclaredType::Enumeration {
                        byte_size: enumeration.size() as u64,
                        values,
                    });
                }
                Type::Enum64(enumeration) => {
                    let mut values: Vec<(String, i64)> =
                        Vec::<(String, i64)>::with_capacity(enumeration.members.len());
                    for value in &enumeration.members {
                        values.push((self.btf.resolve_name(value).ok()?, value.val() as i64));
                    }
                    return Some(DeclaredType::Enumeration {
                        byte_size: enumeration.size() as u64,
                        values,
                    });
                }
                Type::Float(float) => {
                    return Some(DeclaredType::Float {
                        byte_size: float.size() as u64,
                    });
                }
                _ => return Some(DeclaredType::Other),
            }
        }
        return None;
    }

    /// The size in bytes one handle describes,
    /// inside the budget the handles above it left.
    fn size_within(
        &self,
        id: u32,
        budget: usize,
    ) -> Option<u64> {
        return self.declared_within(id, budget)?.byte_size();
    }
}

/// The declaration read through the adopted reader.
impl Declaration for BtfRsDeclaration {
    /// One named structure or union the module declares.
    fn find_container(
        &self,
        name: &str,
        kind: ContainerKind,
    ) -> Option<u32> {
        let ids: Vec<u32> = self.btf.resolve_ids_by_name(name).ok()?;
        return ids.into_iter().find(|id| {
            let found: Option<ContainerKind> = self.own_type(*id).as_ref().and_then(container_kind);
            return found == Some(kind);
        });
    }

    /// One named enumeration the module declares.
    fn find_enumeration(
        &self,
        name: &str,
    ) -> Option<u32> {
        let ids: Vec<u32> = self.btf.resolve_ids_by_name(name).ok()?;
        return ids.into_iter().find(|id| {
            return matches!(self.own_type(*id), Some(Type::Enum(_) | Type::Enum64(_)));
        });
    }

    /// One entry,
    /// through every alias over it and every array element under it.
    fn declared(
        &self,
        id: u32,
    ) -> Option<DeclaredType> {
        return self.declared_within(id, ALIAS_HOPS);
    }
}

/// Which of the two container kinds one entry is,
/// and nothing for any other kind.
fn container_kind(entry: &Type) -> Option<ContainerKind> {
    return match entry {
        Type::Struct(_) => Some(ContainerKind::Struct),
        Type::Union(_) => Some(ContainerKind::Union),
        _ => None,
    };
}
