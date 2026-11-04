//! What:
//!  The state fields of the lane's real state document: the marker, the logical head and tail, the state of every live request and the
//!  consumption map, with their canonical line encoding and their consistency rules.
//! Why:
//!  A field of the document is either a reference (a file of the lane the component derives and checks against the closure) or state (no
//!  file lies behind it). These are state: a newer generation supersedes them, and whether a transition may change one is the protocol's
//!  rule, shown by the models, not the log's. The log keeps them canonical and consistent with the references they name, so a document
//!  never mentions a request it does not adopt or a position outside the adoption order it relies on.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type LaneState = { marker: 'none' | 'materializing' | 'recovering'; head?: string; tail: string[]; requests: RequestState[]; consumption: Consumption[] };
//! ```

use super::commit_integrity::{hex, is_object_name, unhex};

/// The longest resume point: the design names them by a step and a digit (`B1`, `P1`, `T1`), so this leaves room and stops a line from
/// carrying free text.
pub const RESUME_MAX: usize = 16;

/// Whether an effect outside the document is in progress (the design's marker): the document is replaced with the marker set before the
/// effect and with it cleared after, so a reader that sees `None` sees a state the lane was really in.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Marker {
    /// No effect is in progress.
    #[default]
    None,
    /// The index replacement or the install of a published request is in progress.
    Materializing,
    /// A recovery action that changes state outside the document is in progress.
    Recovering,
}

impl Marker {
    /// The word of a marker that is written (`None` is the absence of the line).
    fn word(self) -> Option<&'static str> {
        return match self {
            Marker::None => None,
            Marker::Materializing => Some("materializing"),
            Marker::Recovering => Some("recovering"),
        };
    }

    /// The marker a written word names; `none` is never written, so it is not a word.
    fn from_word(word: &str) -> Option<Marker> {
        return match word {
            "materializing" => Some(Marker::Materializing),
            "recovering" => Some(Marker::Recovering),
            _ => None,
        };
    }
}

/// The non-terminal states of a request, from the design's request state list. A request whose client-visible outcome is final leaves the
/// document after the ledger line that publishes it, so a terminal state is not a state of this document.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RequestPhase {
    /// Captured and bound to its identity.
    Bound,
    /// Published to the inbox.
    Accepted,
    /// Appended to the adoption order.
    Adopted,
    /// Held behind a hook or a foreign program.
    Held,
    /// Built and ready to publish.
    Ready,
    /// Publication has begun.
    Publishing,
    /// Published, its follow-up still to run.
    Published,
    /// Set aside as pending.
    Pending,
    /// Replayed to land after an ambiguous publication.
    Landing,
    /// A merge request whose files are ready to install.
    MergeReady,
    /// A published merge whose files are being installed.
    Installing,
}

/// Every phase with its word, the one place the words live.
const PHASES: [(RequestPhase, &str); 11] = [
    (RequestPhase::Bound, "bound"),
    (RequestPhase::Accepted, "accepted"),
    (RequestPhase::Adopted, "adopted"),
    (RequestPhase::Held, "held"),
    (RequestPhase::Ready, "ready"),
    (RequestPhase::Publishing, "publishing"),
    (RequestPhase::Published, "published"),
    (RequestPhase::Pending, "pending"),
    (RequestPhase::Landing, "landing"),
    (RequestPhase::MergeReady, "merge-ready"),
    (RequestPhase::Installing, "installing"),
];

impl RequestPhase {
    fn word(self) -> &'static str {
        return PHASES.iter().find(|(phase, _)| { return *phase == self; }).map_or("", |(_, word)| { return *word; });
    }

    fn from_word(word: &str) -> Option<RequestPhase> {
        return PHASES.iter().find(|(_, known)| { return *known == word; }).map(|(phase, _)| { return *phase; });
    }
}

/// The state of one live request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequestState {
    /// The request's identity; it must be a request the document adopts.
    pub id: String,
    /// Where the request is.
    pub phase: RequestPhase,
    /// The step to resume at after a restart (`B1`, `P1`, ...), at most `RESUME_MAX` characters of letters, digits and hyphens.
    pub resume: String,
    /// The byte offset of the request's adopt line in the order file the document relies on; it must lie inside that extent.
    pub position: Option<u64>,
}

/// What a consumption entry is for.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConsumptionKind {
    /// The request's published result is still to be applied to that path of the index.
    Install,
    /// The index entry is a historical entry a published request already consumed; nothing is to be installed for it.
    Suppress,
    /// A published merge whose files are still to be installed.
    MergeInstall,
    /// Keeps only the cleanup obligation of a merge; no effect on capture.
    AttachWatch,
}

/// Every kind with its word.
const KINDS: [(ConsumptionKind, &str); 4] = [
    (ConsumptionKind::Install, "install"),
    (ConsumptionKind::Suppress, "suppress"),
    (ConsumptionKind::MergeInstall, "merge-install"),
    (ConsumptionKind::AttachWatch, "attach-watch"),
];

impl ConsumptionKind {
    fn word(self) -> &'static str {
        return KINDS.iter().find(|(kind, _)| { return *kind == self; }).map_or("", |(_, word)| { return *word; });
    }

    fn from_word(word: &str) -> Option<ConsumptionKind> {
        return KINDS.iter().find(|(_, known)| { return *known == word; }).map(|(kind, _)| { return *kind; });
    }
}

