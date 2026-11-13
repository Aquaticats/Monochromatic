#!/usr/bin/env node
/**
 What:
  The proof that a Git command never makes the user service manager of an account do work whose cost grows with the units it
  holds, and that the desktop session's own service survives everything a commit does.

 Why:
  On 2026-11-12T01:24:54-05:00 the desktop session ended because one commit asked the shared user manager for a reload: the
  manager re-read every unit it held for 139,770 ms, a reply to the window manager's unit expired while it did, and the
  manager read the expiry as a lost bus name and stopped the window manager with the whole session
  (`doc/troubleshooting/plasma-session-ended-after-service-manager-timeouts.md`).

 How the proof works:
  Two arms run the same steps, each on a machine of its own (`service-manager-machine.ts`): a container whose system manager
  is its init, a private session bus, a `systemd --user`, thousands of transient units so a reload of it costs what a reload
  of a loaded host costs, and an unrelated `Type=dbus` service bound to a target as the session's own units are
  (`service-manager-desktop.ts`). One arm runs the release the account has installed from `main`, which is the code the
  reload comes from; the other runs the release built from this worktree, on a machine that the release from `main`
  provisioned first, which is the migration (`service-manager-scenario.ts`). The arm from `main` has to end the unrelated
  service and its target, and the arm from this worktree has to keep both and ask for no manager-wide operation at all.

 Usage:
  `node bin/service-manager-proof.ts --image <tag> --main-wrapper <path> [--seed-wrapper <path>] [--units <count>]
  [--probe-deadline <value>]`

  `--seed-wrapper` names the release the migration arm's machine is provisioned by first, and defaults to the release
  `--main-wrapper` names. A machine here can carry any of the templates an earlier release left: `0.6.1-alpha.13`'s (the
  executable of the release that wrote it) or `0.6.1-alpha.14`'s (a link of that release's own), and a proof run names the
  build of each it wants that arm to meet.

  `--harm-seed-wrapper` seeds the control arm's machine with an older release as well, and it is what makes the control arm
  reproduce the harm: the release from `main` asks the manager to read every unit only when a unit *text* changes, which is
  the first commit after a machine carries an earlier release's template and never a release change on a machine this
  release provisioned. A control arm on a fresh machine therefore asks for no reload at all, and a proof run that seeds only
  the migration arm says so and refuses to call itself a pass.
 */
import { resolve, } from 'node:path';

import { NativeVerificationError, } from './native-verification-process.ts';
import { reloadDuration, } from './service-manager-desktop.ts';
import {
  isTransportFailure,
  removeKeptMachines,
  startMachine,
} from './service-manager-machine.ts';
import {
  type ArmReport,
  newArm,
  type Seed,
} from './service-manager-scenario.ts';

/**
 The deadline the unrelated desktop service is given for its own start, as the incident's fixture gave one.
 A reload holds the manager's event loop past it, which is what makes the manager read a queued answer as a lost bus name.
 */
const defaultProbeDeadline = '250ms';

/**
 How many transient units the manager holds before the steps begin.
 Measured: a disposable manager took 346 to 358 ms to reload while holding 1,059 loaded transient services and 194 to 297 ms
 while holding 559, so this machine's reload outlives the stand-in's deadline; the four-thousand-unit machine of the
 measurement in `doc/planning/git-wrapper-reload-remedy.md` spends most of its life waiting for the container runtime on a
 loaded host.
 */
const defaultUnitCount = 1_500;

/**
 The arguments one option takes: its own name and the value after it.
 */
const argumentsPerOption = 2;

/**
 Options of this run.
 */
type Options = {
  /**
   The gate's image, which carries systemd, Git and dbus.
   */
  readonly image: string;
  /**
   The executable of the release the account installed from `main`.
   */
  readonly mainWrapper: string;
  /**
   The executable of the release that provisions the migration arm's machine first, which is the release from `main`
   unless a run names another build of an earlier release.
   */
  readonly seedWrapper: string;
  /**
   The release that provisions the control arm's machine first, or nothing when that arm runs on a fresh machine.
   */
  readonly harmSeed: Seed;
  /**
   How many transient units each machine holds.
   */
  readonly units: number;
  /**
   How long the desktop stand-in's own start may take.
   */
  readonly probeDeadline: string;
};

/**
 Reads the options this run was given.

 @param argv - the arguments after the program's name

 @returns the image, the main release's wrapper, the unit count and the probe deadline

 @example
 ```ts
 options({ argv: ['--image', 'localhost/x', '--main-wrapper', '/opt/git'] });
 ```
 */
