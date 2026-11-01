/**
 `pify` option types and upstream-compatible default resolution.
 
 The resolver reproduces upstream `pify`'s spread-merge verbatim, quirks
 included: an option key explicitly set to `undefined` overwrites its default
 the same way it does upstream, so a mis-typed caller sees upstream's exact
 later failures (`undefined.some`, `new undefined(...)`) instead of silently
 normalized behavior.
 
 Derived from [`pify`](https://github.com/sindresorhus/pify) by Sindre Sorhus
 (MIT); see `LICENSES/MIT.txt` and this package's README for the full
 attribution.
 
 @module
 */

import type { EmptyTuple, } from './type-helpers.ts';

//region Types

/**
 One member-key pattern from `include` or `exclude`: a string matches member
 keys by equality, a `RegExp` matches by test.
 */
export type KeyPattern = string | RegExp;

/**
 Options accepted by `pify`, mirroring upstream `pify`'s `Options` with the
 same defaults and the same per-option semantics.
 
 @typeParam Includes - tuple of member keys to promisify when present
 
 @typeParam Excludes - tuple of member keys to leave untouched when present
 
 @typeParam MultiArgs - whether callbacks report their whole result list
 
 @typeParam ErrorFirst - whether callbacks report an error first argument
 
 @typeParam ExcludeMain - whether a function module's own call stays raw
 
 @example
 ```ts
 const options: PifyOptions = {
   multiArgs: true,
   errorFirst: false,
 };
 ```
 */
export type PifyOptions<
  Includes extends readonly unknown[] = EmptyTuple,
  Excludes extends readonly unknown[] = EmptyTuple,
  MultiArgs extends boolean = false,
  ErrorFirst extends boolean = true,
  ExcludeMain extends boolean = false,
  > = {
  /**
   Collect every callback result into one array instead of only the second
   argument. Rejections then carry the full callback argument list, error
   included, matching upstream `pify`.
   
   @defaultValue false
   */
    readonly multiArgs?: MultiArgs;
  /**
   Member keys to promisify. While present and truthy, only these members are
   promisified and `exclude` is ignored, matching upstream `pify`.
   */
    readonly include?: Includes;
  /**
   Member keys to leave untouched. Only consulted while `include` is absent,
   matching upstream `pify`.
   
   @defaultValue `[/\.+(?:Sync|Stream)$\/]`
   */
    readonly exclude?: Excludes;
  /**
   Whether the callback reports a leading error argument. Falsy means every
   callback argument resolves the promise, matching upstream `pify`.
   
   @defaultValue true
   */
    readonly errorFirst?: ErrorFirst;
  /**
   Promise constructor used instead of the native one, matching upstream
   `pify`'s `promiseModule`.
   
   @defaultValue Promise
   */
    readonly promiseModule?: PromiseConstructor;
  /**
   Keep a function module's own call un-promisified while its members are
   promisified, matching upstream `pify`.
   
   @defaultValue false
   */
    readonly excludeMain?: ExcludeMain;
  };

/**
 Any caller-supplied `pify` options record, before defaults merge: generic
 option slots widened to their runtime ranges so every specialization of
 {@link PifyOptions} is assignable.
 */
export type PifyOptionsInput = PifyOptions<readonly unknown[], readonly unknown[], boolean, boolean, boolean>;

/**
 Option slots consumed by the type-level result computation of one
 promisified function: which result shape `multiArgs` and `errorFirst` imply.
 
 @typeParam Includes - tuple of member keys selected for promisification
 
 @typeParam Excludes - tuple of member keys selected out of promisification
 
 @typeParam MultiArgs - whether callbacks report their whole result list
 
 @typeParam ErrorFirst - whether callbacks report an error first argument
 */
