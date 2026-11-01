//region Rotation salt hooks
// THE LOADER OF THE ROTATION SWEEP (ledger B701). Started as `node --import
// <this file's URL>?salt=N <test file>`, it registers a module customization
// hook that moves the start of `benchRotation` by N seats in every module the
// process loads, so a case that passes only for the sheet text its window
// happens to hold fails for some N.
//
// NO PRODUCTION SOURCE READS THE ENVIRONMENT FOR THIS. The salt travels in this
// module's own URL, which only this module reads; the process under test is not
// told and prints nothing of it, since the built commands' stderr is compared by
// the `bench` and `coverage-census` suites. Hooks run synchronously in the
// thread that loads the modules (`module.registerHooks`, release candidate in
// the Node 26 documentation), so the rewrite is made as the built bundle loads.

import {
  type LoadFnOutput,
  registerHooks,
} from 'node:module';

import { saltedBenchRotation, } from './rotation-salt-source.test-fixture.ts';

/**
 What the hook reads of the context the loader hands it: only the format
 matters, and the context itself is left to the chain.
 */
type SeenContext = {
  readonly conditions: readonly string[];
  readonly format: unknown;
  readonly importAttributes: Readonly<Record<string, unknown>>;
};

/**
 Formats whose source the hook reads for the declaration: bundles and sources
 as modules.
 */
const REWRITTEN_FORMATS: ReadonlySet<unknown> = new Set([
  'module',
  'module-typescript',
],);

/**
 The salt this process was started with; absent or not a whole number from
 zero stops the process before it loads a test, so no sweep can pass for want
 of a salt.
 */
const salt = Number(new URL(import.meta.url,).searchParams
  .get('salt',)
  ?? Number.NaN,);

if ((!Number.isInteger(salt,)) || (salt < 0))
  throw new Error(`rotation salt hook: the URL carries no salt that is a whole number from zero (${String(salt,)})`,);

registerHooks({
  /**
   The default load of a module, rewritten when it declares the rotation.

   @param url - module being loaded

   @param _context - what the loader knows of the import; the chain reads it itself

   @param nextLoad - the rest of the chain

   @returns The loaded module, its text changed when it declares `benchRotation`
   */
  load: function loadWithSalt(
    url: string,
    _context: SeenContext,
    nextLoad: (url: string,) => LoadFnOutput,
  ): LoadFnOutput {
    /**
     What the default chain loaded.
     */
    const loaded = nextLoad(url,);
    if ((!REWRITTEN_FORMATS.has(loaded.format,)) || (loaded.source === undefined))
      return loaded;
    /**
     The module's text.
     */
    const text = (typeof loaded.source) === 'string'
      ? loaded.source
      : new TextDecoder().decode(loaded.source,);
    return {
      ...loaded,
      source: saltedBenchRotation({
        source: text,
        salt,
      },),
    };
  },
},);

//endregion Rotation salt hooks
