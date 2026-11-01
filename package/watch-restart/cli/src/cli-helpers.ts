import type {
  WatchEntityType,
  WatchEventKind,
} from './types.ts';

/**
 Splits a comma-separated string into trimmed non-empty tokens.
 
 Shared helper for `--ext`, `--events`, `--type` so the three flags all
 accept `--flag a,b,c` and repeated `--flag a --flag b` forms with the
 same tokenisation.
 
 @param value - raw string from a CLI flag
 
 @returns trimmed non-empty tokens; never includes empty strings
 
 @example
 ```ts
 splitCommas('.ts, .tsx,, ',); // ['.ts', '.tsx']
 ```
 */
export function splitCommas(value: string,): string[] {
  return value
    .split(',',)
    .map(function trim(s,): string {
      return s.trim();
    },)
    .filter(function nonEmpty(s,): boolean {
      return s.length
        > 0;
    },);
}

/**
 Maps a single CLI event-name token to the internal {@link WatchEventKind}.
 
 `create`/`delete` are the user-facing names (filesystem vocabulary);
 `add`/`unlink` are the chokidar terms surfaced internally. Unknown
 names throw a clear error so the CLI fails fast instead of silently
 treating a typo as "all kinds".
 
 @param token - one CLI event token (e.g. `'create'`)
 
 @returns internal {@link WatchEventKind}
 
 @throws Error when the token is not one of `create`, `change`, `delete`
 
 @example
 ```ts
 cliEventToInternal('create',); // 'add'
 cliEventToInternal('delete',); // 'unlink'
 ```
 */
export function cliEventToInternal(token: string,): WatchEventKind {
  if (token === 'create')
    return 'add';
  if (token === 'change')
    return 'change';
  if (token === 'delete')
    return 'unlink';
  throw new Error(
    `Unknown --events token "${token}"; expected one of create, change, delete`,
  );
}

/**
 Maps a single CLI `--type` token to the internal {@link WatchEntityType}.
 
 Accepts only `'file'` or `'dir'`; anything else throws so a typo fails
 the CLI rather than silently treating an unknown token as "match all".
 
 @param token - one CLI type token
 
 @returns internal {@link WatchEntityType}
 
 @throws Error when the token is not one of `file`, `dir`
 
 @example
 ```ts
 parseTypeToken('file',); // 'file'
 parseTypeToken('dir',);  // 'dir'
 ```
 */
export function parseTypeToken(token: string,): WatchEntityType {
  if ((token === 'file') || (token === 'dir'))
    return token;
  throw new Error(
    `Unknown --type token "${token}"; expected one of file, dir`,
  );
}

/**
 Signal names the kill-signal flag accepts. Locked down because
 NodeJS.Signals is broad (includes unportable POSIX-only signals) and we
 want a clear error when a consumer passes a typo or a signal whose
 disposition is dangerous as a restart trigger (e.g. `SIGSEGV`).
 
 The set covers the common dev-loop choices: `SIGTERM` (default; graceful
 stop), `SIGINT` (Ctrl+C semantics), `SIGHUP` (config-reload signal for
 soft-reload servers), `SIGUSR1`/`SIGUSR2` (custom application signals),
 `SIGKILL` (force kill; rare but the user may explicitly want it).
 
 Typed as `ReadonlySet<string>` (not `ReadonlySet<NodeJS.Signals>`) so the
 `has()` call accepts a raw `string` and acts as the runtime check for
 the type-guard predicate {@link parseKillSignal} relies on; avoiding
 the `as NodeJS.Signals` assertion the type-system would otherwise force.
 */
const KILL_SIGNALS: ReadonlySet<string> = new Set<string>([
  'SIGTERM',
  'SIGINT',
  'SIGHUP',
  'SIGUSR1',
  'SIGUSR2',
  'SIGKILL',
],);

/**
 Type guard: narrows a raw string to {@link NodeJS.Signals} when present
 in {@link KILL_SIGNALS}.
 
 @param raw - signal name as passed to the CLI
 
 @returns `true` when `raw` is an accepted kill-signal name
 
 @example
 ```ts
 if (isKillSignal(raw,)) { ... }
 ```
 */
