//! What:
//!  The seven query forms this repository's `.browserslistrc` uses,
//! matched by exact text.
//! Why:
//!  The user's decision supports only the queries in use and refuses every other spelling with
//!  "not implemented",
//! which also sidesteps every JavaScript-against-Rust whitespace and case difference.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Query = { kind: 'last-versions'; browser: Browser } | { kind: 'firefox-esr' } | { kind: 'node-major' } | { kind: 'not-dead' };
//! ```

/// What:
///  Import `Outcome, not_implemented` from `crate::error`.
/// Why:
///  Failures use the crate's single error type,
/// so callers match one thing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Outcome, notImplemented } from './error';
/// ```
use crate::error::{Outcome, not_implemented};

/// What:
///  The browsers a supported `last 1 <browser> versions` line can name.
/// Why:
///  An enum makes an unsupported browser unrepresentable after parsing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Browser = 'Chrome' | 'ChromeAndroid' | 'Android' | 'FirefoxAndroid';
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Browser {
    /// Desktop Chrome,
    /// caniuse key `chrome`.
    Chrome,
    /// Chrome for Android,
    /// browserslist alias `chromeandroid`,
    /// caniuse key `and_chr`.
    ChromeAndroid,
    /// The Android system browser,
    /// caniuse key `android`.
    Android,
    /// Firefox for Android,
    /// browserslist alias `firefoxandroid`,
    /// caniuse key `and_ff`.
    FirefoxAndroid,
}

/// What:
///  Methods of `Browser`.
/// Why:
///  The resolver turns a parsed browser into its caniuse-lite key here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function agentKey(browser: Browser): string
/// ```
impl Browser {
    /// The caniuse-lite key browserslist resolves the written name to.
    pub fn agent_key(self) -> &'static str {
        match self {
            Self::Chrome => return "chrome",
            Self::ChromeAndroid => return "and_chr",
            Self::Android => return "android",
            Self::FirefoxAndroid => return "and_ff",
        }
    }
}

/// What:
///  One supported query.
/// Why:
///  The resolver matches on this closed set and nothing else can reach it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Query = { kind: 'last-versions'; browser: Browser } | { kind: 'firefox-esr' } | { kind: 'node-major' } | { kind: 'not-dead' };
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Query {
    /// `last 1 <browser> versions`.
    LastVersions(
        /// The browser the line names.
        Browser,
    ),
    /// `Firefox ESR`.
    FirefoxEsr,
    /// `last 1 node major versions`.
    LastNodeMajorVersions,
    /// `not dead`.
    NotDead,
}

/// What:
///  The exact text of each supported query paired with its meaning.
///  A fixed-size array `[T; 7]` (not a `Vec<T>`) because the set never changes at run time.
/// Why:
///  Matching by whole-string equality is the whole parser.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SUPPORTED: readonly [string, Query][] = [['last 1 Chrome versions', { kind: 'last-versions', browser: 'Chrome' }], /* ... */];
/// ```
const SUPPORTED: [(&str, Query); 7] = [
    ("last 1 Chrome versions", Query::LastVersions(Browser::Chrome)),
    ("last 1 ChromeAndroid versions", Query::LastVersions(Browser::ChromeAndroid)),
    ("last 1 Android versions", Query::LastVersions(Browser::Android)),
    ("Firefox ESR", Query::FirefoxEsr),
    ("last 1 FirefoxAndroid versions", Query::LastVersions(Browser::FirefoxAndroid)),
    ("last 1 node major versions", Query::LastNodeMajorVersions),
    ("not dead", Query::NotDead),
];

/// What:
///  Match a trimmed config line against the supported texts,
/// or fail with a "not implemented" error naming the line.
/// Why:
///  A near miss (another count,
/// another case,
/// another browser) must be refused,
///  never approximated as the closest supported form.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseQuery(lineNumber: number, text: string): Query; // throws NotImplemented
/// ```
pub fn parse_query(line_number: usize, text: &str) -> Outcome<Query> {
    for (supported_text, query) in SUPPORTED {
        if text == supported_text {
            return Ok(query);
        }
    }
    return Err(not_implemented(&format!(
        "query {text:?} on line {line_number} of the browserslist config; only the exact text of the seven query lines \
         of this repository's .browserslistrc is supported (last 1 Chrome versions, last 1 ChromeAndroid versions, \
         last 1 Android versions, Firefox ESR, last 1 FirefoxAndroid versions, last 1 node major versions, not dead)"
    )));
}

/// What:
///  The unit tests of `query.rs`,
/// kept in the sidecar file `query_tests.rs`.
/// Why:
///  The repository keeps Rust tests beside the code they cover,
/// out of the source file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import './query.test';
/// ```
#[cfg(test)]
#[path = "query_tests.rs"]
mod tests;
