//! The interface-side fence:
//!  only results for the displayed text and the current server process pass.

/// Results are compared by file generation,
///  revision,
///  and server process.
use super::identity::{DocumentStamp, ServerIdentity};

/// What:
///  A closed set of four names.
///  `Copy` lets it be passed like a number.
/// Why:
///  Tests and logs must be able to say which part of the identity made a result stale.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FenceVerdict = 'accepted' | 'otherFile' | 'staleRevision' | 'oldServer';
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FenceVerdict {
    /// The result describes the displayed text and a current server process.
    Accepted,
    /// The result belongs to a file that is no longer displayed.
    OtherFile,
    /// The result belongs to an earlier revision of the displayed file.
    StaleRevision,
    /// The result comes from a server process that has since been replaced.
    OldServer,
}

/// How many results each verdict has seen since the worker handle was created.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FenceCounts = { accepted: number; otherFile: number; staleRevision: number; oldServer: number };
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FenceCounts {
    /// Results handed to the caller.
    pub accepted: u64,
    /// Results dropped because another file is displayed.
    pub other_file: u64,
    /// Results dropped because the displayed file was reloaded since.
    pub stale_revision: u64,
    /// Results dropped because the server process was replaced.
    pub old_server: u64,
}

/// What:
///  The fence's memory:
///  the displayed text's identity and the current server processes.
///       `Option<DocumentStamp>` is "a stamp,
///  or nothing";
///  `Vec<ServerIdentity>` is a growable list.
/// Why:
///  A reply carries only what it was asked for;
///  whether that is still what the reader
///      shows can be decided only against this record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Fence { displayed?: DocumentStamp; servers: ServerIdentity[] = []; counts = zeroCounts(); }
/// ```
#[derive(Debug, Default)]
pub struct Fence {
    /// Stamp of the text the reader currently displays;
    ///  absent when no file is displayed.
    displayed: Option<DocumentStamp>,
    /// Current process generation of every server named in the latest status.
    servers: Vec<ServerIdentity>,
    /// Running totals per verdict.
    counts: FenceCounts,
}

/// Fence operations.
impl Fence {
    /// Record which text is displayed now.
    ///  `&mut self` allows changing the fence in place.
    pub fn display(&mut self, stamp: Option<DocumentStamp>) {
        self.displayed = stamp;
    }

    /// Read the stamp results are currently compared with.
    pub fn displayed(&self) -> Option<DocumentStamp> {
        return self.displayed;
    }

    /// Replace the list of current server processes with the one from the latest status.
    pub fn set_servers(&mut self, servers: Vec<ServerIdentity>) {
        self.servers = servers;
    }

    /// Read the running totals.
    pub fn counts(&self) -> FenceCounts {
        return self.counts;
    }

    /// What:
    ///  Decide whether a result may be shown,
    ///  without counting it.
    ///  `Option<&ServerIdentity>`
    ///       is "a borrowed identity,
    ///  or nothing" for results no server produced.
    /// Why:
    ///  File generation is compared first,
    ///  then revision,
    ///  then server process,
    ///  so the
    ///      verdict names the outermost identity that moved on.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// verdict(stamp: DocumentStamp, server?: ServerIdentity): FenceVerdict {
    ///   if (this.displayed?.file !== stamp.file) return 'otherFile';
    ///   if (this.displayed.revision !== stamp.revision) return 'staleRevision';
    ///   const current = this.servers.find(known => known.name === server?.name);
    ///   if (server && current && current.instance !== server.instance) return 'oldServer';
    ///   return 'accepted';
    /// }
    /// ```
    pub fn verdict(&self, stamp: DocumentStamp, server: Option<&ServerIdentity>) -> FenceVerdict {
        // What: `let Some(x) = option else { ... }` binds the inner value or runs the else block.
        // Why: With no displayed file, every result belongs to some other file.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (this.displayed === undefined) return 'otherFile';
        // ```
        let Some(displayed) = self.displayed else {
            return FenceVerdict::OtherFile;
        };
        if displayed.file != stamp.file {
            return FenceVerdict::OtherFile;
        }
        if displayed.revision != stamp.revision {
            return FenceVerdict::StaleRevision;
        }
        if let Some(answering) = server {
            // `&self.servers` lends the list to the loop without moving it.
            for known in &self.servers {
                if known.name == answering.name && known.instance != answering.instance {
                    return FenceVerdict::OldServer;
                }
            }
        }
        return FenceVerdict::Accepted;
    }

    /// Decide and count;
    ///  true means the caller may show the result.
    pub fn admit(&mut self, stamp: DocumentStamp, server: Option<&ServerIdentity>) -> bool {
        let verdict = self.verdict(stamp, server);
        match verdict {
            FenceVerdict::Accepted => self.counts.accepted += 1,
            FenceVerdict::OtherFile => self.counts.other_file += 1,
            FenceVerdict::StaleRevision => self.counts.stale_revision += 1,
            FenceVerdict::OldServer => self.counts.old_server += 1,
        }
        if verdict != FenceVerdict::Accepted {
            tracing::debug!(?verdict, ?stamp, ?server, "dropped a stale language result");
        }
        return verdict == FenceVerdict::Accepted;
    }
}

/// Each identity component is moved separately to show which verdict it produces.
#[cfg(test)]
#[path = "fence_tests.rs"]
mod tests;
