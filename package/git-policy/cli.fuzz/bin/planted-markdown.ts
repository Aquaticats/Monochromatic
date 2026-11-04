/**
 Planted defects for the Markdown policy's invariants, spread into the list in `planted-controls.ts`.
 Each `from` must occur exactly once in its file.
 */
import type { Plant } from './planted-table.ts';

/**
 Linter output, event path and configuration defects, one per invariant.
 */
export const markdownPlants: readonly Plant[] = [
  {
    name: 'a linter finding of another severity is accepted',
    file: 'package/git-policy/cli/src/native/markdown/markdown_linter_output.rs',
    edits: [{
      from: 'if finding.severity != Severity::Warn {',
      to: 'if false && finding.severity != Severity::Warn {',
    }],
  },
  {
    name: 'an empty fixed source for a file that is not empty is used',
    file: 'package/git-policy/cli/src/native/markdown/markdown_linter_output.rs',
    edits: [{
      from: 'if run.fixed.is_empty() && !input.is_empty() {',
      to: 'if run.fixed.is_empty() && input.is_empty() {',
    }],
  },
  {
    name: 'a finding of a rule whose name only ends like the selected one is used',
    file: 'package/git-policy/cli/src/native/markdown/markdown_linter_output.rs',
    edits: [{
      from: 'if code.strip_prefix("markdown/") == Some(markdown_rule_name(*rule)) {',
      to: 'if code.ends_with(markdown_rule_name(*rule)) {',
    }],
  },
  {
    name: 'the last label of a finding gives its position',
    file: 'package/git-policy/cli/src/native/markdown/markdown_linter_output.rs',
    edits: [{
      from: 'let Some(label) = finding.labels.first() else {',
      to: 'let Some(label) = finding.labels.last() else {',
    }],
  },
  {
    name: 'a name that is not UTF-8 keeps no exact bytes',
    file: 'package/git-policy/cli/src/native/policy/event_path.rs',
    edits: [{
      from: 'exact: Some(bytes.to_vec()),',
      to: 'exact: None,'
    }],
  },
  {
    name: 'base64 fills the padding of a two-byte group',
    file: 'package/git-policy/cli/src/native/policy/event_path.rs',
    edits: [{
      from: 'if group.len() > 2 {',
      to: 'if group.len() > 1 {'
    }],
  },
  {
    name: 'only the first exclude pattern reaches the linter configuration',
    file: 'package/git-policy/cli/src/native/markdown/markdown_linter_config.rs',
    edits: [{
      from: 'for pattern in exclude {',
      to: 'for pattern in exclude.iter().take(1) {'
    }],
  },
];
