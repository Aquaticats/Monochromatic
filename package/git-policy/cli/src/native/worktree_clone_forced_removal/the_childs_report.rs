//! What:
//!  What one child hands back when it has run one piece of the act in a
//!  namespace:
//!  how the piece ended,
//!  the errno that stopped it,
//!  one line naming the call,
//!  the mount table it read afterwards,
//!  and why no report came back at all.
//! Why:
//!  The handler judges an act by what the child saw inside the namespace,
//!  and a pipe carries bytes,
//!  so the answer is one bounded text with three known header lines and a
//!  length before the table.
//!  A truncated or rewritten report must be refused by name rather than read as
//!  an act that finished
//!  (`doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`,
//!  "The preflight").
//!  The child that fills it is `mount_namespace_child.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const text = report.encode();                      // outcome, errno, detail, table length
//! const back = ActReport.decode(text);               // refused by what it lacks
//! ```

/// The most bytes one report's header takes,
/// its detail line included.
pub const MAXIMUM_HEADER_BYTES: usize = 4096;

/// The most bytes of mount table one report carries,
/// which is the crate's own mount-table bound (`plain_file::MOUNT_TABLE_LIMIT`,
/// four mebibytes).
pub(super) const MAXIMUM_TABLE_BYTES: usize = 4 << 20;

/// What:
///  How one child's piece of the act ended.
/// Why:
///  A refusal the design names,
///  a call the kernel failed,
///  and an act that finished are three answers the handler reports
///  differently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ActOutcome = 'done' | 'refused' | 'failed';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActOutcome {
    /// The child did what it was asked and its own checks passed.
    Done,
    /// The design refused the act here,
    /// as a covered member or a locked copy is refused.
    Refused,
    /// A call the act made answered an error.
    Failed,
}

impl ActOutcome {
    /// The word the report carries.
    pub fn word(self) -> &'static str {
        return match self {
            ActOutcome::Done => "done",
            ActOutcome::Refused => "refused",
            ActOutcome::Failed => "failed",
        };
    }

    /// The outcome one report word names.
    pub fn parse(word: &str) -> Option<ActOutcome> {
        return match word {
            "done" => Some(ActOutcome::Done),
            "refused" => Some(ActOutcome::Refused),
            "failed" => Some(ActOutcome::Failed),
            _ => None,
        };
    }
}

/// What:
///  One child's whole answer:
///  the outcome,
///  the errno that stopped it,
///  one line naming the call,
///  and the table it read after the act.
/// Why:
///  The handler judges an act by what the child saw inside the namespace,
///  and a line that named only "failed" would leave the report unable to say
///  what refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ActReport = { outcome: ActOutcome; errno?: number; detail: string; table: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActReport {
    /// What the act came to.
    pub outcome: ActOutcome,
    /// The errno the stopping call answered.
    pub errno: Option<i32>,
    /// One line naming the call and what it was asked,
    /// with no newline of its own.
    pub detail: String,
    /// `/proc/self/mountinfo` as the child read it after the act.
    pub table: String,
}

impl ActReport {
    /// An act that finished.
    pub fn done(
        detail: String,
        table: String,
    ) -> ActReport {
        return ActReport {
            outcome: ActOutcome::Done,
            errno: None,
            detail,
            table,
        };
    }

    /// An act a call stopped,
    /// with the errno it answered.
    pub fn failed(
        detail: String,
        errno: i32,
        table: String,
    ) -> ActReport {
        return ActReport {
            outcome: ActOutcome::Failed,
            errno: Some(errno),
            detail,
            table,
        };
    }

    /// An act the design refused here.
    pub fn refused(
        detail: String,
        table: String,
    ) -> ActReport {
        return ActReport {
            outcome: ActOutcome::Refused,
            errno: None,
            detail,
            table,
        };
    }

    /// What:
    ///  The report as one bounded text.
    /// Why:
    ///  A pipe carries bytes and the parent must find the fields without
    ///  guessing,
    ///  so the header is three known lines and the table follows one length.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const text = `outcome ${outcome}\nerrno ${errno}\ndetail ${escaped}\ntable ${bytes}\n${table}`;
    /// ```
    pub fn encode(&self) -> Vec<u8> {
        let errno: String = match self.errno {
            Some(number) => number.to_string(),
            None => String::from("none"),
        };
        let mut escaped: String = String::with_capacity(self.detail.len());
        for character in self.detail.chars() {
            if character == '\\' {
                escaped.push_str("\\\\");
            } else if character == '\n' {
                escaped.push_str("\\n");
            } else {
                escaped.push(character);
            }
        }
        let header: String = format!(
            "outcome {}\nerrno {errno}\ndetail {escaped}\ntable {}\n",
            self.outcome.word(),
            self.table.len()
        );
        let mut bytes: Vec<u8> = Vec::with_capacity(header.len() + self.table.len());
        bytes.extend_from_slice(header.as_bytes());
        bytes.extend_from_slice(self.table.as_bytes());
        return bytes;
    }

