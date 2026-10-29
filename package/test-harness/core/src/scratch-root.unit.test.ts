/**
 Tests for `scratchRoot`, which gives module tests a directory for fixtures that
 is never the system temporary directory.

 @module
 */

import {
  existsSync,
  statSync,
} from 'node:fs';
import { tmpdir, } from 'node:os';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core';
import { scratchRoot, } from './scratch-root.ts';

await describe({
  name: scratchRoot.name,
  children: [
    it({
      name: 'RETURNS AN EXISTING DIRECTORY, so a caller can create a fixture inside it without making the '
        + 'parent first',
      fn: async function returnsAnExistingDirectory() {
        const root = scratchRoot();

        expect(existsSync(root,),).toBe(true,);
        expect(statSync(root,).isDirectory(),).toBe(true,);
      },
    },),

    it({
      name: 'IS NOT INSIDE THE SYSTEM TEMPORARY DIRECTORY, which is the whole point: a fixture there fills a '
        + 'shared tmpfs quota when a test leaks it',
      fn: async function staysOutOfTheSystemTemporaryDirectory() {
        const root = scratchRoot();

        expect(root.startsWith(tmpdir(),),).toBe(false,);
      },
    },),

    it({
      name: 'LANDS IN THE USER CACHE DIRECTORY without an override, outside every checkout, so a fixture is '
        + 'never found by a tool that walks upward looking for a repository',
      fn: async function landsInTheUserCache() {
        const previous = process.env['MONOCHROMATIC_TEST_SCRATCH'];
        delete process.env['MONOCHROMATIC_TEST_SCRATCH'];
        try {
          expect(scratchRoot().endsWith('monochromatic-dev/test-scratch',),).toBe(true,);
        } finally {
          if (previous !== undefined) {
            process.env['MONOCHROMATIC_TEST_SCRATCH'] = previous;
          }
        }
      },
    },),

    it({
      name: 'FOLLOWS THE OVERRIDE VARIABLE when it names a directory, creating it if it is missing',
      fn: async function followsTheOverride() {
        const previous = process.env['MONOCHROMATIC_TEST_SCRATCH'];
        const base = scratchRoot();
        const overridden = `${base}/override-check`;
        process.env['MONOCHROMATIC_TEST_SCRATCH'] = overridden;
        try {
          expect(scratchRoot(),).toBe(overridden,);
          expect(existsSync(overridden,),).toBe(true,);
        } finally {
          if (previous === undefined) {
            delete process.env['MONOCHROMATIC_TEST_SCRATCH'];
          } else {
            process.env['MONOCHROMATIC_TEST_SCRATCH'] = previous;
          }
        }
      },
    },),
  ],
},);
