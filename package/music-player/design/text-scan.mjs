// Linear scans of authored text, with no regular expression.
// What: the cursor scans behind the checks the music-player design tools run over a page, a page
// template or a builder's own source.
// Why: each check is a literal test, a delimiter walk or a word-boundary count, and one cursor pass
// answers it in the same O(n) time the pattern it replaced took, so no pattern earns its place under
// the repo's regex-necessity criterion.
// Order: predicates, needRuleNames, ruleNameAt, skipWhitespace.

//region Character predicates

const UNDERSCORE = '_'.charCodeAt(0);
const DIGIT_ZERO = '0'.charCodeAt(0);
const DIGIT_NINE = '9'.charCodeAt(0);
const UPPER_A = 'A'.charCodeAt(0);
const UPPER_Z = 'Z'.charCodeAt(0);
const LOWER_A = 'a'.charCodeAt(0);
const LOWER_Z = 'z'.charCodeAt(0);
const CASE_OFFSET = LOWER_A - UPPER_A;

// The ECMAScript `\s` set, written out because a pattern is the one place it used to live.
const WHITESPACE = new Set([
  '\u0009',
  '\u000A',
  '\u000B',
  '\u000C',
  '\u000D',
  '\u0020',
  '\u00A0',
  '\u1680',
  '\u2000',
  '\u2001',
  '\u2002',
  '\u2003',
  '\u2004',
  '\u2005',
  '\u2006',
  '\u2007',
  '\u2008',
  '\u2009',
  '\u200A',
  '\u2028',
  '\u2029',
  '\u202F',
  '\u205F',
  '\u3000',
  '\uFEFF'
]);

// Whether `code` is a word character as a non-Unicode word boundary reads it: an ASCII letter, an
// ASCII digit or an underscore.
function isWordCode(code) {
  return code === UNDERSCORE
    || ((code >= DIGIT_ZERO) && (code <= DIGIT_NINE))
    || ((code >= UPPER_A) && (code <= UPPER_Z))
    || ((code >= LOWER_A) && (code <= LOWER_Z));
}

// Whether a word boundary holds at `index` in `text`, which needs a word character on exactly one
// side of the position.
function isBoundaryAt({
  text,
  index
}) {
  const before = index > 0 ? isWordCode(text.charCodeAt(index - 1)) : false;
  const after = index < text.length ? isWordCode(text.charCodeAt(index)) : false;
  return before !== after;
}

// Whether `text` holds a whitespace character at `index`.
function isWhitespaceAt({
  text,
  index
}) {
  return (index < text.length) && WHITESPACE.has(text[index]);
}

// Whether `code` is an ASCII lower-case letter or a hyphen, the run `[a-z-]` reads.
function isLowercaseRunCode(code) {
  return ((code >= LOWER_A) && (code <= LOWER_Z)) || (code === '-'.charCodeAt(0));
}
//endregion

//region ASCII case folding

// Whether `literal` appears at `index` in `text`, comparing ASCII letters without case.
// What: a per-character comparison that folds only A to Z.
// Why: a non-Unicode `/i` pattern folds only ASCII letters onto each other, so folding anything else
// (JavaScript's `toLowerCase` maps KELVIN SIGN to `k`) would accept text the pattern rejected.
// `literal` must already be lower case.
export function asciiEqualsAt({
  text,
  index,
  literal
}) {
  if ((index < 0) || ((index + literal.length) > text.length)) return false;
  for (let offset = 0; offset < literal.length; offset += 1) {
    let code = text.charCodeAt(index + offset);
    if ((code >= UPPER_A) && (code <= UPPER_Z)) code += CASE_OFFSET;
    if (code !== literal.charCodeAt(offset)) return false;
  }
  return true;
}
//endregion

//region Builder source fields

const NEED = 'need(';
const RULE_FIELD = 'rule:';

// Every rule name a `need({ rule: '<name>',` call in `source` declares, in the order they appear.
// What: find `need(`, then read `{`, `rule:`, then the quoted name, tolerating any whitespace between
// them; a call that does not declare a rule in that shape is skipped.
// Why: `need\(\{ rule: '([a-z-]+)',` with `g` read that field, but it pinned one space after `{`, so a
// reformatted builder that puts `rule:` on its own line silently declares no rules at all. The field
// is the contract here, not its layout.
export function* needRuleNames({ source }) {
  let from = 0;
  while (from < source.length) {
    const at = source.indexOf(
      NEED,
      from
    );
    if (at === (-1)) return;
    from = at + NEED.length;
    const name = ruleNameAt({
      source,
      callStart: at
    });
    if (name !== '') yield name;
  }
}

// The rule name the `need(` call starting at `callStart` declares, or the empty string when the call
// is not the `need({ rule: '<name>'` shape.
// What: skip whitespace, then read `{`, `rule:`, `'`, a run of ASCII lower-case letters and hyphens,
// and the closing `'`.
// Why: the name is what the builder and its test both key on, so the test can name the call it must
// delete without depending on how the call is laid out.
export function ruleNameAt({
  source,
  callStart
}) {
  let index = skipWhitespace({
    source,
    index: callStart + NEED.length
  });
  if (source[index] !== '{') return '';
  index = skipWhitespace({
    source,
    index: index + 1
  });
  if (!asciiEqualsAt({
    text: source,
    index,
    literal: RULE_FIELD
  })) return '';
  index = skipWhitespace({
    source,
    index: index + RULE_FIELD.length
  });
  if (source[index] !== "'") return '';
  const start = index + 1;
  let end = start;
  while ((end < source.length) && isLowercaseRunCode(source.charCodeAt(end))) end += 1;
  if ((end === start) || (source[end] !== "'")) return '';
  return source.slice(
    start,
    end
  );
}

// Where the first non-whitespace character at or after `index` sits.
function skipWhitespace({
  source,
  index
}) {
  let cursor = index;
  while ((cursor < source.length) && WHITESPACE.has(source[cursor])) cursor += 1;
  return cursor;
}
//endregion
