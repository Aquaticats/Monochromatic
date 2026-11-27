#!/usr/bin/env node
/**
 Drive plan step 20's Sanoid arms in a throwaway `mvm` guest.

 The arms read a real dataset tree: recursive coverage, the exclusion sections and their order, the
 `recursive = zfs` control, the markers, retention pruning and the coverage comparison. They run Sanoid
 2.3.0 beside the Ubuntu package's own 2.2.0 as a control, because the project's measurements of the
 section-order effect were made on 2.3.0 and the earlier guest had 2.2.0.

 This driver owns the guest through the ZFS binding's own recipe
 (`package/zfs/request-to-openzfs-kernel-device/e2e/run-fixture.ts`): it creates the Ubuntu 24.04 guest,
 prepares it with OpenZFS 2.4.4, Rust and cargo-nextest, installs both Sanoid releases, builds this
 crate's binaries and tests inside it, runs the arm, and destroys the guest. The arm itself creates the
 file-backed pool, the decided layout, the markers and the configurations, and asserts every claim.

 Every line a run must find is printed by the arm as `SANOID-GUEST <claim>`. `create` refuses a name
 `mvm list` already reports, and `destroy` removes only a guest this driver created.

 Usage: node e2e/sanoid-coverage-guest.ts <create|prepare|build|arm|destroy|all> [--name <guest>] [--log-dir <dir>]
          [--keep-guest]
 */
import {
  mkdir,
  rm,
} from 'node:fs/promises';
import {
  join,
  resolve,
} from 'node:path';

import { runGateCommand, } from '../bin/native-gate-run.ts';
import { copyWrapperSnapshot, } from '../bin/native-wrapper-snapshot.ts';
import {
  holdsWordLine,
  missingMarkerRefusal,
  missingMarkers,
} from './guest-driver-arm-log.ts';
import {
  flagGiven,
  flagValue,
} from './guest-driver-command-line.ts';
import {
  GuestDriverError,
  refusalOf,
} from './guest-driver-error.ts';
import { holdsGuestName, } from './guest-driver-guest-listing.ts';
import { runGuestScript, } from './guest-driver-guest-script.ts';
import { runGuestRecipeStep, } from './guest-driver-recipe-step.ts';
import {
  readStepLog,
  runRequiredStepCommand,
} from './guest-driver-step-command.ts';

//region Constants

/**
 Where the crate is unpacked and built inside the guest.
 */
const CRATE_DIR = '/opt/git-policy-cli-sanoid-coverage';

/**
 The file-backed pool the arm's layout sits on, which only the guest ever sees.
 */
const POOL = 'sanoidstep20';

/**
 The variable the arm reads: its value is the guest's name, so a run outside the guest prints SKIP.
 */
const GUEST_VARIABLE = 'CLI_GIT_ZFS_SANOID_GUEST';

/**
 The variable the arm reads for the pool's name.
 */
const POOL_VARIABLE = 'CLI_GIT_ZFS_SANOID_POOL';

/**
 The release of Sanoid the arms measure.
 */
const SANOID_RELEASE = '2.3.0';

/**
 Where the Sanoid 2.3.0 release is unpacked inside the guest.
 */
const SANOID_SOURCE = `/opt/sanoid-${SANOID_RELEASE}`;

/**
 The GitHub source archive of that release, which the guest downloads and checks.
 */
const SANOID_URL = `https://github.com/jimsalterjrs/sanoid/archive/refs/tags/v${SANOID_RELEASE}.tar.gz`;

/**
 sha256 of that archive, measured on 2026-11-25 from the tag `v2.3.0`
 (commit 8d4abf14b2bf8d6b98fc8e163fa9895486e8cdeb).
 A published archive that GitHub regenerates fails this check rather than measuring other bytes.
 */
const SANOID_SHA256 = '1d8735a271a34ec87ea46313a66f6f20bd38b583886924574d3c1f72ea173620';

