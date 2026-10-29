//! Hybrid runtime matcher for exact literals and restricted regular expressions.
//!
//! Bare literals stay out of `RegexSet`:
//!  one Aho-Corasick automaton handles them
//! directly,
//!  while explicit and multiline regex rules retain the in-house engine.
//! Both subset-local matchers map back to original runtime rule ids before findings.

/// Imports overlapping multi-literal matcher.
use aho_corasick::{AhoCorasick, MatchKind};
/// Imports engine set and static compile reason.
use forbidden_regex::{CompileError, RegexSet};
/// Imports literal de-duplication map.
use std::collections::HashMap;

/// Imports parser output retaining exact-literal kind.
use crate::rule::frx::{parse_runtime_rules, LoadError, RuntimeRuleInput, RuntimeRuleKind};

/// Bare literals shorter than this byte count retain incumbent word-boundary gating.
const SHORT_LITERAL_THRESHOLD: usize = 8;

/// One unique literal plus every original rule id declaring it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LiteralGroup {
    /// Exact UTF-8 source bytes matched without dialect interpretation.
    pub(crate) bytes: Vec<u8>,
    /// Original runtime rule ids reported when literal satisfies boundaries.
    pub(crate) rule_ids: Vec<usize>,
}

/// Redacted hybrid matcher construction failure.
#[derive(Debug)]
pub(crate) enum RuntimeMatcherError {
    /// Existing rule-format parser rejected source.
    Load(
        /// Redacted parser reason.
        LoadError,
    ),
    /// Existing regex engine rejected original rule id.
    Regex {
        /// Original runtime rule id,
        ///  not subset-local index.
        index: usize,
        /// Static engine reason that never carries pattern bytes.
        reason: CompileError,
    },
    /// Combined regex-only set failed after every individual rule validated.
    RegexSet {
        /// Static set-level engine reason without fabricated rule index.
        reason: CompileError,
    },
    /// Aho-Corasick could not build literal automaton.
    LiteralBuild,
    /// Decoded artifact mappings or counts disagree.
    InvalidArtifact,
}

/// One runtime set exposing global-id batch matching behind small interface.
pub(crate) struct RuntimeRules {
    /// Rule identities parallel to original global runtime ids.
    names: Vec<Option<String>>,
    /// Unique exact literals and original id fanout.
    literal_groups: Vec<LiteralGroup>,
    /// Runtime-only automaton rebuilt from literal groups.
    literal_matcher: Option<AhoCorasick>,
    /// Precompiled set containing only restricted-regex subset.
    regex_set: Option<RegexSet>,
    /// Regex subset-local id to original runtime id.
    regex_rule_ids: Vec<usize>,
}

/// Hybrid matcher construction,
///  artifact projection,
///  and scan behavior.
impl RuntimeRules {
    /// Parses and compiles authoritative runtime source.
    pub(crate) fn compile(text: &str) -> Result<Self, RuntimeMatcherError> {
        let inputs = parse_runtime_rules(text).map_err(RuntimeMatcherError::Load)?;
        return Self::from_inputs(inputs)
    }

    /// Rebuilds runtime-only Aho matcher around decoded artifact data.
    pub(crate) fn from_artifact(
        names: Vec<Option<String>>,
        literal_groups: Vec<LiteralGroup>,
        regex_set: Option<RegexSet>,
        regex_rule_ids: Vec<usize>,
    ) -> Result<Self, RuntimeMatcherError> {
        validate_mappings(&names, &literal_groups, regex_set.as_ref(), &regex_rule_ids)?;
        let literal_matcher = build_literal_matcher(&literal_groups)?;
        return Ok(Self {
            names,
            literal_groups,
            literal_matcher,
            regex_set,
            regex_rule_ids,
        })
    }

