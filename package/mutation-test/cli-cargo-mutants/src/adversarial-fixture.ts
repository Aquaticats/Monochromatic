/**
 Values that stress every place a file name or option value is interpolated: spaces, both quote kinds, `$`, command
 separators, command substitution, globs, backslashes, a leading hyphen, an empty-looking value and newlines.
 */
export const adversarialValues: readonly string[] = [
  'plain.rs',
  'with space.rs',
  `single'quote.rs`,
  'double"quote.rs',
  'dollar$HOME.rs',
  'semi;colon.rs',
  'amp&&pipe||bar|.rs',
  '`backtick`.rs',
  '$(substitution).rs',
  'glob*?[x].rs',
  String.raw`back\slash.rs`,
  '-leading-hyphen.rs',
  'new\nline.rs',
  'carriage\rreturn.rs',
  `'; touch pwned; echo '`,
  'tab\tseparated.rs',
  'dir/nested/file.rs',
  'unicode-é中.rs',
];