/**
 The Sanoid release the Ubuntu package carries, kept beside the measured one as a control.
 */
const CONTROL_RELEASE = '2.2';

/**
 Milliseconds one host command may take.
 */
const HOST_COMMAND_TIMEOUT_MS = 120_000;

/**
 Milliseconds the guest's Sanoid install may take.
 */
const SANOID_INSTALL_TIMEOUT_MS = 600_000;

/**
 Milliseconds the guest's crate build may take.
 */
const CRATE_BUILD_TIMEOUT_MS = 2_400_000;

/**
 Milliseconds one arm may take.
 */
const ARM_TIMEOUT_MS = 1_200_000;

/**
 Seconds the guest's own `timeout` fires before `mvm`'s kill, so the guest reports a status rather than dying with it.
 */
const GUEST_TIMEOUT_MARGIN_SECONDS = 60;

/**
 The guest name a run takes when the command line names none.
 */
const DEFAULT_GUEST_NAME = 'git-policy-sanoid-coverage';

/**
 The log directory a run takes when the command line names none: this package's build folder.
 */
const DEFAULT_LOG_DIRECTORY_NAME = 'sanoid-coverage-guest-logs';

/**
 The word a test binary prints at the start of a line when it returned without running its arm.
 */
const SKIP_WORD = 'SKIP';

/**
 The lines a green arm must print, one per claim it makes.
 */
const GREEN_MARKERS: readonly string[] = [
  'SANOID-GUEST layout-built',
  'SANOID-GUEST coverage-passes',
  'SANOID-GUEST 2-3-0-covers-home-only',
  'SANOID-GUEST 2-3-0-order-captures-the-exclusion',
  'SANOID-GUEST 2-3-0-zfs-recursion-captures-the-exclusion',
  'SANOID-GUEST 2-2-0-covers-home-only',
  'SANOID-GUEST 2-2-0-order-captures-the-exclusion',
  'SANOID-GUEST prune-keeps-the-pinned-origin',
  'SANOID-GUEST marker-finding-and-correction-apply',
  'SANOID-GUEST order-finding-and-correction-apply',
  'SANOID-GUEST recursion-finding-and-correction-apply',
  'SANOID-GUEST store-finding-and-nothing-reaches-it',
  'SANOID-GUEST guard-scope-names-what-it-cannot-prune',
];

//endregion Constants

//region Arguments

/**
 The step, the guest name, the log directory and whether a run keeps the guest it made.
 */
type Options = {
  readonly step: string;
  readonly name: string;
  readonly logDirectory: string;
  readonly keepGuest: boolean;
};

/**
 The log directory a run keeps its logs in, which this package's build folder holds unless the command line names one.

 @returns the directory this package's build folder holds for this driver's logs

 @example
 ```ts
 defaultLogDirectory(); // <package>/target/sanoid-coverage-guest-logs
 ```
 */
function defaultLogDirectory(): string {
  return resolve(
    import.meta.dirname,
    '..',
    'target',
    DEFAULT_LOG_DIRECTORY_NAME,
  );
}

/**
 The step and the values the command line gave it.

 @returns the step, the guest's name, the log directory and whether the guest is kept

 @example
 ```ts
 const chosen = options();
 ```
 */
function options(): Options {
  /**
   The words this run was started with, after the program and this file's own path.
   */
  const argumentLine = process.argv
    .slice(2,);
  return {
    step: argumentLine[0] ?? '',
    name: flagValue({
      argumentWords: argumentLine,
      flag: '--name',
      fallback: DEFAULT_GUEST_NAME,
    },),
    logDirectory: flagValue({
      argumentWords: argumentLine,
      flag: '--log-dir',
      fallback: defaultLogDirectory(),
    },),
    keepGuest: flagGiven({
      argumentWords: argumentLine,
      flag: '--keep-guest',
    },),
  };
}

//endregion Arguments

//region Steps