    /// Builds both matcher subsets from parsed rules in original order.
    fn from_inputs(inputs: Vec<RuntimeRuleInput>) -> Result<Self, RuntimeMatcherError> {
        let names: Vec<Option<String>> = inputs
            .iter()
            .map(|input| return input.name.clone())
            .collect();
        let mut literal_groups: Vec<LiteralGroup> = Vec::new();
        let mut literal_indices: HashMap<Vec<u8>, usize> = HashMap::new();
        let mut regex_patterns: Vec<String> = Vec::new();
        let mut regex_rule_ids: Vec<usize> = Vec::new();

        for (rule_id, input) in inputs.into_iter().enumerate() {
            match input.kind {
                RuntimeRuleKind::ExactLiteral(bytes) => {
                    if let Some(&group_index) = literal_indices.get(&bytes) {
                        literal_groups[group_index].rule_ids.push(rule_id);
                    } else {
                        let group_index = literal_groups.len();
                        literal_indices.insert(bytes.clone(), group_index);
                        literal_groups.push(LiteralGroup { bytes, rule_ids: vec![rule_id] });
                    }
                }
                RuntimeRuleKind::RestrictedRegex(pattern) => {
                    if let Err(reason) = RegexSet::new(std::slice::from_ref(&pattern)) {
                        return Err(RuntimeMatcherError::Regex { index: rule_id, reason });
                    }
                    regex_patterns.push(pattern);
                    regex_rule_ids.push(rule_id);
                }
            }
        }

        let regex_set = if regex_patterns.is_empty() {
            None
        } else {
            Some(RegexSet::new(&regex_patterns).map_err(|reason| {
                return RuntimeMatcherError::RegexSet { reason }
            })?)
        };
        return Self::from_artifact(names, literal_groups, regex_set, regex_rule_ids)
    }

    /// Returns ordered rule identities for finding attribution.
    pub(crate) fn names(&self) -> &[Option<String>] {
        return &self.names
    }

    /// Returns unique literal groups for artifact encoding.
    pub(crate) fn literal_groups(&self) -> &[LiteralGroup] {
        return &self.literal_groups
    }

    /// Returns optional regex-only engine set for artifact encoding.
    pub(crate) fn regex_set(&self) -> Option<&RegexSet> {
        return self.regex_set.as_ref()
    }

    /// Returns regex subset mapping for artifact encoding.
    pub(crate) fn regex_rule_ids(&self) -> &[usize] {
        return &self.regex_rule_ids
    }

    /// Returns original runtime rule count.
    pub(crate) fn len(&self) -> usize {
        return self.names.len()
    }

    /// Reports every `(line index, original rule id)` pair in stable order.
    pub(crate) fn line_matches(&self, buf: &[u8], starts: &[usize]) -> Vec<(usize, usize)> {
        let mut hits = self.regex_pairs(buf, starts);
        for line_index in 0..starts.len() {
            let start = starts[line_index];
            let end = line_end(buf, starts, line_index);
            if end == start {
                continue;
            }
            self.append_literal_matches(&buf[start..end], line_index, &mut hits);
        }
        hits.sort_unstable();
        hits.dedup();
        return hits
    }

    /// Maps regex subset pairs back to original runtime ids.
    fn regex_pairs(&self, buf: &[u8], starts: &[usize]) -> Vec<(usize, usize)> {
        let Some(regex_set) = &self.regex_set else {
            return Vec::new();
        };
        return regex_set
            .line_matches(buf, starts)
            .into_iter()
            .map(|(line, local_rule)| return (line, self.regex_rule_ids[local_rule]))
            .collect()
    }

    /// Appends boundary-valid literal matches for one line.
    fn append_literal_matches(
        &self,
        line: &[u8],
        line_index: usize,
        hits: &mut Vec<(usize, usize)>,
    ) {
        let Some(matcher) = &self.literal_matcher else {
            return;
        };
        for found in matcher.find_overlapping_iter(line) {
            let group_index = found.pattern().as_usize();
            let group = &self.literal_groups[group_index];
            if literal_boundaries_match(line, found.start(), found.end(), &group.bytes) {
                hits.extend(group.rule_ids.iter().map(|&rule_id| return (line_index, rule_id)));
            }
        }
    }
}

