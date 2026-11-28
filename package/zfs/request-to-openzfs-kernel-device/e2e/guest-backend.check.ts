// Holds the one place the recipe takes its backend from, without a guest.
//
// What:  resolves the backend from a run's own word, its environment and the default; reads
//        what each backend answers for the four facts a guest step needs; and refuses the
//        steps whose subject is a reboot or a power cut on a backend that cannot power-cycle.
// Why the mvm source too: the variable and the default are mvm's own, so the check reads
//        them out of `package/mvm/cli/src/backend/registry.ts` rather than trusting a copy.
//        A recipe that drifted from mvm's variable would still pass its own tests and would
//        quietly make a local guest while the run asked for a cloud one.
// Why the recipe's own text: the fault this change was written against is a literal at a
//        call site, not a wrong answer in one function. A step that names `'libvirt'` or
//        `/mnt/shared` itself works on one backend whatever this module answers.
//
// Usage: node e2e/guest-backend.check.ts

import { readFileSync, } from 'node:fs';
import { join, } from 'node:path';

import {
  DEFAULT_GUEST_BACKEND_KIND,
  GUEST_BACKENDS,
  GUEST_BACKEND_ENV_VAR,
  isGuestBackendKind,
  POWER_CYCLE_STEPS,
  refuseAStepThatNeedsAPowerCycle,
  resolveGuestBackend,
} from './guest-backend.ts';

//region Holding claims

/** The claims this check has held so far, printed once at the end. */
let held: number = 0;

/**
 Hold one claim, which throws and ends the check when it no longer holds.

 @param claim - what the check asserts

 @param holds - whether it held

 @throws Error naming the claim that no longer holds

 @example
 ```ts
 hold('the default is libvirt', backend.kind === 'libvirt');
 ```
 */
function hold(
  claim: string,
  holds: boolean,
): void {
  if (!holds) {
    throw new Error(`the guest backend no longer holds: ${claim}`,);
  }
  held += 1;
  console.log(`held: ${claim}`,);
}

//endregion Holding claims

//region mvm's own words

/** The registry that resolves a backend for every `mvm`, read as text. */
const registry = readFileSync(
  join(
    import.meta.dirname,
    '..',
    '..',
    '..',
    'mvm',
    'cli',
    'src',
    'backend',
    'registry.ts',
  ),
  'utf8',
);

hold(
  `the variable read here is mvm's own, ${GUEST_BACKEND_ENV_VAR}`,
  registry.includes(`export const BACKEND_ENV_VAR = '${GUEST_BACKEND_ENV_VAR}';`,),
);
hold(
  `the default here is mvm's own, ${DEFAULT_GUEST_BACKEND_KIND}`,
  registry.includes(`export const DEFAULT_BACKEND_KIND: BackendKind = '${DEFAULT_GUEST_BACKEND_KIND}';`,),
);
/** The recipe's text, read once, so a literal at a call site is what the claims below are about. */
const RECIPE_TEXT = readFileSync(
  join(
    import.meta.dirname,
    'run-fixture.ts',
  ),
  'utf8',
);

hold(
  'mvm still resolves a flag word before the variable and the variable before the default',
  registry.includes('const candidate = ((raw === undefined) || (raw === \'\'))',),
);

//endregion mvm's own words

//region Resolution

hold(
  'a run that says nothing gets mvm\'s default',
  resolveGuestBackend({ env: {}, },).kind === DEFAULT_GUEST_BACKEND_KIND,
);
hold(
  `a run with ${GUEST_BACKEND_ENV_VAR}=ovh gets ovh`,
  resolveGuestBackend({ env: { [GUEST_BACKEND_ENV_VAR]: 'ovh', }, },).kind === 'ovh',
);
hold(
  'a run\'s own --backend word wins over the variable',
  resolveGuestBackend({
    raw: 'libvirt',
    env: { [GUEST_BACKEND_ENV_VAR]: 'ovh', },
  },).kind === 'libvirt',
);
hold(
  'an empty --backend word falls through to the variable',
  resolveGuestBackend({
    raw: '',
    env: { [GUEST_BACKEND_ENV_VAR]: 'ovh', },
  },).kind === 'ovh',
);
hold(
  'an empty variable is no word at all',
  resolveGuestBackend({ env: { [GUEST_BACKEND_ENV_VAR]: '', }, },).kind === DEFAULT_GUEST_BACKEND_KIND,
);

/**
 The words a run can give that name nothing this recipe drives: one `mvm` knows and this recipe does not, and one no
 backend has.
 */
const UNKNOWN_WORDS: readonly string[] = ['hetzner', 'kvm',];

for (const word of UNKNOWN_WORDS) {
  /** What the refusal said, so the claim rests on the words a person reads. */
  let said = '';
  try {
    resolveGuestBackend({ raw: word, env: {}, },);
  }
  catch (caught) {
    said = String(caught,);
  }
  hold(
    `'${word}' is refused rather than silently filled in`,
    said.includes(`unknown backend '${word}'`,),
  );
  hold(
    `the refusal of '${word}' names every backend this recipe drives`,
    Object.keys(GUEST_BACKENDS,)
      .every((kind: string,): boolean => said.includes(kind,)),
  );
}

