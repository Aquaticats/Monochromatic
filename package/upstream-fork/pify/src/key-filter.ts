/**
 Member selection for promisification: decides which members of a wrapped
 module are promisified and which pass through untouched.
 
 Derived from [`pify`](https://github.com/sindresorhus/pify) by Sindre Sorhus
 (MIT); see `LICENSES/MIT.txt` and this package's README for the full
 attribution. Two upstream quirks are reproduced verbatim because the fuzz
 sidecar's differential oracle compares member selection against upstream
 `pify` 6.1.0 directly:
 
 1. The decision cache is module-level and keyed by target object only, so the
 first `pify` call to read a member freezes that member's promisification
 decision for every later `pify` call on the same object, whatever those
 later calls' `include` / `exclude` options say.
 2. The cache starts as a plain object and membership is tested with `in`, so
 keys living on `Object.prototype` (`'constructor'`, `'toString'`, ...) skip
 include / exclude evaluation entirely and fall through to the `get` trap's
 `Function.prototype` comparison.
 
 @module
 */

import type {
  KeyPattern,
  ResolvedPifyOptions,
} from './pify-options.ts';

//region Types

/**
 Decides whether the member behind one key of a target object is promisified.
 
 Matches upstream `pify`'s `filter(target, key)`: `true` means the member is
 selected for promisification (subject to the caller being a function).
 
 @example
 ```ts
 const keyFilter = createKeyFilter({ options: resolvedOptions, });
 keyFilter({ target: fs, key: 'readFile', }); // => true
 keyFilter({ target: fs, key: 'readFileSync', }); // => false
 ```
 */
export type KeyFilter = ({
  target,
  key,
}: {
  readonly target: object;
  readonly key: PropertyKey;
}) => boolean;

//endregion Types

//region Decision cache

/**
 Promisification decisions memoized per target object and shared by every
 `pify` call (upstream `pify`'s `filterCache`).
 
 Kept module-level and keyed by target only to reproduce upstream's
 first-touch-wins behavior; see this module's header for why the fork does not
 fix that.
 */
const filterCache = new WeakMap<object, Record<PropertyKey, unknown>>();

//endregion Decision cache

//region Factory

/**
 Builds one `pify` call's key filter over the shared decision cache.
 
 @param options - Resolved options whose `include` / `exclude` patterns select
 members; read at decision time exactly like upstream `pify`.
 
 @returns Filter answering whether one member key is promisified.
 
 @example
 ```ts
 const keyFilter = createKeyFilter({ options: resolvePifyOptions({}), });
 keyFilter({ target: nodeFs, key: 'readFile', }); // => true
 ```
 */
export function createKeyFilter(
  {
    options,
  }: {
    readonly options: ResolvedPifyOptions;
  },
): KeyFilter {
  /**
   Answers whether the member behind one key is promisified, memoizing the
   answer in the shared cache.
   
   @param target - Object whose member is classified.
   
   @param key - Member key to classify.
   
   @returns Whether the member is selected for promisification.
   */
  function keyFilter(
    {
      target,
      key,
    }: {
      readonly target: object;
      readonly key: PropertyKey;
    },
  ): boolean {
    /**
     Decisions already computed for this target, created on first touch.
     */
    let cached = filterCache.get(target);
    if (!cached) {
      cached = {};
      filterCache.set(
        target,
        cached
      );
    }
    /**
     Upstream `pify` tests `key in cached` on this plain object, so keys on
     `Object.prototype` short-circuit here to their prototype value's
     truthiness before any include / exclude rule runs. Reproduced verbatim.
     */
    if (key in cached)
      return Boolean(cached[key]);

    /**
     Matches one include / exclude pattern against the key: strings by
     equality, everything else by `test` call, exactly like upstream `pify`.
     A symbol key never matches a pattern and is compared by identity only.
     
     @param pattern - Include / exclude entry to match.
     
     @returns Whether the pattern selects this key.
     */
    function matches(pattern: KeyPattern): boolean {
      return ((typeof pattern) === 'string') || ((typeof key) === 'symbol')
        ? key === pattern
        // String() reproduces RegExp.prototype.test's own ToString coercion
        // for the string and number keys that reach this branch; symbol keys
        // are filtered out above exactly like upstream `pify`.
        : pattern.test(String(key,),);
    }

    /**
     Own property descriptor as it exists right now, looked up once exactly
     like upstream `pify`.
     */
    const descriptor = Reflect.getOwnPropertyDescriptor(
      target,
      key,
    );
    /**
     Whether the member has no own descriptor, or its own descriptor is
     writable or configurable. Non-writable, non-configurable own members must
     pass through raw or the proxy's `get` invariant would throw.
     */
    const writableOrConfigurableOwn = (descriptor === undefined)
      || Boolean(descriptor.writable,)
      || Boolean(descriptor.configurable);
    /**
     Whether the caller supplied a truthy `include` list at runtime; the
     truthiness coercion mirrors upstream `pify`'s `options.include ? ... : ...`
     exactly: a present `include` (even `[]`, which is truthy and means
     "promisify nothing") ignores `exclude`.
     */
    const includeProvided = Boolean(options.include,);
    /**
     Whether include / exclude selection picks this key at all.
     */
    const included = includeProvided
      ? options.include
        .some(function byIncludePattern(pattern: KeyPattern): boolean {
          return matches(pattern,);
        },)
      : !options.exclude
        .some(function byExcludePattern(pattern: KeyPattern): boolean {
          return matches(pattern,);
        },);
    /**
     Whether the member behind this key is promisified.
     */
    const shouldPromisify = included && writableOrConfigurableOwn;
    cached[key] = shouldPromisify;
    return shouldPromisify;
  }

  return keyFilter;
}

//endregion Factory