/// Builds overlapping standard matcher over unique literal groups.
fn build_literal_matcher(
    groups: &[LiteralGroup],
) -> Result<Option<AhoCorasick>, RuntimeMatcherError> {
    if groups.is_empty() {
        return Ok(None);
    }
    let patterns: Vec<&[u8]> = groups.iter().map(|group| return group.bytes.as_slice()).collect();
    let matcher = AhoCorasick::builder()
        .match_kind(MatchKind::Standard)
        .build(&patterns)
        .map_err(|_| return RuntimeMatcherError::LiteralBuild)?;
    return Ok(Some(matcher))
}

/// Validates decoded global-id mappings before matcher construction.
fn validate_mappings(
    names: &[Option<String>],
    groups: &[LiteralGroup],
    regex_set: Option<&RegexSet>,
    regex_rule_ids: &[usize],
) -> Result<(), RuntimeMatcherError> {
    if regex_set.map_or(0, RegexSet::len) != regex_rule_ids.len() {
        return Err(RuntimeMatcherError::InvalidArtifact);
    }
    let rule_count = names.len();
    let mut seen_rule_ids = vec![false; rule_count];
    for &rule_id in regex_rule_ids {
        if rule_id >= rule_count || seen_rule_ids[rule_id] {
            return Err(RuntimeMatcherError::InvalidArtifact);
        }
        seen_rule_ids[rule_id] = true;
    }
    let mut seen_literals: std::collections::HashSet<&[u8]> = std::collections::HashSet::new();
    for group in groups {
        if group.bytes.is_empty()
            || group.rule_ids.is_empty()
            || !seen_literals.insert(&group.bytes)
        {
            return Err(RuntimeMatcherError::InvalidArtifact);
        }
        for &rule_id in &group.rule_ids {
            if rule_id >= rule_count || seen_rule_ids[rule_id] {
                return Err(RuntimeMatcherError::InvalidArtifact);
            }
            seen_rule_ids[rule_id] = true;
        }
    }
    if seen_rule_ids.iter().any(|seen| return !seen) {
        return Err(RuntimeMatcherError::InvalidArtifact);
    }
    return Ok(())
}

/// Returns line content end excluding one LF and one preceding CR.
fn line_end(buf: &[u8], starts: &[usize], line_index: usize) -> usize {
    let start = starts[line_index];
    let mut end = starts.get(line_index + 1).copied().unwrap_or(buf.len());
    if end > start && buf[end - 1] == b'\n' {
        end -= 1;
    }
    if end > start && buf[end - 1] == b'\r' {
        end -= 1;
    }
    return end
}

/// Reports whether byte belongs to incumbent ASCII word-boundary alphabet.
fn is_word_byte(byte: u8) -> bool {
    return byte.is_ascii_alphanumeric() || byte == b'_'
}

/// Applies short bare-literal boundary semantics to one exact Aho match.
fn literal_boundaries_match(
    line: &[u8],
    start: usize,
    end: usize,
    literal: &[u8],
) -> bool {
    if literal.len() >= SHORT_LITERAL_THRESHOLD {
        return true;
    }
    let left_ok = !is_word_byte(literal[0]) || start == 0 || !is_word_byte(line[start - 1]);
    let right_ok = !is_word_byte(literal[literal.len() - 1])
        || end == line.len()
        || !is_word_byte(line[end]);
    return left_ok && right_ok
}

/// Renders only redacted construction reasons.
impl std::fmt::Display for RuntimeMatcherError {
    /// Writes existing redacted parser or engine reason without literal bytes.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Load(error) => return write!(formatter, "{error}"),
            Self::Regex { index, reason } => return write!(formatter, "rule {index}: {reason}"),
            Self::RegexSet { reason } => return write!(formatter, "runtime regex set: {reason}"),
            Self::LiteralBuild => return formatter.write_str("runtime literal matcher build failed"),
            Self::InvalidArtifact => return formatter.write_str("runtime matcher artifact is invalid"),
        }
    }
}

/// Standard error marker for runtime matcher construction.
impl std::error::Error for RuntimeMatcherError {}

/// Registers hybrid matcher semantic tests.
#[cfg(test)]
#[path = "runtime_matcher_tests.rs"]
mod tests;
