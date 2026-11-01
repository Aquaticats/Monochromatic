/**
 Tests for file-based IPC ({@link writeCompactFile}, {@link readCompactFile}).
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { existsSync, } from 'node:fs';
import {
  readCompactFile,
  writeCompactFile,
  type WriteCompactFileResult,
} from './ipc-file.ts';

/** Wrap a writeCompactFile result so `await using` calls cleanup automatically. */
function usingCompactFile(
  result: WriteCompactFileResult,
): AsyncDisposable & {
  filePath: string;
} {
  return {
    filePath: result.filePath,
    [Symbol.asyncDispose]: result.cleanup,
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: writeCompactFile.name,
      children: [
        it({
          name: 'writes text to a temp file under /tmp',
          fn: async () => {
            const text = 'compressed context content';
            await using file = usingCompactFile(await writeCompactFile(text,),);

            expect(file.filePath,).toContain('morph-compact-',);
            expect(existsSync(file.filePath,),).toBe(true,);

            const content = await readCompactFile(file.filePath,);
            expect(content,).toBe(text,);
          },
        },),
        it({
          name: 'handles empty string',
          fn: async () => {
            await using file = usingCompactFile(await writeCompactFile('',),);

            const content = await readCompactFile(file.filePath,);
            expect(content,).toBe('',);
          },
        },),
        it({
          name: 'handles large text (100KB+)',
          fn: async () => {
            const text = 'x'.repeat(150_000,);
            await using file = usingCompactFile(await writeCompactFile(text,),);

            const content = await readCompactFile(file.filePath,);
            expect(content,).toBe(text,);
            expect(content.length,).toBe(150_000,);
          },
        },),
        it({
          name: 'handles text with special characters',
          fn: async () => {
            const text = 'hello\nworld\t"quotes" \'single\' $var `backtick` \\slash\\';
            await using file = usingCompactFile(await writeCompactFile(text,),);

            const content = await readCompactFile(file.filePath,);
            expect(content,).toBe(text,);
          },
        },),
        it({
          name: 'handles binary-safe Unicode content',
          fn: async () => {
            const text = 'unicode: \u0000\u0001\u0002 emoji: 🎉🚀 cjk: 中文日本語';
            await using file = usingCompactFile(await writeCompactFile(text,),);

            const content = await readCompactFile(file.filePath,);
            expect(content,).toBe(text,);
          },
        },),
        it({
          name: 'creates unique paths for each call',
          fn: async () => {
            const first = await writeCompactFile('first',);
            const second = await writeCompactFile('second',);

            expect(first.filePath,).not.toBe(second.filePath,);

            await first.cleanup();
            await second.cleanup();
          },
        },),
      ],
    },),
    describe({
      name: 'readCompactFile.name',
      children: [
        it({
          name: 'reads content written by writeCompactFile',
          fn: async () => {
            const text = 'round-trip test data';
            await using file = usingCompactFile(await writeCompactFile(text,),);

            const content = await readCompactFile(file.filePath,);
            expect(content,).toBe(text,);
          },
        },),
      ],
    },),
    describe({
      name: 'cleanup',
      children: [
        it({
          name: 'removes the temp directory after cleanup',
          fn: async () => {
            const { filePath, cleanup, } = await writeCompactFile('data',);
            const dir = filePath.split('/',).slice(0, -1,).join('/',);

            expect(existsSync(filePath,),).toBe(true,);

            await cleanup();

            expect(existsSync(filePath,),).toBe(false,);
            expect(existsSync(dir,),).toBe(false,);
          },
        },),
        it({
          name: 'is safe to call multiple times',
          fn: async () => {
            const { cleanup, } = await writeCompactFile('data',);

            await cleanup();
            // Second call should not throw
            await cleanup();
          },
        },),
      ],
    },),
  ],
},);
