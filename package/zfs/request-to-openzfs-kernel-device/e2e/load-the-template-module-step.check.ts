// Holds the shell a template guest's prepare runs to load its module, without a guest.
//
// What:  runs the step's own shell in `sh` once per state a guest made from the step 23
//        template can be in, against a stand-in `/sys` tree and stand-in `modinfo` and
//        `modprobe`, and reads what it printed, what it ran and what it left loaded.
// Why stand-ins: the step asks the kernel to load a module, so only a guest shows it
//        working against a real kernel. What a host can hold is the decision it makes
//        from the version files it reads; the stand-ins stand for `/sys/module`, the
//        module files `modinfo` reads and `modprobe` itself, and no claim about a real
//        kernel rests on them. The guest runs the change was measured with are in
//        [`../doc/prepare-in-a-prepared-guest-2026-11-27.md`](../doc/prepare-in-a-prepared-guest-2026-11-27.md).
// Why the template has its own step: the template installs its module as a package, in
//        `extramodules/`, and builds none, so the built module's step would look for
//        files that are not there and insert with `insmod` where the package's own
//        `modprobe` is what loads it. This step is the smaller one that reads what the
//        template holds.
// Why behaviour and not the text alone: the fault is a state, not a sentence. A step
//        that hid `modprobe`'s failure, or that compared no version, would still carry
//        every sentence it must say.

import { spawnSync, } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync, rmSync, writeFileSync, } from 'node:fs';
import { join, } from 'node:path';

import {
  ALREADY_RUNNING_WORDS,
  loadTheTemplateModuleStep,
  LOADED_WORDS,
  NOTHING_LOADED_WORDS,
  REFUSAL_WORDS,
} from './load-the-template-module-step.ts';

//region The stand-ins

/** The kernel release the stand-in `modprobe` says it looked in, which is the template's own. */
const STAND_IN_KERNEL = '7.3.5-1-cachyos';

/** The release the template installs and this recipe pins. */
const TEMPLATE_RELEASE = '2.4.4-1';

/** The release another module states, which the step must refuse. */
const OTHER_RELEASE = '2.2.2';

/** The release a rebuilt template states, which the recipe's pin must refuse. */
const REBUILT_RELEASE = '2.4.5-1';

/** The release the stand-in `modprobe` reports for a module the scenario says comes up as another build. */
const WRONG_VERSION = '9.9.9';

/** Where the template's package puts the module file this step names. */
const MODULE_FILE = `/lib/modules/${STAND_IN_KERNEL}/extramodules/zfs.ko.zst`;

/**
 The stand-in `modinfo` and `modprobe`, as shell functions.

 A stand-in, and every part of it says so: a real `modinfo` reads the module file's own
 metadata from the tree `modules.dep` names, and a real `modprobe` asks the kernel to
 load it. Here `modinfo` answers a field from a file of that scenario's own and
 `modprobe` writes the version files a loaded module publishes, or fails, as the
 scenario's control files say.

 Every call is appended to `calls.log`, so a check can read which programs ran, in
 which order, and that a refused run loaded nothing.
 */
const STAND_INS: readonly string[] = [
  'modinfo() {',
  '  printf \'modinfo %s\\n\' "$*" >> "$CHECK_ROOT/calls.log"',
  '  field=$2',
  '  module_name=$3',
  '  if test -f "$CHECK_ROOT/modinfo/$module_name.$field.unreadable"; then',
  '    printf \'modinfo: ERROR: could not read %s\\n\' "$module_name" >&2',
  '    return 1',
  '  fi',
  '  if test -f "$CHECK_ROOT/modinfo/$module_name.$field"; then cat "$CHECK_ROOT/modinfo/$module_name.$field"; fi',
  '  return 0',
  '}',
  'modprobe() {',
  '  printf \'modprobe %s\\n\' "$*" >> "$CHECK_ROOT/calls.log"',
  '  action=load',
  '  if test -f "$CHECK_ROOT/modprobe"; then action=$(cat "$CHECK_ROOT/modprobe"); fi',
  '  case "$action" in',
  `    fail) printf 'modprobe: FATAL: Module zfs not found in directory /lib/modules/${STAND_IN_KERNEL}\\n' >&2; return 1 ;;`,
  '    fail-quietly) return 1 ;;',
  '    keep) return 0 ;;',
  '  esac',
  '  for loaded in spl zfs; do',
  '    published=$CHECK_ROOT/modinfo/$loaded.version',
  // A scenario may name the version a fresh load comes up as, which is how a module that
  // answers with another build than its file states is held here.
  '    if test -f "$CHECK_ROOT/loaded-versions/$loaded"; then published=$CHECK_ROOT/loaded-versions/$loaded; fi',
  '    if test -f "$published"; then',
  '      mkdir --parents "$CHECK_ROOT/sys/module/$loaded"',
  '      cat "$published" > "$CHECK_ROOT/sys/module/$loaded/version"',
  '    fi',
  '  done',
  '  return 0',
  '}',
];

