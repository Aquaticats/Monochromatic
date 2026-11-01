//region Repeat census patterns
// The line patterns of the repeat census, as the design measured them. They are
// kept in one file so that each scoped disable carries its rationale once. Every
// pattern reads one line of a page: none spans lines, each opens with a fixed
// character or class, and none nests a quantifier, so each runs in time linear
// in the line's length.

/* oxlint-disable no-restricted-syntax/no-regex -- Fence markers are two fixed sequences at the line start, one anchored scan of a single line with no nested quantifier, so it cannot backtrack. */
/**
 Line that opens or closes a fenced code block.
 */
export const FENCE_LINE: RegExp = /^\s*(?:```|~~~)/u;
/* oxlint-enable no-restricted-syntax/no-regex */

/* oxlint-disable no-restricted-syntax/no-regex -- Anchored at the line start with bounded counts and no nested quantifier, so one line is scanned once. */
/**
 Line that is an ATX heading.
 */
export const HEADING_LINE: RegExp = /^\s{0,3}#{1,6}\s/u;
/* oxlint-enable no-restricted-syntax/no-regex */

/* oxlint-disable no-restricted-syntax/no-regex -- Anchored at the line start with one bounded digit run and no nested quantifier, so one line is scanned once. */
/**
 Line that is a list item.
 */
export const LIST_LINE: RegExp = /^\s*(?:[-*+]|\d+[.)])\s/u;
/* oxlint-enable no-restricted-syntax/no-regex */

/* oxlint-disable no-restricted-syntax/no-regex -- Anchored at the line start with a single fixed character, so one line is scanned once. */
/**
 Line that is a blockquote.
 */
export const QUOTE_LINE: RegExp = /^\s*>/u;
/* oxlint-enable no-restricted-syntax/no-regex */

/* oxlint-disable no-restricted-syntax/no-regex -- Anchored at the line start with a fixed prefix and one character class, so one line is scanned once. */
/**
 Line that opens with an HTML tag.
 */
export const HTML_LINE: RegExp = /^\s*<\/?[a-zA-Z]/u;
/* oxlint-enable no-restricted-syntax/no-regex */

/* oxlint-disable no-restricted-syntax/no-regex -- Anchored at the line start; the two negated classes are each bounded by a closing character on the same line, so one line is scanned once. */
/**
 Line that is a footnote definition.
 */
export const FOOTNOTE_LINE: RegExp = /^\s*\[\^[^\]]+\]:/u;
/* oxlint-enable no-restricted-syntax/no-regex */

/* oxlint-disable no-restricted-syntax/no-regex -- Anchored at the line start with a fixed alternation and a word boundary, so one line is scanned once. */
/**
 Line that opens with an import or an export.
 */
export const JSX_KEYWORD_LINE: RegExp = /^\s*(?:import|export)\b/u;
/* oxlint-enable no-restricted-syntax/no-regex */

/* oxlint-disable no-restricted-syntax/no-regex -- Fixed alternatives with no quantifier beyond an optional letter, so one line is scanned once. */
/**
 Line carrying a template expression or an image extension.
 */
export const JSX_MARK_LINE: RegExp = /\$\{|\.(?:webp|png|jpe?g|gif|svg)\b/u;
/* oxlint-enable no-restricted-syntax/no-regex */

/* oxlint-disable no-restricted-syntax/no-regex -- Anchored at the line start with one character class, so one line is scanned once. */
/**
 Line that opens with a JSX or object character.
 */
export const JSX_OPEN_LINE: RegExp = /^\s*[<{[]/u;
/* oxlint-enable no-restricted-syntax/no-regex */

/* oxlint-disable no-restricted-syntax/no-regex -- Anchored at the line start with one class repeated and a fixed colon, so one line is scanned once. */
/**
 Line that is a front-matter key.
 */
export const FRONT_KEY_LINE: RegExp = /^\s*[a-zA-Z_]+:\s/u;
/* oxlint-enable no-restricted-syntax/no-regex */

/* oxlint-disable no-restricted-syntax/no-regex -- A fixed scheme followed by one negated class, so each match is found in one forward pass over the line. */
/**
 Bare URL, which the census blanks.
 */
export const BARE_URL: RegExp = /https?:\/\/\S+/gu;
/* oxlint-enable no-restricted-syntax/no-regex */

/* oxlint-disable no-restricted-syntax/no-regex -- A fixed bracket and parenthesis around one negated class, so each match is found in one forward pass over the line. */
/**
 Link destination, which the census drops and keeps the bracket of.
 */
export const LINK_DESTINATION: RegExp = /\]\([^)]*\)/gu;
/* oxlint-enable no-restricted-syntax/no-regex */

/* oxlint-disable no-restricted-syntax/no-regex -- A fixed angle bracket around one negated class, so each match is found in one forward pass over the line. */
/**
 Inline tag, which the census blanks.
 */
export const INLINE_TAG: RegExp = /<[^>]*>/gu;
/* oxlint-enable no-restricted-syntax/no-regex */

/* oxlint-disable no-restricted-syntax/no-regex -- A fixed prefix around one negated class, so each match is found in one forward pass over the line. */
/**
 Template expression, which the census blanks.
 */
export const TEMPLATE_EXPRESSION: RegExp = /\$\{[^}]*\}/gu;
/* oxlint-enable no-restricted-syntax/no-regex */

/* oxlint-disable no-restricted-syntax/no-regex -- A fixed brace around one negated class, so each match is found in one forward pass over the line. */
/**
 Object expression, which the census blanks.
 */
export const BRACE_EXPRESSION: RegExp = /\{[^}]*\}/gu;
/* oxlint-enable no-restricted-syntax/no-regex */

/* oxlint-disable no-restricted-syntax/no-regex -- A single code-point range with no quantifier, matched once per character. */
/**
 One CJK ideograph, counted for the JSX line test.
 */
export const CJK_IDEOGRAPH: RegExp = /[一-鿿]/gu;
/* oxlint-enable no-restricted-syntax/no-regex */

/* oxlint-disable no-restricted-syntax/no-regex -- One class with a plus quantifier, so each run is consumed once in a forward pass. */
/**
 Run of whitespace, which the census removes before comparing wording.
 */
export const WHITESPACE_RUN: RegExp = /\s+/gu;
/* oxlint-enable no-restricted-syntax/no-regex */

//endregion Repeat census patterns
