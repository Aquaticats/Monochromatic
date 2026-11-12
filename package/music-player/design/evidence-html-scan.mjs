// Regex-free scans over the tags and attributes of a generated design review.
//
// The review builders in `package/music-player/design` validate their own HTML before it is committed. Every check
// here answers `package/config/oxlint`'s `no-restricted-syntax/no-regex`: that rule asks for an index scan, a parser or
// a string API instead of a pattern, and these functions are the index scans the review validators share.
//
// Each scan walks its input once from left to right and keeps no backtracking state, so its cost is linear in the
// document length for any input.
//
// This module is `.mjs` rather than `.ts` because the package's fixture harnesses run copies of a builder from
// `node_modules/.monochromatic`, and Node refuses to strip types for any file under `node_modules`
// (`ERR_UNSUPPORTED_NODE_MODULES_TYPE_STRIPPING`). A `.ts` sibling of a copied builder cannot load there.
// `doc/troubleshooting/node-type-stripping-under-node-modules.md` records the probe.

import {
  matchesAtWithin,
  findWithin
} from './evidence-text-scan.mjs';

//region Character classes

// Whether one UTF-16 code unit is a JavaScript word character: an ASCII letter, digit or underscore.
//
// `\b` in a pattern is decided by this class alone, so every boundary check here reads it.
function isAsciiWordCharacter({ character }) {
  if (character === '')
    return false;
  if ((character >= 'A') && (character <= 'Z'))
    return true;
  if ((character >= 'a') && (character <= 'z'))
    return true;
  if ((character >= '0') && (character <= '9'))
    return true;
  return character === '_';
}

//endregion Character classes

//region Tag bodies

// Position of the next `<tagName` whose name ends at a word boundary, or -1 when the document holds no such opening.
//
// `<tag\b` is the shape every scan here starts from: a longer tag name such as `<scripted` is a different element.
function findTagOpening({
  text,
  tagOpen,
  from,
  ignoreAsciiCase
}) {
  for (let searchFrom = from; searchFrom < text.length;) {
    const tagAt = findWithin({
      haystack: text,
      needle: tagOpen,
      from: searchFrom,
      to: text.length,
      ignoreAsciiCase
    });
    if (tagAt === (-1))
      return -1;
    const afterTagName = tagAt + tagOpen.length;
    if (!isAsciiWordCharacter({ character: text.charAt(afterTagName) }))
      return tagAt;
    searchFrom = afterTagName;
  }
  return -1;
}

// End of the tag body that starts at `from`, which is the first `>` or the end of the document.
//
// A pattern's `[^>]*` run stops there too, so an attribute after that position belongs to a later tag even when the
// document never closes the current one.
function tagBodyEnd({
  text,
  from
}) {
  const closeAt = text.indexOf(
    '>',
    from
  );
  return closeAt === (-1) ? text.length : closeAt;
}

// Whether `attributeName` follows the tag open at `tagNameStart` as a whole word, with one of `valuePrefixes` after it.
function tagOpenHasAttribute(
  {
    text,
    tagNameStart,
    tagName,
    attributeName,
    valuePrefixes,
    ignoreAsciiCase
  }
) {
  const bodyStart = tagNameStart + tagName.length
    + 1;
  const bodyEnd = tagBodyEnd({
    text,
    from: bodyStart
  });
  for (let searchFrom = bodyStart; searchFrom < bodyEnd;) {
    const attributeAt = findWithin({
      haystack: text,
      needle: attributeName,
      from: searchFrom,
      to: bodyEnd,
      ignoreAsciiCase
    });
    if (attributeAt === (-1))
      return false;
    const afterAttribute = attributeAt + attributeName.length;
    // `\b<name>`: the code unit before the name must not extend a longer word.
    if (!isAsciiWordCharacter({ character: text.charAt(attributeAt - 1) })) {
      if (valuePrefixes.length === 0) {
        // `\b<name>\b`: the name alone, bounded on the right as well.
        if (!isAsciiWordCharacter({ character: text.charAt(afterAttribute) }))
          return true;
      } else {
        for (const prefix of valuePrefixes) {
          if (matchesAtWithin({
            haystack: text,
            needle: prefix,
            at: afterAttribute,
            ignoreAsciiCase
          }))
            return true;
        }
      }
    }
    searchFrom = afterAttribute;
  }
  return false;
}

// Whether the document opens `tagName` with `attributeName` inside that tag's body.
//
// The scan answers `<tag\b[^>]*\battr` and its value-bound variants: `valuePrefixes` holds the literal text that must
// immediately follow the attribute name, and an empty list asks for the bare attribute name bounded on both sides.
// Prose and quoted attribute values are not parsed, exactly as the pattern was not: the first `>` ends the body.
export function hasTagAttribute(
  {
    text,
    tagName,
    attributeName,
    valuePrefixes,
    ignoreAsciiCase
  }
) {
  const tagOpen = `<${tagName}`;
  for (let searchFrom = 0; searchFrom < text.length;) {
    const tagAt = findTagOpening({
      text,
      tagOpen,
      from: searchFrom,
      ignoreAsciiCase
    });
    if (tagAt === (-1))
      return false;
    if (tagOpenHasAttribute({
      text,
      tagNameStart: tagAt,
      tagName,
      attributeName,
      valuePrefixes,
      ignoreAsciiCase
    }))
      return true;
    searchFrom = tagAt + tagOpen.length;
  }
  return false;
}

// Whether the document opens `tagName` exactly once, counting openings the way `<tag\b` did.
//
// The review asks for a page with one form and one inline script, so a count compared to one becomes this predicate and
// stops scanning once a second opening exists.
export function hasExactlyOneTagOpening({
  text,
  tagName,
  ignoreAsciiCase
}) {
  const tagOpen = `<${tagName}`;
  const first = findTagOpening({
    text,
    tagOpen,
    from: 0,
    ignoreAsciiCase
  });
  if (first === (-1))
    return false;
  const second = findTagOpening({
    text,
    tagOpen,
    from: first + 1,
    ignoreAsciiCase
  });
  return second === (-1);
}

//endregion Tag bodies
