import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  AsnDatabaseError,
  normalizeAsn,
  validateNetwork,
} from '../dist/final/node/asn-network.mjs';

/**
 Captures synchronous error from operation expected to throw.
 
 @param operation - Operation expected to throw.
 
 @returns Thrown value.
 
 @example
 ```ts
 captureThrown({ operation: () => { throw new Error('fixture'); } });
 ```
 */
function captureThrown(
  { operation, }: { readonly operation: () => unknown; },
): unknown {
  try {
    operation();
  }
  catch (error: unknown) {
    return error;
  }
  throw new Error('Expected operation to throw.',);
}

await describe({
  name: '',
  children: [
    describe({
      name: normalizeAsn.name,
      children: [
        it({
          name: 'trims and normalizes valid lowercase input',
          fn: async () => {
            expect(normalizeAsn('  as64500  ',),).toBe('AS64500',);
          },
        },),
        ...[
          '',
          '64500',
          'AS',
          'AS64x00',
        ].map(function invalidAsn(asn: string,) {
          return it({
            name: `rejects ${JSON.stringify(asn,)}`,
            fn: async () => {
              /**
               Validation failure for invalid ASN fixture.
               */
              const error = captureThrown({
                operation: function normalizeInvalidAsn(): string {
                  return normalizeAsn(asn,);
                },
              },);
              expect(error,).toBeInstanceOf(AsnDatabaseError,);
              expect(String(error,),).toContain('ASN must use AS<number> syntax',);
            },
          },);
        },),
      ],
    },),
    describe({
      name: validateNetwork.name,
      children: [
        ...[
          '192.0.2.1',
          '2001:db8::1',
          '192.0.2.0/24',
          '2001:db8::/32',
        ].map(function validNetwork(network: string,) {
          return it({
            name: `accepts ${network}`,
            fn: async () => {
              expect(validateNetwork({
                network,
                targetAsn: 'AS64500',
              },),).toBe(network,);
            },
          },);
        },),
        ...[
          'not-a-network',
          '/24',
          '192.0.2.0/24/25',
          '192.0.2.0/prefix',
          '192.0.2.0/33',
          '2001:db8::/129',
        ].map(function invalidNetwork(network: string,) {
          return it({
            name: `rejects ${network}`,
            fn: async () => {
              /**
               Validation failure for invalid network fixture.
               */
              const error = captureThrown({
                operation: function validateInvalidNetwork(): string {
                  return validateNetwork({
                    network,
                    targetAsn: 'AS64500',
                  },);
                },
              },);
              expect(error,).toBeInstanceOf(AsnDatabaseError,);
              expect(String(error,),).toContain(`Invalid network for AS64500: ${network}`,);
            },
          },);
        },),
      ],
    },),
  ],
},);
