//! What:
//!  Raw and generated `git cat-file --batch` replies through the reply-reading invariants.
//! Why:
//!  Reply content is file content,
//!  which is arbitrary and may imitate a reply,
//!  so
//!      framing is checked over truncated,
//!  oversized,
//!  malformed and missing-object streams.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! fuzz(bytes, data => { checkBatchReply(...requestAndStream(data)); checkGeneratedReply(data); });
//! ```
#![no_main]

/// Import the libFuzzer entry macro and the sidecar's shared generators and checks.
use git_policy_cli_fuzz::batch::{check_batch_reply, check_generated_reply, request_and_stream};
use libfuzzer_sys::fuzz_target;

// Run both views of every input: bytes split at the first line feed into a request and a
// stream, and bytes mapped to a built reply with a known outcome.
// The `|data: &[u8]|` form is the macro's input grammar, not a closure.
fuzz_target!(|data: &[u8]| {
    let (request, stream): (&[u8], &[u8]) = request_and_stream(data);
    check_batch_reply(request, stream);
    check_generated_reply(data);
});
