//! What:
//!  The commit message Git makes of the `-m` texts when no editor opens.
//! Why:
//!  The service lands the scoped commit an agent or a person types most
//! (`git commit -m <message> -- <paths>`),
//! and Git's own clean-up of those paragraphs is what the message must be
//! (`service_commit_args.rs`).
//! It is a step of its own because it reads the finished texts and nothing else:
//! no option and no repository.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const message = cleanupMessage(['subject', 'body']) // undefined when nothing is left
//! ```

/// What:
///  The commit message Git makes of the given texts when no editor opens.
/// Why:
///   Each text is a paragraph;
/// Git strips trailing whitespace from every line,
/// drops leading and trailing blank lines,
///  squeezes runs of blank lines into one and ends the message with a newline.
/// An empty result is no message.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// cleanupMessage(['subject', 'body'])
/// ```
pub fn cleanup_message(messages: &[String]) -> Option<String> {
    let joined: String = messages.join("\n\n");
    let mut lines: Vec<&str> = Vec::new();
    let mut previous_blank: bool = true;
    for line in joined.split('\n') {
        let trimmed: &str = line.trim_end();
        let blank: bool = trimmed.is_empty();
        if blank && previous_blank {
            continue;
        }
        lines.push(trimmed);
        previous_blank = blank;
    }
    while matches!(lines.last(), Some(line) if line.is_empty()) {
        lines.pop();
    }
    if lines.is_empty() {
        return None;
    }
    return Some(format!("{}\n", lines.join("\n")));
}
