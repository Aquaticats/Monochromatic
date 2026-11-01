/**
 Linear-cost string helpers shared by config parsing.
 
 These avoid pattern-based trimming so a very long `AllowedIPs` value is
 processed in time proportional to its length, never quadratically.
 
 @module
 */

/**
 Reports whether one code unit is ASCII whitespace significant to config lines.
 
 @param code - UTF-16 code unit to classify.
 
 @returns True for space, tab, carriage return, newline, and vertical whitespace.
 
 @example
 ```ts
 isSpace({ code: 32 });
 ```
 */
export function isSpace({ code, }: { readonly code: number; },): boolean {
  /**
   Horizontal tab code unit.
   */
  const TAB = 9;
  /**
   Carriage return code unit.
   */
  const CR = 13;
  /**
   Space code unit.
   */
  const SPACE = 32;
  return (code === SPACE) || ((code >= TAB) && (code <= CR));
}

/**
 Finds the index of the first code unit that is not ASCII whitespace.
 
 @param value - Text to scan from the left.
 
 @returns Index of the first non-whitespace code unit, or the length when all whitespace.
 
 @example
 ```ts
 firstNonSpace({ value: '  x' });
 ```
 */
export function firstNonSpace({ value, }: { readonly value: string; },): number {
  return (function scan(): number {
    /**
     Cursor walking rightward past whitespace.
     */
    let index = 0;
    while ((index < value.length) && isSpace({ code: value.codePointAt(index,) ?? 0, },))
      index += 1;
    return index;
  })();
}

/**
 Finds one past the last code unit that is not ASCII whitespace.
 
 @param value - Text to scan from the right.
 
 @param start - Leftmost index that is still content.
 
 @returns End index exclusive of trailing whitespace.
 
 @example
 ```ts
 lastNonSpace({ value: 'x  ', start: 0 });
 ```
 */
export function lastNonSpace(
  {
    value,
    start,
  }: {
    readonly value: string;
    readonly start: number;
  },
): number {
  return (function scan(): number {
    /**
     Cursor walking leftward past trailing whitespace.
     */
    let end = value.length;
    while ((end > start) && isSpace({ code: value.codePointAt(end - 1,) ?? 0, },))
      end -= 1;
    return end;
  })();
}

/**
 Trims ASCII whitespace from both ends using index scanning rather than a
 pattern, so cost stays linear even on a very long `AllowedIPs` value.
 
 @param value - Raw value text that may carry surrounding whitespace.
 
 @returns Value without leading or trailing whitespace.
 
 @example
 ```ts
 trimLinear({ value: '  0.0.0.0/0  ' });
 ```
 */
export function trimLinear({ value, }: { readonly value: string; },): string {
  /**
   First index at which a non-whitespace code unit appears.
   */
  const start = firstNonSpace({ value, },);
  /**
   One past the last non-whitespace code unit.
   */
  const end = lastNonSpace({
    value,
    start,
  },);
  return value.slice(
    start,
    end,
  );
}

/**
 Reports whether one code unit is an ASCII digit.
 
 @param code - UTF-16 code unit to classify.
 
 @returns True for `0` through `9`.
 
 @example
 ```ts
 isDigit({ code: 48 });
 ```
 */
function isDigit({ code, }: { readonly code: number; },): boolean {
  /**
   Code unit for `0`.
   */
  const ZERO = 48;
  /**
   Code unit for `9`.
   */
  const NINE = 57;
  return (code >= ZERO) && (code <= NINE);
}

/**
 Reports whether one code unit may appear in a network device name.
 
 @param code - UTF-16 code unit to classify.
 
 @returns True for letters, digits, and common device-name punctuation.
 
 @example
 ```ts
 isDeviceChar({ code: 97 });
 ```
 */
function isDeviceChar({ code, }: { readonly code: number; },): boolean {
  /**
   Code unit for `a`.
   */
  const A_LOWER = 97;
  /**
   Code unit for `z`.
   */
  const Z_LOWER = 122;
  /**
   Code unit for `A`.
   */
  const A_UPPER = 65;
  /**
   Code unit for `Z`.
   */
  const Z_UPPER = 90;
  /**
   Code unit for `.`.
   */
  const DOT = 46;
  /**
   Code unit for `-`.
   */
  const DASH = 45;
  /**
   Code unit for `_`.
   */
  const UNDERSCORE = 95;
  return isDigit({ code, })
    || ((code >= A_LOWER) && (code <= Z_LOWER))
    || ((code >= A_UPPER) && (code <= Z_UPPER))
    || (code === DOT)
    || (code === DASH)
    || (code === UNDERSCORE);
}

