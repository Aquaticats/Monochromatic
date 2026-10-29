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

await describe({
  name: 'loopback exclusion warning',
  children: [
    it({
      name: 'warns for missing or partial coverage and accepts exact or broader coverage',
      fn: async ({ sinon, }) => {
        /**
         Console warning spy observing built logger output after its microtask flush.
         */
        const warningSpy = sinon.spy(
          console,
          'warn',
        );
        /**
         Result produced while both loopback ranges remain uncovered.
         */
        const missingOutput = await generateAllowedIpsWithLookup({
          allowedText: '192.0.2.1',
          disallowedText: '',
          lookupAddresses: fixtureLookup,
          lookupAsnNetworks: fixtureAsnLookup,
        },);
        /**
         Result produced while half of IPv4 loopback remains uncovered.
         */
        const partialOutput = await generateAllowedIpsWithLookup({
          allowedText: '192.0.2.1',
          disallowedText: '127.0.0.0/9\n::1/128',
          lookupAddresses: fixtureLookup,
          lookupAsnNetworks: fixtureAsnLookup,
        },);
        /**
         Result produced with exact complete loopback coverage.
         */
        const exactOutput = await generateAllowedIpsWithLookup({
          allowedText: '192.0.2.1',
          disallowedText: '127.0.0.0/8\n::1/128',
          lookupAddresses: fixtureLookup,
          lookupAsnNetworks: fixtureAsnLookup,
        },);
        /**
         Result produced with broader networks covering every loopback address.
         */
        const broaderOutput = await generateAllowedIpsWithLookup({
          allowedText: '192.0.2.1',
          disallowedText: '0.0.0.0/0\n::/0',
          lookupAddresses: fixtureLookup,
          lookupAsnNetworks: fixtureAsnLookup,
        },);
        await Promise.resolve();
        /**
         Every console warning argument flattened for occurrence checks.
         */
        const warningText = warningSpy.args
          .map(function warningArgument([message,],): string {
            return String(message,);
          },)
          .join('\n',);
        expect(missingOutput,).toBe('192.0.2.1/32\n',);
        expect(partialOutput,).toBe('192.0.2.1/32\n',);
        expect(exactOutput,).toBe('192.0.2.1/32\n',);
        expect(broaderOutput,).toBe('',);
        expect(
          warningText.split(
            'disallowed IPs do not cover all loopback ranges; uncovered: 127.0.0.0/8, ::1/128',
          ).length - 1,
        ).toBe(1,);
        expect(
          warningText.split(
            'disallowed IPs do not cover all loopback ranges; uncovered: 127.128.0.0/9',
          ).length - 1,
        ).toBe(1,);
        expect(
          warningText.split('disallowed IPs do not cover all loopback ranges',).length - 1,
        ).toBe(2,);
      },
    },),
  ],
},);
