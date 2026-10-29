/**
 Tests for reading a text setting from the environment (ledger D15): an
 exported-but-empty variable reads as unset, as `artifact-pool.ts` reads its
 own, rather than as a seed or a basename nobody chose.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { textSettingOf, } from '../../dist/final/node/index.mjs';

await describe({
  name: textSettingOf.name,
  children: [
    it({
      name: 'TAKES THE FALLBACK FOR AN UNSET OR EMPTY VARIABLE and the value otherwise: DAMAGE_SAMPLE_SEED and '
        + 'VERIFY_SHEET_BASENAME exported empty were used as the empty string',
      fn: async () => {
        expect([
          textSettingOf({ env: {}, name: 'WHISKERS_SEED', fallback: 'tabby', },),
          textSettingOf({ env: { WHISKERS_SEED: '', }, name: 'WHISKERS_SEED', fallback: 'tabby', },),
          textSettingOf({ env: { WHISKERS_SEED: 'calico', }, name: 'WHISKERS_SEED', fallback: 'tabby', },),
        ],).toEqual(['tabby', 'tabby', 'calico',],);
      },
    },),
  ],
},);