//endregion The stand-ins

//region The fixture

/** What one run starts from: what the template's files state and what is loaded. */
interface Scenario {
  readonly name: string;
  /** The version each file the template installs states; an empty string states none. */
  readonly files?: { readonly spl?: string; readonly zfs?: string };
  /** The version each loaded module publishes; `null` publishes none. */
  readonly loaded?: { readonly spl?: string | null; readonly zfs?: string | null };
  /** The version a fresh `modprobe` load comes up as, where it differs from what the file states. */
  readonly loadedVersions?: { readonly spl?: string; readonly zfs?: string };
  /** What the stand-in `modprobe` does: `fail`, `fail-quietly`, `keep`, or nothing for a load. */
  readonly modprobe?: string;
  /** A module whose `modinfo` refuses to answer, which is a file this step cannot read a version from. */
  readonly unreadable?: 'spl' | 'zfs';
}

/** The fixture folder one run works in, which every run starts by emptying. */
const WORK = join(import.meta.dirname, '..', 'target', 'template-module-step-check');

/** Where one run's stand-in trees and control files live. */
interface Trees {
  readonly root: string;
  readonly sys: string;
}

/**
 Write one scenario's stand-in trees and control files.

 @param scenario - what the run starts from

 @returns the stand-in `/sys` tree the step is handed

 @example
 ```ts
 const trees = makeTrees({ name: 'a fresh guest', files: { spl: '2.4.4-1', zfs: '2.4.4-1' } });
 ```
 */
