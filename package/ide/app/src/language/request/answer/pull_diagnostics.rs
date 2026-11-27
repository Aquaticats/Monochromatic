//! Storing a pull-diagnostics answer:
//!  the full and unchanged reports,
//!  and asking again when the server asked for it.

/// The retry bounds and the two steps that use them.
use super::retries::{
    PULL_RETRY_DELAY,
    record,
    unanswered,
};
/// The ticket the request was sent under.
use crate::language::request::Ticket;
/// The worker whose state these steps change.
use crate::language::worker::Worker;
/// The protocol's data types.
use helix_lsp::lsp;
/// What:
///  `Duration` is a time span.
/// Why:
///  A timed-out pull request is sent again at once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ZERO_WAIT = 0;
/// ```
use std::time::Duration;

/// What:
///  Store a pull-diagnostics answer,
///  or send the request again when the server asks for
///       that or stayed silent past its timeout.
/// Why:
///  The store fences the answer by stamp;
///  a failure whose data says `retriggerRequest` is
///      retried as Helix does (`helix-term/src/handlers/diagnostics.rs`),
///  a bounded number of
///      times.
///  A timed-out request is sent again at once,
///  because the wait already happened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pulled(worker: Worker, ticket: Ticket, result: Report | Error): void
/// ```
pub(super) fn pulled(
    worker: &mut Worker,
    ticket: Ticket,
    result: helix_lsp::Result<lsp::DocumentDiagnosticReportResult>,
) {
    if result.is_ok() {
        record(worker, &ticket, false);
    }
    let store = &mut worker.session.diagnostics;
    match result {
        Ok(lsp::DocumentDiagnosticReportResult::Report(lsp::DocumentDiagnosticReport::Full(
            report,
        ))) => {
            let full = report.full_document_diagnostic_report;
            let accepted = store.pulled_full(
                &ticket.server,
                ticket.encoding,
                ticket.stamp,
                full.result_id,
                full.items,
            );
            tracing::debug!(server = %ticket.server.name, accepted, "stored pulled diagnostics");
        }
        Ok(lsp::DocumentDiagnosticReportResult::Report(
            lsp::DocumentDiagnosticReport::Unchanged(report),
        )) => {
            let identifier = report.unchanged_document_diagnostic_report.result_id;
            let accepted = store.pulled_unchanged(&ticket.server, ticket.stamp, identifier);
            tracing::debug!(server = %ticket.server.name, accepted, "kept pulled diagnostics the server reported unchanged");
        }
        Ok(lsp::DocumentDiagnosticReportResult::Partial(_)) => {
            tracing::debug!(server = %ticket.server.name, "ignored a partial pull-diagnostics answer");
        }
        Err(error) => {
            let wants_retry = match &error {
                // `clone` copies the error data so it can be decoded by value.
                helix_lsp::Error::Rpc(rpc) => match rpc.data.clone() {
                    Some(data) => {
                        match serde_json::from_value::<lsp::DiagnosticServerCancellationData>(data)
                        {
                            Ok(decoded) => decoded.retrigger_request,
                            Err(_) => false,
                        }
                    }
                    None => false,
                },
                _ => false,
            };
            if worker.session.stamp() != Some(ticket.stamp) {
                tracing::debug!(server = %ticket.server.name, %error, "pull diagnostics failed for text that is no longer displayed");
                return;
            }
            // What: Pick the wait before the request is sent again. `matches!` tests the error's
            //       variant; `Some(...)` is the "value present" variant of `Option`.
            // Why: The server names its own wait by asking for a retrigger; a timeout already
            //      waited, so that request goes out at once; every other failure is final.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const wait = wantsRetry ? PULL_RETRY_DELAY : error instanceof TimeoutError ? 0 : undefined;
            // ```
            let wait = if wants_retry {
                Some(PULL_RETRY_DELAY)
            } else if matches!(error, helix_lsp::Error::Timeout(_)) {
                Some(Duration::ZERO)
            } else {
                None
            };
            unanswered(worker, ticket, wait, &error);
        }
    }
}
