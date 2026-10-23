/**
 Tests for the run's corpus pin (`run-corpus-pin.ts`): the built-in clone and commit, and the refusal of an
 unreadable override, both read from an environment the case hands in.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import { homedir, } from 'node:os';
import { join, } from 'node:path';

import { CORPUS_COMMIT_SHA, } from '@monochromatic-dev/translation-repair-corpus-source';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import { readRunCorpusPinSetting, } from '../../dist/final/node/index.mjs';

await describe({
  name: readRunCorpusPinSetting.name,
  children: [
    it({
      name: 'READS THE BUILT-IN CLONE AND THE PINNED COMMIT from a handed environment that overrides neither half',
      fn: async () => {
        expect(readRunCorpusPinSetting({ env: {}, },),).toEqual({
          pin: {
            cloneDir: join(
              homedir(),
              'one-among-us',
              'data',
            ),
            commitSha: CORPUS_COMMIT_SHA,
          },
          cloneDirSource: 'fallback',
          commitSource: 'fallback',
        },);
      },
    },),

    it({
      name: 'REFUSES AN UNREADABLE COMMIT in the handed environment as a stated refusal naming the variable',
      fn: async () => {
          /**
           What the reader threw for a commit that is a cat's name.
           */
        const refusal = caught(function readsUnderBadCommit(): unknown {
          return readRunCorpusPinSetting({ env: { TRANSLATION_REPAIR_CORPUS_COMMIT: 'whiskers', }, },);
        },);
        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect(String(refusal,),).toBe(
          'StatedRefusalError: TRANSLATION_REPAIR_CORPUS_COMMIT must be one full lowercase hexadecimal '
            + 'commit name, and whiskers is not one',
        );
      },
    },),
  ],
},);
