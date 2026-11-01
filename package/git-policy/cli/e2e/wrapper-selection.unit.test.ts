import {
  mkdir,
  mkdtempDisposable,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  parseWrapperKind,
  verifyWrapperExecutable,
  WRAPPER_BINS,
  WrapperSelectionError,
} from './wrapper-selection-fixture.ts';

await describe({
  name: parseWrapperKind.name,
  children: [
    it({
      name: 'selects the native executable for an empty value, the only wrapper there is',
      fn: async () => {
        expect(parseWrapperKind('',),).toBe('native',);
      },
    },),
    it({
      name: 'selects the native executable by name',
      fn: async () => {
        expect(parseWrapperKind('native',),).toBe('native',);
      },
    },),
    it({
      name: 'rejects other values, including the retired incumbent, instead of falling back to a default',
      fn: async () => {
        expect(function wrongCase() {
          return parseWrapperKind('Native',);
        },).toThrow(WrapperSelectionError,);
        expect(function retired() {
          return parseWrapperKind('incumbent',);
        },).toThrow(WrapperSelectionError,);
        expect(function unknown() {
          return parseWrapperKind('shell',);
        },).toThrow(WrapperSelectionError,);
      },
    },),
    it({
      name: 'maps the native kind to its directory',
      fn: async () => {
        expect(WRAPPER_BINS.native,).toBe('/opt/native/bin',);
      },
    },),
  ],
},);

await describe({
  name: verifyWrapperExecutable.name,
  children: [
    it({
      name: 'accepts an ELF git as native, and rejects a script git and an absent git',
      fn: async () => {
        /**
         Disposable directory holding a fake ELF `git` and a fake script `git`.
         */
        await using root = await mkdtempDisposable(join(scratchRoot(), 'wrapper-selection-',),);
        const elf = join(root.path, 'elf',);
        const script = join(root.path, 'script',);
        await Promise.all([elf, script,].map(async function prepare(directory,) {
          await mkdir(directory,);
        },),);
        await writeFile(join(elf, 'git',), Buffer.from('\u007FELF\u0000\u0000', 'latin1',),);
        await writeFile(join(script, 'git',), '#!/usr/bin/env node\n',);
        expect(await verifyWrapperExecutable({ kind: 'native', bin: elf, },),)
          .toBe(`wrapper=native ${join(elf, 'git',)} (ELF executable)`,);
        await expect(verifyWrapperExecutable({ kind: 'native', bin: script, },),)
          .rejects.toThrow(WrapperSelectionError,);
        await expect(verifyWrapperExecutable({ kind: 'native', bin: join(root.path, 'absent',), },),)
          .rejects.toThrow(WrapperSelectionError,);
      },
    },),
  ],
},);
