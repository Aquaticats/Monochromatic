import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { generateAllowedIpsWithLookup, } from '../dist/final/node/generate-with-lookup.mjs';
import {
  captureError,
  fixtureAsnLookup,
  fixtureLookup,
} from './test-fixtures.ts';

/**
 Invalid CIDR syntax fixture and expected diagnostic fragment.
 */
type InvalidCidrCase = {
  readonly entry: string;
  readonly expected: string;
};

/**
 Dependency parser failure paths that must propagate.
 */
const INVALID_CIDR_CASES: readonly InvalidCidrCase[] = [
  {
    entry: '192.0.2.1/24junk',
    expected: 'not a CIDR or IP',
  },
  {
    entry: '192.0.2.1/24/25',
    expected: 'not a CIDR or IP',
  },
  {
    entry: '/24',
    expected: 'Invalid IP address',
  },
];

await describe({
  name: generateAllowedIpsWithLookup.name,
  children: [
    //region Input lines

    it({
      name: 'trims lines and skips blanks and whole-line comments',
      fn: async () => {
        /**
         Output from active Windows- and Unix-terminated lines.
         */
        const output = await generateAllowedIpsWithLookup({
          allowedText: '  \r\n # comment\r\n 192.0.2.1 \r\n 2001:db8::1\n',
          disallowedText: '# none\n',
          lookupAddresses: fixtureLookup,
          lookupAsnNetworks: fixtureAsnLookup,
        },);
        expect(output,).toBe('192.0.2.1/32, 2001:db8::1/128\n',);
      },
    },),

    it({
      name: 'normalizes CIDR host bits before subtraction',
      fn: async () => {
        /**
         Normalized dual-stack networks.
         */
        const output = await generateAllowedIpsWithLookup({
          allowedText: '192.0.2.7/24\n2001:db8::7/126',
          disallowedText: '',
          lookupAddresses: fixtureLookup,
          lookupAsnNetworks: fixtureAsnLookup,
        },);
        expect(output,).toBe('192.0.2.0/24, 2001:db8::4/126\n',);
      },
    },),

    it({
      name: 'treats inline comment text as part of a domain',
      fn: async () => {
        /**
         Host route returned for exact inline-comment-looking domain fixture.
         */
        const output = await generateAllowedIpsWithLookup({
          allowedText: 'inline#comment.example',
          disallowedText: '',
          lookupAddresses: fixtureLookup,
          lookupAsnNetworks: fixtureAsnLookup,
        },);
        expect(output,).toBe('198.51.100.7/32\n',);
      },
    },),

    //endregion Input lines

    //region Domain resolution

    it({
      name: 'adds every domain address and subtracts resolved disallowed addresses',
      fn: async () => {
        /**
         Domain result after removing its IPv4 address.
         */
        const output = await generateAllowedIpsWithLookup({
          allowedText: 'allowed.example',
          disallowedText: 'disallowed.example',
          lookupAddresses: fixtureLookup,
          lookupAsnNetworks: fixtureAsnLookup,
        },);
        expect(output,).toBe('2001:db8::1/128\n',);
      },
    },),

    it({
      name: 'skips every domain whose lookup returns ENOTFOUND',
      fn: async () => {
        /**
         Direct route retained while each unresolved domain contributes nothing.
         */
        const output = await generateAllowedIpsWithLookup({
          allowedText: '192.0.2.1\nmissing-one.example',
          disallowedText: 'missing-two.example',
          lookupAddresses: fixtureLookup,
          lookupAsnNetworks: fixtureAsnLookup,
        },);
        expect(output,).toBe('192.0.2.1/32\n',);
      },
    },),

    it({
      name: 'propagates lookup failure other than ENOTFOUND',
      fn: async () => {
        /**
         Error from unregistered deterministic resolver hostname.
         */
        const error = await captureError({
          operation: async function generateUnknownDomain(): Promise<string> {
            return await generateAllowedIpsWithLookup({
              allowedText: 'unknown.example',
              disallowedText: '',
              lookupAddresses: fixtureLookup,
              lookupAsnNetworks: fixtureAsnLookup,
            },);
          },
        },);
        expect(String(error,),).toContain('Unexpected lookup: unknown.example',);
      },
    },),

    it({
      name: 'rejects an invalid address returned by the resolver',
      fn: async () => {
        /**
         Error naming invalid resolver output and its domain.
         */
        const error = await captureError({
          operation: async function generateInvalidResolvedAddress(): Promise<string> {
            return await generateAllowedIpsWithLookup({
              allowedText: 'invalid-address.example',
              disallowedText: '',
              lookupAddresses: fixtureLookup,
              lookupAsnNetworks: fixtureAsnLookup,
            },);
          },
        },);
        expect(String(error,),).toContain('Invalid IP address from domain invalid-address.example: not-an-ip',);
      },
    },),

    //endregion Domain resolution

    //region ASN resolution

    it({
      name: 'adds case-insensitive ASN networks and subtracts another ASN',
      fn: async () => {
        /**
         Dual-stack ASN result after subtracting first half of each network.
         */
        const output = await generateAllowedIpsWithLookup({
          allowedText: 'as64500',
          disallowedText: 'AS64501',
          lookupAddresses: fixtureLookup,
          lookupAsnNetworks: fixtureAsnLookup,
        },);
        expect(output,).toBe('192.0.2.128/25, 2001:db8:100:8000::/49\n',);
      },
    },),

    it({
      name: 'turns single addresses from ASN database into host routes',
      fn: async () => {
        /**
         Host routes from database records without prefixes.
         */
        const output = await generateAllowedIpsWithLookup({
          allowedText: 'AS64502',
          disallowedText: '',
          lookupAddresses: fixtureLookup,
          lookupAsnNetworks: fixtureAsnLookup,
        },);
        expect(output,).toBe('198.51.100.9/32, 2001:db8::9/128\n',);
      },
    },),

    it({
      name: 'warns once for each ASN contributing no networks',
      fn: async ({ sinon, }) => {
        /**
         Console warning spy observing built logger output after its microtask flush.
         */
        const warningSpy = sinon.spy(
          console,
          'warn',
        );
        /**
         Direct route retained while empty ASNs in both sets contribute nothing.
         */
        const output = await generateAllowedIpsWithLookup({
          allowedText: '192.0.2.1\nAS64503',
          disallowedText: 'AS64506',
          lookupAddresses: fixtureLookup,
          lookupAsnNetworks: fixtureAsnLookup,
        },);
        await Promise.resolve();
        /**
         Every console warning argument flattened for per-ASN occurrence checks.
         */
        const warningText = warningSpy.args
          .map(function warningArgument([message,],): string {
            return String(message,);
          },)
          .join('\n',);
        expect(output,).toBe('192.0.2.1/32\n',);
        expect(
          warningText.split('ASN AS64503 contributed no networks; skipping ASN',).length - 1,
        ).toBe(1,);
        expect(
          warningText.split('ASN AS64506 contributed no networks; skipping ASN',).length - 1,
        ).toBe(1,);
      },
    },),

    it({
      name: 'rejects an invalid network returned by ASN database',
      fn: async () => {
        /**
         Invalid ASN database record diagnostic.
         */
        const error = await captureError({
          operation: async function generateInvalidAsnNetwork(): Promise<string> {
            return await generateAllowedIpsWithLookup({
              allowedText: 'AS64504',
              disallowedText: '',
              lookupAddresses: fixtureLookup,
              lookupAsnNetworks: fixtureAsnLookup,
            },);
          },
        },);
        expect(String(error,),).toContain('Invalid network from ASN AS64504: not-a-network',);
      },
    },),

    it({
      name: 'propagates ASN lookup failure',
      fn: async () => {
        /**
         Error from unregistered deterministic ASN resolver fixture.
         */
        const error = await captureError({
          operation: async function generateUnknownAsn(): Promise<string> {
            return await generateAllowedIpsWithLookup({
              allowedText: 'AS64505',
              disallowedText: '',
              lookupAddresses: fixtureLookup,
              lookupAsnNetworks: fixtureAsnLookup,
            },);
          },
        },);
        expect(String(error,),).toContain('Unexpected ASN lookup: AS64505',);
      },
    },),

    //endregion ASN resolution

    //region Empty allowed set

    it({
      name: 'rejects blank and comment-only allowed input',
      fn: async () => {
        /**
         Empty-allowed diagnostic.
         */
        const error = await captureError({
          operation: async function generateEmptyAllowed(): Promise<string> {
            return await generateAllowedIpsWithLookup({
              allowedText: ' \n# comment\n',
              disallowedText: '',
              lookupAddresses: fixtureLookup,
              lookupAsnNetworks: fixtureAsnLookup,
            },);
          },
        },);
        expect(String(error,),).toContain('Allowed input must contain at least one address',);
      },
    },),

    it({
      name: 'rejects an allowed domain resolving to no addresses',
      fn: async () => {
        /**
         Empty-resolver-result diagnostic.
         */
        const error = await captureError({
          operation: async function generateEmptyResolvedAllowed(): Promise<string> {
            return await generateAllowedIpsWithLookup({
              allowedText: 'empty.example',
              disallowedText: '',
              lookupAddresses: fixtureLookup,
              lookupAsnNetworks: fixtureAsnLookup,
            },);
          },
        },);
        expect(String(error,),).toContain('Allowed input must contain at least one address',);
      },
    },),

    //endregion Empty allowed set

    //region CIDR validation

    it({
      name: 'rejects nonstandard shorthand IPv4 accepted by the dependency parser',
      fn: async () => {
        /**
         Original-address validation diagnostic.
         */
        const error = await captureError({
          operation: async function generateShorthandIpv4(): Promise<string> {
            return await generateAllowedIpsWithLookup({
              allowedText: '127.1/24',
              disallowedText: '',
              lookupAddresses: fixtureLookup,
              lookupAsnNetworks: fixtureAsnLookup,
            },);
          },
        },);
        expect(String(error,),).toContain('Invalid IP address in CIDR entry: 127.1/24',);
      },
    },),

    it({
      name: 'rejects an IPv4 prefix above 32 and names the entry',
      fn: async () => {
        /**
         IPv4 family-bound diagnostic.
         */
        const error = await captureError({
          operation: async function generateOversizedIpv4Prefix(): Promise<string> {
            return await generateAllowedIpsWithLookup({
              allowedText: '192.0.2.1/33',
              disallowedText: '',
              lookupAddresses: fixtureLookup,
              lookupAsnNetworks: fixtureAsnLookup,
            },);
          },
        },);
        expect(String(error,),).toContain('IPv4 maximum 32: 192.0.2.1/33',);
      },
    },),

    it({
      name: 'rejects an IPv6 prefix above 128 and names the entry',
      fn: async () => {
        /**
         IPv6 family-bound diagnostic.
         */
        const error = await captureError({
          operation: async function generateOversizedIpv6Prefix(): Promise<string> {
            return await generateAllowedIpsWithLookup({
              allowedText: '2001:db8::1/129',
              disallowedText: '',
              lookupAddresses: fixtureLookup,
              lookupAsnNetworks: fixtureAsnLookup,
            },);
          },
        },);
        expect(String(error,),).toContain('IPv6 maximum 128: 2001:db8::1/129',);
      },
    },),

    ...INVALID_CIDR_CASES.map(function invalidCidrCase({
      entry,
      expected,
    }: InvalidCidrCase,) {
      return it({
        name: `propagates parser rejection for ${entry}`,
        fn: async () => {
          /**
           Dependency parser diagnostic.
           */
          const error = await captureError({
            operation: async function generateInvalidCidr(): Promise<string> {
              return await generateAllowedIpsWithLookup({
                allowedText: entry,
                disallowedText: '',
                lookupAddresses: fixtureLookup,
                lookupAsnNetworks: fixtureAsnLookup,
              },);
            },
          },);
          expect(String(error,),).toContain(expected,);
        },
      },);
    },),

    //endregion CIDR validation
  ],
},);
