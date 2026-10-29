/**
 Unit tests for the provider output parsers.

 These lock two defects a blind review found in `providers.parse.ts`:
 rows separated by a carriage return or a Unicode line terminator
 were swallowed by the row before them,
 and the `cargo binstall --list` reader normalized only one of its two
 version columns.
 */

import {
  describe,
  expect as verify,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  parseBinList,
  parseCargoBinstallList,
} from '../../dist/final/node/index.mjs';

await describe({
  name: 'provider output parsers',
  children: [
    it({
      name: 'parses a plain bin list row',
      fn: async () => {
        const versions = parseBinList('/bin/tool  v1.2.3  https://example.test  ok',);
        verify(versions.get('tool',),).toEqual('1.2.3',);
      },
    },),
    it({
      name: 'keeps rows separated by carriage returns',
      fn: async () => {
        const versions = parseBinList('/bin/a  v1.2.3  https://a.test  ok\r/bin/b  v2.0.0  https://b.test  ok',);
        verify(versions.get('a',),).toEqual('1.2.3',);
        verify(versions.get('b',),).toEqual('2.0.0',);
      },
    },),
    it({
      name: 'keeps rows separated by Unicode line terminators',
      fn: async () => {
        const versions = parseBinList('/bin/a  v1.2.3  https://a.test  ok\u2028/bin/b  v2.0.0  https://b.test  ok',);
        verify(versions.get('a',),).toEqual('1.2.3',);
        verify(versions.get('b',),).toEqual('2.0.0',);
      },
    },),
    it({
      name: 'normalizes the binstall version column like the cargo one',
      fn: async () => {
        const versions = parseCargoBinstallList('ripgrep  v14.1.0  v14.1.0  prebuilt',);
        verify(versions.get('ripgrep',),).toEqual('14.1.0',);
      },
    },),
    it({
      name: 'still normalizes the cargo version when binstall prints a dash',
      fn: async () => {
        const versions = parseCargoBinstallList('ripgrep  v14.1.0  -  compiled',);
        verify(versions.get('ripgrep',),).toEqual('14.1.0',);
      },
    },),
  ],
},);