export type PromisifyOptions<
  Includes extends readonly unknown[],
  Excludes extends readonly unknown[],
  MultiArgs extends boolean,
  ErrorFirst extends boolean,
  > = {
    readonly multiArgs: MultiArgs;
    readonly include: Includes;
    readonly exclude: Excludes;
    readonly errorFirst: ErrorFirst;
  };

/**
 Options record after {@link resolvePifyOptions}: every slot present with the
 value upstream `pify` would observe, default or caller-supplied.
 
 @example
 ```ts
 const resolved: ResolvedPifyOptions = {
   multiArgs: false,
   include: ['readFile'],
   exclude: [/.+(?:Sync|Stream)$/],
   errorFirst: true,
   promiseModule: Promise,
   excludeMain: false,
 };
 ```
 */
export type ResolvedPifyOptions = {
  /**
   Caller's `multiArgs` when supplied, `false` otherwise.
   */
  readonly multiArgs: boolean;
  /**
   Caller's `include` when supplied. Absent and explicitly `undefined` stay
   indistinguishable at runtime: `pify` only ever tests this slot's
   truthiness, exactly like upstream `pify`, so the type states the contract
   a well-typed caller supplies and the empty-selection marker is `undefined`,
   never `[]` (`[]` is truthy and would mean "promisify nothing").
   */
  readonly include: readonly KeyPattern[];
  /**
   Caller's `exclude` when supplied, upstream's default pattern otherwise.
   */
  readonly exclude: readonly KeyPattern[];
  /**
   Caller's `errorFirst` when supplied, `true` otherwise.
   */
  readonly errorFirst: boolean;
  /**
   Caller's `promiseModule` when supplied, native `Promise` otherwise.
   */
  readonly promiseModule: PromiseConstructor;
  /**
   Caller's `excludeMain` when supplied, `false` otherwise.
   */
  readonly excludeMain: boolean;
};

//endregion Types

//region Resolution

/**
 Merges caller options over upstream `pify`'s defaults with upstream's exact
 spread semantics.
 
 A bare `{}` means upstream's defaults. A caller key explicitly set to
 `undefined` wins over its default, because upstream spreads the caller
 record after the defaults and never re-normalizes; keeping that behavior
 keeps mis-typed callers on upstream's exact failure modes instead of on
 invented ones.
 
 @param options - Caller-supplied options, or nothing for all defaults.
 
 @returns Options record whose slots hold the values upstream `pify` would
 observe.
 
 @example
 ```ts
 resolvePifyOptions({ options: { multiArgs: true, }, });
 // => { multiArgs: true, errorFirst: true, ... }
 ```
 */
export function resolvePifyOptions(
  {
    options,
  }: {
    // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- mirrors upstream `pify` 6.1.0's `pify(input, options?)`, where an explicitly `undefined` options record still means the defaults and callers porting option spreads rely on that acceptance; exactOptionalPropertyTypes would otherwise reject the pass-through binding
    readonly options?: PifyOptionsInput | undefined;
  },
): ResolvedPifyOptions {
  /* oxlint-disable typescript/no-unsafe-type-assertion -- the spread-merge below preserves upstream `pify`'s explicit-`undefined` option override and mis-typed-caller failure modes verbatim; the assertion states only the well-typed caller contract at this boundary */
  return {
    multiArgs: false,
    // oxlint-disable-next-line no-restricted-syntax/no-regex, eslint/require-unicode-regexp -- Verbatim upstream pify 6.1.0 default `exclude` pattern so the differential oracle sees identical member selection; input is member-key strings bounded by property-name length; `.+` then a `$`-anchored suffix alternation scans linearly with no nesting, and the literal carries no `u` flag on purpose, since adding one would diverge from upstream on lone-surrogate keys.
    exclude: [/.+(?:Sync|Stream)$/],
    errorFirst: true,
    promiseModule: Promise,
    excludeMain: false,
    ...options,
  } as ResolvedPifyOptions;
  /* oxlint-enable typescript/no-unsafe-type-assertion */
}

//endregion Resolution
