/**
 The command line that runs one of the four privileged tests inside a machine.

 A run downloads the two toolchains the repository locks, packs the checkout,
 serves both to a fresh Ubuntu guest, provisions that guest, runs the test's
 commands there, prints what they printed, and destroys the machine. The
 invoking host is never the machine the test runs in, and nothing on it changes
 besides this package's cache directory.

 @module
 */

import { randomBytes, } from 'node:crypto';
import { appendFile, } from 'node:fs/promises';
import { dirname, } from 'node:path';

import {
  haveArchive,
  lockedArchive,
} from './locked-archives.ts';
import {
  createMachine,
  GUEST_ACCOUNT,
  machineName,
} from './machine.ts';
import {
  PAYLOAD_PATH,
  writePayload,
} from './payload.ts';
import { provisionGuest, } from './provisioning.ts';
import { serveFiles, } from './served-files.ts';
import {
  definitionOf,
  type Options,
  parseArguments,
  renderPhaseCommand,
} from './run-request.ts';
import { renderCommandLine, } from './shell-command.ts';
import {
  GUEST_CRATE_DIRECTORY,
  GUEST_LOG_DIRECTORY,
  type TestDefinition,
} from './test-definitions.ts';

/**
 Exit status when a test's own phases fail inside the machine.
 */
const STATUS_TEST_FAILED = 1;

/**
 Exit status when this command line itself is wrong.
 */
const STATUS_USAGE = 2;

/**
 The account's runtime directory inside the guest, as the tests' manager paths
 name it.
 */
const GUEST_ACCOUNT_RUNTIME_DIRECTORY = '/run/user/1000';

/**
 Runs every phase of one test in the machine and prints what each printed.

 @param definition - the test to run

 @param machine - the provisioned machine

 @param testBinaryPath - guest path of the crate's test executable

 @param write - where the run's whole output goes

 @returns whether every phase passed
 */
async function runPhases(
  {
    definition,
    machine,
    testBinaryPath,
    write,
  }: {
    readonly definition: TestDefinition;
    readonly machine: Awaited<ReturnType<typeof createMachine>>;
    readonly testBinaryPath: string;
    readonly write: (text: string,) => Promise<void>;
  },
): Promise<boolean> {
  /**
   Whether every phase passed so far.
   */
  let passed = true;
  for (const phase of definition.phases({ testBinaryPath, },)) {
    /**
     Guest file this phase's output goes to.
     */
    const guestLog = `${GUEST_LOG_DIRECTORY}/${phase.id}.log`;
    await write(`\n=== ${definition.id}: ${phase.id} (as ${phase.caller}) ===\n`,);
    /**
     Result of the phase's command, whose status is the test's own.
     */
    const result = await machine.execAllowingFailure(renderPhaseCommand({
      logPath: guestLog,
      phase,
    },),);
    /**
     What the phase printed, read back from the guest.
     */
    const printed = await machine.exec(renderCommandLine({ words: [
      'cat',
      guestLog,
    ], },),);
    await write(printed,);
    await write(`\n=== ${definition.id}: ${phase.id} ended with status ${String(result.status,)} ===\n`,);
    if (result.status !== 0) {
      passed = false;
    }
  }
  return passed;
}

/**
 What a phase could have created in the guest, and how each is listed.

 A failing phase stops before its own removals, so the inventory says what it
 left behind while the machine still holds it.
 */
const LEFTOVER_PROBES: readonly {
  readonly label: string;
  readonly words: readonly string[];
}[] = [
  {
    label: 'BPF pins',
    words: ['find', '/sys/fs/bpf', '-mindepth', '1', '-maxdepth', '2',],
  },
  {
    label: 'cgroups',
    words: ['find', '/sys/fs/cgroup', '-mindepth', '1', '-maxdepth', '1', '-name', 'wg-quicker-exempt*',],
  },
  {
    label: 'network namespaces',
    words: ['find', '/run/netns', '-mindepth', '1', '-maxdepth', '1',],
  },
  {
    label: 'runtime files',
    words: ['find', '/run', '-mindepth', '1', '-maxdepth', '1', '-name', 'wg-quicker*',],
  },
];

