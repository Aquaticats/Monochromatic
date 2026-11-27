//! What:
//!  The wait for the answer to a request that was written:
//!  reads the service's messages until one of them answers,
//!  tells the person where the commit stands while it waits,
//!  and ends the command at the ceiling or when the person interrupts.
//! Why:
//!  A request is answered by several messages and by none for a long time,
//!  and what the wait remembers between two of them is what decides:
//!  the place a person was told once and not again,
//!  the words of the write the request is inside,
//!  the scratch directory's renewal,
//!  and the instants both of its bounds are measured from.
//!  It lives here rather than in the loop that sends,
//!  so that loop reads as the tries it makes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const end = theAnswer(wait, reader); // { answered } | 'wentSilent' | 'closed'
//! ```

use crate::commit_timing::{
    SubmitTrace,
    holder_kind_after,
};
use crate::service_client_interrupted::waking_the_read_of;
use crate::service_client_listen_and_ask_to_retire::{
    Heard,
    a_message_is_already_here,
    listen,
};
use crate::service_client_scratch_renewal::renew_scratch;
use crate::service_client_settings_and_watches::ClientConfig;
use crate::service_client_the_failure_a_person_reads::failure_for;
use crate::service_client_waiting_words::{
    place_number,
    say_the_write_it_is_inside,
    say_where_it_stands,
};
use crate::service_client_what_a_request_came_to::Submitted;
use crate::service_client_why_withdrawn::WhyWithdrawn;
use crate::service_client_withdraw_a_request::withdraw;
use crate::service_messages::ClientFailure;
use crate::service_protocol::{
    CommitRequest,
    FailureCode,
    ServiceMessage,
};
use std::io::{
    BufReader,
    Write,
};
use std::os::unix::net::UnixStream;
use std::time::Instant;

/// What:
///  What one written request's wait remembers between two messages of the service.
/// Why:
///   Every field outlives a single message:
///  the place and the write a person was told once,
///  the blocker the error is worded with,
///  the scratch directory's renewal,
///  and the instants the silence bound and the ceiling are measured from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class TheWaitForTheAnswer { config; request; notices; trace; parked; toldPlace; insideTold; lastBlocker; lastRenewed; insideWrite; holder; waitingSince; answerDeadline; writtenAt; }
/// ```
pub(crate) struct TheWaitForTheAnswer<'a> {
    /// The bounds and the words one client run was configured with.
    pub(crate) config: &'a ClientConfig,
    /// The request whose answer is waited for.
    pub(crate) request: &'a CommitRequest,
    /// Where the words the person reads are written.
    pub(crate) notices: &'a mut dyn Write,
    /// The timing record this command fills as it waits.
    pub(crate) trace: &'a mut SubmitTrace,
    /// When the request line first reached the service,
    /// and nothing when none did yet.
    pub(crate) parked: Option<Instant>,
    /// The place the person was told last,
    /// which outlives the connection that named it.
    pub(crate) told_place: &'a mut Option<Option<usize>>,
    /// The words about the write this request is inside that the person was told last.
    pub(crate) inside_told: &'a mut Option<String>,
    /// The words the service last said about what the landing waits for,
    /// which the failure the command ends with names.
    pub(crate) last_blocker: &'a mut Option<String>,
    /// The renewal of the command's scratch directory,
    /// which the wait keeps due.
    pub(crate) last_renewed: &'a mut Option<Instant>,
    /// Whether the service said this request is inside its branch write that is not ending.
    pub(crate) inside_write: &'a mut bool,
    /// Who the service last named as holding what the commit waits for.
    pub(crate) holder: &'a mut Option<String>,
    /// When the wait began, which the ceiling is measured from.
    pub(crate) waiting_since: Instant,
    /// When the extra wait for a write the request is inside ends.
    pub(crate) answer_deadline: Instant,
    /// When the request line was written, which this wait is measured from.
    pub(crate) written_at: Instant,
}

/// What:
///  What the wait came to.
/// Why:
///   An answered request and an ended command leave this wait differently:
///  the wait that ended the command has already counted its own time in the trace,
///  and the one a message answered is counted by the loop that sent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheEndOfTheWait = { kind: 'decidedByTheWait'; done: Submitted | ClientFailure } | { kind: 'answered'; done: Submitted | ClientFailure } | { kind: 'wentSilent' } | { kind: 'closed' };
/// ```
pub(crate) enum TheEndOfTheWait {
    /// A stop or the ceiling withdrew the request or ended it,
    /// and this wait is already counted.
    DecidedByTheWait(Result<Submitted, ClientFailure>),
    /// A message answered the request,
    /// and this wait is counted by the loop that sent it.
    Answered(Result<Submitted, ClientFailure>),
    /// The service said nothing for the whole silence bound.
    WentSilent,
    /// The service closed the connection before answering.
    Closed,
}

