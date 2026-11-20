//! What:
//!  How a name becomes a value a request can carry:
//! the fixed field it is
//!  bound for,
//! the check against that field's bound,
//! and the copy into it.
//! Why:
//!  Every request names a dataset,
//! pool or snapshot,
//! and the kernel reads that
//!  name out of a fixed byte array rather than a length-prefixed string.
//!  Keeping the check and the copy together means no request can write an
//!  unchecked name,
//! and no caller has to remember which field is which size.
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `name_field.rs`:
//!   names the three fixed text fields and the bytes each holds.
//! - `checked_name.rs`:
//!   checks a name against its field and copies it in with its NUL.

/// The name check and the copy into its field.
mod checked_name;
/// The three fixed text fields and their bounds.
mod name_field;

/// The checked name and its refusal.
pub use checked_name::{CheckedName, NameError, NameProblem};
/// The field a name is bound for.
pub use name_field::NameField;
