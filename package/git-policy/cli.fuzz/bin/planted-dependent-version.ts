/**
 Planted defects for the dependent-version invariants, spread into the list in `planted-controls.mjs`.
 Each `from` must occur exactly once in its file.
 */

/** A nested key is rewritten, a subpath import is missed, and an escaped quote ends a string literal. */
export const dependentVersionPlants = [
  {
    name: 'a nested version key is rewritten',
    file: 'package/git-policy/cli/src/native/dependent_version_text.rs',
    edits: [{
      from: 'if depth == 1',
      to: 'if depth >= 1'
    }],
  },
  {
    name: 'a subpath import is missed',
    file: 'package/git-policy/cli/src/native/dependent_version_imports.rs',
    edits: [{
      from: "(following == Some(&quote) || following == Some(&b'/'))",
      to: '(following == Some(&quote))'
    }],
  },
  {
    name: 'an escaped quote ends a string literal',
    file: 'package/git-policy/cli/src/native/dependent_version_text.rs',
    edits: [{
      from: 'escaped = true;',
      to: 'escaped = false;'
    }],
  },
];