impl TheWaitForTheAnswer<'_> {
    /// What:
    ///  Reads the service's messages until one answers,
    /// the service falls silent,
    /// or its connection closes.
    /// Why:
    ///   This is the whole of the wait:
    /// every message is read,
    /// the person is told where the commit stands before each wait,
    /// and the two bounds are judged after a message has been read,
    /// so a notice that arrived with the ceiling still counts.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// while (true) { if (stopped) { countTime(); return { decidedByTheWait: withdraw(...) }; } tellWhereItStands(); const heard = listen(reader); ... }
    /// ```
    pub(crate) fn the_answer(
        &mut self,
        reader: &mut BufReader<UnixStream>,
    ) -> TheEndOfTheWait {
        // What:
        //  The place the service named,
        // which belongs to this connection and to no other.
        // Why:
        //  The place is a position in a batch the answering service decided,
        // and a request sent again goes to a service that decides
        //  its own,
        // so a place read here cannot outlive the connection it came from.
        // It is therefore held inside this method,
        // and
        //  what the person was already told (`told_place`) is what outlives it.
        let mut last_place: Option<(usize, usize)> = None;
        // What:
        //  The words the service on this connection said about the write this command's own request is inside,
        // held until the wait has mattered.
        // Why:
        //  They name that service's write,
        // so they do not outlive the connection that said them,
        // as the place of a batch does not;
        // what the person was told (`inside_told`) is what outlives it.
        let mut inside_words: Option<String> = None;
        loop {
            // What:
            //  A person who interrupts the commit withdraws its request.
            // Why:
            //  Native Git makes no commit when it is interrupted,
            //  and the request this command sent is the service's,
            //  so ending here would leave it to land with nobody to read the answer.
            //  The withdrawal is asked for and its answer is the command's end:
            //  a commit that landed before the signal came stays landed and says so,
            //  and one the service gives up was not made.
            if let Some((signal, sender)) = crate::service_client_interrupted::stop() {
                self.trace.answer_wait += self.written_at.elapsed();
                return TheEndOfTheWait::DecidedByTheWait(withdraw(
                    self.config,
                    self.request.id.as_str(),
                    self.last_blocker.take(),
                    Instant::now(),
                    WhyWithdrawn::Interrupted {
                        signal,
                        sender,
                    },
                ));
            }
            // What:
            //  Where the commit stands, and that it is inside a write, are said here, before each wait, so a service that
            //  sends nothing at all is answered in the same words as one that only beats: neither has started on this commit.
            // Why:
            //  A notice is said to a wait that is still on,
            // and a message the service has already sent may be the answer:
            //  a client the machine held up spoke of a write over the landing it had not read yet.
            // Nothing between two passes waits, so the words that came with the message just read are said here too.
            if !a_message_is_already_here(reader) {
                say_where_it_stands(
                    self.notices,
                    self.config,
                    self.parked,
                    last_place,
                    self.told_place,
                );
                say_the_write_it_is_inside(
                    self.notices,
                    self.config,
                    self.parked,
                    inside_words.as_deref(),
                    self.inside_told,
                );
            }
            // What: The connection is declared for this one read, so a stop delivered to another thread of this process can
            //  wake it.
            // Why:  The kernel hands a process-directed signal to any thread, and a thread that is not the waiting one only
            //  writes the flag the wait reads after its own read gives up, which can be a whole silence bound later.
            let heard: Heard = {
                let _awake = waking_the_read_of(reader.get_ref());
                listen(reader)
            };
            if matches!(heard, Heard::Message(_)) {
                renew_scratch(self.config, self.last_renewed);
            }
            // A notice that arrives with the ceiling counts, so it is read before the ceiling is judged.
            if matches!(heard, Heard::Message(ServiceMessage::InsideWrite { .. })) {
                *self.inside_write = true;
            }
            // The wait ends at the ceiling whatever the service sends: the request is withdrawn, and the answer to that is
            //  the command's end (landed after all, or the error).
            //  A request the service says is inside its branch write is not withdrawn: its commit may be on the branch already, so the
            //  client waits the short extra wait for that write (owner, 2026-11-04, "Short extra wait"), listening for the landing,
            //  and then reports that the result is not known; the write is left running.
            if self.waiting_since.elapsed() >= self.config.answer_ceiling
                && !matches!(
                    heard,
                    Heard::Message(
                        ServiceMessage::Landed { .. }
                            | ServiceMessage::Held { .. }
                            | ServiceMessage::RetryPrepare { .. }
                            | ServiceMessage::Failed { .. }
                    )
                )
            {
                if !*self.inside_write {
                    self.trace.answer_wait += self.written_at.elapsed();
                    return TheEndOfTheWait::DecidedByTheWait(withdraw(
                        self.config,
                        self.request.id.as_str(),
                        self.last_blocker.take(),
                        self.answer_deadline,
                        WhyWithdrawn::Ceiling {
                            minutes: self.config.answer_ceiling.as_secs() / 60,
                        },
                    ));
                }
                if Instant::now() >= self.answer_deadline + self.config.extra_wait {
                    self.trace.answer_wait += self.written_at.elapsed();
                    return TheEndOfTheWait::DecidedByTheWait(Err(
                        ClientFailure::WithdrawalUnconfirmed {
                            blocker: self.last_blocker.take(),
                        },
                    ));
                }
            }
            match heard {
                Heard::Message(ServiceMessage::Position {
                    number,
                    of,
                }) => {
                    // Every place the service names is recorded, printed or not; the person is told it once the wait has
                    //  mattered, so a commit that lands at once says nothing new. The record keeps every place every service
                    //  named, which is what `positions` states; what may be printed is only ever this connection's place.
                    if last_place.map(place_number) != Some(number) {
                        last_place = Some((number, of));
                        self.trace.positions.push(number);
                    }
                }
                Heard::Message(ServiceMessage::Landed {
                    commit,
                    already,
                    landing,
                    ..
                }) => {
                    // What the landing spent and held, for the record of this command: the service measured it, and only its
                    //  answer can carry it.
                    self.trace.landing = landing;
                    return TheEndOfTheWait::Answered(Ok(Submitted::Landed {
                        commit,
                        already,
                    }));
                }
                Heard::Message(ServiceMessage::Held {
                    commit,
                    words,
                    ..
                }) => {
                    return TheEndOfTheWait::Answered(Ok(Submitted::Held {
                        commit,
                        words,
                    }));
                }
                Heard::Message(ServiceMessage::RetryPrepare {
                    base,
                    landed,
                    ..
                }) => {
                    return TheEndOfTheWait::Answered(Ok(Submitted::PrepareAgain {
                        base,
                        landed,
                    }));
                }
                Heard::Message(ServiceMessage::Failed {
                    code: FailureCode::BranchMoved,
                    ..
                }) => return TheEndOfTheWait::Answered(Ok(Submitted::Moved)),
                Heard::Message(ServiceMessage::Failed {
                    code: FailureCode::IndexInUse,
                    ..
                }) => {
                    return TheEndOfTheWait::Answered(Err(ClientFailure::IndexInUse(
                        match self.holder.take() {
                            Some(who) => who,
                            None => String::from("Another program"),
                        },
                    )));
                }
                Heard::Message(ServiceMessage::Failed {
                    code: FailureCode::ReferenceLocked,
                    ..
                }) => {
                    // The service named the locks it left and the command that clears each; without that answer the
                    //  message names the lock by the branch's own name.
                    let words: String = match self.holder.take() {
                        Some(who) => who,
                        None => format!(
                            "The reference lock of {} stands in the way, and nothing proves that a Git process still holds it.",
                            self.request.reference
                        ),
                    };
                    return TheEndOfTheWait::Answered(Err(ClientFailure::ReferenceLocked {
                        branch: self.request.reference.short().spoken(),
                        words,
                    }));
                }
                Heard::Message(ServiceMessage::Failed {
                    code,
                    ..
                }) => return TheEndOfTheWait::Answered(Err(failure_for(code, self.request))),
                Heard::Message(ServiceMessage::Waiting {
                    text,
                }) => {
                    *self.last_blocker = Some(text.trim().to_string());
                    self.trace.waiting_notices += 1;
                    self.trace.holder_kind =
                        Some(holder_kind_after(self.trace.holder_kind, text.as_str()));
                    let _written = write!(self.notices, "{text}");
                }
                // The landing is inside a write: said of the commit, not of a holder, so the kind of holder is left as it was.
                Heard::Message(ServiceMessage::InsideWrite {
                    text,
                }) => {
                    *self.last_blocker = Some(text.trim().to_string());
                    self.trace.waiting_notices += 1;
                    inside_words = Some(text);
                }
                Heard::Message(ServiceMessage::Holder {
                    who,
                }) => *self.holder = Some(who),
                // Each message, this one too, restarts the silence bound; nothing else is needed.
                // A drawn shadow and the answer that there is none are the other question's answers (`service_client_shadow_hand_out.rs`),
                // which a commit never sees on the connection of its request.
                Heard::Message(
                    ServiceMessage::Welcome {
                        ..
                    }
                    | ServiceMessage::Heartbeat
                    | ServiceMessage::Withdrawn {
                        ..
                    }
                    | ServiceMessage::Drawn {
                        ..
                    }
                    | ServiceMessage::NoShadow,
                ) => {}
                Heard::Silent => return TheEndOfTheWait::WentSilent,
                Heard::Closed => return TheEndOfTheWait::Closed,
            }
        }
    }
}
