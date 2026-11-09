//! What:
//!  The named facts one audit record carries beside the fields every record has,
//!  as the JSON object
//!       appended to its line.
//! Why:
//!  A record about a form that is only recorded,
//!  or about an exemption the environment chose,
//!  has to
//!       name what the decision saw (a worktree,
//!  a branch,
//!  the variable that exempted it),
//!  and a typed pair keeps
//!       the line valid JSON without a second reader.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // recordAudit({ ..., facts: [{ kind: 'flag', name: 'force', value: true }] })
//! ```

/// Quoting one JSON string,
///  as every other field of the line is quoted.
use crate::json_record::quote;

/// What:
///  One named fact a record carries beside the fields every record has.
///  An `enum` is a closed set of named alternatives.
/// Why:
///   A record about a form that is only recorded,
///  or about an exemption the environment chose,
///  has to
///       name what the decision saw (a worktree,
///  a branch,
///  the variable that exempted it),
/// and a typed pair keeps the
///  line valid JSON without a second reader.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AuditFact = { kind: 'text'; name: string; value: string } | { kind: 'flag'; name: string; value: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuditFact {
    /// Text the decision read,
    ///  such as a path,
    ///  a branch name or a variable name.
    Text(&'static str, String),
    /// A yes-or-no answer,
    ///  written as JSON `true` or `false`.
    Flag(&'static str, bool),
}

impl AuditFact {
    /// The field's name.
    pub fn name(&self) -> &'static str {
        match self {
            AuditFact::Text(name, _) | AuditFact::Flag(name, _) => return name,
        }
    }

    /// The field's value as JSON.
    pub fn json(&self) -> String {
        match self {
            AuditFact::Text(_, value) => return quote(value),
            AuditFact::Flag(_, value) => return String::from(if *value { "true" } else { "false" }),
        }
    }
}

/// What:
///  The named facts of one record as a JSON object,
/// or empty text when it has none.
/// Why:
///   The object is additive:
///  a reader that ignores unknown fields is unaffected,
/// and the line stays one
///  valid JSON record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const facts = facts.length === 0 ? '' : `,"facts":{${facts.map(f => `"${f.name()}":${f.json()}`).join(',')}}`;
/// ```
pub fn facts_object(facts: &[AuditFact]) -> String {
    if facts.is_empty() {
        return String::new();
    }
    let mut rendered: Vec<String> = Vec::with_capacity(facts.len());
    for fact in facts {
        rendered.push(format!("{}:{}", quote(fact.name()), fact.json()));
    }
    return format!(",\"facts\":{{{}}}", rendered.join(","));
}
