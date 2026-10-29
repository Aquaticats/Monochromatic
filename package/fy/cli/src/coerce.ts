import { tagged, } from '@monochromatic-dev/module-logger/ts';

/**
 Logger root for cli-fy after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'cli-fy', },);

/**
 Coerces a string argument to a JS value by attempting `JSON.parse`.
 Falls back to the raw string if parsing fails.
 
 @param arg - Raw CLI argument string
 
 @returns Parsed value (number, boolean, null, object, array) or original string
 
 @example
 ```ts
 coerceArg({ arg: '42' });    // => 42
 coerceArg({ arg: 'true' });  // => true
 coerceArg({ arg: 'hello' }); // => 'hello'
 coerceArg({ arg: '[1,2]' }); // => [1, 2]
 ```
 */
export function coerceArg({ arg, }: { readonly arg: string; },): unknown {
  /**
   Tagged logger scoped to this function so log lines identify the coercion call site.
   */
  const rl = tagged({
    tag: coerceArg.name,
    l,
  },);
  try {
    /**
     Parsed value from `JSON.parse`; typed `unknown` so callers must narrow before use.
     */
    const parsed: unknown = JSON.parse(arg,);
    rl.info(`"${arg}" => ${typeof parsed} ${String(parsed,)}`,);
    return parsed;
  }
  catch (parseError: unknown) {
    rl.info(`"${arg}" => string (raw), JSON parse failed: ${String(parseError,)}`,);
    return arg;
  }
}