    /// What:
    ///  Read one report,
    ///  or say which field is not one this build writes.
    /// Why:
    ///  A truncated or rewritten report must be refused by name rather than
    ///  read as an act that finished.
    ///
    /// # Errors
    /// A line is absent,
    /// the outcome is not a word this build writes,
    /// the table's length is not a number,
    /// or the bytes do not carry the table the length names.
    pub fn decode(bytes: &[u8]) -> Result<ActReport, String> {
        let mut at: usize = 0;
        let mut outcome: Option<ActOutcome> = None;
        let mut errno: Option<i32> = None;
        let mut detail: Option<String> = None;
        let mut table_bytes: Option<usize> = None;
        for _line in 0..4 {
            let rest: &[u8] = &bytes[at..];
            let Some(end) = rest.iter().position(|byte| return *byte == b'\n') else {
                return Err(String::from(
                    "the report's header ended before its four lines did",
                ));
            };
            let Ok(line) = std::str::from_utf8(&rest[..end]) else {
                return Err(String::from("a header line is not text"));
            };
            at += end + 1;
            if let Some(word) = line.strip_prefix("outcome ") {
                outcome = ActOutcome::parse(word);
                if outcome.is_none() {
                    return Err(format!("the outcome `{word}` is not one this build writes"));
                }
            } else if let Some(number) = line.strip_prefix("errno ") {
                if number == "none" {
                    errno = None;
                } else {
                    errno = Some(
                        number
                            .parse::<i32>()
                            .map_err(|_error| return String::from("the errno is not a number"))?,
                    );
                }
            } else if let Some(text) = line.strip_prefix("detail ") {
                let mut plain: String = String::with_capacity(text.len());
                let mut characters = text.chars();
                while let Some(character) = characters.next() {
                    if character != '\\' {
                        plain.push(character);
                        continue;
                    }
                    match characters.next() {
                        Some('n') => plain.push('\n'),
                        Some('\\') => plain.push('\\'),
                        _ => {
                            return Err(String::from(
                                "the detail line's escape is not one this build writes",
                            ));
                        }
                    }
                }
                detail = Some(plain);
            } else if let Some(number) = line.strip_prefix("table ") {
                table_bytes = Some(number.parse::<usize>().map_err(|_error| {
                    return String::from("the table's length is not a number");
                })?);
            } else {
                return Err(format!(
                    "the header line `{line}` is not one this build writes"
                ));
            }
        }
        let (Some(answered), Some(named), Some(wanted)) = (outcome, detail, table_bytes) else {
            return Err(String::from(
                "the report's header lacks one of its four lines",
            ));
        };
        if wanted > MAXIMUM_TABLE_BYTES {
            return Err(String::from(
                "the table is longer than the crate's own bound",
            ));
        }
        let table: &[u8] = &bytes[at..];
        if table.len() != wanted {
            return Err(format!(
                "the report carries {} table bytes and its length says {wanted}",
                table.len()
            ));
        }
        let Ok(text) = std::str::from_utf8(table) else {
            return Err(String::from("the table is not text"));
        };
        return Ok(ActReport {
            outcome: answered,
            errno,
            detail: named,
            table: String::from(text),
        });
    }
}

/// What:
///  Why no child's report came back.
/// Why:
///  A fork the system refused,
///  a pipe,
///  a wait and a report that is not one this build writes are four different
///  failures,
///  and the act must never read a missing report as a finished one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ChildRefusal = { calling: string; errno?: number; said: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChildRefusal {
    /// Which call refused.
    pub calling: &'static str,
    /// The errno the call answered,
    /// where it answered one.
    pub errno: Option<i32>,
    /// What it said,
    /// in plain words.
    pub said: String,
}

impl std::fmt::Display for ChildRefusal {
    /// One sentence naming the call,
    /// the errno and what it said.
    fn fmt(
        &self,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        return match self.errno {
            Some(errno) => write!(
                formatter,
                "{} answered errno {errno}: {}",
                self.calling, self.said
            ),
            None => write!(
                formatter,
                "{} could not be made: {}",
                self.calling, self.said
            ),
        };
    }
}
