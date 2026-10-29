import { tagged, } from '@monochromatic-dev/module-logger/ts';

/**
 Logger root for terminal-exec after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l: parentLogger, },);
 ```
 */
const parentLogger = tagged({ tag: 'terminal-exec', },);
/**
 Tokenizes desktop entry `Exec` values following the freedesktop Desktop Entry Specification.
 Handles double-quote escaping, `%` field code stripping, and rejects unquoted shell metacharacters.
 
 @module
 */

/**
 Tagged logger for this module.
 */
const l = tagged({
  tag: 'tokenize',
  l: parentLogger,
},);

/**
 Sentinel returned by {@link tokenizeExec} when the Exec value contains
 invalid syntax (unquoted shell metacharacter or unterminated quote). A
 `unique symbol`; callers narrow with `=== INVALID_EXEC`.
 */
export const INVALID_EXEC: unique symbol = Symbol('terminal-exec/exec value contains invalid syntax',);

/**
 Shell metacharacters that are invalid unquoted in Exec values.
 */
const UNQUOTED_REJECT = new Set([
  '$',
  '`',
  '>',
  '<',
  '|',
  '&',
  ';',
  '(',
  ')',
],);

/**
 Recognized `%` field codes to strip from Exec values.
 */
const FIELD_CODES = new Set([
  'f',
  'F',
  'u',
  'U',
  'i',
  'c',
  'k',
  'd',
  'D',
  'n',
  'N',
  'v',
  'm',
],);

/**
 Tokenizes a desktop entry `Exec` value into an argument array.
 Strips `%` field codes and rejects lines with unquoted shell metacharacters.
 
 @param exec - Raw Exec value from the desktop entry.
 
 @returns Array of argument tokens, or {@link INVALID_EXEC} if the value contains invalid syntax.
 
 @example
 ```ts
 tokenizeExec({ exec: '/usr/bin/ghostty --gtk-single-instance=true' })
 // ['/usr/bin/ghostty', '--gtk-single-instance=true']
 ```
 */
export function tokenizeExec({ exec, }: { readonly exec: string; },): readonly string[] | typeof INVALID_EXEC {
  /**
   Output accumulator; pushed when whitespace ends a token.
   */
  const tokens: string[] = [];
  /**
   In-progress token characters; reset on whitespace.
   */
  let current = '';
  /**
   Quote-state gate; toggled by unescaped `"` characters.
   */
  let inQuote = false;
  /**
   Input cursor; multi-character constructs advance by 2 to skip the escaped second char.
   */
  let i = 0;

  while (i < exec
    .length) {
    /**
     Current input character, scoped to the loop iteration.
     */
    const ch = exec[i];
    if (ch === undefined)
      break;

    if (inQuote) {
      if (ch === '"') {
        inQuote = false;
        i++;
        continue;
      }
      if ((ch === '\\') && ((i + 1) < exec
        .length)) {
        /**
         Lookahead char for the quoted-backslash escape branch.
         */
        const next = exec[i + 1];
        if (next === undefined)
          break; // unreachable (length checked above)
        if ((next === '"') || (next === '`')
          || (next === '$')
          || (next === '\\')) {
          current += next;
          i += 2;
          continue;
        }
      }
      current += ch;
      i++;
      continue;
    }

    if (ch === '"') {
      inQuote = true;
      i++;
      continue;
    }

    if ((ch === ' ') || (ch === '\t')) {
      if (current.length
        > 0) {
        tokens.push(current,);
        current = '';
      }
      i++;
      continue;
    }

    if (UNQUOTED_REJECT.has(ch,)) {
      l.debug(`rejected unquoted character '${ch}' in exec: ${exec}`,);
      return INVALID_EXEC;
    }

    //region % field code stripping
    if ((ch === '%') && ((i + 1) < exec
      .length)) {
      /**
       Lookahead char for the `%` field-code branch.
       */
      const next = exec[i + 1];
      if (next === undefined)
        break; // unreachable (length checked above)
      if (next === '%') {
        current += '%';
        i += 2;
        continue;
      }
      if (FIELD_CODES.has(next,)) {
        i += 2;
        continue;
      }
    }
    //endregion

    current += ch;
    i++;
  }

  if (inQuote) {
    l.debug(`unterminated quote in exec: ${exec}`,);
    return INVALID_EXEC;
  }

  if (current.length
    > 0)
    tokens.push(current,);

  l.debug(`tokenized: ${JSON.stringify(tokens,)}`,);
  return tokens;
}
