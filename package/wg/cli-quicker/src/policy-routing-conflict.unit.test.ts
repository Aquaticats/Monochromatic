import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { isIvpnSplitRule, } from '../dist/final/node/policy-routing-conflict.mjs';

await describe({
  name: isIvpnSplitRule.name,
  children: [
    ...[
      {
        name: 'accepts canonical hex mark with numeric table text',
        value: {
          fwmark: '0xca6c',
          table: '17',
        },
      },
      {
        name: 'accepts zero-padded hex mark with numeric table',
        value: {
          fwmark: '0x0000ca6c',
          table: 17,
        },
      },
      {
        name: 'accepts decimal mark with named table',
        value: {
          fwmark: 51_820,
          table: 'ivpn-exclude-tbl',
        },
      },
    ].map(function validRule({ name, value, },) {
      return it({
        name,
        fn: async () => {
          expect(isIvpnSplitRule(value,),).toBe(true,);
        },
      },);
    },),
    ...[
      {
        name: 'rejects null',
        value: null,
      },
      {
        name: 'rejects absent mark',
        value: {
          table: '17',
        },
      },
      {
        name: 'rejects unrelated mark',
        value: {
          fwmark: '0x22b8',
          table: '17',
        },
      },
      {
        name: 'rejects unrelated table',
        value: {
          fwmark: '0xca6c',
          table: 'main',
        },
      },
      {
        name: 'rejects malformed mark',
        value: {
          fwmark: 'ca6c',
          table: '17',
        },
      },
    ].map(function invalidRule({ name, value, },) {
      return it({
        name,
        fn: async () => {
          expect(isIvpnSplitRule(value,),).toBe(false,);
        },
      },);
    },),
  ],
},);