function isKillSignal(raw: string,): raw is NodeJS.Signals {
  return KILL_SIGNALS.has(raw,);
}

/**
 Validates and narrows a raw `--signal` string to {@link NodeJS.Signals}.
 
 @param raw - signal name as passed to the CLI
 
 @returns narrowed `NodeJS.Signals` token
 
 @throws Error when the raw name is not in the allowed set
 
 @example
 ```ts
 parseKillSignal('SIGHUP',); // 'SIGHUP'
 ```
 */
export function parseKillSignal(raw: string,): NodeJS.Signals {
  if (isKillSignal(raw,))
    return raw;
  throw new Error(
    `Unknown --signal "${raw}"; expected one of ${[...KILL_SIGNALS,].join(', ',)}`,
  );
}

/**
 Compiles a CLI regex pattern into a {@link RegExp}.
 
 Trivial wrapper; explicit function so the compile call site reads
 intent-first (`compileRegex(pattern,)`) rather than `new RegExp(pattern,)`
 scattered through {@link argsToOptions}, and so the throw site for an
 invalid pattern is one stack frame away from the field it failed on.
 
 @param pattern - regex source string
 
 @returns compiled `RegExp` (no flags; case-sensitive)
 
 @throws SyntaxError when the pattern fails {@link RegExp} compilation
 
 @example
 ```ts
 compileRegex('\\.story\\.ts$',);
 ```
 */
export function compileRegex(pattern: string,): RegExp {
  // oxlint-disable-next-line no-restricted-syntax/no-regex, eslint/require-unicode-regexp -- user-supplied CLI regex source: the pattern shape is user-defined (filename matchers like \.story\.ts$) so no string-API substitute fits; input is bounded by argv length, and the regex runs against relative paths (bounded by filesystem path limits) on each event with no nested quantifiers introduced by this site. The `u` flag is intentionally omitted: it would change matching of arbitrary user patterns (turning previously-valid escapes into SyntaxErrors), so the un-flagged form preserves the patterns the user supplies.
  return new RegExp(pattern,);
}

/**
 Real sentinel for "neither `--flag` nor `--no-flag` was passed". A unique
 `Symbol` rather than `undefined` so {@link resolveBoolPair}'s tri-state
 stays free of a nullish union; the caller maps it to "omit the option and
 defer to the orchestrator's documented default".
 
 @example
 ```ts
 const hidden = resolveBoolPair({ positive, negative, flag: 'hidden', },);
 if (hidden !== FLAG_UNSET) options.hidden = hidden;
 ```
 */
export const FLAG_UNSET: unique symbol = Symbol('cli boolean flag pair not provided',);

/**
 Collapses a `--flag` / `--no-flag` pair into a single tri-state.
 
 {@link FLAG_UNSET} means "caller did not pass either flag, defer to the
 orchestrator's documented default"; this lets the same helper power
 default-off and default-on toggles without each call site duplicating
 the both-true conflict check.
 
 @param positive - whether the positive flag (e.g. `--hidden`) was passed
 
 @param negative - whether the negative flag (e.g. `--no-hidden`) was passed
 
 @param flag - user-facing flag name (without leading `--`) for the error message
 
 @returns `true` / `false` / {@link FLAG_UNSET} (see description)
 
 @throws Error when both positive and negative forms are passed in the same argv
 
 @example
 ```ts
 resolveBoolPair({ positive: args.hidden, negative: args.noHidden, flag: 'hidden', },);
 ```
 */
export function resolveBoolPair(
  {
    positive,
    negative,
    flag,
  }: {
    readonly positive: boolean;
    readonly negative: boolean;
    readonly flag: string;
  },
): boolean | typeof FLAG_UNSET {
  if (positive && negative) {
    throw new Error(
      `Cannot pass both --${flag} and --no-${flag} in the same invocation`,
    );
  }
  if (positive)
    return true;
  if (negative)
    return false;
  return FLAG_UNSET;
}
