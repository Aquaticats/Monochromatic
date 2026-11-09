//! What:
//! Classifies what a tool's standard input is attached to:
//! nothing,
//! a regular file,
//! a pipe,
//! or a socket.
//! Why:
//! The policies decide on it.
//! With a pipe and no search path a tool reads the input and walks nothing,
//! and with a pipe and a named search path ripgrep reads the path and drops the input.
//! A redirected file and a socket are deliberate redirections,
//! so they must not be taken for a pipe.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // stdin-kind.ts
//! export type StdinKind = 'none' | 'file' | 'pipe' | 'socket';
//! export function stdinKindOf(fileType: FileType): StdinKind { ... }
//! ```

/// Imports the trait that adds `is_fifo()` and `is_socket()` to a file type.
/// A pipe on standard input is a FIFO,
/// and the difference between it and a redirected file is the whole lost-input rule.
use std::os::unix::fs::FileTypeExt;

/// Imports `FileType`,
/// the result of asking the file system what a path is,
/// so the classification is a pure function a test can call.
use std::fs::FileType;

/// What:
/// `pub enum StdinKind { ... }` declares a type whose value is exactly one of four named cases.
/// `#[derive(Debug, PartialEq, Eq, Clone, Copy)]` asks the compiler to generate printing,
/// `==`,
/// copying and comparison.
/// Why:
/// One value says which kind of input a tool would read,
/// where two true/false facts would let an impossible pair exist.
/// It is `Copy` because the policies take it by value and the caller keeps it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StdinKind = 'none' | 'file' | 'pipe' | 'socket';
/// ```
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum StdinKind {
    /// Standard input is closed or attached to a terminal,
    /// so a search reads no input.
    None,
    /// Standard input is a regular file (`rg PATTERN < file`),
    /// which a search reads.
    File,
    /// Standard input is a pipe (`command | rg PATTERN`),
    /// which a search reads.
    Pipe,
    /// Standard input is a socket,
    /// which a search reads.
    Socket,
}

/// The questions a policy asks about the kind of input:
/// whether a tool could read anything at all,
/// and whether that input is a pipe.
impl StdinKind {
    /// What:
    /// `pub fn of(file_type: &FileType) -> StdinKind` declares an associated function (it takes no `self`) that maps a
    /// file type to the kind of input.
    /// Why:
    /// The shadow asks the file system once per run;
    /// this mapping is the whole decision and stays callable from a test without a real pipe.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static of(fileType: FileType): StdinKind { ... }
    /// ```
    pub fn of(file_type: &FileType) -> StdinKind {
        // A terminal and `/dev/null` are character devices, and a directory is neither of the four: all give `None`.
        if file_type.is_file() {
            return StdinKind::File;
        }
        if file_type.is_fifo() {
            return StdinKind::Pipe;
        }
        if file_type.is_socket() {
            return StdinKind::Socket;
        }
        return StdinKind::None;
    }

    /// What:
    /// `pub fn reads_input(self) -> bool` declares a method taking the value by copy and returning true when a tool
    /// could read the input.
    /// Why:
    /// A pipe,
    /// a file and a socket all give a tool something to read;
    /// only `StdinKind::None` leaves it nothing.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// readsInput(): boolean { return this !== 'none'; }
    /// ```
    pub fn reads_input(self) -> bool {
        return !matches!(self, StdinKind::None);
    }

    /// What:
    /// `pub fn is_pipe(self) -> bool` declares a method returning true only for `StdinKind::Pipe`.
    /// Why:
    /// The lost-input rule turns on a pipe exactly.
    /// Counting a redirected file or a socket as well would refuse a redirection the caller meant.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// isPipe(): boolean { return this === 'pipe'; }
    /// ```
    pub fn is_pipe(self) -> bool {
        return matches!(self, StdinKind::Pipe);
    }
}

/// The unit test of the classification.
#[cfg(test)]
#[path = "stdin_kind_tests.rs"]
mod tests;
