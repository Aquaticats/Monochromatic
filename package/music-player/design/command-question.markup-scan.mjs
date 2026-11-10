// Text scans over the built command questionnaire page.
//
// Each scan answers one question the withdrawn patterns asked about the page's
// markup: a fixed literal set, a fixed alphabet run, or a fixed tag shape. None
// of them needs a pattern engine, and each states its rule in code.
//
//region The clipboard guard reads literals and one opening call, not a pattern
/** Characters ECMAScript's `\s` matches, so the scan folds the whitespace a pattern did. */
const WHITESPACE = new Set([
  ' ',
  '\t',
  '\n',
  '\v',
  '\f',
  '\r',
  '\u00a0',
  '\u1680',
  '\u2028',
  '\u2029',
  '\u202f',
  '\u205f',
  '\u3000',
  '\ufeff',
]);
// EN QUAD through HAIR SPACE are the remaining code points `\s` admits.
for (let codePoint = 0x2000; codePoint <= 0x200a; codePoint += 1) WHITESPACE.add(String.fromCharCode(codePoint));

/** Opens the clipboard write call the guard refuses, whatever spacing follows it. */
const EXEC_COMMAND_OPEN = 'execCommand(';

/** Quotes the copy command after that opening call. */
const COPY_COMMAND = 'copy';

/** Names the clipboard objects a questionnaire must not reach for. */
const CLIPBOARD_OBJECTS = [
  'navigator.clipboard',
  'clipboardData',
];

/**
 * Answers whether one whitespace run and one quoted `copy` follow the opening
 * call at `openAt`, resuming the search past it when they do not.
 * @param text - page markup to read
 * @param openAt - index of `EXEC_COMMAND_OPEN` inside `text`
 * @returns offset of the opening call whose command is a quoted copy, or -1
 */
function execCommandCopyAt(text, openAt) {
  let quoteAt = openAt + EXEC_COMMAND_OPEN.length;
  while (quoteAt < text.length && WHITESPACE.has(text[quoteAt])) quoteAt += 1;
  const quote = text.at(quoteAt);
  if ((quote !== "'") && (quote !== '"')) return -1;
  if (!text.startsWith(COPY_COMMAND, quoteAt + 1)) return -1;
  return openAt;
}

/** Rejects a page that reaches for a clipboard object or runs a copy command. */
export function usesClipboardApi(text) {
  if (CLIPBOARD_OBJECTS.some((object) => text.includes(object))) return true;
  let cursor = 0;
  for (;;) {
    const openAt = text.indexOf(EXEC_COMMAND_OPEN, cursor);
    if (openAt === -1) return false;
    if (execCommandCopyAt(text, openAt) !== -1) return true;
    cursor = openAt + 1;
  }
}
//endregion

//region The raster guard reads the prefix and its alphabet run, not a pattern
/** Prefix every embedded raster carries before its base64 payload. */
const PNG_DATA_PREFIX = 'data:image/png;base64,';

/** Characters a base64 payload may carry, padding included. */
const BASE64_ALPHABET = new Set('ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=');

/**
 * Reads every embedded raster payload, in page order.
 * The prefix has no proper prefix that is also a suffix, so a prefix with no
 * payload after it resumes past the whole prefix without skipping a later match.
 */
export function pngPayloads(text) {
  const payloads = [];
  let cursor = 0;
  for (;;) {
    const prefixAt = text.indexOf(PNG_DATA_PREFIX, cursor);
    if (prefixAt === -1) return payloads;
    const payloadAt = prefixAt + PNG_DATA_PREFIX.length;
    let end = payloadAt;
    while (end < text.length && BASE64_ALPHABET.has(text[end])) end += 1;
    if (end > payloadAt) payloads.push(text.slice(payloadAt, end));
    cursor = end;
  }
}
//endregion

//region The preselect guard reads one input tag, not a pattern
/** Opens the tag a preselection would sit in. */
const INPUT_TAG_OPEN = '<input';

/** Names the choice a questionnaire must not preselect. */
const RADIO_TYPE_ATTRIBUTE = 'type="radio"';

/** Marks a control the page rendered as chosen. */
const CHECKED_ATTRIBUTE = 'checked';

/**
 * Rejects a page whose radio input arrives already checked.
 * A tag ends at its next `>`, so both attributes have to sit inside that one tag
 * and in that order, which is exactly what the withdrawn pattern required.
 */
export function hasPreselectedRadio(text) {
  let cursor = 0;
  for (;;) {
    const openAt = text.indexOf(INPUT_TAG_OPEN, cursor);
    if (openAt === -1) return false;
    const closeAt = text.indexOf('>', openAt);
    const tag = text.slice(openAt, closeAt === -1 ? text.length : closeAt);
    const typeAt = tag.indexOf(RADIO_TYPE_ATTRIBUTE);
    if (typeAt !== -1 && tag.indexOf(CHECKED_ATTRIBUTE, typeAt + RADIO_TYPE_ATTRIBUTE.length) !== -1) return true;
    cursor = openAt + INPUT_TAG_OPEN.length;
  }
}
//endregion

//region Occurrence counts read a literal, not a pattern
/**
 * Counts non-overlapping occurrences of one literal, left to right.
 * Each occurrence consumes its own length before the search resumes, which is
 * how a global match counted, so `aa` in `aaa` is one occurrence and not two.
 */
export function occurrencesOf(text, literal) {
  if (literal === '') throw new Error('occurrencesOf counts a literal, so it cannot read an empty one.');
  let count = 0;
  let cursor = 0;
  for (;;) {
    const at = text.indexOf(literal, cursor);
    if (at === -1) return count;
    count += 1;
    cursor = at + literal.length;
  }
}
//endregion
