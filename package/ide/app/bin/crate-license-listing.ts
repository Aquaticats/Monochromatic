// The crate list a `--licenses` section prints, read with cursor scans instead of regular expressions.
// What: the label that starts a section's crate list, and the `name version` words that list holds.
// Why: what the executable prints is the specification, so every scan here answers exactly what the pattern it
// replaced answered. The printed form is `Used by: ` followed by `name version` items ending in commas, wrapped at
// the width the Rust side formats to, with each continuation line indented by the label's length, so items separate
// on commas and on runs of whitespace.

/// The label that starts a section's crate list; every continuation line holds spaces of the same length instead.
const USED_BY_PREFIX = 'Used by: ';

/**
 The crate list of one license section without the label it starts with.

 What: a prefix comparison, the answer an anchored `Used by: ` pattern gives at the start of the block.

 Why: only the first line carries the label; the continuations carry spaces, so a block that starts with anything else
 is returned unchanged rather than dropped.

 @param usedBy - one section's crate list, its lines joined
 @returns the list without its label
 */
export function withoutUsedByPrefix(usedBy: string): string {
  return usedBy.startsWith(USED_BY_PREFIX) ? usedBy.slice(USED_BY_PREFIX.length) : usedBy;
}

/**
 Whether one UTF-16 code unit is a character `\s` reaches.

 What: the WhiteSpace and LineTerminator set, the same one `String.prototype.trim` uses, spelled as the ranges it is.

 Why: a scan that splits on whitespace has to answer exactly what the pattern answered, and every other member of the
 set (a no-break space, an ideographic space, a byte-order mark) would otherwise end a token the pattern ended too.

 @param codeUnit - one code unit of the text, from `String#charCodeAt`
 @returns whether the character is whitespace
 */
function isPatternWhitespace(codeUnit: number): boolean {
  return (codeUnit >= 0x0009 && codeUnit <= 0x000D)
    || codeUnit === 0x0020
    || codeUnit === 0x00A0
    || codeUnit === 0x1680
    || (codeUnit >= 0x2000 && codeUnit <= 0x200A)
    || codeUnit === 0x2028
    || codeUnit === 0x2029
    || codeUnit === 0x202F
    || codeUnit === 0x205F
    || codeUnit === 0x3000
    || codeUnit === 0xFEFF;
}

/**
 The words of `text`, in order, split on runs of whitespace.

 What: one linear pass that keeps the span between whitespace runs, the answer `split(/\s+/).filter(Boolean)` gave, so
 empty words at either end and between two runs are absent.

 Why: the printed crate list separates its `name version` items with commas and spaces and continues on lines indented by
 spaces, so a run of any length is one separator and no item is empty.

 @param text - a printed crate list with its commas already read as spaces
 @returns its words, in order
 */
export function whitespaceSeparatedTokens(text: string): string[] {
  const tokens: string[] = [];
  let start = -1;
  for (let index = 0; index < text.length; index += 1) {
    if (isPatternWhitespace(text.charCodeAt(index))) {
      if (start >= 0) {
        tokens.push(text.slice(start, index));
        start = -1;
      }
      continue;
    }
    if (start < 0) start = index;
  }
  if (start >= 0) tokens.push(text.slice(start));
  return tokens;
}