function options({ argv, }: { readonly argv: readonly string[]; }): Options {
  /**
   The options as given, the last one winning, which is how a person's later value replaces a default.
   */
  const given = new Map<string, string>();
  for (let index = 0; index < argv.length; index += argumentsPerOption) {
    /**
     The option's name and its value.
     */
    const [name = '', value = ''] = argv.slice(
      index,
      index + argumentsPerOption,
    );
    given.set(
      name,
      value,
    );
  }
  /**
   The gate's image, which the caller always names.
   */
  const image = given.get('--image',);
  /**
   The release from `main`, whose executable the control arm runs.
   */
  const mainWrapper = given.get('--main-wrapper',);
  if ((image === undefined) || (mainWrapper === undefined)) {
    throw new NativeVerificationError('Usage: node bin/service-manager-proof.ts --image <tag> --main-wrapper <path> [--units <count>] [--probe-deadline <value>]');
  }
  return {
    image,
    mainWrapper,
    seedWrapper: given.get('--seed-wrapper',) ?? mainWrapper,
    harmSeed: given.has('--harm-seed-wrapper',)
      ? {
        kind: 'release',
        wrapper: String(given.get('--harm-seed-wrapper',),),
      }
      : { kind: 'none', },
    units: Number(given.get('--units',) ?? defaultUnitCount,),
    probeDeadline: given.get('--probe-deadline',) ?? defaultProbeDeadline,
  };
}

/**
 What:
  Runs one arm: its machine, its release, its steps, and the reading of what its manager was asked for.

 Why:
  The container is removed by `await using` whatever the steps found, so a failing proof leaves no manager holding thousands
  of units on this host.

 @param name - the arm's name

 @param image - the gate's image

 @param wrapper - the executable this arm runs, at both of its release paths

 @param probeDeadline - how long the desktop stand-in's own start may take

 @param units - how many transient units the machine holds

 @param seed - the release from `main`, which provisions the machine first when this arm is the migration

 @returns this arm's report

 @example
 ```ts
 await runArm({ name: 'main', image, wrapper: '/tmp/git', probeDeadline: '200ms', units: 4000 });
 ```
 */
async function runArm({
  name,
  image,
  wrapper,
  probeDeadline,
  units,
  seed,
}: {
  readonly name: string;
  readonly image: string;
  readonly wrapper: string;
  readonly probeDeadline: string;
  readonly units: number;
  readonly seed: Seed;
}): Promise<ArmReport> {
  /**
   The machine this arm runs on, removed when this function returns or throws.
   */
  await using running = await startMachine({
    image,
    units,
  });
  /**
   The arm this machine belongs to.
   */
  const arm = newArm({
    machine: running.machine,
    name,
  });
  await arm.prepare({
    wrapper,
    seed,
  });
  return arm.run({ probeDeadline, });
}

/**
 What:
  Runs one arm, and starts it again on a fresh machine when the container runtime itself failed.

 Why:
  Podman on a loaded host answers a session in a container holding thousands of units with a timeout, or loses the container
  it just made, and a proof that reported that would report the host instead of the wrapper. The steps are only ever run on
  a machine that was made for them, so a retry repeats no step the wrapper performed.

 @param attempt - how many times this arm has been started

 @param attempts - how many times it may be started

 @param name - the arm's name

 @param image - the gate's image

 @param wrapper - the executable this arm runs

 @param probeDeadline - how long the desktop stand-in's own start may take

 @param units - how many transient units the machine holds

 @param seed - the release that provisions the machine first, or the fact that it is not seeded

 @returns this arm's report

 @example
 ```ts
 await runArmAgain({ attempt: 0, attempts: 3, name: 'main', image, wrapper, probeDeadline, units, seed: { kind: 'none' } });
 ```
 */
async function runArmAgain({
  attempt,
  attempts,
  name,
  image,
  wrapper,
  probeDeadline,
  units,
  seed,
}: {
  readonly attempt: number;
  readonly attempts: number;
  readonly name: string;
  readonly image: string;
  readonly wrapper: string;
  readonly probeDeadline: string;
  readonly units: number;
  readonly seed: Seed;
}): Promise<ArmReport> {
  try {
    return await runArm({
      name,
      image,
      wrapper,
      probeDeadline,
      units,
      seed,
    });
  } catch (error) {
    if ((!(error instanceof NativeVerificationError)) || (!isTransportFailure({ message: error.message, }))) throw error;
    if ((attempt + 1) >= attempts) throw error;
    console.error(`arm ${name}: the container runtime failed, so this arm starts again on a fresh machine (${String(attempt + 1)} of ${String(attempts - 1)})`);
    return runArmAgain({
      attempt: attempt + 1,
      attempts,
      name,
      image,
      wrapper,
      probeDeadline,
      units,
      seed,
    });
  }
}

