//! What:
//!  The place one line of progress goes,
//! as a trait a struct can implement.
//! Why:
//!  The ordered runner hands every item a sink that sends each line to the consumer with the item's index,
//! and a conversion says
//!  its progress through whatever sink it is given.
//! A sink that carries state (the sender and the index) is a named struct
//!  instead of a closure,
//! so the code that says lines is the same whether a thread or a test supplies the sink.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! interface LineSink { say(text: string): void }
//! ```

/// What:
///  Where one item says its lines while it runs.
/// Why:
///   The runner sends each line to the consumer with the item's index,
/// which a plain function cannot know;
/// the sink carries it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface LineSink { say(text: string): void }
/// ```
pub trait LineSink {
    /// Say one line.
    fn say(&mut self, text: &str);
}

/// A list of text is a sink that keeps every line in the order it was said,
/// for the places that report the lines after the work.
impl LineSink for Vec<String> {
    fn say(&mut self, text: &str) {
        self.push(text.to_string());
    }
}

/// Line sink controls stay out of the release executable.
#[cfg(test)]
#[path = "line_sink_tests.rs"]
mod tests;
