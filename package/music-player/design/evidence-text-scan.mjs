// Regex-free scans over the text of a generated design review.
//
// The review builders in `package/music-player/design` validate their own output before it is committed. Every check
// here answers `package/config/oxlint`'s `no-restricted-syntax/no-regex`: that rule asks for an index scan, a parser or
// a string API instead of a pattern, and these functions are the index scans the validators share.
//
// Each scan walks its input once from left to right and keeps no backtracking state, so its cost is linear in the
// document length for any input.
//
// This module is `.mjs` rather than `.ts` because the package's fixture harnesses run copies of a builder from
// `node_modules/.monochromatic`, and Node refuses to strip types for any file under `node_modules`
// (`ERR_UNSUPPORTED_NODE_MODULES_TYPE_STRIPPING`). A `.ts` sibling of a copied builder cannot load there.
// `doc/troubleshooting/node-type-stripping-under-node-modules.md` records the probe.

//region Constants

// Literal that opens an embedded base64 PNG payload inside a `data:` URL.
const PNG_DATA_URL_PREFIX = 'data:image/png;base64,';

// Characters the base64 alphabet adds to the ASCII letters and digits: the two extra symbols and the padding
// character.
const BASE64_SYMBOLS = '+/=';

// Fixed head every search-filename capture name starts with.
const SEARCH_FILENAME_CAPTURE_PREFIX = 'search-filename-';

// Fixed tail every search-filename capture name ends with.
const SEARCH_FILENAME_CAPTURE_SUFFIX = '.png';

// Source axes the filename study captured, each naming one side of its comparison.
const SEARCH_FILENAME_CAPTURE_SOURCES = [
  'comparison',
  'actual-player'
];

//endregion Constants

//region ASCII case folding

// Lower-cases one ASCII letter and leaves every other code unit alone.
//
// A Unicode `toLowerCase()` would fold code points the validators never wrote, and the `i` flag these scans replace
// only ever decided ASCII letters in the literals they matched.
function asciiLoweredCharacter({ character }) {
  if ((character >= 'A') && (character <= 'Z'))
    return String.fromCharCode((character.charCodeAt(0) + ('a'.charCodeAt(0) - 'A'.charCodeAt(0))));
  return character;
}

// Whether `needle` sits at `at` in `haystack` under the requested ASCII case folding.
export function matchesAtWithin({ haystack, needle, at, ignoreAsciiCase }) {
  if ((at < 0) || ((at + needle.length) > haystack.length))
    return false;
  for (let offset = 0; offset < needle.length; offset += 1) {
    const haystackCharacter = haystack.charAt(at + offset);
    const needleCharacter = needle.charAt(offset);
    if (needleCharacter === haystackCharacter)
      continue;
    if (!ignoreAsciiCase)
      return false;
    if (asciiLoweredCharacter({ character: haystackCharacter })
      !== asciiLoweredCharacter({ character: needleCharacter }))
      return false;
  }
  return true;
}

// First position in `[from, to - needle.length]` where `needle` occurs, or -1 when it occurs nowhere in that range.
export function findWithin({ haystack, needle, from, to, ignoreAsciiCase }) {
  for (let at = from; (at + needle.length) <= to; at += 1) {
    if (matchesAtWithin({
      haystack,
      needle,
      at,
      ignoreAsciiCase
    }))
      return at;
  }
  return -1;
}

// Whether the document holds any of `candidates`.
//
// The pattern an alternation of literals wrote becomes a list of those literals: one left-to-right pass per candidate
// over the same text, which is the order the alternation tried them in and keeps the cost linear.
export function containsAnyLiteral({ text, candidates, ignoreAsciiCase }) {
  for (const candidate of candidates) {
    if (candidate.length === 0)
      return true;
    if (findWithin({
      haystack: text,
      needle: candidate,
      from: 0,
      to: text.length,
      ignoreAsciiCase
    })
    !== -1)
      return true;
  }
  return false;
}

//endregion ASCII case folding

//region Base64 payloads

// Whether one UTF-16 code unit belongs to the base64 alphabet.
function isBase64Character({ character }) {
  if (character === '')
    return false;
  if ((character >= 'A') && (character <= 'Z'))
    return true;
  if ((character >= 'a') && (character <= 'z'))
    return true;
  if ((character >= '0') && (character <= '9'))
    return true;
  return BASE64_SYMBOLS.includes(character);
}

