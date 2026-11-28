// Holds the shell that leaves a guest running the built module, without a guest.
//
// What:  runs the step's own shell in `sh` once per state a guest can be in, against
//        stand-in trees and stand-in kmod programs, and reads what it printed, what it
//        ran and what it left loaded.
// Why stand-ins: the step asks the kernel to load a module, so only a guest can show
//        it working against a real kernel. What a host can hold is the decision it
//        makes from the two versions it reads; the stand-ins stand for `/sys`, `/proc`,
//        the module files kmod reads and kmod's three programs, and no claim about a
//        real kernel rests on them. The runs this change was measured with are in
//        [`../doc/prepare-in-a-prepared-guest-2026-11-27.md`](../doc/prepare-in-a-prepared-guest-2026-11-27.md).
// Why behaviour and not the text alone: the fault this step was written against is a
//        state, not a sentence. A step that compared only one module, or that inserted
//        over a removal that failed, would still carry every sentence it must say.

import { spawnSync, } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync, rmSync, writeFileSync, } from 'node:fs';
import { join, } from 'node:path';

import {
  ALREADY_RUNNING_WORDS,
  loadTheBuiltModuleStep,
  NOTHING_LOADED_WORDS,
  NOW_RUNNING_WORDS,
  OTHER_MODULE_WORDS,
  REFUSAL_WORDS,
} from './load-the-built-module-step.ts';

//region The stand-ins

/** The kernel release the stand-in `uname` answers with, which is the release the plan pins. */
const STAND_IN_KERNEL = '6.8.0-147-generic';

/** The version the stand-in module files state, which is the release the design pins. */
const BUILT_VERSION = '2.4.4-1';

/** The version a module of the distribution states, which the wrapper's gate refuses. */
const OTHER_VERSION = '2.2.2';

/** The version the stand-in `insmod` reports for a module the scenario says comes up as another build. */
const WRONG_VERSION = '9.9.9';

/** The reference count the stand-in kernel's `refcnt` file holds where a scenario sets one. */
const HELD_REFERENCES = '10';

/**
 The kmod programs and `uname`, as shell functions.

 A stand-in, and every part of it says so: a real `modinfo` reads the module's own
 metadata, a real `modprobe --remove` asks the kernel, and a real `insmod` puts the
 module in. Here `modinfo` prints the version file beside the module file, `modprobe
 --remove` fails when the scenario's control file says the module is held, and
 `insmod` writes the version file where a loaded module's version lives.

 Every call is appended to `calls.log`, so a check can read which programs ran, in
 which order, and that a refused run inserted nothing.
 */