/**
 Refuse a name another guest already holds.

 @param chosen - the run's options

 @throws GuestDriverError when the listing names this guest, or the listing could not be read

 @example
 ```ts
 await create({ chosen });
 ```
 */
async function create({ chosen, }: { readonly chosen: Options; },): Promise<void> {
  /**
   What `mvm list` answered, which names every guest this account holds.
   */
  const listed = await runGateCommand({
    program: 'mvm',
    args: ['list',],
    deadlineMilliseconds: HOST_COMMAND_TIMEOUT_MS,
  },);
  if (holdsGuestName({
    listing: listed.stdout,
    name: chosen.name,
  },)) {
    throw new GuestDriverError(`a guest named ${chosen.name} is already listed; name another with --name`,);
  }
  await runGuestRecipeStep({
    name: chosen.name,
    logDirectory: chosen.logDirectory,
    step: 'create',
  },);
}

/**
 Install OpenZFS 2.4.4, Rust and cargo-nextest, then both Sanoid releases.

 @param chosen - the run's options

 @throws GuestDriverError when the recipe or the install refused

 @example
 ```ts
 await prepare({ chosen });
 ```
 */
async function prepare({ chosen, }: { readonly chosen: Options; },): Promise<void> {
  await runGuestRecipeStep({
    name: chosen.name,
    logDirectory: chosen.logDirectory,
    step: 'prepare',
  },);
  /**
   The script the guest runs: the Ubuntu package, then the measured release from its published archive.
   */
  const script = [
    'set -eu',
    'export DEBIAN_FRONTEND=noninteractive',
    'apt-get --option Acquire::Retries=2 --yes install sanoid > /root/sanoid-install.log 2>&1',
    'installed=$(command -v sanoid)',
    'installed_version=$("$installed" --version)',
    'echo "installed $installed_version at $installed" >> /root/sanoid-install.log',
    `case "$installed_version" in *${CONTROL_RELEASE}*) ;; *) echo "the Ubuntu package is not ${CONTROL_RELEASE}: $installed_version" >&2; exit 1;; esac`,
    `curl --fail --location --retry 3 --max-time 600 --output /root/sanoid-${SANOID_RELEASE}.tar.gz ${SANOID_URL} >> /root/sanoid-install.log 2>&1`,
    `printf '%s  %s\\n' ${SANOID_SHA256} /root/sanoid-${SANOID_RELEASE}.tar.gz | sha256sum --check --strict - >> /root/sanoid-install.log 2>&1`,
    `tar --extract --gzip --file /root/sanoid-${SANOID_RELEASE}.tar.gz --directory /opt`,
    `measured=$(perl ${SANOID_SOURCE}/sanoid --version)`,
    `case "$measured" in *${SANOID_RELEASE}*) ;; *) echo "the archive is not ${SANOID_RELEASE}: $measured" >&2; exit 1;; esac`,
    'echo "measured $measured from the archive" >> /root/sanoid-install.log',
    'tail --lines=6 /root/sanoid-install.log',
  ].join('\n',);
  await runGuestScript({
    logDirectory: chosen.logDirectory,
    name: chosen.name,
    script,
    log: 'guest-sanoid-install.log',
    timeoutMilliseconds: SANOID_INSTALL_TIMEOUT_MS,
    guestTimeoutMarginSeconds: GUEST_TIMEOUT_MARGIN_SECONDS,
  },);
}

/**
 Push a snapshot of this worktree and build this crate's binaries and tests inside the guest.

 @param chosen - the run's options

 @throws GuestDriverError when a step refused, or when the guest's build did

 @example
 ```ts
 await build({ chosen });
 ```
 */
