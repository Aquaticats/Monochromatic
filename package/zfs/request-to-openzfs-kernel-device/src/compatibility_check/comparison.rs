//! What:
//!  The walk that compares the loaded module's declared layout with the
//! expectation,
//!  in a fixed order,
//! and stops at the first difference.
//! Why:
//!  A gate that reported a different difference on every run would be one no
//! test could
//!  pin,
//! and the arms that prove it refuses a changed module assert WHICH finding
//! comes first.
//!  The order is the design's:
//! the container's own size,
//! one pass over its members by name or
//!  by kind and position,
//! the member count,
//! then the interiors of the containers it covers,
//!  in the order their containers stood in the parent.
//!  A named member that meets an anonymous container where it stands,
//! or the
//!  reverse,
//! is a `Type` mismatch,
//! which is what a release that kept the total size and moved
//!  four fields into an anonymous union meets first.
//!  The walk reads the reader-neutral declaration alone,
//! so a reader picked later
//!  fills one adapter and changes no line here.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! compareContainer(ZFS_CMD, idOfZfsCmd);
//! compareRequests();
//! ```

/// The expectation vocabulary.
use super::declaration::{ContainerKind, DeclaredMember, DeclaredType, Declaration};
use super::expectations::{ExpectedContainer, ExpectedMember, ZFS_CMD};
/// The one difference value this walk returns.
use super::mismatch::Mismatch;
/// The step one difference is reported as.
use super::refusal::Step;

/// One container still to walk,
/// with the path its members are read as.
type Descent<'a> = (String, &'a ExpectedContainer, u32);

/// Compare the module's declaration with the expectation,
/// stopping at the first difference.
///
/// # Errors
/// A mismatch naming the step,
/// the field path and both values.
pub fn compare(module: &dyn Declaration) -> Result<(), Mismatch> {
    let found: Option<u32> = module.find_container(ZFS_CMD.name, ZFS_CMD.kind);
    let Some(id): Option<u32> = found else {
        return Err(Mismatch {
            step: Step::Type,
            detail: format!(
                "the loaded module holds no structure named {}, which every request needs",
                ZFS_CMD.name
            ),
        });
    };
    return compare_container(module, &ZFS_CMD, id, ZFS_CMD.name);
}

/// Compare one container's size,
/// its members and the interiors it covers.
///
/// # Errors
/// A mismatch at the first difference the order reaches.
fn compare_container(
    module: &dyn Declaration,
    expected: &ExpectedContainer,
    id: u32,
    path: &str,
) -> Result<(), Mismatch> {
    let Some(declared): Option<DeclaredType> = module.declared(id) else {
        return Err(Mismatch {
            step: Step::Type,
            detail: format!("{path} is named and its type entry could not be read"),
        });
    };
    let DeclaredType::Record {
        kind,
        byte_size,
        members,
    } = declared
    else {
        return Err(not_a_container(path, declared.words(), expected));
    };
    if kind != expected.kind {
        return Err(not_a_container(path, kind.words(), expected));
    }
    if byte_size != u64::from(expected.byte_size) {
        return Err(Mismatch {
            step: Step::Type,
            detail: format!(
                "{path} holds {byte_size} bytes where {} was expected",
                expected.byte_size
            ),
        });
    }
    let mut descent: Vec<Descent<'_>> = Vec::<Descent<'_>>::new();
    for wanted in expected.members {
        let member: &DeclaredMember = match_member(module, &members, wanted, path)?;
        if let Some(interior) = wanted.interior {
            descent.push((field_path(path, wanted), interior, member.type_id));
        }
    }
    if members.len() != expected.members.len() {
        return Err(Mismatch {
            step: Step::Member,
            detail: format!(
                "{path} declares {} members where {} were expected{}",
                members.len(),
                expected.members.len(),
                first_extra(module, &members, expected)
            ),
        });
    }
    for (interior_path, interior, interior_id) in descent {
        compare_container(module, interior, interior_id, &interior_path)?;
    }
    return Ok(());
}

/// The member one expectation matches,
/// checked for its position and its type's size.
///
/// # Errors
/// A mismatch naming the step that differs.
fn match_member<'a>(
    module: &dyn Declaration,
    found: &'a [DeclaredMember],
    wanted: &ExpectedMember,
    path: &str,
) -> Result<&'a DeclaredMember, Mismatch> {
    let index: Option<usize> = covered_index(module, found, wanted);
    let Some(member): Option<&'a DeclaredMember> = index.and_then(|at| {
        return found.get(at);
    }) else {
        return Err(not_found(found, wanted, path));
    };
    let field: String = field_path(path, wanted);
    if member.bit_offset != wanted.bit_offset {
        return Err(Mismatch {
            step: Step::Member,
            detail: format!(
                "{field} sits at {} bits where {} was expected",
                member.bit_offset, wanted.bit_offset
            ),
        });
    }
    if let Some(width) = member.bitfield_bits {
        return Err(Mismatch {
            step: Step::Member,
            detail: format!("{field} is a {width}-bit field, and this crate reads whole members"),
        });
    }
    let Some(size): Option<u64> = module.byte_size(member.type_id) else {
        return Err(Mismatch {
            step: Step::BaseType,
            detail: format!(
                "{field} is of a kind whose size this reader cannot compute, and {} bytes were expected",
                wanted.byte_size
            ),
        });
    };
    if size != u64::from(wanted.byte_size) {
        return Err(Mismatch {
            step: Step::Member,
            detail: format!(
                "{field} holds {size} bytes where {} was expected",
                wanted.byte_size
            ),
        });
    }
    if let Some(element_bytes) = wanted.element_bytes {
        check_array(module, member, element_bytes, &field)?;
    }
    return Ok(member);
}

