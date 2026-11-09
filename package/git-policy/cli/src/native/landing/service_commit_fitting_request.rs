//! What:
//!  Whether the request the embedded route would send fits the one line the service reads,
//! and the fallback a commit takes when it does not.
//! Why:
//!   The service refuses a line over `MAX_LINE_BYTES` as too long,
//! and "Commits should never fail":
//! a commit whose changed paths would not fit is built by Git instead,
//! which is the route it took before a changed path carried its bytes.
//! A name that is not text carries its bytes beside the readable spelling,
//! so the most changes at the path length that ceiling was measured for no longer fit one line.
//! The decision is split from `service_commit.rs`,
//! which holds the loop that uses it and is at its line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! embedded = embeddedThatFits(embedded, ready, inputs, id, tree); // undefined when Git must build the commit
//! ```

use crate::service_commit_input_notes_and_embedded_commit::Embedded;
use crate::service_commit_requests_and_landing_report::request_for_embedded;
use crate::service_prepare::{NativeInputs, Ready};
use crate::service_protocol::commit_request_fits;

/// What:
///  The commit the embedded route may ask the service to make:
/// the one `embedded` holds when its request fits the line the service reads,
/// and nothing when Git must build the commit instead.
/// Why:
///   A request over the line is refused as too long and the commit would fail,
/// so the loop gives up the embedded route and takes the one where Git builds the commit,
/// which sends no list of changes and so fits whatever the names are.
/// A commit whose preparation needs Git already names nothing here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const embeddedThatFits = (embedded, ready, inputs, id, tree) => {
///   const made = embedded; if (made === undefined || ready.needsNative) return undefined;
///   return fitsTheLine(ready, inputs, id, made, tree) ? made : undefined;
/// };
/// ```
pub(crate) fn embedded_that_fits(embedded: Option<Embedded>, ready: &Ready, native_inputs: &NativeInputs, id: &str, tree: &Option<String>) -> Option<Embedded> {
    let made: &Embedded = embedded.as_ref()?;
    if ready.needs_native {
        return None;
    }
    return fits_the_line(ready, native_inputs, id, made, tree).then_some(embedded).flatten();
}

/// Whether the request the embedded route would send is one the service accepts.
fn fits_the_line(ready: &Ready, native_inputs: &NativeInputs, id: &str, made: &Embedded, tree: &Option<String>) -> bool {
    return commit_request_fits(&request_for_embedded(ready, native_inputs, id, made, tree));
}