async function build({ chosen, }: { readonly chosen: Options; },): Promise<void> {
  /**
   The build context the wrapper's tree is copied into before it is packed.
   */
  const context = join(
    chosen.logDirectory,
    'snapshot',
  );
  await rm(
    context,
    {
      recursive: true,
      force: true,
    },
  );
  await mkdir(
    context,
    {
      recursive: true,
      mode: 0o700,
    },
  );
  await copyWrapperSnapshot({ context, },);
  /**
   The packed tree this run hands the guest.
   */
  const tarball = join(
    chosen.logDirectory,
    'subject.tar.gz',
  );
  await runRequiredStepCommand({
    logDirectory: chosen.logDirectory,
    program: 'tar',
    argumentWords: [
      '--create',
      '--gzip',
      '--file',
      tarball,
      '--exclude=target',
      '--exclude=node_modules',
      '--directory',
      context,
      '.',
    ],
    log: 'host-package.log',
    timeoutMilliseconds: HOST_COMMAND_TIMEOUT_MS,
  },);
  await runRequiredStepCommand({
    logDirectory: chosen.logDirectory,
    program: 'mvm',
    argumentWords: [
      '--backend',
      'libvirt',
      'push',
      chosen.name,
      tarball,
      '/root/subject.tar.gz',
    ],
    log: 'guest-push.log',
    timeoutMilliseconds: HOST_COMMAND_TIMEOUT_MS,
  },);
  /**
   The script the guest runs to unpack and build the crate.
   */
  const script = [
    'set -eu',
    'if test -f /mnt/shared/subject.tar.gz; then cp /mnt/shared/subject.tar.gz /root/subject.tar.gz; fi',
    `rm --recursive --force ${CRATE_DIR}`,
    `mkdir --parents ${CRATE_DIR}`,
    `tar --extract --gzip --file /root/subject.tar.gz --directory ${CRATE_DIR}`,
    `cd ${CRATE_DIR}/package/git-policy/cli`,
    'export PATH=/root/.cargo/bin:$PATH',
    // Two jobs and a debug build keep rustc and the linker inside the guest's memory: the host backs the guest's
    // pages, and a release build at one compiler per vCPU was SIGKILLed by the guest's own killer while the host was
    // short. The arm runs the same test binary either way.
    'export CARGO_BUILD_JOBS=2',
    'if cargo build --locked --tests > /root/sanoid-coverage-build.log 2>&1; then status=0; else status=$?; fi',
    'grep --extended-regexp "^(error|warning)" --after-context=6 /root/sanoid-coverage-build.log | tail --lines=80 || true',
    'tail --lines=5 /root/sanoid-coverage-build.log',
    'exit $status',
  ].join('\n',);
  await runGuestScript({
    logDirectory: chosen.logDirectory,
    name: chosen.name,
    script,
    log: 'guest-crate-build.log',
    timeoutMilliseconds: CRATE_BUILD_TIMEOUT_MS,
    guestTimeoutMarginSeconds: GUEST_TIMEOUT_MARGIN_SECONDS,
  },);
}

/**
 Run the arm and refuse unless every claim it prints was printed.

 @param chosen - the run's options

 @throws GuestDriverError when the arm skipped, refused, or printed fewer claims than it must

 @example
 ```ts
 await arm({ chosen });
 ```
 */