const STAND_INS: readonly string[] = [
  'uname() { printf \'%s\\n\' "$STAND_IN_KERNEL"; }',
  'modinfo() {',
  '  module_file=$3',
  '  printf \'modinfo %s\\n\' "$*" >> "$CHECK_ROOT/calls.log"',
  '  if test -f "$CHECK_ROOT/built/$(basename "$module_file").unreadable"; then',
  '    printf \'modinfo: ERROR: could not read %s\\n\' "$module_file" >&2',
  '    return 2',
  '  fi',
  '  if test ! -f "$module_file"; then',
  '    printf \'modinfo: ERROR: Module %s: No such file or directory\\n\' "$module_file" >&2',
  '    return 1',
  '  fi',
  '  if test -f "$CHECK_ROOT/built/$(basename "$module_file")"; then cat "$CHECK_ROOT/built/$(basename "$module_file")"; fi',
  '  return 0',
  '}',
  'modprobe() {',
  '  module_name=$2',
  '  printf \'modprobe %s\\n\' "$*" >> "$CHECK_ROOT/calls.log"',
  '  action=remove',
  '  if test -f "$CHECK_ROOT/remove-$module_name"; then action=$(cat "$CHECK_ROOT/remove-$module_name"); fi',
  '  case "$action" in',
  '    refuse) printf \'modprobe: FATAL: Module %s is in use.\\n\' "$module_name" >&2; return 1 ;;',
  '    refuse-quietly) return 1 ;;',
  '    keep) return 0 ;;',
  '  esac',
  '  rm --recursive --force "$CHECK_ROOT/sys/module/$module_name"',
  '  return 0',
  '}',
  'insmod() {',
  '  module_file=$1',
  '  module_name=$(basename "$module_file" .ko)',
  '  printf \'insmod %s\\n\' "$*" >> "$CHECK_ROOT/calls.log"',
  '  action=insert',
  '  if test -f "$CHECK_ROOT/insert-$module_name"; then action=$(cat "$CHECK_ROOT/insert-$module_name"); fi',
  '  if test "$action" = fail; then',
  '    printf \'insmod: ERROR: could not insert module %s: Invalid module format\\n\' "$module_file" >&2',
  '    return 1',
  '  fi',
  '  version=$(cat "$CHECK_ROOT/built/$(basename "$module_file")")',
  `  if test "$action" = other-version; then version=${WRONG_VERSION}; fi`,
  '  mkdir --parents "$CHECK_ROOT/sys/module/$module_name"',
  // A build that records no version publishes no version file either, which is the
  // shape a real module without one has.
  '  if test -n "$version"; then printf \'%s\\n\' "$version" > "$CHECK_ROOT/sys/module/$module_name/version"; fi',
  '  return 0',
  '}',
];

//endregion The stand-ins

//region The fixture

/**
 What one run of the step starts from: the version each built module file states, what
 is loaded, and how the stand-in programs behave.
 */
interface Scenario {
  readonly name: string;
  /**
   The version each built module file states. `undefined` writes no module file at all,
   which is a tree that was never built, and an empty string writes a file whose
   `modinfo` answers nothing, which is a build that records no version.
   */
  readonly built?: { readonly spl?: string; readonly zfs?: string };
  /** What is loaded before the step runs, and the version each loaded module publishes; `null` publishes none. */
  readonly loaded?: { readonly spl?: string | null; readonly zfs?: string | null };
  /** What `modprobe --remove` does for each module: `refuse`, `refuse-quietly`, `keep`, or nothing for a removal. */
  readonly removals?: { readonly spl?: string; readonly zfs?: string };
  /** What `insmod` does for each module: `fail`, `other-version`, or nothing for an insert at the built version. */
  readonly inserts?: { readonly spl?: string; readonly zfs?: string };
  /** The reference count each loaded module publishes, which the refusal names. */
  readonly refcounts?: { readonly spl?: string; readonly zfs?: string };
  /** The pool directories the stand-in kernel's listing holds, which stand for imported pools. */
  readonly pools?: readonly string[];
  /** The datasets the stand-in kernel's mount table holds, as `pool/dataset`; the mountpoint is the stand-in's own. */
  readonly mounts?: readonly string[];
  /** A module whose `modinfo` refuses to answer, which is a file the step cannot read a version from. */
  readonly unreadable?: 'spl' | 'zfs';
}

/** The fixture folder one run works in, which every run starts by emptying. */
const WORK = join(import.meta.dirname, '..', 'target', 'module-step-check');

/** Where one run's stand-in trees and control files live. */
interface Trees {
  readonly root: string;
  readonly sys: string;
  readonly proc: string;
  readonly modules: string;
}

/**
 Write one scenario's stand-in trees and control files.

 @param scenario - what the run starts from

 @returns the three trees the step is handed

 @example
 ```ts
 const trees = makeTrees({ name: 'a fresh guest', built: { spl: '2.4.4-1', zfs: '2.4.4-1' } });
 ```
 */
