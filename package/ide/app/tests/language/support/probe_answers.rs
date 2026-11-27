//! What the worker's answers and states say:
//!  the replies to a request,
//!  the diagnostics and hints it holds,
//!  its notice,
//!  and the states a test waits for.

use super::child_process::SERVER;
use ide_app::language::notice::SOURCE as NOTICE_SOURCE;
use ide_app::language::{
    identity::DocumentStamp,
    reply::LanguageReply,
    status::ServerState,
};

use super::probe::Probe;

impl Probe {
    /// True when some reply answers request `number`.
    pub fn has_reply(
        &self,
        number: u64,
    ) -> bool {
        for reply in &self.replies {
            if reply.request == number {
                return true;
            }
        }
        return false;
    }

    /// True when the last reply of request `number` arrived.
    pub fn is_answered(
        &self,
        number: u64,
    ) -> bool {
        for reply in &self.replies {
            if reply.request == number && reply.remaining == 0 {
                return true;
            }
        }
        return false;
    }

    /// True when the held diagnostics carry `stamp`.
    pub fn diagnostics_stamped(
        &self,
        stamp: DocumentStamp,
    ) -> bool {
        return match self.diagnostics.as_ref() {
            Some(found) => found.stamp == stamp,
            None => false,
        };
    }

    /// True when the held hints carry `stamp`.
    pub fn hints_stamped(
        &self,
        stamp: DocumentStamp,
    ) -> bool {
        return match self.hints.as_ref() {
            Some(held) => held.stamp == stamp,
            None => false,
        };
    }

    /// True when the notice entry exists and its text contains `part`.
    pub fn notice_contains(
        &self,
        part: &str,
    ) -> bool {
        return match self.notice() {
            Some(entry) => entry.contains(part),
            None => false,
        };
    }

    /// State of the named server in the latest status.
    pub fn state(
        &self,
        name: &str,
    ) -> Option<&ServerState> {
        for row in &self.status.servers {
            if row.server.name == name {
                return Some(&row.state);
            }
        }
        return None;
    }

    /// Wait until the scripted server is ready.
    pub fn until_ready(&mut self) {
        self.wait_for("the scripted server to be ready");
        while self.state(SERVER) != Some(&ServerState::Ready) {
            self.step();
        }
    }

    /// Wait for the last reply of request `number` and return every reply it produced.
    pub fn answers(
        &mut self,
        number: u64,
    ) -> Vec<LanguageReply> {
        self.wait_for("the request to be answered");
        while !self.is_answered(number) {
            self.step();
        }
        let mut found = Vec::new();
        for reply in &self.replies {
            if reply.request == number {
                found.push(reply.clone());
            }
        }
        return found;
    }

    /// Every message of the servers' own diagnostics on display,
    ///  in group and position order;
    ///  the entry that explains missing language support is not one of them (see `notice`).
    pub fn messages(&self) -> Vec<String> {
        let mut found = Vec::new();
        if let Some(snapshot) = &self.diagnostics {
            for group in &snapshot.groups {
                if group.source == NOTICE_SOURCE {
                    continue;
                }
                for item in &group.items {
                    found.push(item.message.clone());
                }
            }
        }
        return found;
    }

    /// The message of the entry that explains missing language support,
    ///  when the displayed diagnostics hold it.
    pub fn notice(&self) -> Option<String> {
        let snapshot = self.diagnostics.as_ref()?;
        let mut noticed = None;
        for group in snapshot.groups.iter() {
            if group.source == NOTICE_SOURCE {
                noticed = Some(group);
                break;
            }
        }
        let group = noticed?;
        let item = group.items.first()?;
        return Some(item.message.clone());
    }
}
