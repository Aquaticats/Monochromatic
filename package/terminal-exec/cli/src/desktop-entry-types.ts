/**
 Types and constants for desktop entry parsing.
 
 @module
 */

/**
 Parsed result from a `.desktop` file containing terminal-relevant fields.
 */
export type DesktopEntry = {
  /**
   Raw Exec value from the desktop entry.
   */
  readonly exec: string;
  /**
   Whether `TerminalEmulator` appears in Categories.
   */
  readonly isTerminal: boolean;
  /**
   Hidden flag.
   */
  readonly hidden: boolean;
  /**
   TryExec binary path, if specified.
   */
  readonly tryExec: string;
  /**
   OnlyShowIn desktop list.
   */
  readonly onlyShowIn: readonly string[];
  /**
   NotShowIn desktop list.
   */
  readonly notShowIn: readonly string[];
  /**
   X-TerminalArgExec or TerminalArgExec value.
   */
  readonly execArg: string;
  /**
   X-TerminalArgAppId or TerminalArgAppId value.
   */
  readonly appIdArg: string;
  /**
   X-TerminalArgTitle or TerminalArgTitle value.
   */
  readonly titleArg: string;
  /**
   X-TerminalArgDir or TerminalArgDir value.
   */
  readonly dirArg: string;
  /**
   X-TerminalArgHold or TerminalArgHold value.
   */
  readonly holdArg: string;
};

/**
 Desktop entry string escapes per the spec.
 */
export const ESCAPE_MAP: Record<string, string> = {
  s: ' ',
  n: '\n',
  t: '\t',
  r: '\r',
  '\\': '\\',
};

/**
 Mutable version of {@link DesktopEntry} used during parsing.
 */
export type MutableDesktopEntry = {
  exec: string;
  isTerminal: boolean;
  hidden: boolean;
  tryExec: string;
  onlyShowIn: string[];
  notShowIn: string[];
  execArg: string;
  appIdArg: string;
  titleArg: string;
  dirArg: string;
  holdArg: string;
};

/**
 Expands desktop entry string escapes (`\s`, `\n`, `\t`, `\r`, `\\`).
 
 Walks `s` character by character: on backslash, consumes the next
 character and looks it up in {@link ESCAPE_MAP}; otherwise emits the
 character verbatim. A trailing lone backslash is emitted as-is.
 
 @param s - Raw value from a desktop entry key.
 
 @returns Expanded string with escapes resolved.
 
 @example
 ```ts
 expandEscapes({ s: 'hello\\sworld' }) // → 'hello world'
 ```
 */
export function expandEscapes({ s, }: { readonly s: string; },): string {
  return (function build(): string {
    /**
     Output segments in order; joined once at the end so the build is O(n) time.
     */
    const out: string[] = [];
    /**
     Scan cursor; advances by 2 across a resolved escape, by 1 otherwise.
     */
    let idx = 0;
    while (idx < s
      .length) {
      /**
       Current character under the cursor.
       */
      const c = s.charAt(idx,);
      if ((c === '\\') && ((idx + 1) < s
        .length)) {
        /**
         Character following the backslash; looked up in the escape map.
         */
        const next = s.charAt(idx + 1,);
        out.push(ESCAPE_MAP[next]
          ?? next,);
        idx += 2;
      }
      else {
        out.push(c,);
        idx += 1;
      }
    }
    return out.join('',);
  })();
}

/**
 Creates a fresh mutable desktop entry with default values.
 
 @returns Empty mutable desktop entry
 
 @example
 ```ts
 const entry = createEmptyEntry();
 entry.exec = '/usr/bin/xterm';
 ```
 */
export function createEmptyEntry(): MutableDesktopEntry {
  return {
    exec: '',
    isTerminal: false,
    hidden: false,
    tryExec: '',
    onlyShowIn: [],
    notShowIn: [],
    execArg: '',
    appIdArg: '',
    titleArg: '',
    dirArg: '',
    holdArg: '',
  };
}