function makeTrees(scenario: Scenario,): Trees {
  rmSync(WORK, { recursive: true, force: true, },);
  const root = WORK;
  const sys = join(root, 'sys',);
  const proc = join(root, 'proc',);
  const modules = join(root, 'modules',);
  const built = join(root, 'built',);
  mkdirSync(join(modules, STAND_IN_KERNEL, 'extra',), { recursive: true, },);
  mkdirSync(join(sys, 'module',), { recursive: true, },);
  mkdirSync(join(proc, 'spl', 'kstat', 'zfs',), { recursive: true, },);
  mkdirSync(join(proc, 'self',), { recursive: true, },);
  mkdirSync(built, { recursive: true, },);
  writeFileSync(join(root, 'calls.log',), '',);
  for (const name of ['spl', 'zfs',] as const) {
    const version = scenario.built?.[name];
    if (version !== undefined) {
      // The module file stands for the `.ko` this recipe installs. The stand-in reads
      // only its name and the version beside it, so an empty file is enough.
      writeFileSync(join(modules, STAND_IN_KERNEL, 'extra', `${name}.ko`,), '',);
      writeFileSync(join(built, `${name}.ko`,), version,);
    }
    const loaded = scenario.loaded?.[name];
    if (loaded !== undefined) {
      mkdirSync(join(sys, 'module', name,), { recursive: true, },);
      if (loaded !== null)
        writeFileSync(join(sys, 'module', name, 'version',), loaded,);
    }
    const references = scenario.refcounts?.[name];
    if (references !== undefined)
      writeFileSync(join(sys, 'module', name, 'refcnt',), references,);
    const removal = scenario.removals?.[name];
    if (removal !== undefined)
      writeFileSync(join(root, `remove-${name}`,), removal,);
    const insert = scenario.inserts?.[name];
    if (insert !== undefined)
      writeFileSync(join(root, `insert-${name}`,), insert,);
  }
  if (scenario.unreadable !== undefined)
    writeFileSync(join(built, `${scenario.unreadable}.ko.unreadable`,), '',);
  for (const pool of scenario.pools ?? [])
    mkdirSync(join(proc, 'spl', 'kstat', 'zfs', pool,), { recursive: true, },);
  // The mount table, in the kernel's own shape of /proc/self/mountinfo: a mountpoint
  // field, then ` - zfs <dataset> ` and the superblock's options. A stand-in, and only
  // the dataset name after ` - zfs ` is read.
  if ((scenario.mounts ?? []).length > 0) {
    const lines = (scenario.mounts ?? []).map(function mountLine(dataset: string, index: number,): string {
      return `53 ${String(index + 29,)} 0:46 / /home/zfs-step-2-fixture/${dataset.split('/')[1] ?? dataset} rw,relatime shared:214 - zfs ${dataset} rw,xattr,noacl,casesensitive`;
    },);
    writeFileSync(join(proc, 'self', 'mountinfo',), `${lines.join('\n',)}\n`, { encoding: 'utf8', flag: 'w', },);
  }
  return { root, sys, proc, modules, };
}

/** What one run of the step printed and left behind. */
interface Run {
  readonly status: number;
  readonly out: string;
  readonly err: string;
  readonly calls: readonly string[];
  /** The modules loaded once the step ended, as `name version` for each. */
  readonly loaded: readonly string[];
}

/**
 Read the version one stand-in module publishes.

 @param trees - the trees the step is handed

 @param name - the module's name, `zfs` or `spl`

 @returns the version, `unknown` for a module loaded without publishing one, or `null` when it is not loaded
 */
function readLoadedVersion(trees: Trees, name: string,): string | null {
  const moduleDirectory = join(trees.sys, 'module', name,);
  if (!existsSync(moduleDirectory))
    return null;
  const versionFile = join(moduleDirectory, 'version',);
  if (!existsSync(versionFile))
    return 'unknown';
  return readFileSync(versionFile, 'utf8',).trim();
}

/**
 Run the step once against one scenario's trees.

 @param trees - the three trees the step is handed

 @returns what the step printed, what it ran and what it left loaded
 */