/**
 What:
  Prints one arm's report, one line per step and one per reload.

 Why:
  A run that fails still says what each arm saw, which is what a reader needs to tell a fixture that proves nothing from one
  that proves the defect gone.

 @param report - the arm's report

 @example
 ```ts
 printReport({ report });
 ```
 */
function printReport({ report, }: { readonly report: ArmReport; }): void {
  console.log(`\narm ${report.name}:`);
  for (const step of report.steps) console.log(`  ${step}`);
  for (const line of report.reloads) console.log(`  ${reloadDuration({ line, })}`);
  console.log(`  commits that did not land: ${report.failures
    .length
    === 0 ? 'none' : report.failures
      .join('; ')}`);
  console.log(`  the desktop stand-in ended: ${report.deaths
    .length
    === 0 ? 'never' : report.deaths
      .join('; ')}`);
}

/**
 What:
  Runs both arms and judges them.

 Why:
  The arm from `main` shows the defect the issue is about, and the arm from this worktree has to show it gone; a control
  that only counted `Reload` calls in a stand-in manager would prove nothing about a real manager.
 */
async function main_(): Promise<void> {
  /**
   How many times one arm is started before a container-runtime failure ends this run.
   */
  const armAttempts = 3;
  /**
   What this run was asked to do.
   */
  const {
    image,
    mainWrapper,
    seedWrapper,
    harmSeed,
    units,
    probeDeadline,
  } = options({ argv: process.argv
    .slice(2,), });
  /**
   The release built from this worktree, which the task builds before it runs this file.
   */
  const candidate = process.env
    .GPC_CANDIDATE_WRAPPER
    ?? resolve(
      import.meta.dirname,
      '..',
      'target',
      'debug',
      'monochromatic-git-policy-cli',
    );
  /**
   The arm that runs the release the account has installed from `main`, on a fresh machine.
   */
  const onMain = await runArmAgain({
    attempt: 0,
    attempts: armAttempts,
    name: 'main',
    image,
    wrapper: mainWrapper,
    probeDeadline,
    units,
    seed: harmSeed,
  });
  printReport({ report: onMain, });
  /**
   The arm that runs the release built from this worktree, on a machine the release from `main` provisioned first.
   */
  const fixed = await runArmAgain({
    attempt: 0,
    attempts: armAttempts,
    name: 'this worktree',
    image,
    wrapper: candidate,
    probeDeadline,
    units,
    seed: {
      kind: 'release',
      wrapper: seedWrapper,
    },
  });
  printReport({ report: fixed, });
  console.log(`\nthe arm from main asked for ${String(onMain.reloads
    .length)} manager-wide reload(s); the arm from this worktree asked for ${String(fixed.reloads
      .length)}`);
  /**
   Everything that makes this run stop proving anything.
   */
  const problems: string[] = [];
  if (fixed.failures
    .length
    > 0) problems.push(`the release from this worktree failed its own steps: ${fixed.failures
      .join('; ')}`);
  if (fixed.deaths
    .length
    > 0) problems.push(`the release from this worktree ended the unrelated desktop: ${fixed.deaths
      .join('; ')}`);
  if (fixed.reloads
    .length
    > 0) problems.push(`the release from this worktree asked the manager to re-read its units ${String(fixed.reloads
      .length)} time(s)`);
  if (onMain.reloads
    .length
    === 0) problems.push('the release from main asked for no reload, so this fixture would prove nothing about it');
  if (onMain.deaths
    .length
    === 0) problems.push('the release from main did not end the unrelated desktop, so this fixture does not reproduce the harm');
  // What: Every machine this run started is removed here, whatever the judgement below says.
  // Why:  The two arms' containers are kept until both have run, because removing one keeps this host's container storage
  //       busy long enough that the next machine's sessions are refused; they are ended before the verdict is reported.
  await removeKeptMachines();
  if (problems.length > 0) throw new NativeVerificationError(`THE PROOF FAILED: ${problems.join('; ')}`);
}

await main_();