/**
 Prints what the phases left in the guest, before the machine is destroyed.

 @param machine - the machine the phases ran in

 @param write - where the run's whole output goes
 */
async function reportLeftovers(
  {
    machine,
    write,
  }: {
    readonly machine: Awaited<ReturnType<typeof createMachine>>;
    readonly write: (text: string,) => Promise<void>;
  },
): Promise<void> {
  await write('\n=== what the phases left inside the machine ===\n',);
  for (const probe of LEFTOVER_PROBES) {
    /**
     The probe's output, which names nothing when the guest holds nothing.
     */
    const listed = await machine.execAllowingFailure(renderCommandLine({ words: probe.words, },),);
    await write(`${probe.label}: ${listed.output.trim() || 'nothing'}\n`,);
  }
  /**
   The scopes the account's user manager still holds.
   */
  const scopes = await machine.execAllowingFailure(renderCommandLine({ words: [
    'runuser',
    '--user',
    GUEST_ACCOUNT,
    '--',
    'sh',
    '-lc',
    `XDG_RUNTIME_DIR=${GUEST_ACCOUNT_RUNTIME_DIRECTORY} systemctl --user list-units --type=scope --no-legend --no-pager`,
  ], },),);
  await write(`the account's scopes: ${scopes.output.trim() || 'nothing'}\n`,);
}

/**
 Runs one test in a machine of its own.

 @param options - the command line's request

 @returns the process exit status

 @throws when the run cannot be prepared at all
 */
async function main({
  logPath,
  test,
}: Options,): Promise<number> {
  /**
   The test to run.
   */
  const definition = definitionOf(test,);
  /**
   Every line this run printed, so it can be written to the caller's file.
   */
  const lines: string[] = [];
  /**
   Prints one piece of the run and keeps it for the file.
   */
  const write = async function write(text: string,): Promise<void> {
    process.stdout.write(text,);
    lines.push(text,);
  };

  await write(`=== ${definition.id}: ${definition.description} ===\n`,);
  /**
   The locked Node archive, downloaded once and verified.
   */
  const nodeArchive = await haveArchive({ archive: await lockedArchive({ tool: 'node', },), },);
  /**
   The locked pnpm archive, downloaded once and verified.
   */
  const pnpmArchive = await haveArchive({ archive: await lockedArchive({ tool: 'pnpm', },), },);
  /**
   The checkout as the guest receives it.
   */
  const payloadPath = await writePayload({ destination: PAYLOAD_PATH, },);
  /**
   The host's file server for the guest.
   */
  const served = await serveFiles({ directory: dirname(payloadPath,), },);
  /**
   Name this run's machine carries, unique among runs at once.
   */
  const name = machineName({
    id: definition.id,
    suffix: randomBytes(4,)
      .toString('hex',),
  },);
  await write(`=== machine ${name}, image ubuntu ===\n`,);
  /**
   The machine, created before the first guest command.
   */
  const machine = await createMachine({ name, },);
  /**
   Whether every phase passed.
   */
  let passed = false;
  try {
    /**
     What provisioning left the guest holding.
     */
    const provisioned = await provisionGuest({
      crateDirectory: definition.needsRustCrate ? GUEST_CRATE_DIRECTORY : undefined,
      machine,
      needsNodeWorkspace: definition.needsNodeWorkspace,
      nodeArchive,
      payloadPath,
      pnpmArchive,
      served,
    },);
    passed = await runPhases({
      definition,
      machine,
      testBinaryPath: provisioned.testBinaryPath,
      write,
    },);
    await reportLeftovers({
      machine,
      write,
    },);
  }
  finally {
    await machine.destroy();
    await served.close();
    await write(`=== machine ${name} destroyed ===\n`,);
    if (logPath !== undefined) {
      await appendFile(
        logPath,
        lines.join('',),
      );
    }
  }
  return passed ? 0 : STATUS_TEST_FAILED;
}

try {
  process.exitCode = await main(parseArguments(process.argv.slice(2,),),);
}
catch (error) {
  process.stderr.write(`${
    error instanceof Error
      ? error.message
      : String(error,)
  }\n`,);
  process.exitCode = STATUS_USAGE;
}