function runStep(trees: Trees,): Run {
  const script = [
    'set -eu',
    `CHECK_ROOT=${trees.root}`,
    `STAND_IN_KERNEL=${STAND_IN_KERNEL}`,
    ...STAND_INS,
    ...loadTheBuiltModuleStep({
      label: 'check',
      sysDirectory: trees.sys,
      procDirectory: trees.proc,
      modulesDirectory: trees.modules,
    }),
  ].join('\n',);
  const result = spawnSync('sh', ['-ec', script,], {
    encoding: 'utf8',
    env: { PATH: process.env.PATH ?? '', },
  },);
  if (result.status === null)
    throw new Error(`the step could not be run at all: ${result.error?.message ?? 'no status'}`,);
  const calls = readFileSync(join(trees.root, 'calls.log',), 'utf8',)
    .split('\n',)
    .filter(function hasText(line: string,): boolean {
      return line.length > 0;
    },);
  const loaded: string[] = [];
  for (const name of ['zfs', 'spl',]) {
    const version = readLoadedVersion(trees, name,);
    if (version !== null)
      loaded.push(`${name} ${version}`,);
  }
  return { status: result.status, out: result.stdout, err: result.stderr, calls, loaded, };
}

//endregion The fixture

//region The claims

/** Every claim one scenario makes, said in the words a reader needs when it fails. */
interface Expectation {
  readonly scenario: Scenario;
  readonly says: readonly string[];
  readonly status: 'zero' | 'nonzero';
  readonly calls: readonly string[];
  readonly loaded: readonly string[];
}

/** The module file the stand-in `insmod` is handed, which every call expectation spells out. */
const BUILT_SPL = join(WORK, 'modules', STAND_IN_KERNEL, 'extra', 'spl.ko',);

/** The zfs module file, which every call expectation spells out. */
const BUILT_ZFS = join(WORK, 'modules', STAND_IN_KERNEL, 'extra', 'zfs.ko',);

/**
 The two reads every run that has both module files begins with, in the order the step
 makes them: this recipe's own zfs file first, which is the version the whole decision
 rests on, then spl's.
 */
const READS: readonly string[] = [`modinfo --field version ${BUILT_ZFS}`, `modinfo --field version ${BUILT_SPL}`,];

