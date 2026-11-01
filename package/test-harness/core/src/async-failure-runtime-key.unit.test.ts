/**
 The realm protocol key must have one declaration its Node observer and browser probe both reach. @module
 */

import { readFile, } from 'node:fs/promises';

import { describe, expect, it, } from '@monochromatic-dev/test-harness-core';

/**
 Reads one module's authored source beside this test file.

 @param fileName - module file name in this directory

 @returns The file's source text

 @example
 ```ts
 const source = await sourceOf('execution-node.ts');
 ```
 */
async function sourceOf(fileName: string,): Promise<string> {
  return await readFile(new URL(fileName, import.meta.url,), 'utf8',);
}

await describe({
  name: 'async failure runtime key',
  children: [
    it({
      name: 'is declared once and reaches both consumers through the shared declaration rather than a repeated literal',
      fn: async () => {
        /**
         Authored sources of the Node observer, the browser probe, and the entry that carries the key to it.
         */
        const [
          observerSource,
          fixtureSource,
          entrySource,
        ] = await Promise.all([
          sourceOf('execution-node.ts',),
          sourceOf('sinon-browser-fixture.ts',),
          sourceOf('index.ts',),
        ],);
        /**
         The observer imports the shared declaration instead of owning a copy.
         */
        expect(observerSource,).toContain("from './async-failure-runtime-key.ts'",);
        expect(observerSource,).not.toContain('Symbol.for(',);
        /**
         The browser fixture reads the key from the built entry the package ships.
         */
        expect(fixtureSource,).toContain("from '../dist/final/neutral/index.mjs'",);
        expect(fixtureSource,).toContain('RUNTIME_KEY',);
        expect(fixtureSource,).not.toContain('Symbol.for(',);
        expect(fixtureSource,).not.toContain("from './async-failure-runtime-key.ts'",);
        /**
         That entry re-exports the shared declaration, closing the chain.
         */
        expect(entrySource,).toContain("from './async-failure-runtime-key.ts'",);
        expect(entrySource,).toContain('RUNTIME_KEY',);
        /**
         The shared declaration holds the only protocol literal, so a version bump reaches every consumer.
         */
        const declared = await sourceOf('async-failure-runtime-key.ts',);
        expect(declared.split('Symbol.for(',),).toHaveLength(2,);
      },
    },),
  ],
},);
