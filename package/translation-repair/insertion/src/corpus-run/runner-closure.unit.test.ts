/**
 Tests for the runner closure reading.

 The case that matters is the MINIFIED import form. Built runners here are one
 long line whose imports read `from"./chunk.mjs"` with no space, and a scan
 expecting `from './` finds nothing and reports a clean closure for a file full
 of imports. That false null was hit while measuring the settled audit, and it looks
 exactly like a self-contained bundle, which is a legitimate state.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { writeFile, } from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { readRunnerClosure, } from '../../dist/final/node/index.mjs';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

/**
 Writes one entry file into a caller-owned directory and returns its path.

 ON A THROWAWAY, per `THR`: this reads files, so it gets its own directory
 rather than any path the repository cares about.

 @param dir - case-owned directory to write into

 @param text - entry contents

 @returns Path written

 @example
 ```ts
 const path = await entryWith({ dir: scratch.path, text: 'export {};', },);
 ```
 */
async function entryWith(
  { dir, text, }: { readonly dir: string; readonly text: string; },
): Promise<string> {
  /**
   Where the fixture entry goes.
   */
  const path = join(
    dir,
    'tabby-probe.mjs',
  );
  await writeFile(
    path,
    text,
    'utf8',
  );
  return path;
}

await describe({
  name: readRunnerClosure.name,
  children: [
    it({
      name: 'READS THE MINIFIED FORM, which is the one that matters: built runners are one long '
        + 'line whose imports carry no space, and a scan written for readable source reports a '
        + 'clean closure for a file full of them',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'runner-closure-', },);
        /**
         An entry shaped the way the bundler actually emits one.
         */
        const path = await entryWith({
          dir: scratch.path,
          text: 'import{a as tagged}from"./run-config-ABC123.mjs";import{b}from"./whisker-DEF456.mjs";'
            + 'import{join}from"node:path";const x=1;export{};',
        },);

        /**
         What it imports.
         */
        const closure = await readRunnerClosure({ entryPath: path, },);

        expect(closure.kind,).toBe('read',);
        if (closure.kind !== 'read')
          throw new Error('read by construction',);
        expect(closure.chunks,).toEqual([
          'run-config-ABC123.mjs',
          'whisker-DEF456.mjs',
        ],);
        expect(closure.entry,).toBe('tabby-probe.mjs',);
      },
    },),

    it({
      name: 'READS THE SPACED FORM TOO, so an unminified build is not silently reported as '
        + 'importing nothing',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'runner-closure-', },);
        /**
         Ordinary readable source.
         */
        const path = await entryWith({
          dir: scratch.path,
          text: 'import { tagged, } from \'./run-config.mjs\';\n'
            + 'import { join, } from "node:path";\n',
        },);

        /**
         What it imports.
         */
        const closure = await readRunnerClosure({ entryPath: path, },);
        if (closure.kind !== 'read')
          throw new Error('read by construction',);
        expect(closure.chunks,).toEqual(['run-config.mjs',],);
      },
    },),

    it({
      name: 'READS A SPECIFIER CLOSED BY A SINGLE QUOTE where no double quote follows it in the file',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'runner-closure-', },);
        /**
         Readable source whose only quotes are single.
         */
        const path = await entryWith({
          dir: scratch.path,
          text: 'import { tagged, } from \'./run-config.mjs\';\n',
        },);

        /**
         What it imports.
         */
        const closure = await readRunnerClosure({ entryPath: path, },);
        if (closure.kind !== 'read')
          throw new Error('read by construction',);
        expect(closure.chunks,).toEqual(['run-config.mjs',],);
      },
    },),

    it({
      name: 'SKIPS A SPECIFIER WHOSE QUOTE NEVER CLOSES, a file cut off inside an import, rather than naming '
        + 'the rest of the file a chunk',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'runner-closure-', },);
        /**
         An entry cut off inside its second import.
         */
        const path = await entryWith({
          dir: scratch.path,
          text: 'import{a}from"./run-config-ABC123.mjs";import{b}from"./whisker-DEF',
        },);

        /**
         What it imports.
         */
        const closure = await readRunnerClosure({ entryPath: path, },);
        if (closure.kind !== 'read')
          throw new Error('read by construction',);
        expect(closure.chunks,).toEqual(['run-config-ABC123.mjs',],);
      },
    },),

    it({
      name: 'IGNORES BARE AND NODE SPECIFIERS, since only relative chunks belong to this build: '
        + 'a package name identifies a dependency and not the code that was executed',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'runner-closure-', },);
        const path = await entryWith({
          dir: scratch.path,
          text: 'import{a}from"node:fs";import{b}from"nano-spawn";import{c}from"@scope/pkg";',
        },);
        const closure = await readRunnerClosure({ entryPath: path, },);
        if (closure.kind !== 'read')
          throw new Error('read by construction',);
        expect(closure.chunks,).toEqual([],);
      },
    },),

    it({
      name: 'SORTS AND DEDUPLICATES, so two runs of one build compare equal by string equality '
        + 'regardless of the order the bundler happened to emit',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'runner-closure-', },);
        const path = await entryWith({
          dir: scratch.path,
          text: 'import{a}from"./zebra.mjs";import{b}from"./alpha.mjs";import{c}from"./zebra.mjs";',
        },);
        const closure = await readRunnerClosure({ entryPath: path, },);
        if (closure.kind !== 'read')
          throw new Error('read by construction',);
        expect(closure.chunks,).toEqual([
          'alpha.mjs',
          'zebra.mjs',
        ],);
      },
    },),

    it({
      name: 'SEPARATES "imports nothing" FROM "could not be read", which is the whole reason this '
        + 'is a tagged union: an inlined bundle whose closure is itself and a file nobody could '
        + 'open are opposite findings, and comparing two unreadable ones would call two unknown '
        + 'builds the same',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'runner-closure-', },);
        /**
         A real entry that genuinely imports nothing relative.
         */
        const inlined = await readRunnerClosure({
          entryPath: await entryWith({ dir: scratch.path, text: 'const x=1;export{};', },),
        },);
        expect(inlined.kind,).toBe('read',);
        if (inlined.kind === 'read')
          expect(inlined.chunks,).toEqual([],);

        /**
         A path that is not there.
         */
        const missing = await readRunnerClosure({
          entryPath: join(
            scratchRoot(),
            'no-such-tabby-probe-anywhere.mjs',
          ),
        },);
        expect(missing.kind,).toBe('unavailable',);

        /**
         No path at all, which is what a source run gives.
         */
        const none = await readRunnerClosure({ entryPath: '', },);
        expect(none.kind,).toBe('unavailable',);
      },
    },),
  ],
},);
