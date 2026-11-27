//! What:
//!  Every clone whose request no receipt of this boot answers.
//! Why:
//!  A receipt names the nonce it answers and the boot it was filed in,
//!  so a request with no matching receipt,
//!  or one answered in another boot,
//!  is the request nobody has done yet,
//!  and a clone that asked to be mounted or unmounted and got no answer is the
//!  one state a person can act on.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = checkUnresolvedRequests(requests, liveBoot); // pass | finding('request')
//! ```

/// One check's answer,
/// and the two constructors this check builds it from.
use super::check_outcome::{
    CheckOutcome,
    finding,
    pass,
};
//region The unresolved requests

/// What:
///  One clone's durable request and the receipt that answers it,
///  where either is there.
/// Why:
///  A receipt names the nonce it answers and the boot it was filed in,
///  so a request with no matching receipt,
///  or one answered in another boot,
///  is the request nobody has done yet.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RequestFacts = { clone: string; intentNonce?: string; receiptNonce?: string; receiptBoot?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequestFacts {
    /// The clone's dataset name.
    pub clone: String,
    /// The nonce its local request names,
    /// or nothing when it asks for no work.
    pub intent_nonce: Option<String>,
    /// The nonce the newest receipt answers,
    /// or nothing when there is no receipt.
    pub receipt_nonce: Option<String>,
    /// The boot the newest receipt belongs to.
    pub receipt_boot: Option<String>,
}

/// What:
///  Every clone whose request no receipt of this boot answers.
/// Why:
///  A clone that asked to be mounted or unmounted and got no answer is the one
///  state a person can act on,
///  while a request answered in another boot is not answered here,
///  and the report names each clone rather than a count.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = check_unresolved_requests(requests, liveBoot); // pass | finding('request')
/// ```
pub fn check_unresolved_requests(
    requests: &[RequestFacts],
    live_boot: &str,
) -> CheckOutcome {
    let mut unresolved: Vec<String> = Vec::new();
    let mut asked: usize = 0;
    for request in requests {
        let Some(nonce) = &request.intent_nonce else {
            continue;
        };
        asked += 1;
        let answered: bool = request.receipt_nonce.as_ref() == Some(nonce)
            && request.receipt_boot.as_deref() == Some(live_boot);
        if !answered {
            unresolved.push(format!("`{}` asks under nonce {nonce}", request.clone));
        }
    }
    if !unresolved.is_empty() {
        return finding(
            "request",
            format!(
                "{} clone request(s) have no receipt from this boot: {}",
                unresolved.len(),
                unresolved.join(", ")
            ),
        );
    }
    return pass(format!(
        "{asked} clone request(s) are each answered by a receipt of this boot"
    ));
}

//endregion