// Every base64 PNG payload a document embeds, in document order.
//
// The regular expression this replaced, `[...text.matchAll(/data:image\/png;base64,([A-Za-z0-9+/=]+)/g)]`, reported
// the maximal base64 run after each prefix and resumed after that run, so a payload was never reported twice and a
// prefix with no base64 character after it reported nothing. This scan keeps both properties: the run is maximal
// because the walk stops at the first code unit outside the alphabet, and the next search resumes one code unit past a
// prefix that carried no payload, which is the position the pattern engine itself retried from.
export function findPngDataPayloads({ text }) {
  const payloads = [];
  // A `for` initializer rather than a function-root `let`: the counter belongs to the loop, and
  // `no-restricted-syntax/no-function-root-let` keeps a mutable root binding from leaking into every statement after
  // it.
  for (let searchFrom = 0; searchFrom < text.length;) {
    const prefixAt = text.indexOf(
      PNG_DATA_URL_PREFIX,
      searchFrom
    );
    if (prefixAt === (-1))
      break;
    const payloadStart = prefixAt + PNG_DATA_URL_PREFIX.length;
    let payloadEnd = payloadStart;
    while ((payloadEnd < text.length) && isBase64Character({ character: text.charAt(payloadEnd) }))
      payloadEnd += 1;
    if (payloadEnd === payloadStart) {
      // The pattern engine advanced one code unit after a prefix that carried no payload; so does this scan.
      searchFrom = payloadStart;
      continue;
    }
    payloads.push(text.slice(
      payloadStart,
      payloadEnd
    ));
    searchFrom = payloadEnd;
  }
  return payloads;
}

//endregion Base64 payloads

//region Capture names

// Whether one UTF-16 code unit can appear in the slug of a search-filename capture name.
//
// The slug class is `[a-z0-9-]`: lowercase ASCII letters, digits and the hyphen the study uses between a panel and its
// scene.
function isSearchFilenameSlugCharacter({ character }) {
  if ((character >= 'a') && (character <= 'z'))
    return true;
  if ((character >= '0') && (character <= '9'))
    return true;
  return character === '-';
}

// Whether `text` is a non-empty slug whose every code unit the caller's predicate accepts.
//
// Both capture-name grammars end in a `+` character class, so an empty run and one stray code unit are the same
// rejection: the pattern needed at least one character and nothing outside the class.
function isNonEmptyNamedSlug({
  text,
  isSlugCharacter
}) {
  if (text.length === 0)
    return false;
  for (let index = 0; index < text.length; index += 1) {
    if (!isSlugCharacter({ character: text.charAt(index) }))
      return false;
  }
  return true;
}

// Whether a name is one the filename review may embed.
//
// The regular expression this replaced, `/^search-filename-(?:comparison|actual-player)-[a-z0-9-]+\.png$/`, accepted
// one of two fixed source axes, then a non-empty slug from `[a-z0-9-]`, then the `.png` suffix, anchored at both ends.
// The suffix is stripped before the slug is read, so a dot inside the slug cannot pass as the suffix; both axes are
// tried, which is how the alternation committed when the first axis left no valid slug.
export function isSearchFilenameCaptureName({ name }) {
  if (!name.startsWith(SEARCH_FILENAME_CAPTURE_PREFIX))
    return false;
  if (!name.endsWith(SEARCH_FILENAME_CAPTURE_SUFFIX))
    return false;
  // Name text between the fixed prefix and the `.png` suffix, which holds the axis and the slug.
  const body = name.slice(
    SEARCH_FILENAME_CAPTURE_PREFIX.length,
    name.length - SEARCH_FILENAME_CAPTURE_SUFFIX.length
  );
  for (const source of SEARCH_FILENAME_CAPTURE_SOURCES) {
    if (!body.startsWith(`${source}-`))
      continue;
    if (isNonEmptyNamedSlug({
      text: body.slice(source.length + 1),
      isSlugCharacter: isSearchFilenameSlugCharacter
    }))
      return true;
  }
  return false;
}

//endregion Capture names