/// The index one expectation covers:
/// by name,
/// or by kind and bit offset for an anonymous container.
fn covered_index(module: &dyn Declaration, found: &[DeclaredMember], wanted: &ExpectedMember) -> Option<usize> {
    for (index, member) in found.iter().enumerate() {
        if covers(module, wanted, member) {
            return Some(index);
        }
    }
    return None;
}

/// Whether one member of the module is the one an expectation covers.
fn covers(module: &dyn Declaration, wanted: &ExpectedMember, member: &DeclaredMember) -> bool {
    return match (wanted.name, wanted.container) {
        (Some(name), _) => member.name.as_deref() == Some(name),
        (None, Some(kind)) => {
            member.name.is_none()
                && member.bit_offset == wanted.bit_offset
                && module.is_container(member.type_id, kind)
        }
        (None, None) => false,
    };
}

/// Why an expectation found no member,
/// naming the container that stands where it was expected.
fn not_found(found: &[DeclaredMember], wanted: &ExpectedMember, path: &str) -> Mismatch {
    let field: String = field_path(path, wanted);
    for member in found {
        if member.bit_offset != wanted.bit_offset {
            continue;
        }
        let wanted_is_anonymous: bool = wanted.name.is_none();
        if member.name.is_none() != wanted_is_anonymous {
            return Mismatch {
                step: Step::Type,
                detail: format!(
                    "{path} holds {} at {} bits where {field} was expected",
                    member_name(member),
                    member.bit_offset
                ),
            };
        }
    }
    return Mismatch {
        step: Step::Member,
        detail: format!(
            "{field} is missing, and a member at {} bits was expected there",
            wanted.bit_offset
        ),
    };
}

/// Compare an array member's element size with the size this crate assumed.
///
/// # Errors
/// A mismatch naming the element type's size and the expected one.
fn check_array(
    module: &dyn Declaration,
    member: &DeclaredMember,
    element_bytes: u32,
    field: &str,
) -> Result<(), Mismatch> {
    let Some(DeclaredType::Array { element, .. }) = module.declared(member.type_id) else {
        return Err(Mismatch {
            step: Step::Type,
            detail: format!("{field} is not an array of whole elements"),
        });
    };
    if !module.is_base_integer(element) {
        return Err(Mismatch {
            step: Step::Type,
            detail: format!("{field} is an array whose element is no base integer"),
        });
    }
    let Some(size): Option<u64> = module.byte_size(element) else {
        return Err(Mismatch {
            step: Step::BaseType,
            detail: format!("{field} is an array whose element size cannot be computed"),
        });
    };
    if size != u64::from(element_bytes) {
        return Err(Mismatch {
            step: Step::BaseType,
            detail: format!(
                "{field} holds elements of {size} bytes where {element_bytes} was expected"
            ),
        });
    }
    return Ok(());
}

/// Why a container's own entry is not the container the expectation names.
fn not_a_container(path: &str, kind: &str, expected: &ExpectedContainer) -> Mismatch {
    return Mismatch {
        step: Step::Type,
        detail: format!("{path} is {kind} where {} was expected", expected.kind_name()),
    };
}

/// One member's name,
/// or the word for an anonymous container.
fn member_name(member: &DeclaredMember) -> String {
    return match member.name.as_deref() {
        Some(name) => name.to_string(),
        None => "an anonymous container".to_string(),
    };
}

/// The path one member is read as.
fn field_path(path: &str, wanted: &ExpectedMember) -> String {
    return match wanted.name {
        Some(name) => format!("{path}.{name}"),
        None => format!("{path}.{{anonymous {}}}", anonymous_word(wanted)),
    };
}

/// The word one anonymous container's kind is read as.
fn anonymous_word(wanted: &ExpectedMember) -> &'static str {
    return match wanted.container {
        Some(ContainerKind::Union) => "union",
        Some(ContainerKind::Struct) => "structure",
        None => "member",
    };
}

/// The first member the expectation does not cover,
/// as the words a member-count mismatch ends with.
fn first_extra(module: &dyn Declaration, found: &[DeclaredMember], expected: &ExpectedContainer) -> String {
    for member in found {
        let covered: bool = expected.members.iter().any(|wanted| {
            return covers(module, wanted, member);
        });
        if !covered {
            return format!(
                ", and {} at {} bits is one this crate does not expect",
                member_name(member),
                member.bit_offset
            );
        }
    }
    return String::new();
}

/// The two words one expectation's kind is read as.
impl ExpectedContainer {
    /// The container kind in the words a mismatch prints.
    pub fn kind_name(&self) -> &'static str {
        return self.kind.words();
    }
}