function makeTrees(scenario: Scenario,): Trees {
  rmSync(WORK, { recursive: true, force: true, },);
  const root = WORK;
  const sys = join(root, 'sys',);
  mkdirSync(join(sys, 'module',), { recursive: true, },);
  mkdirSync(join(root, 'modinfo',), { recursive: true, },);
  writeFileSync(join(root, 'calls.log',), '',);
  for (const name of ['spl', 'zfs',] as const) {
    const version = scenario.files?.[name];
    if (version !== undefined)
      writeFileSync(join(root, 'modinfo', `${name}.version`,), version,);
    const loaded = scenario.loaded?.[name];
    if (loaded !== undefined) {
      mkdirSync(join(sys, 'module', name,), { recursive: true, },);
      if (loaded !== null)
        writeFileSync(join(sys, 'module', name, 'version',), loaded,);
    }
  }
  if (scenario.files?.zfs !== undefined)
    writeFileSync(join(root, 'modinfo', 'zfs.filename',), MODULE_FILE,);
  const loadedVersions = scenario.loadedVersions;
  if (loadedVersions !== undefined) {
    mkdirSync(join(root, 'loaded-versions',), { recursive: true, },);
    for (const name of ['spl', 'zfs',] as const) {
      const version = loadedVersions[name];
      if (version !== undefined)
        writeFileSync(join(root, 'loaded-versions', name,), version,);
    }
  }
  if (scenario.unreadable !== undefined)
    writeFileSync(join(root, 'modinfo', `${scenario.unreadable}.version.unreadable`,), '',);
  if (scenario.modprobe !== undefined)
    writeFileSync(join(root, 'modprobe',), scenario.modprobe,);
  return { root, sys, };
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

 @param trees - the stand-in `/sys` tree the step is handed

 @returns what the step printed, what it ran and what it left loaded
 */
function runStep(trees: Trees,): Run {
  const script = [
    'set -eu',
    `CHECK_ROOT=${trees.root}`,
    ...STAND_INS,
    ...loadTheTemplateModuleStep({
      label: 'check',
      release: TEMPLATE_RELEASE,
      sysDirectory: trees.sys,
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

/**
 The three reads every run begins with, in the order the step makes them: the file this
 template installs, the version that file states, and the version its spl file states.
 The step reads these before it loads anything, so a file that is not there or carries
 another release is refused with nothing loaded.
 */
const READS: readonly string[] = [
  'modinfo --field filename zfs',
  'modinfo --field version zfs',
  'modinfo --field version spl',
];

/** The claims, one per state the step can meet, and the awkward halves of each. */
const EXPECTATIONS: readonly Expectation[] = [
  {
    scenario: {
      name: 'a fresh guest with nothing loaded gets the module the template installs',
      files: { spl: TEMPLATE_RELEASE, zfs: TEMPLATE_RELEASE, },
    },
    says: [NOTHING_LOADED_WORDS, LOADED_WORDS, `zfs ${TEMPLATE_RELEASE}`, `from ${MODULE_FILE}`,],
    status: 'zero',
    calls: [...READS, 'modprobe zfs',],
    loaded: [`zfs ${TEMPLATE_RELEASE}`, `spl ${TEMPLATE_RELEASE}`,],
  },
  {
    scenario: {
      name: 'a guest already running the template`s module is left alone and says so',
      files: { spl: TEMPLATE_RELEASE, zfs: TEMPLATE_RELEASE, },
      loaded: { spl: TEMPLATE_RELEASE, zfs: TEMPLATE_RELEASE, },
      modprobe: 'keep',
    },
    says: [ALREADY_RUNNING_WORDS, 'nothing to load', `zfs ${TEMPLATE_RELEASE}`,],
    status: 'zero',
    calls: [...READS, 'modprobe zfs',],
    loaded: [`zfs ${TEMPLATE_RELEASE}`, `spl ${TEMPLATE_RELEASE}`,],
  },
  {
    scenario: {
      name: 'a load that fails is refused with modprobe`s own words, not hidden',
      files: { spl: TEMPLATE_RELEASE, zfs: TEMPLATE_RELEASE, },
      modprobe: 'fail',
    },
    says: [
      REFUSAL_WORDS,
      'modprobe zfs could not load',
      'modprobe: FATAL: Module zfs not found',
      'loaded now: zfs none, spl none',
      `the file this template installs: ${MODULE_FILE} stating zfs ${TEMPLATE_RELEASE}`,
      `this recipe pins OpenZFS ${TEMPLATE_RELEASE}`,
    ],
    status: 'nonzero',
    calls: [...READS, 'modprobe zfs',],
    loaded: [],
  },
  {
    scenario: {
      name: 'a load that fails without a word is refused with words of the step`s own',
      files: { spl: TEMPLATE_RELEASE, zfs: TEMPLATE_RELEASE, },
      modprobe: 'fail-quietly',
    },
    says: [REFUSAL_WORDS, 'printed nothing and exited with a failure', 'loaded now: zfs none, spl none',],
    status: 'nonzero',
    calls: [...READS, 'modprobe zfs',],
    loaded: [],
  },
  {
    scenario: {
      name: 'another zfs module already loaded is refused, and both versions and the file are named',
      files: { spl: TEMPLATE_RELEASE, zfs: TEMPLATE_RELEASE, },
      loaded: { spl: OTHER_RELEASE, zfs: OTHER_RELEASE, },
      modprobe: 'keep',
    },
    says: [
      REFUSAL_WORDS,
      'runs a module this template did not install',
      `loaded: zfs ${OTHER_RELEASE}, spl ${OTHER_RELEASE}`,
      `stating zfs ${TEMPLATE_RELEASE}`,
      `this recipe pins OpenZFS ${TEMPLATE_RELEASE}`,
      'reboot the guest',
      'modprobe --remove zfs',
    ],
    status: 'nonzero',
    calls: [...READS, 'modprobe zfs',],
    loaded: [`zfs ${OTHER_RELEASE}`, `spl ${OTHER_RELEASE}`,],
  },
  {
    scenario: {
      name: 'a loaded spl that publishes another version than its file is refused',
      files: { spl: TEMPLATE_RELEASE, zfs: TEMPLATE_RELEASE, },
      loaded: { spl: OTHER_RELEASE, zfs: TEMPLATE_RELEASE, },
      modprobe: 'keep',
    },
    says: [REFUSAL_WORDS, `loaded: zfs ${TEMPLATE_RELEASE}, spl ${OTHER_RELEASE}`, 'runs a module this template did not install',],
    status: 'nonzero',
    calls: [...READS, 'modprobe zfs',],
    loaded: [`zfs ${TEMPLATE_RELEASE}`, `spl ${OTHER_RELEASE}`,],
  },
  {
    scenario: {
      name: 'a module that comes up as another version than its file states is refused',
      files: { spl: TEMPLATE_RELEASE, zfs: TEMPLATE_RELEASE, },
      loadedVersions: { zfs: WRONG_VERSION, },
    },
    says: [
      REFUSAL_WORDS,
      `loaded: zfs ${WRONG_VERSION}, spl ${TEMPLATE_RELEASE}`,
      `stating zfs ${TEMPLATE_RELEASE}`,
      'runs a module this template did not install',
    ],
    status: 'nonzero',
    calls: [...READS, 'modprobe zfs',],
    loaded: [`zfs ${WRONG_VERSION}`, `spl ${TEMPLATE_RELEASE}`,],
  },
  {
    scenario: {
      name: 'a spl file that states no version is compared as a build that records none',
      files: { spl: '', zfs: TEMPLATE_RELEASE, },
      loaded: { spl: null, zfs: TEMPLATE_RELEASE, },
      modprobe: 'keep',
    },
    says: [ALREADY_RUNNING_WORDS, 'nothing to load',],
    status: 'zero',
    calls: [...READS, 'modprobe zfs',],
    loaded: [`zfs ${TEMPLATE_RELEASE}`, 'spl unknown',],
  },
  {
    scenario: {
      name: 'a zfs file that states another release than this recipe pins is refused before anything is loaded',
      files: { spl: REBUILT_RELEASE, zfs: REBUILT_RELEASE, },
    },
    says: [
      REFUSAL_WORDS,
      `states zfs ${REBUILT_RELEASE} where this recipe pins`,
      `this recipe pins OpenZFS ${TEMPLATE_RELEASE}`,
      `the file: ${MODULE_FILE}`,
      'nothing was loaded here',
    ],
    status: 'nonzero',
    calls: [...READS,],
    loaded: [],
  },
  {
    scenario: {
      name: 'an spl file that states another release than this recipe pins is refused before anything is loaded',
      files: { spl: REBUILT_RELEASE, zfs: TEMPLATE_RELEASE, },
    },
    says: [
      REFUSAL_WORDS,
      `states spl ${REBUILT_RELEASE} where this recipe pins`,
      `this recipe pins OpenZFS ${TEMPLATE_RELEASE}`,
      'nothing was loaded here',
    ],
    status: 'nonzero',
    calls: [...READS,],
    loaded: [],
  },
  {
    scenario: {
      name: 'a modinfo that refuses to answer is refused in words, and nothing is loaded',
      files: { spl: TEMPLATE_RELEASE, zfs: TEMPLATE_RELEASE, },
      unreadable: 'zfs',
    },
    says: [
      REFUSAL_WORDS,
      'nothing here says which module this template installs',
      'modinfo: ERROR: could not read zfs',
      'nothing was loaded here',
    ],
    status: 'nonzero',
    calls: ['modinfo --field filename zfs', 'modinfo --field version zfs',],
    loaded: [],
  },
  {
    scenario: {
      name: 'a modprobe that exits zero and loads nothing is refused, not read as loaded',
      files: { spl: TEMPLATE_RELEASE, zfs: TEMPLATE_RELEASE, },
      modprobe: 'keep',
    },
    says: [REFUSAL_WORDS, 'no zfs module is loaded', `the file this template installs: ${MODULE_FILE}`,],
    status: 'nonzero',
    calls: [...READS, 'modprobe zfs',],
    loaded: [],
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
    throw new Error(`the template module step no longer holds: ${claim}`,);
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

// The recipe's own text: the template's prepare calls this step, and no line of it loads
// a module of its own, because a second copy of the old lines would be a second place
// with the old fault.
const recipe = readFileSync(join(import.meta.dirname, 'run-fixture.ts',), 'utf8',);
const templatePrepare = recipe.split('function prepareFromTemplate(',)[1]?.split('function prepare(',)[0]
  ?? '';
hold(
  'the recipe imports the template step rather than repeating its shell',
  recipe.includes(`from './load-the-template-module-step.ts'`,),
);
hold(
  'the template`s prepare calls the step',
  templatePrepare.includes('loadTheTemplateModuleStep(',),
);
hold(
  'no line of the template`s prepare loads a module itself',
  !templatePrepare.includes('modprobe'),
);
hold(
  'the load whose failure was hidden is gone from the recipe',
  !recipe.includes('modprobe --quiet zfs'),
);

rmSync(WORK, { recursive: true, force: true, },);
console.log(`the template module step holds ${held} claims, and no guest was needed for any of them`,);

//endregion The check