/** The claims, one per state the step can meet, and the awkward halves of each. */
const EXPECTATIONS: readonly Expectation[] = [
  {
    scenario: {
      name: 'a fresh guest gets the built modules',
      built: { spl: BUILT_VERSION, zfs: BUILT_VERSION, },
    },
    says: ['check:', NOTHING_LOADED_WORDS, NOW_RUNNING_WORDS, `zfs ${BUILT_VERSION}`,],
    status: 'zero',
    calls: [...READS, `insmod ${BUILT_SPL}`, `insmod ${BUILT_ZFS}`,],
    loaded: [`zfs ${BUILT_VERSION}`, `spl ${BUILT_VERSION}`,],
  },
  {
    scenario: {
      name: 'a guest already running the built modules keeps them, and its pools',
      built: { spl: BUILT_VERSION, zfs: BUILT_VERSION, },
      loaded: { spl: BUILT_VERSION, zfs: BUILT_VERSION, },
      pools: ['zfsstep2-account',],
    },
    says: [ALREADY_RUNNING_WORDS, 'nothing was removed and nothing was inserted', `stating ${BUILT_VERSION}`,],
    status: 'zero',
    calls: [...READS,],
    loaded: [`zfs ${BUILT_VERSION}`, `spl ${BUILT_VERSION}`,],
  },
  {
    scenario: {
      name: 'a guest running the distribution module gets the built one, zfs first out and zfs last in',
      built: { spl: BUILT_VERSION, zfs: BUILT_VERSION, },
      loaded: { spl: OTHER_VERSION, zfs: OTHER_VERSION, },
    },
    says: [OTHER_MODULE_WORDS, `removed zfs ${OTHER_VERSION}`, `removed spl ${OTHER_VERSION}`, NOW_RUNNING_WORDS,],
    status: 'zero',
    calls: [...READS, `modprobe --remove zfs`, `modprobe --remove spl`, `insmod ${BUILT_SPL}`, `insmod ${BUILT_ZFS}`,],
    loaded: [`zfs ${BUILT_VERSION}`, `spl ${BUILT_VERSION}`,],
  },
  {
    scenario: {
      name: 'a module an imported pool holds is refused, and the pools and both versions are named',
      built: { spl: BUILT_VERSION, zfs: BUILT_VERSION, },
      loaded: { spl: OTHER_VERSION, zfs: OTHER_VERSION, },
      removals: { zfs: 'refuse', },
      refcounts: { zfs: HELD_REFERENCES, },
      pools: ['zfsstep2-account', 'zfsstep4',],
      mounts: ['zfsstep2-account/checkout',],
    },
    says: [
      REFUSAL_WORDS,
      `loaded: zfs ${OTHER_VERSION}`,
      `built: zfs ${BUILT_VERSION}`,
      'Module zfs is in use',
      `the kernel counts ${HELD_REFERENCES} references to module zfs`,
      'the kernel lists these ZFS datasets mounted: zfsstep2-account/checkout',
      'the kernel lists these pools imported: zfsstep2-account zfsstep4',
      'a mounted dataset is what holds module zfs',
      'zfs unmount <dataset>',
      `left running: zfs ${OTHER_VERSION}, spl ${OTHER_VERSION}`,
    ],
    status: 'nonzero',
    calls: [...READS, `modprobe --remove zfs`,],
    loaded: [`zfs ${OTHER_VERSION}`, `spl ${OTHER_VERSION}`,],
  },
  {
    scenario: {
      name: 'a held module with no dataset mounted is refused, and the reading says what the counts were',
      built: { spl: BUILT_VERSION, zfs: BUILT_VERSION, },
      loaded: { spl: OTHER_VERSION, zfs: OTHER_VERSION, },
      removals: { zfs: 'refuse', },
      pools: ['zfsstep3',],
    },
    says: [
      REFUSAL_WORDS,
      'the kernel lists these ZFS datasets mounted:\n',
      'the kernel lists these pools imported: zfsstep3',
      'a mounted dataset is what holds module zfs',
      'at 0 with a pool imported and nothing mounted',
    ],
    status: 'nonzero',
    calls: [...READS, `modprobe --remove zfs`,],
    loaded: [`zfs ${OTHER_VERSION}`, `spl ${OTHER_VERSION}`,],
  },
  {
    scenario: {
      name: 'a removal that exits zero and leaves the module loaded is refused, not passed over',
      built: { spl: BUILT_VERSION, zfs: BUILT_VERSION, },
      loaded: { spl: OTHER_VERSION, zfs: OTHER_VERSION, },
      removals: { zfs: 'keep', },
    },
    says: [REFUSAL_WORDS, 'printed nothing, and module zfs is still loaded',],
    status: 'nonzero',
    calls: [...READS, `modprobe --remove zfs`,],
    loaded: [`zfs ${OTHER_VERSION}`, `spl ${OTHER_VERSION}`,],
  },
  {
    scenario: {
      name: 'a removal that fails without a word is refused with words of the step`s own',
      built: { spl: BUILT_VERSION, zfs: BUILT_VERSION, },
      loaded: { spl: OTHER_VERSION, zfs: OTHER_VERSION, },
      removals: { zfs: 'refuse-quietly', },
    },
    says: [REFUSAL_WORDS, 'printed nothing, and module zfs is still loaded',],
    status: 'nonzero',
    calls: [...READS, `modprobe --remove zfs`,],
    loaded: [`zfs ${OTHER_VERSION}`, `spl ${OTHER_VERSION}`,],
  },
  {
    scenario: {
      name: 'a spl module that publishes no version is still the built one when zfs matches',
      built: { spl: '', zfs: BUILT_VERSION, },
      loaded: { spl: null, zfs: BUILT_VERSION, },
    },
    says: [ALREADY_RUNNING_WORDS, 'nothing was removed and nothing was inserted',],
    status: 'zero',
    calls: [...READS,],
    loaded: [`zfs ${BUILT_VERSION}`, 'spl unknown',],
  },
  {
    scenario: {
      name: 'a versionless built spl still refuses a loaded spl that publishes another version',
      built: { spl: '', zfs: BUILT_VERSION, },
      loaded: { spl: OTHER_VERSION, zfs: BUILT_VERSION, },
    },
    says: [OTHER_MODULE_WORDS, `removed spl ${OTHER_VERSION}`, NOW_RUNNING_WORDS,],
    status: 'zero',
    calls: [...READS, `modprobe --remove zfs`, `modprobe --remove spl`, `insmod ${BUILT_SPL}`, `insmod ${BUILT_ZFS}`,],
    loaded: [`zfs ${BUILT_VERSION}`, 'spl unknown',],
  },
  {
    scenario: {
      name: 'a guest holding spl alone gets both built modules',
      built: { spl: BUILT_VERSION, zfs: BUILT_VERSION, },
      loaded: { spl: OTHER_VERSION, },
    },
    says: [OTHER_MODULE_WORDS, `removed spl ${OTHER_VERSION}`, NOW_RUNNING_WORDS,],
    status: 'zero',
    calls: [...READS, `modprobe --remove spl`, `insmod ${BUILT_SPL}`, `insmod ${BUILT_ZFS}`,],
    loaded: [`zfs ${BUILT_VERSION}`, `spl ${BUILT_VERSION}`,],
  },
  {
    scenario: {
      name: 'a guest holding zfs alone gets spl and then zfs, because a module rests on what it links',
      built: { spl: BUILT_VERSION, zfs: BUILT_VERSION, },
      loaded: { zfs: OTHER_VERSION, },
    },
    says: [OTHER_MODULE_WORDS, `removed zfs ${OTHER_VERSION}`, NOW_RUNNING_WORDS,],
    status: 'zero',
    calls: [...READS, `modprobe --remove zfs`, `insmod ${BUILT_SPL}`, `insmod ${BUILT_ZFS}`,],
    loaded: [`zfs ${BUILT_VERSION}`, `spl ${BUILT_VERSION}`,],
  },
  {
    scenario: {
      name: 'a built module file that is not there is refused, and nothing is inserted',
      built: { spl: BUILT_VERSION, },
    },
    says: [REFUSAL_WORDS, 'is not there', 'make install',],
    status: 'nonzero',
    calls: [],
    loaded: [],
  },
  {
    scenario: {
      name: 'a built zfs module that states no version is refused, because nothing could be compared',
      built: { spl: BUILT_VERSION, zfs: '', },
    },
    says: [REFUSAL_WORDS, 'states no version',],
    status: 'nonzero',
    calls: [`modinfo --field version ${BUILT_ZFS}`,],
    loaded: [],
  },
  {
    scenario: {
      name: 'a modinfo that refuses to answer is refused in words rather than by a bare status',
      built: { spl: BUILT_VERSION, zfs: BUILT_VERSION, },
      unreadable: 'zfs',
    },
    says: [REFUSAL_WORDS, 'modinfo could not read a version', 'could not read'],
    status: 'nonzero',
    calls: [`modinfo --field version ${BUILT_ZFS}`,],
    loaded: [],
  },
  {
    scenario: {
      name: 'an insert that fails names the file and insmod`s own words, and says what is loaded',
      built: { spl: BUILT_VERSION, zfs: BUILT_VERSION, },
      inserts: { zfs: 'fail', },
    },
    says: [REFUSAL_WORDS, 'Invalid module format', `left running: zfs none, spl ${BUILT_VERSION}`,],
    status: 'nonzero',
    calls: [...READS, `insmod ${BUILT_SPL}`, `insmod ${BUILT_ZFS}`,],
    loaded: [`spl ${BUILT_VERSION}`,],
  },
  {
    scenario: {
      name: 'a module that comes up as another version than its file states is refused',
      built: { spl: BUILT_VERSION, zfs: BUILT_VERSION, },
      inserts: { zfs: 'other-version', },
    },
    says: [REFUSAL_WORDS, `reports ${WRONG_VERSION}`, `states ${BUILT_VERSION}`,],
    status: 'nonzero',
    calls: [...READS, `insmod ${BUILT_SPL}`, `insmod ${BUILT_ZFS}`,],
    loaded: [`zfs ${WRONG_VERSION}`, `spl ${BUILT_VERSION}`,],
  },
  {
    scenario: {
      name: 'zfs gone and spl held is refused, and the guest is said to be left with spl alone',
      built: { spl: BUILT_VERSION, zfs: BUILT_VERSION, },
      loaded: { spl: OTHER_VERSION, zfs: OTHER_VERSION, },
      removals: { spl: 'refuse', },
      pools: ['zfsstep3',],
    },
    says: [
      REFUSAL_WORDS,
      `removed zfs ${OTHER_VERSION}`,
      'module spl would not go',
      `left running: zfs none, spl ${OTHER_VERSION}`,
      'the kernel lists these pools imported: zfsstep3',
    ],
    status: 'nonzero',
    calls: [...READS, `modprobe --remove zfs`, `modprobe --remove spl`,],
    loaded: [`spl ${OTHER_VERSION}`,],
  },
];

