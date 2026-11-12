// The guards that keep a generated design review inside its declared boundary: the page it writes is offline
// evidence rather than a ballot, and every witness it reads is a file this package holds.
//
// Each guard was a regular expression until the regex-necessity audit filed it as unnecessary
// (doc/audit/regex-necessity.md, issues #776 to #786), so each is now one pass over the text. Those patterns remain
// the spec for the answers these scans give, down to the `\b`, `\d` and `\s` sets and the ASCII-only case folding
// of the `i` flag, and offline-review-guards-test.mjs replays the answers recorded from them.

//region Character classes, each one the exact set its removed pattern named
/** Reports whether one code unit is a character `\d` accepts. */
function isDigit(character) {
  return (character >= '0') && (character <= '9');
}

/** Reports whether one code unit is a character `\w` accepts, which is the set `\b` reads. */
function isWordCharacter(character) {
  return ((character >= 'A') && (character <= 'Z'))
    || ((character >= 'a') && (character <= 'z'))
    || isDigit(character)
    || (character === '_');
}
//endregion

//region Tag scans, the markup questions the validate passes ask
/**
 * Counts the openings of one tag name that a `\b` follows.
 *
 * The comparison is exact on purpose: the count answered `/<\w+\b/` without the `i` flag, so `<FORM` never
 * counted. A `<form` inside another tag's attribute text does count, exactly as it did before.
 */
export function countTagOpenings({ page, tagName }) {
  const opening = `<${tagName}`;
  let count = 0;
  let from = 0;
  for (;;) {
    const at = page.indexOf(opening, from);
    if (at === -1) return count;
    const after = at + opening.length;
    if ((after === page.length) || (!isWordCharacter(page[after]))) {
      count += 1;
      from = after;
    } else {
      from = at + 1;
    }
  }
}
//endregion