async function arm({ chosen, }: { readonly chosen: Options; },): Promise<void> {
  /**
   The script the guest runs to perform the arm and print its log.
   */
  const script = [
    'set -eu',
    `cd ${CRATE_DIR}/package/git-policy/cli`,
    'export PATH=/opt/openzfs-2.4.4/sbin:/opt/openzfs-2.4.4/bin:/root/.cargo/bin:$PATH',
    'export LD_LIBRARY_PATH=/opt/openzfs-2.4.4/lib',
    'export CARGO_BUILD_JOBS=2',
    `export ${GUEST_VARIABLE}=${chosen.name}`,
    `export ${POOL_VARIABLE}=${POOL}`,
    'if /root/.cargo/bin/cargo nextest run --locked --test native_binary --no-capture --status-level all ' +
      '--filterset "test(zfs_sanoid_coverage_guest)" > /root/sanoid-coverage-arm.log 2>&1; then status=0; else status=$?; fi',
    'cat /root/sanoid-coverage-arm.log',
    'exit $status',
  ].join('\n',);
  /**
   What the guest printed, which is the arm's own log.
   */
  const pulled = await runGuestScript({
    logDirectory: chosen.logDirectory,
    name: chosen.name,
    script,
    log: 'guest-arm.log',
    timeoutMilliseconds: ARM_TIMEOUT_MS,
    guestTimeoutMarginSeconds: GUEST_TIMEOUT_MARGIN_SECONDS,
  },);
  if (holdsWordLine({
    text: pulled,
    word: SKIP_WORD,
  },)) {
    throw new GuestDriverError('the arm printed SKIP inside the guest: its variables did not reach it',);
  }
  /**
   The first claim the arm's own output lacks a line for, which the run refuses on.
   */
  const [firstMissing] = missingMarkers({
    text: pulled,
    markers: GREEN_MARKERS,
  },);
  if (firstMissing !== undefined)
    throw missingMarkerRefusal({ marker: firstMissing, },);
  process.stdout
    .write(`sanoid coverage arm passed ${String(GREEN_MARKERS.length,)} claim(s)\n`,);
}

/**
 Destroy the guest this driver made, or keep it and say its name.

 @param chosen - the run's options

 @throws GuestDriverError when the recipe's destruction refused

 @example
 ```ts
 await destroy({ chosen });
 ```
 */
async function destroy({ chosen, }: { readonly chosen: Options; },): Promise<void> {
  if (chosen.keepGuest) {
    process.stdout
      .write(`kept ${chosen.name}\n`,);
    return;
  }
  await runGuestRecipeStep({
    name: chosen.name,
    logDirectory: chosen.logDirectory,
    step: 'destroy',
  },);
}

/**
 How the work of one `all` run went: what refused, or nothing when nothing did.
 */
type WorkOutcome = {
  readonly refusal?: unknown;
};

/**
 Prepare, build and run the arm, reporting what refused instead of raising it.

 The three are one answer because the guest goes whatever any of them said: `all` destroys the guest it created even
 when the work refused, and a destruction that refused is the failure the run reports.

 @param chosen - the run's options

 @returns what refused, or nothing when the whole of the work passed

 @example
 ```ts
 const outcome = await attemptWork({ chosen });
 ```
 */
async function attemptWork({ chosen, }: { readonly chosen: Options; },): Promise<WorkOutcome> {
  try {
    await prepare({ chosen, },);
    await build({ chosen, },);
    await arm({ chosen, },);
    return {};
  }
  catch (caught) {
    return { refusal: caught, };
  }
}

/**
 Create, prepare, build, arm and destroy, in that order, destroying the guest whatever the work said.

 @param chosen - the run's options

 @throws GuestDriverError naming the step that refused; a failed destruction is what the run reports then

 @example
 ```ts
 await all({ chosen });
 ```
 */
async function all({ chosen, }: { readonly chosen: Options; },): Promise<void> {
  await create({ chosen, },);
  /**
   How the work went, kept until the guest is gone so a destruction that refused is the failure a person reads.
   */
  const outcome = await attemptWork({ chosen, },);
  await destroy({ chosen, },);
  if (outcome.refusal !== undefined)
    throw refusalOf({ caught: outcome.refusal, },);
}

//endregion Steps

//region The command line

/**
 The steps this driver knows, and nothing else.
 */
const steps: Record<string, (run: { readonly chosen: Options; }) => Promise<void>> = {
  create,
  prepare,
  build,
  arm,
  destroy,
  all,
};

/**
 The options this run was given.
 */
const chosen = options();

/**
 The step this run performs, or nothing when the command line named one this driver does not know.
 */
const step = steps[chosen.step];
if (step === undefined) {
  throw new GuestDriverError(`unknown step ${chosen.step}; give create, prepare, build, arm, destroy or all`,);
}
await mkdir(
  chosen.logDirectory,
  {
    recursive: true,
    mode: 0o700,
  },
);
await step({ chosen, },);

//endregion The command line
