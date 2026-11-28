//! What:
//!  What one of the two words answers:
//!  the status this process exits with and the text each stream carries.
//! Why:
//!  The management grammar turns an answer into the process's own exit,
//!  and a command that answered with a value of its own would have to know how
//!  this crate prints one.
//!  Three fields are the whole of what the wrapper's exits carry
//!  (`invocation::action::Action::Exit`),
//!  so the two commands answer with those three and the grammar writes them
//!  where it knows how.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type AnAnswer = { code: number; stdout: string; stderr: string };
//! ```

/// What:
///  One finished invocation:
///  its status,
///  what standard output carries and what standard error carries.
/// Why:
///  A person reading one stream sees a whole report or a whole refusal and
///  never half of each,
///  and a caller that captures one stream reads the same text every run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer: AnAnswer = { code: 0, stdout: '', stderr: 'cancelled the removal of `one`...' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnAnswer {
    /// The status this process exits with.
    pub code: i32,
    /// The text written on standard output,
    /// line-terminated where it is not empty.
    pub stdout: String,
    /// The text written on standard error,
    /// line-terminated where it is not empty.
    pub stderr: String,
}

impl AnAnswer {
    /// What:
    ///  One answer whose text a person reads on standard output.
    /// Why:
    ///  A report a command was run for belongs there,
    ///  where a shell's pipeline can read it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// AnAnswer.onStandardOutput(0, 'settled the records of `one`.\n');
    /// ```
    pub fn on_standard_output(
        code: i32,
        text: String,
    ) -> AnAnswer {
        return AnAnswer {
            code,
            stdout: text,
            stderr: String::new(),
        };
    }

    /// What:
    ///  One answer whose text a person reads on standard error.
    /// Why:
    ///  A refusal and the cancellation's own report sit there:
    ///  the cancel command's module doc says its lines go to standard error,
    ///  because the stream rule keeps a forwarded command's standard output for
    ///  Git.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// AnAnswer.onStandardError(2, 'no slot was named, so there is nothing to cancel.\n');
    /// ```
    pub fn on_standard_error(
        code: i32,
        text: String,
    ) -> AnAnswer {
        return AnAnswer {
            code,
            stdout: String::new(),
            stderr: text,
        };
    }

    /// What:
    ///  The same answer with one line feed after its text,
    ///  which every sentence and every report a person reads ends in.
    /// Why:
    ///  A stream whose last line carries no line feed runs into the shell's next
    ///  prompt,
    ///  and the command's own sentences are written without one.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// answer.lineTerminated();
    /// ```
    pub fn line_terminated(mut self) -> AnAnswer {
        if !self.stdout.is_empty() && !self.stdout.ends_with('\n') {
            self.stdout.push('\n');
        }
        if !self.stderr.is_empty() && !self.stderr.ends_with('\n') {
            self.stderr.push('\n');
        }
        return self;
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "the_answer_one_word_prints_tests.rs"]
mod tests;