//endregion The claims

//region The check

/** What the check has held so far, printed one line per claim it held. */
let held = 0;

/**
 Hold one claim, or end the check naming it.

 @param claim - what is held, in the words a person reads when it fails

 @param holds - whether it holds
 */
function hold(claim: string, holds: boolean,): void {
  if (!holds)
    throw new Error(`the module step no longer holds: ${claim}`,);
  held += 1;
  console.log(`held: ${claim}`,);
}

for (const expectation of EXPECTATIONS) {
  const trees = makeTrees(expectation.scenario,);
  const run = runStep(trees,);
  const said = `${run.out}${run.err}`;
  if (expectation.status === 'zero')
    hold(`${expectation.scenario.name}: ends 0`, run.status === 0,);
  else
    hold(`${expectation.scenario.name}: refuses`, run.status !== 0,);
  for (const words of expectation.says)
    hold(`${expectation.scenario.name}: says ${words}`, said.includes(words,),);
  hold(
    `${expectation.scenario.name}: runs ${expectation.calls.join(', ') || 'nothing'}`,
    JSON.stringify(run.calls,) === JSON.stringify(expectation.calls,),
  );
  hold(
    `${expectation.scenario.name}: leaves ${expectation.loaded.join(', ') || 'nothing'} loaded`,
    JSON.stringify([...run.loaded,].sort(),) === JSON.stringify([...expectation.loaded,].sort(),),
  );
}

// The recipe's own text, which must not remove or insert a module anywhere else: a
// second copy of the old lines would be a second place with the old fault.
const recipe = readFileSync(join(import.meta.dirname, 'run-fixture.ts',), 'utf8',);
hold(
  'the recipe imports the step rather than repeating its shell',
  recipe.includes(`from './load-the-built-module-step.ts'`,),
);
hold(
  'no line of the recipe asks modprobe to remove a module itself',
  !recipe.includes('modprobe --remove',),
);
hold(
  'no line of the recipe asks insmod to insert a module itself',
  !recipe.includes('insmod',),
);
hold(
  'the recipe calls the step at each of the four places that load a module',
  recipe.split('loadTheBuiltModuleStep(',).length - 1 >= 4,
);

rmSync(WORK, { recursive: true, force: true, },);
console.log(`the module step holds ${held} claims, and no guest was needed for any of them`,);

//endregion The check
