/**
 Tests for the clipboard read and the file-manager stub.
 
 @module
 */

import {
  mkdtemp,
  readFile,
  rm,
  stat,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  describe,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  readClipboard,
  stubScript,
  writeFileManagerStub,
} from './clipboard.ts';
import { runCommand, } from './command.ts';

await describe({
  name: stubScript.name,
  children: [
    it({
      name: 'starts with a Node shebang and appends the arguments to the record file',
      fn: async ({ expect, }) => {
        const script = stubScript('/work/record "x".txt',);
        expect(script.startsWith('#!/usr/bin/env node\n',),).toBe(true,);
        expect(script,).toContain(JSON.stringify('/work/record "x".txt',),);
        expect(script,).toContain('appendFileSync',);
      },
    },),
  ],
},);

await describe({
  name: writeFileManagerStub.name,
  children: [
    it({
      name: 'writes an executable xdg-open that records its arguments',
      fn: async ({ expect, }) => {
        const dir = await mkdtemp(join(scratchRoot(), 'e2e-stub-',),);
        const record = join(dir, 'record.txt',);
        await writeFile(record, '',);
        await writeFileManagerStub({ binDir: join(dir, 'bin',), record, },);
        const stub = join(dir, 'bin', 'xdg-open',);
        expect(((await stat(stub,)).mode & 0o111) !== 0,).toBe(true,);
        const ran = await runCommand({ command: stub, args: ['/music/Zzz Tones',], },);
        expect(ran.status,).toBe(0,);
        expect(await readFile(record, 'utf8',),).toBe('/music/Zzz Tones\n',);
        await rm(dir, { recursive: true, force: true, },);
      },
      timeout: 30_000,
    },),
  ],
},);

await describe({
  name: readClipboard.name,
  children: [
    it({
      name: 'reports a failure note when no clipboard is reachable on the display',
      fn: async ({ expect, }) => {
        expect((await readClipboard('wayland-no-such-display',)).startsWith('<wl-paste failed:',),).toBe(true,);
      },
      timeout: 30_000,
    },),
  ],
},);
