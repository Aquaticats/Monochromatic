import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { generateAllowedIpsWithLookup, } from '../dist/final/node/generate-with-lookup.mjs';
import {
  fixtureAsnLookup,
  fixtureLookup,
} from './test-fixtures.ts';

/**
 Exact set-operation fixture.
 */
type SetCase = {
  readonly name: string;
  readonly allowedText: string;
  readonly disallowedText: string;
  readonly expected: string;
};

/**
 Distinct arithmetic and formatting paths required by the command contract.
 */
const SET_CASES: readonly SetCase[] = [
  {
    name: 'subtracts an IPv4 half-network',
    allowedText: '10.0.0.0/8',
    disallowedText: '10.0.0.0/9',
    expected: '10.128.0.0/9\n',
  },
  {
    name: 'subtracts an IPv6 half-network',
    allowedText: '2001:db8::/126',
    disallowedText: '2001:db8::/127',
    expected: '2001:db8::2/127\n',
  },
  {
    name: 'merges duplicates and overlaps before partial subtraction',
    allowedText: '10.0.0.0/8\n10.0.0.0/9\n10.128.0.0/9',
    disallowedText: '10.0.0.0/10\n192.0.2.0/24',
    expected: '10.64.0.0/10, 10.128.0.0/9\n',
  },
  {
    name: 'minimizes adjacent blocks with an empty disallowed set',
    allowedText: '192.0.2.0/25\n192.0.2.128/25\n::/128\n::1/128',
    disallowedText: '',
    expected: '192.0.2.0/24, ::/127\n',
  },
  {
    name: 'returns empty output for complete dual-stack subtraction',
    allowedText: '0.0.0.0/0\n::/0',
    disallowedText: '0.0.0.0/0\n::/0',
    expected: '',
  },
  {
    name: 'sorts IPv4 before numerically smaller IPv6 output',
    allowedText: '255.255.255.255\n::',
    disallowedText: '',
    expected: '255.255.255.255/32, ::/128\n',
  },
  {
    name: 'removes multiple disjoint bases covered by one exclusion',
    allowedText: '10.0.0.0/30\n10.0.0.8/30',
    disallowedText: '10.0.0.0/28',
    expected: '',
  },
];

await describe({
  name: generateAllowedIpsWithLookup.name,
  children: SET_CASES.map(function setCase({
    name,
    allowedText,
    disallowedText,
    expected,
  }: SetCase,) {
    return it({
      name,
      fn: async () => {
        /**
         Output from built internal seam.
         */
        const output = await generateAllowedIpsWithLookup({
          allowedText,
          disallowedText,
          lookupAddresses: fixtureLookup,
          lookupAsnNetworks: fixtureAsnLookup,
        },);
        expect(output,).toBe(expected,);
      },
    },);
  },),
},);
