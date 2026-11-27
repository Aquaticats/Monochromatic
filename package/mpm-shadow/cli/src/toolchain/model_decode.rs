//! What:
//! Decoding the plan and lock files' JSON into the model:
//! one decoder for a declaration,
//! one for a lock entry,
//! each naming the path of every wrong field.
//! Why:
//! The model file stays small and holds only types;
//! this file holds the checks that make the types true.
//! The checks are one file per thing decoded,
//! named for what each holds:
//! a request and the postinstall step it may carry (`a_request_and_its_postinstall`),
//! the options a route takes (`the_options_a_route_takes`),
//! a declaration (`a_declaration`),
//! the artifacts and their digests (`the_artifacts_and_their_digests`),
//! and a lock entry (`a_lock_entry`).
//! A caller imports from this file and never from a part.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // model-decode.ts
//! export function decodePlanEntry(id: string, value: unknown, path: string): Declaration
//! export function decodeLockEntry(id: string, value: unknown, path: string): LockedTool
//! export * from './model_decode/a_request_and_its_postinstall.ts';
//! export * from './model_decode/the_options_a_route_takes.ts';
//! export * from './model_decode/a_declaration.ts';
//! export * from './model_decode/the_artifacts_and_their_digests.ts';
//! export * from './model_decode/a_lock_entry.ts';
//! ```

/// The fields a plan entry and a lock entry share.
mod a_declaration;
/// One lock entry.
mod a_lock_entry;
/// The request text and the postinstall step an entry may carry.
mod a_request_and_its_postinstall;
/// The artifact rows of one version.
mod the_artifacts_and_their_digests;
/// The options a route takes.
mod the_options_a_route_takes;

/// Re-exports the decoder of one plan entry.
pub use self::a_declaration::decode_plan_entry;
/// Re-exports the decoder of one lock entry.
pub use self::a_lock_entry::decode_lock_entry;
/// Re-exports the check that a digest is 64 lowercase hex characters.
pub use self::the_artifacts_and_their_digests::is_sha256_hex;
