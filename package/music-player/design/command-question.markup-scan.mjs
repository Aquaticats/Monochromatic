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