hold(
  'every kind the registry answers for is a kind this recipe drives',
  Object.keys(GUEST_BACKENDS,)
    .every((kind: string,): boolean => isGuestBackendKind(kind,)),
);
hold(
  'a word that is not a kind is no kind',
  !isGuestBackendKind('libvirt-2',),
);

//endregion Resolution

//region What each backend answers

/** The four facts every guest step reads, as each backend answers them. */
const EXPECTED_FACTS: Readonly<Record<
  string,
  {
    readonly hostname: string;
    readonly rootWords: readonly string[];
    readonly pushDestination: string;
    readonly pushedFolder: string;
    readonly powerCycles: boolean;
    readonly makesTheFolder: boolean;
  }
>> = {
  libvirt: {
    hostname: 'guest-a',
    rootWords: [],
    pushDestination: '/shared/a.tar.gz',
    pushedFolder: '/mnt/shared',
    powerCycles: true,
    makesTheFolder: false,
  },
  ovh: {
    hostname: 'mvm-guest-a',
    rootWords: [
      'sudo',
      '--non-interactive',
      '--',
    ],
    pushDestination: '/mnt/shared/a.tar.gz',
    pushedFolder: '/mnt/shared',
    powerCycles: false,
    makesTheFolder: true,
  },
};

for (const [kind, expected,] of Object.entries(EXPECTED_FACTS,)) {
  /** The backend under test. */
  const backend = GUEST_BACKENDS[kind as keyof typeof GUEST_BACKENDS];
  hold(
    `${kind} names itself on mvm's command line`,
    JSON.stringify(backend.mvmArgumentWords,) === JSON.stringify(['--backend', kind,],),
  );
  hold(
    `${kind} answers the guest hostname`,
    backend.hostnameOf('guest-a',) === expected.hostname,
  );
  hold(
    `${kind} starts a root command with ${expected.rootWords.join(' ',) || 'nothing'}`,
    JSON.stringify(backend.rootWords,) === JSON.stringify(expected.rootWords,),
  );
  hold(
    `${kind} pushes a file to ${expected.pushDestination}`,
    backend.pushDestinationOf('a.tar.gz',) === expected.pushDestination,
  );
  hold(
    `${kind} reads a pushed file back at ${expected.pushedFolder}`,
    backend.pushedFolder === expected.pushedFolder,
  );
  hold(
    `${kind} ${expected.powerCycles ? 'can' : 'cannot'} power-cycle its guest`,
    backend.powerCycles === expected.powerCycles,
  );
  hold(
    `${kind} ${expected.makesTheFolder ? 'makes' : 'does not make'} the folder its pushes land in`,
    (backend.makesThePushedFolder !== undefined) === expected.makesTheFolder,
  );
}

//endregion What each backend answers

//region Steps refused by name

hold(
  'every power-cycle step is a step the recipe knows',
  POWER_CYCLE_STEPS.every((step: string,): boolean => RECIPE_TEXT.includes(`'${step}':`,)),
);

for (const step of POWER_CYCLE_STEPS) {
  /** What the refusal said, so the claim rests on the words a person reads. */
  let said = '';
  try {
    refuseAStepThatNeedsAPowerCycle({
      backend: GUEST_BACKENDS.ovh,
      step,
    },);
  }
  catch (caught) {
    said = String(caught,);
  }
  hold(
    `ovh refuses '${step}' by name before anything is created`,
    said.includes(`step '${step}'`,) && said.includes(`backend 'ovh'`,),
  );
  hold(
    `libvirt still runs '${step}'`,
    ((): boolean => {
      refuseAStepThatNeedsAPowerCycle({
        backend: GUEST_BACKENDS.libvirt,
        step,
      },);
      return true;
    })(),
  );
}

hold(
  'a step whose subject is not a power cycle runs on every backend',
  ((): boolean => {
    refuseAStepThatNeedsAPowerCycle({
      backend: GUEST_BACKENDS.ovh,
      step: 'zfs-attach-arm',
    },);
    return true;
  })(),
);

//endregion Steps refused by name

//region The recipe's own text

hold(
  'the recipe names no backend of its own',
  !RECIPE_TEXT.includes('\'libvirt\'',),
);
hold(
  'the recipe names no backend\'s shared folder of its own',
  !RECIPE_TEXT.includes('/mnt/shared',),
);
hold(
  'the recipe starts every mvm command with the resolved backend\'s own words',
  RECIPE_TEXT.includes('options.backend.mvmArgumentWords',),
);
hold(
  'the recipe refuses a power-cycle step through the module rather than itself',
  RECIPE_TEXT.includes('refuseAStepThatNeedsAPowerCycle(',),
);
hold(
  'the recipe makes the folder its pushes land in, through the module, before any step pushes',
  RECIPE_TEXT.includes('makesThePushedFolder',) && RECIPE_TEXT.includes('mvm-shared-folder.log',),
);
hold(
  'every guest variable the arms read carries the guest hostname, which is what they compare `gethostname()` with',
  !/[*_]GUEST=\${options\.name}/.test(RECIPE_TEXT,),
);
hold(
  'the recipe hands the arms that hostname through the backend',
  RECIPE_TEXT.includes('_GUEST=${options.backend.hostnameOf(options.name)}',),
);

//endregion The recipe's own text

console.log(`the guest backend holds ${String(held,)} claims, and no guest was needed for any of them`,);
