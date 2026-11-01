/**
 TypeScript fork of [`pify`](https://github.com/sindresorhus/pify) by
 Sindre Sorhus (MIT): promisify a callback-style function or module.
 
 Rewritten to this repository's TypeScript standards with the upstream
 `pify` 6.1.0 behavior preserved: a `Proxy` view whose function members
 return promises, member selection through `include` / `exclude` with
 upstream's exact quirks, and upstream's option merge semantics. The only
 API-shape deviation is lint-mandated: `pify({ input, options })` takes one
 destructured object and promisified calls pass an `args` tuple instead of a
 rest parameter.
 
 @example
 ```ts
 import { pify, } from '\@monochromatic-dev/upstream-fork-pify';
 
 const readFile = pify({ input: nodeFs.readFile, });
 const data = await readFile({ args: ['package.json', 'utf8'], });
 ```
 
 @packageDocumentation
 */

export { InvalidInputError, } from './errors.ts';

export {
  type KeyPattern,
  type PifyOptions,
  type PromisifyOptions,
} from './pify-options.ts';

export {
  pify,
  type PromisifiedModule,
} from './pify.ts';

export {
  type PromisifiedCall,
  type Promisify,
  type WrappedFunction,
} from './promisify-function.ts';

export {
  type DropLastArrayElement,
  type EmptyTuple,
  type LastArrayElement,
  type StringEndsWith,
} from './type-helpers.ts';
