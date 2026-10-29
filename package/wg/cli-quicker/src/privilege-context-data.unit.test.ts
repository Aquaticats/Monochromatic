import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { parsePrivilegeContext, } from '../dist/final/node/privilege-context-data.mjs';

/**
 Valid serialized caller context with every optional field.
 */
const VALID_CONTEXT = {
  environment: {
    HOME: '/home/caller',
    IPINFO_TOKEN: 'token',
    WG_ALLOWEDIPS_CACHE_DIRECTORY: '/home/caller/.cache/allowedips',
    WG_QUICKER_CALLER_PATH: '/home/caller/bin:/usr/bin',
    WG_QUICKER_EXEMPT_COMMAND: '/repo/target/release/wg-quicker-exempt',
    WG_QUICKER_EXEMPT_UID: '1000',
    WG_QUICKER_OPENSNITCH_DAEMON_CONFIG: '/etc/opensnitchd/default-config.json',
    WG_QUICKER_OPENSNITCH_SYSTEM_FIREWALL_CONFIG: '/etc/opensnitchd/system-fw.json',
    WG_QUICKER_RUNTIME_DIRECTORY: '/run/wg-quicker',
    XDG_CACHE_HOME: '/home/caller/.cache',
  },
  uid: 1_000,
  version: 1,
};

/**
 Invalid serialized context cases and expected diagnostics.
 */
const INVALID_CONTEXTS = [
  {
    name: 'malformed JSON',
    text: '{',
    diagnostic: 'not valid JSON',
  },
  {
    name: 'non-object root',
    text: '[]',
    diagnostic: 'invalid shape',
  },
  {
    name: 'unsupported version',
    text: JSON.stringify({ ...VALID_CONTEXT, version: 2, },),
    diagnostic: 'invalid version',
  },
  {
    name: 'zero UID',
    text: JSON.stringify({ ...VALID_CONTEXT, uid: 0, },),
    diagnostic: 'invalid UID',
  },
  {
    name: 'non-object environment',
    text: JSON.stringify({ ...VALID_CONTEXT, environment: [], },),
    diagnostic: 'environment is invalid',
  },
  {
    name: 'unknown environment key',
    text: JSON.stringify({
      ...VALID_CONTEXT,
      environment: {
        ...VALID_CONTEXT.environment,
        PATH: '/untrusted/bin',
      },
    },),
    diagnostic: 'environment is invalid',
  },
  {
    name: 'non-string environment value',
    text: JSON.stringify({
      ...VALID_CONTEXT,
      environment: {
        ...VALID_CONTEXT.environment,
        IPINFO_TOKEN: 1,
      },
    },),
    diagnostic: 'environment is invalid',
  },
  {
    name: 'empty home',
    text: JSON.stringify({
      ...VALID_CONTEXT,
      environment: {
        ...VALID_CONTEXT.environment,
        HOME: '',
      },
    },),
    diagnostic: 'HOME is invalid',
  },
] as const;

await describe({
  name: parsePrivilegeContext.name,
  children: [
    it({
      name: 'reconstructs allowlisted environment',
      fn: async () => {
        expect(parsePrivilegeContext({ text: JSON.stringify(VALID_CONTEXT,), },),).toEqual(
          VALID_CONTEXT,
        );
      },
    },),
    ...INVALID_CONTEXTS.map(function invalidContext(testCase,) {
      return it({
        name: `rejects ${testCase.name}`,
        fn: async () => {
          expect(() => parsePrivilegeContext({ text: testCase.text, },),).toThrow(
            testCase.diagnostic,
          );
        },
      },);
    },),
  ],
},);
