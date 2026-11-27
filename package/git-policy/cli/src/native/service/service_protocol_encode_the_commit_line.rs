//! What:
//!  The line for one commit request,
//!  and whether that line is one the service accepts.
//! Why:
//!   The preparation asks the same function whether the request fits the ceiling both sides accept,
//!  so the line the service is given and the line that was measured cannot differ.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const commitLine = (request: CommitRequest) => JSON.stringify({ type: 'commit', version: PROTOCOL_VERSION, ...request });
//! ```

use crate::path_bytes_in_records as path_bytes;
use crate::service_protocol_encode_the_fields_of_a_message::{
    change_json,
    cleanup_json,
    programs_json,
    put,
};
use crate::service_protocol_messages_and_name_checks::{
    CommitRequest,
    MAX_LINE_BYTES,
    PROTOCOL_VERSION,
};
use serde_json::{
    Map,
    Value,
};
/// What:
///  The line for one commit request.
/// Why:
///   `encode_client` writes it,
/// and the preparation asks the same function whether the request fits the ceiling both sides accept,
/// so the line the service is given and the line that was measured cannot differ.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const commitLine = (request: CommitRequest) => JSON.stringify({ type: 'commit', version: PROTOCOL_VERSION, ...request });
/// ```
pub fn commit_line(request: &CommitRequest) -> String {
    let mut fields: Map<String, Value> = Map::new();
    put(&mut fields, "type", Value::from("commit"));
    put(&mut fields, "version", Value::from(PROTOCOL_VERSION));
    put(&mut fields, "id", Value::from(request.id.as_str()));
    path_bytes::write_field(&mut fields, "repository", request.repository.as_path());
    path_bytes::write_field(&mut fields, "commonDir", request.common_dir.as_path());
    path_bytes::write_field(&mut fields, "gitDir", request.git_dir.as_path());
    path_bytes::write_field(&mut fields, "index", request.index.as_path());
    path_bytes::write_field(&mut fields, "scratch", request.scratch.as_path());
    if let Some(shadow) = request.shadow.as_deref() {
        path_bytes::write_field(&mut fields, "shadow", shadow);
    }
    // A name that is not text travels as its bytes in the field the reader prefers, and the plain field holds the text
    //  spelling only when text spells every byte of the name (`path_bytes_in_records`).
    if request.reference.is_text() {
        put(
            &mut fields,
            "reference",
            Value::from(request.reference.to_string()),
        );
    }
    fields.insert(
        String::from("referenceHex"),
        Value::from(request.reference.hex()),
    );
    put(&mut fields, "base", Value::from(request.base.as_str()));
    put(
        &mut fields,
        "changes",
        Value::Array(request.changes.iter().map(change_json).collect()),
    );
    put(
        &mut fields,
        "message",
        Value::from(request.message.as_str()),
    );
    put(
        &mut fields,
        "author",
        request.author.as_deref().map_or(Value::Null, Value::from),
    );
    put(
        &mut fields,
        "committer",
        Value::from(request.committer.as_str()),
    );
    put(&mut fields, "amend", Value::from(request.amend));
    put(
        &mut fields,
        "tree",
        request.tree.as_deref().map_or(Value::Null, Value::from),
    );
    put(
        &mut fields,
        "commit",
        request.commit.as_deref().map_or(Value::Null, Value::from),
    );
    put(&mut fields, "captured", Value::from(request.captured));
    put(&mut fields, "holder", Value::from(request.holder.as_str()));
    put(
        &mut fields,
        "programs",
        programs_json(request.programs.as_ref()),
    );
    put(
        &mut fields,
        "cleanup",
        request.cleanup.as_ref().map_or(Value::Null, cleanup_json),
    );
    return Value::Object(fields).to_string();
}
/// What:
///  Whether the line that carries `request` is one the service accepts.
/// Why:
///   The reader refuses a line over `MAX_LINE_BYTES` (`decode_client`),
/// and the client writes the line and one line break,
/// so a request at the ceiling is one whose line and break fit together.
/// A request that does not fit is a commit the wrapper builds itself instead (`service_commit.rs`):
/// "Commits should never fail."
/// The request measured is the request the service is sent,
/// so this command's program search list and transport flag are
///  counted here too:
/// they travel with every request (`service_client::submit_traced`),
/// and a request measured without them could fit alone and be refused as too long once they travelled.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const commitRequestFits = (request: CommitRequest) => Buffer.byteLength(commitLine(withThisProcessPrograms(request))) + 1 <= MAX_LINE_BYTES;
/// ```
pub fn commit_request_fits(request: &CommitRequest) -> bool {
    let mut delivered: CommitRequest = request.clone();
    // A request that already carries a list keeps it, so a caller that gave one is not overruled by this process's environment.
    if delivered.programs.is_none() {
        delivered.programs = Some(crate::service_push_programs_held_in_memory::of_this_process());
    }
    // The reader counts the line break it reads as part of the line, so the break is part of the budget here too.
    return commit_line(&delivered).len() + 1 <= MAX_LINE_BYTES;
}
