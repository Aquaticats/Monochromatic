//! What:
//!  The registration half of the ZFS add route:
//!  the add's claim and durable binding record,
//!  the two snapshot inventories,
//!  the managed Git lock's sentence and comparison,
//!  the finder that recognises what a kill inside Git's own `worktree add` left,
//!  the recipe that clears an incomplete registration,
//!  the add's read of a removal's plan,
//!  and the one command that is true for a state a resume meets.
//! Why:
//!  Plan step 10 builds the add route from registration to verified fill,
//!  and the two designs the project accepted on 2026-11-23 own its records and its lock:
//!  `doc/planning/cli-git-zfs-add-binding-record-design.md` and
//!  `doc/planning/cli-git-zfs-add-time-git-lock-design.md`.
//!  This folder carries the parts of that build that need no pool:
//!  every record's grammar,
//!  its publication and its refusals,
//!  the words the lock holds,
//!  and the states a resume tells apart.
//!  The route's pool steps,
//!  the snapshot,
//!  the clone,
//!  the mount and the fill,
//!  are the route's own files beside this folder.
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `binding_words.rs`:
//!   the header,
//!   the eight phases,
//!   the seven settle reasons,
//!   the three request forms and the keys' order.
//! - `registration_home.rs`:
//!   the folder,
//!   the four name forms and the plan folder.
//! - `registration_gitdir.rs`:
//!   the path one registration's `gitdir` file names.
//! - `add_state_commands.rs`:
//!   the command each state names.
//! - `binding_bound.rs`:
//!   the byte bounds computed from the kernel limits they rest on.
//! - `binding_encoding.rs`:
//!   the four encodings a record writes.
//! - `binding_inventory.rs`:
//!   the two inventories' grammar and reading.
//! - `binding_record.rs`:
//!   the binding's fields,
//!   which phase proves each,
//!   and its bytes.
//! - `lock_words.rs`:
//!   the lock's sentence and the comparison that proves it the wrapper's.
//! - `record_refusal.rs`:
//!   why bytes are not a record,
//!   and the sentence a person reads.
//! - `binding_field_set.rs`:
//!   the fields one binding parse has read so far,
//!   each still optional until the whole record answers.
//! - `binding_field_set_line.rs`:
//!   one line's key and value recorded into those fields,
//!   or the value the field cannot hold.
//! - `binding_field_set_record.rs`:
//!   the record the fields describe,
//!   and the check that every field its phase needs is present.
//! - `binding_inventory_parse.rs`:
//!   reading one inventory's bytes as the file its name says it is.
//! - `binding_parse.rs`:
//!   reading one binding's bytes,
//!   field by field and in order.
//! - `registration_finder.rs`:
//!   Git's sanitiser,
//!   the candidate names and the rows a kill leaves.
//! - `plan_folder_read.rs`:
//!   the plan that names the add's clone,
//!   and the fields its stop reads.
//! - `record_publication.rs`:
//!   the flushed replacing write,
//!   the write-once inventories and the reading that refuses a foreign file.
//! - `registration_removals.rs`:
//!   the removals below held descriptors,
//!   the names one run works from,
//!   and why a run stopped.
//! - `registration_recipe.rs`:
//!   the eight steps in their order and the one lock completion.
//! - `registration_resume.rs`:
//!   what the resume does with each row and with a listing that holds none.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! import { parseBinding, lockWords, findRegistrations, commandFor } from '@monochromatic-dev/git-policy-cli/worktree-clone-binding-record';
//! ```

pub mod add_state_commands;
pub mod binding_bound;
pub mod binding_encoding;
pub mod binding_field_set;
pub mod binding_field_set_line;
pub mod binding_field_set_record;
pub mod binding_inventory;
pub mod binding_inventory_parse;
pub mod binding_parse;
pub mod binding_record;
pub mod binding_words;
pub mod lock_words;
pub mod plan_folder_read;
pub mod record_publication;
pub mod record_refusal;
pub mod registration_finder;
pub(crate) mod registration_gitdir;
pub mod registration_home;
pub mod registration_recipe;
pub mod registration_removals;
pub mod registration_resume;

pub use add_state_commands::{
    ClonePresence,
    NextCommand,
    PlanHeadWord,
    StateFacts,
    THE_REMOVALS_WORDS,
    command_for,
};
pub use binding_bound::{
    BINDING_MAXIMUM_BYTES,
    BINDING_MAXIMUM_LINE_BYTES,
    INVENTORY_ENTRY_LINE_MAXIMUM_BYTES,
    INVENTORY_FIXED_BYTES,
};
pub use binding_encoding::{
    FolderIdentity,
    Request,
    bytes_as_hexadecimal,
    path_as_hexadecimal,
};
pub use binding_inventory::{
    EntryKind,
    InventoryEntry,
    InventoryKind,
    InventoryRecord,
};
pub use binding_inventory_parse::parse_inventory;
pub use binding_parse::{
    parse_binding,
    slot_hexadecimal,
};
pub use binding_record::{
    BindingRecord,
    required_keys_for,
};
pub use binding_words::{
    BINDING_HEADER,
    BINDING_KEYS_IN_ORDER,
    INVENTORY_HEADER,
    Phase,
    RequestForm,
    SettleReason,
};
pub use lock_words::{
    LOCK_WORDS_MAXIMUM_BYTES,
    LOCK_WORDS_PREFIX,
    LOCK_WORDS_SUFFIX,
    LockStanding,
    lock_file_bytes,
    lock_standing_of,
    lock_words,
};
pub use plan_folder_read::{
    APlanThatNamesTheClone,
    PlanUnreadable,
    is_canonical_guid_name,
    the_plan_naming_the_clone,
};
pub use record_publication::{
    RecordRefusalAtTheFile,
    publish_inventory,
    read_binding,
    read_inventory,
    write_binding,
};
pub use record_refusal::RecordRefusal;
pub use registration_finder::{
    EntryFacts,
    EntryRow,
    candidate_names,
    git_sanitized_name,
};
pub use registration_home::{
    BINDING_SUFFIX,
    CLONE_INVENTORY_SUFFIX,
    GRAMMAR_VERSION_FOLDER,
    ORIGIN_INVENTORY_SUFFIX,
    RecordKind,
    RecordName,
    WORKTREE_CLONES_FOLDER,
    binding_name,
    claim_prefix,
    clone_inventory_name,
    home_folder,
    open_or_create_home,
    origin_inventory_name,
    plans_folder,
    record_name_of,
    slot_of,
};
pub use registration_recipe::{
    REMOVAL_STEPS,
    RecipeSeams,
    RemovalStep,
    clear_registration,
    complete_a_cut_short_lock,
};
pub use registration_removals::{
    RecipeRefusal,
    ToClear,
};
pub use registration_resume::{
    EntryDecision,
    Finding,
    ListingDecision,
    decide,
    find_registrations,
};
