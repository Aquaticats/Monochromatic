/**
 Single-line values that stress every place a release note or name reaches a command line, a commit subject or a
 terminal: quotes of both kinds, `$` expansions, command separators, substitution, globs, backslashes, option-looking
 text, and non-ASCII text.
 */
export const adversarialLines: readonly string[] = [
  'plain note',
  `it's quoted`,
  'say "hello"',
  'costs $5 and $HOME',
  'semi;colon; touch pwned',
  'amp && pipe || bar | tee',
  '`backtick`',
  '$(substitution)',
  'glob * ? [x]',
  String.raw`back\slash and \n literal`,
  '--looks-like-an-option',
  '-x',
  `'; touch pwned; echo '`,
  'unicode é 中 🙂',
  '%s %d %%',
  '# not a comment',
  'trailing space ',
  ' leading space',
];

/**
 Values that hold a line break or other control character, which no single-line commit subject can hold.
 */
export const controlCharacterValues: readonly string[] = [
  'line one\nline two',
  'carriage\rreturn',
  'tab\tseparated',
  'nul\0byte',
  'escape\u001bsequence',
  'delete\u007fcharacter',
  '\n',
];
