/**
 Planted defects for the Markdown policy's invariants, spread into the list in `planted-controls.mjs`.
 Each `from` must occur exactly once in its file.
 */

/** Linter output, event path and configuration defects, one per invariant. */
export const markdownPlants = [
  {
    name: 'a linter record of another severity is accepted',
    file: 'package/git-policy/cli/src/native/markdown_linter_output.rs',
    edits: [{
      from: 'if required_text(entries, "severity")? != "warn" {',
      to: 'if required_text(entries, "severity")? == "error" {',
    }],
  },
  {
    name: 'linter standard error without a final line feed is accepted',
    file: 'package/git-policy/cli/src/native/markdown_linter_output.rs',
    edits: [{
      from: String.raw`let Some(body) = text.strip_suffix('\n') else {`,
      to: String.raw`let Some(body) = text.strip_suffix('\n').or(Some(text)) else {`,
    }],
  },
  {
    name: 'a linter run that exited 1 is used',
    file: 'package/git-policy/cli/src/native/markdown_linter_output.rs',
    edits: [{
      from: 'ChildExit::Code(0) => {}',
      to: 'ChildExit::Code(0) | ChildExit::Code(1) => {}'
    }],
  },
  {
    name: 'a name that is not UTF-8 keeps no exact bytes',
    file: 'package/git-policy/cli/src/native/event_path.rs',
    edits: [{
      from: 'exact: Some(bytes.to_vec()),',
      to: 'exact: None,'
    }],
  },
  {
    name: 'base64 fills the padding of a two-byte group',
    file: 'package/git-policy/cli/src/native/event_path.rs',
    edits: [{
      from: 'if group.len() > 2 {',
      to: 'if group.len() > 1 {'
    }],
  },
  {
    name: 'only the first exclude pattern reaches the linter configuration',
    file: 'package/git-policy/cli/src/native/markdown_linter_config.rs',
    edits: [{
      from: 'for pattern in exclude {',
      to: 'for pattern in exclude.iter().take(1) {'
    }],
  },
];
