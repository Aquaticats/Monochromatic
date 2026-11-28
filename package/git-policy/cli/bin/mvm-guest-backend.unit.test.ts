import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  DEFAULT_GUEST_BACKEND_KIND,
  GUEST_BACKENDS,
  GUEST_BACKEND_ENV_VAR,
  guestBackend,
  isGuestBackendKind,
  resolveGuestBackend,
} from './mvm-guest-backend.ts';

/**
 One backend's facts, as every guest step reads them.
 */
type ExpectedFacts = {
  readonly hostname: string;
  readonly rootWords: readonly string[];
  readonly sharedFolder: string;
  readonly pushDestination: string;
  readonly makesTheFolder: boolean;
};

/**
 The facts every guest step reads, as each backend answers them.
 A table rather than one expectation per claim, so a new backend is one row here.
 */
const EXPECTED_FACTS: Readonly<Record<string, ExpectedFacts>> = {
  libvirt: {
    hostname: 'guest-a',
    rootWords: [],
    sharedFolder: '/mnt/shared',
    pushDestination: '/shared/a.tar.gz',
    makesTheFolder: false,
  },
  ovh: {
    hostname: 'mvm-guest-a',
    rootWords: [
      'sudo',
      '--non-interactive',
      '--',
    ],
    sharedFolder: '/mnt/shared',
    pushDestination: '/mnt/shared/a.tar.gz',
    makesTheFolder: true,
  },
};

await describe({
  name: resolveGuestBackend.name,
  children: [
    it({
      name: 'answers the default when neither the run nor the environment names a backend',
      fn: async () => {
        const backend = resolveGuestBackend({ env: {}, },);
        expect(backend.kind,).toBe(DEFAULT_GUEST_BACKEND_KIND,);
        expect(backend.kind,).toBe('libvirt',);
      },
    },),
    it({
      name: `answers the environment's backend, ${GUEST_BACKEND_ENV_VAR}`,
      fn: async () => {
        const backend = resolveGuestBackend({ env: { [GUEST_BACKEND_ENV_VAR]: 'ovh', }, },);
        expect(backend.kind,).toBe('ovh',);
        expect(backend.mvmArgumentWords,).toEqual(['--backend', 'ovh',],);
      },
    },),
    it({
      name: 'lets the run\'s own word win over the environment',
      fn: async () => {
        const backend = resolveGuestBackend({
          raw: 'libvirt',
          env: { [GUEST_BACKEND_ENV_VAR]: 'ovh', },
        },);
        expect(backend.kind,).toBe('libvirt',);
      },
    },),
    it({
      name: 'falls through an empty word to the environment, as mvm does',
      fn: async () => {
        const backend = resolveGuestBackend({
          raw: '',
          env: { [GUEST_BACKEND_ENV_VAR]: 'ovh', },
        },);
        expect(backend.kind,).toBe('ovh',);
      },
    },),
    it({
      name: 'reads an empty variable as no word at all',
      fn: async () => {
        const backend = resolveGuestBackend({ env: { [GUEST_BACKEND_ENV_VAR]: '', }, },);
        expect(backend.kind,).toBe(DEFAULT_GUEST_BACKEND_KIND,);
      },
    },),
    it({
      name: 'refuses a word no guest driver can drive, naming the word and the ones it can',
      fn: async () => {
        // `hetzner` is a backend `mvm` knows and this package's guest steps do not drive.
        for (const word of ['hetzner', 'kvm',]) {
          let said = '';
          try {
            resolveGuestBackend({ raw: word, env: {}, },);
          }
          catch (caught) {
            said = String(caught,);
          }
          expect(said,).toContain(`unknown backend '${word}'`,);
          for (const kind of Object.keys(GUEST_BACKENDS,)) {
            expect(said,).toContain(kind,);
          }
        }
      },
    },),
    it({
      name: 'reads this process\'s own backend from its environment',
      fn: async () => {
        /** The word this process started with, restored before the claim is read. */
        const saved = process.env[GUEST_BACKEND_ENV_VAR];
        process.env[GUEST_BACKEND_ENV_VAR] = 'ovh';
        /** What this process answers while the variable names ovh. */
        const answered = guestBackend().kind;
        if (saved === undefined) {
          Reflect.deleteProperty(process.env, GUEST_BACKEND_ENV_VAR,);
        }
        else {
          process.env[GUEST_BACKEND_ENV_VAR] = saved;
        }
        expect(answered,).toBe('ovh',);
      },
    },),
  ],
},);

await describe({
  name: isGuestBackendKind.name,
  children: [
    it({
      name: 'reads each kind a driver can drive',
      fn: async () => {
        for (const kind of Object.keys(GUEST_BACKENDS,)) {
          expect(isGuestBackendKind(kind,),).toBe(true,);
        }
      },
    },),
    it({
      name: 'reads a word that is no kind, or nearly one, as no kind',
      fn: async () => {
        expect(isGuestBackendKind('libvirt-2',),).toBe(false,);
        expect(isGuestBackendKind('ovh2',),).toBe(false,);
        expect(isGuestBackendKind('',),).toBe(false,);
      },
    },),
  ],
},);

/**
 One backend's case: the facts every guest step reads, as that backend answers them.

 @param kind - the backend's kind, which is how the case names itself

 @param expected - the facts that backend owes

 @returns the case

 @example
 ```ts
 aBackendCase('ovh', EXPECTED_FACTS.ovh);
 ```
 */
function aBackendCase(
  kind: string,
  expected: {
    readonly hostname: string;
    readonly rootWords: readonly string[];
    readonly sharedFolder: string;
    readonly pushDestination: string;
    readonly makesTheFolder: boolean;
  },
) {
  return it({
    name: `${kind} answers them`,
    fn: async () => {
      const backend = GUEST_BACKENDS[kind as keyof typeof GUEST_BACKENDS];
      expect(backend.mvmArgumentWords,).toEqual(['--backend', kind,],);
      expect(backend.hostnameOf('guest-a',),).toBe(expected.hostname,);
      expect(backend.rootWords,).toEqual(expected.rootWords,);
      expect(backend.guestSharedFolder,).toBe(expected.sharedFolder,);
      expect(backend.pushDestinationOf('a.tar.gz',),).toBe(expected.pushDestination,);
      expect(backend.makesTheSharedFolder !== undefined,).toBe(expected.makesTheFolder,);
    },
  },);
}

await describe({
  name: 'the facts every guest step reads',
  children: Object.entries(EXPECTED_FACTS,)
    .map(function aKind(entry: readonly [string, (typeof EXPECTED_FACTS)[string]],) {
      return aBackendCase(entry[0], entry[1],);
    }),
},);