/**
 Extracts the token following a keyword in command output via index scanning.
 
 Used to read `dev <name>` from a route line or `mtu <n>` from link output
 without a pattern, keeping the scan linear and allocation-light.
 
 @param value - Command output to scan.
 
 @param keyword - Bare keyword preceding the desired token.
 
 @param isChar - Classifier for code units allowed in the token.
 
 @returns A result whose `found` flag guards the extracted `token`.
 
 @example
 ```ts
 tokenAfter({ value: 'default dev eth0', keyword: 'dev', isChar: isDeviceChar });
 ```
 */
export function tokenAfter(
  {
    value,
    keyword,
    isChar,
  }: {
    readonly value: string;
    readonly keyword: string;
    readonly isChar: (args: { readonly code: number; }) => boolean;
  },
): {
  readonly found: boolean;
  readonly token: string;
} {
  return (function scan(): {
    readonly found: boolean;
    readonly token: string;
  } {
    /**
     Cursor searching for the keyword at a word boundary.
     */
    let from = 0;
    for (;;) {
      /**
       Index of the next keyword occurrence, or -1 when exhausted.
       */
      const at = value.indexOf(
        keyword,
        from,
      );
      if (at === (-1))
        return {
          found: false,
          token: '',
        };
      /**
       Whether the match starts a whole word (no letter/digit just before).
       */
      const boundaryBefore = (at === 0)
        || isSpace({ code: value.codePointAt(at - 1,) ?? 0, });
      /**
       Index just past the keyword.
       */
      const after = at + keyword.length;
      /**
       First non-space token start after the keyword.
       */
      const tokenStart = firstNonSpace({ value: value.slice(after,), },) + after;
      if (boundaryBefore && (tokenStart > after)) {
        /**
         One past the last token code unit.
         */
        let tokenEnd = tokenStart;
        while ((tokenEnd < value.length)
          && isChar({ code: value.codePointAt(tokenEnd,) ?? 0, },))
          tokenEnd += 1;
        return {
          found: true,
          token: value.slice(
            tokenStart,
            tokenEnd,
          ),
        };
      }
      from = after;
    }
  })();
}

/**
 Classifier allowing device-name code units, exported for `tokenAfter` callers.
 
 @param args - Code unit wrapper.
 
 @returns True when the code unit may appear in a device name.
 
 @example
 ```ts
 deviceChar({ code: 95 });
 ```
 */
export function deviceChar(args: { readonly code: number; },): boolean {
  return isDeviceChar(args,);
}

/**
 Classifier allowing ASCII digits, exported for `tokenAfter` callers.
 
 @param args - Code unit wrapper.
 
 @returns True when the code unit is a digit.
 
 @example
 ```ts
 digitChar({ code: 50 });
 ```
 */
export function digitChar(args: { readonly code: number; },): boolean {
  return isDigit(args,);
}

/**
 Splits text into whitespace-separated words using an index scan, avoiding a
 regex so cost stays linear and no suppression is needed.
 
 @param line - Text to split.
 
 @returns Non-empty whitespace-delimited tokens in order.
 
 @example
 ```ts
 splitWords({ line: 'default via 192.168.1.1 dev eth0' });
 ```
 */
export function splitWords({ line, }: { readonly line: string; },): readonly string[] {
  return (function scan(): readonly string[] {
    /**
     Accumulated words.
     */
    const words: string[] = [];
    /**
     Cursor walking the line.
     */
    let index = 0;
    while (index < line.length) {
      /**
       Start of the next word after skipping whitespace.
       */
      const start = firstNonSpace({ value: line.slice(index,), },) + index;
      if (start >= line.length)
        break;
      /**
       One past the end of the current word.
       */
      let end = start;
      while ((end < line.length) && (!isSpace({ code: line.codePointAt(end,) ?? 0, },)))
        end += 1;
      words.push(line.slice(
        start,
        end,
      ),);
      index = end;
    }
    return words;
  })();
}
