/**
 URI/URL detection helpers used by the asset resolvers and the CSS/HTML
 parsers to decide whether a candidate path is a local relative reference
 or an external/absolute URL.
 */

/**
 Returns true when `s` begins with a URI scheme of the form
 `[a-z][a-z0-9+.-]*:` (case-insensitively). Matches the prior
 `/^[a-z][a-z0-9+.-]*:/i` regex test without backtracking surface:
 a single linear scan checks the first character, then advances over
 the scheme body until a non-scheme character or the colon terminator.
 
 @param s - candidate string (usually a `src`/`href`/`url(...)` value)
 
 @returns whether the string opens with an absolute URI scheme
 
 @example
 ```ts
 startsWithUriScheme('https://example.com'); // true
 startsWithUriScheme('data:image/png;base64,...'); // true
 startsWithUriScheme('./local.png'); // false
 ```
 */
export function startsWithUriScheme(s: string,): boolean {
  if (s.length
    < 2)
    return false;
  /**
   Lower-cased copy so the alpha range check covers `[A-Za-z]`.
   */
  const lowered = s.toLowerCase();
  /**
   First character; must be an ASCII letter to open a scheme.
   */
  const first = lowered.charAt(0,);
  if ((first < 'a') || (first > 'z'))
    return false;
  // Linear scan from index 1 over the scheme body; the first non-scheme-body
  // character decides the result. A scheme is valid only when that terminator
  // is `:`. Indexed `charAt` (not `for...of`) preserves the prior UTF-16
  // code-unit semantics: a non-ASCII body char stops the run at its first code
  // unit, exactly as the recursion did, in O(n) time and O(1) stack.
  for (let cursorIndex = 1; cursorIndex < lowered
    .length; cursorIndex += 1) {
    /**
     Char at cursor; non-scheme characters end the run.
     */
    const c = lowered.charAt(cursorIndex,);
    /**
     Whether the cursor sits on a scheme-body character.
     */
    const ok = ((c >= 'a') && (c <= 'z'))
      || ((c >= '0') && (c <= '9'))
      || (c === '+')
      || (c === '.')
      || (c === '-');
    if (!ok)
      return c === ':';
  }
  return false;
}