/// One entry of the consumption map: a path of the index, the entry it had before the request consumed it, the request and its version.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Consumption {
    /// The index path, as bytes.
    pub path: Vec<u8>,
    /// What the entry is for.
    pub kind: ConsumptionKind,
    /// The consuming request; it must be a request the document adopts.
    pub request: String,
    /// The version of the request.
    pub version: u64,
    /// The pre-entry with every field, as the bytes the caller encoded; opaque to the log, non-empty.
    pub entry: Vec<u8>,
}

/// The state of the lane that has no file behind it.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LaneState {
    /// Whether an effect outside the document is in progress.
    pub marker: Marker,
    /// The logical head: the newest published-or-pending, non-held commit, or the branch tip (an object name).
    pub head: Option<String>,
    /// The logical tail: the requests whose commits are built on the head, oldest first; each must be adopted.
    pub tail: Vec<String>,
    /// The state of every live request, sorted by identity.
    pub requests: Vec<RequestState>,
    /// The consumption map, sorted by path, request and version.
    pub consumption: Vec<Consumption>,
}

impl LaneState {
    /// Appends this state's lines to `text`, in the order the parser requires.
    pub(super) fn encode_into(&self, text: &mut String) {
        if let Some(word) = self.marker.word() {
            text.push_str(&format!("marker {}\n", word));
        }
        if let Some(head) = &self.head {
            text.push_str(&format!("head {}\n", head));
        }
        for id in &self.tail {
            text.push_str(&format!("tail {}\n", id));
        }
        for request in &self.requests {
            let position: String = request.position.map_or_else(|| { return "-".to_string(); }, |at| { return at.to_string(); });
            text.push_str(&format!("state {} {} {} {}\n", request.id, request.phase.word(), request.resume, position));
        }
        for entry in &self.consumption {
            text.push_str(&format!("consume {} {} {} {} {}\n", entry.kind.word(), entry.request, entry.version, hex(&entry.path), hex(&entry.entry)));
        }
    }

    /// Reads one line (`key value`) into this state; `None` when the key is not a state key or the value does not decode. Repeats of a
    /// single-valued line are refused here; ordering and consistency are checked by `is_consistent` and by the canonical re-encoding.
    pub(super) fn apply_line(&mut self, key: &str, value: &str) -> Option<()> {
        match key {
            "marker" if self.marker == Marker::None => self.marker = Marker::from_word(value)?,
            "head" if self.head.is_none() => self.head = Some(value.to_string()),
            "tail" => self.tail.push(value.to_string()),
            "state" => {
                let parts: Vec<&str> = value.split(' ').collect();
                let [id, phase, resume, written_position] = parts[..] else { return None };
                let position: Option<u64> = if written_position == "-" { None } else { Some(written_position.parse().ok()?) };
                self.requests.push(RequestState { id: id.to_string(), phase: RequestPhase::from_word(phase)?, resume: resume.to_string(), position });
            }
            "consume" => {
                let parts: Vec<&str> = value.split(' ').collect();
                let [kind, request, version, path, entry] = parts[..] else { return None };
                self.consumption.push(Consumption {
                    path: unhex(path)?,
                    kind: ConsumptionKind::from_word(kind)?,
                    request: request.to_string(),
                    version: version.parse().ok()?,
                    entry: unhex(entry)?,
                });
            }
            _ => return None,
        }
        return Some(());
    }

    /// Whether this state is consistent with the references of the document: `adopted` is the sorted list of request identities the document
    /// adopts and `order_length` the order extent it relies on. A state for a request the document does not adopt, a position outside the
    /// extent, an unsorted or repeated entry, a head that is not an object name or a resume point that is not a short token is refused.
    pub(super) fn is_consistent(&self, adopted: &[String], order_length: Option<u64>) -> bool {
        let adopts = |id: &String| -> bool { return adopted.binary_search(id).is_ok(); };
        let head_ok: bool = self.head.as_deref().is_none_or(is_object_name);
        let tail_ok: bool = self.tail.iter().all(adopts) && self.tail.iter().enumerate().all(|(at, id)| { return !self.tail[..at].contains(id); });
        let requests_ok: bool = self.requests.windows(2).all(|pair| { return pair[0].id < pair[1].id; })
            && self.requests.iter().all(|request| {
                let token: bool = !request.resume.is_empty() && request.resume.len() <= RESUME_MAX && request.resume.chars().all(|c| { return c.is_ascii_alphanumeric() || c == '-'; });
                let position_ok: bool = request.position.is_none_or(|at| { return order_length.is_some_and(|length| { return at < length; }); });
                return adopts(&request.id) && token && position_ok;
            });
        let key = |entry: &Consumption| -> (Vec<u8>, String, u64) { return (entry.path.clone(), entry.request.clone(), entry.version); };
        let consumption_ok: bool = self.consumption.windows(2).all(|pair| { return key(&pair[0]) < key(&pair[1]); })
            && self.consumption.iter().all(|entry| { return adopts(&entry.request) && !entry.path.is_empty() && !entry.entry.is_empty(); });
        return head_ok && tail_ok && requests_ok && consumption_ok;
    }
}
