/**
 The steps that turn a fresh Ubuntu cloud guest into a machine the four tests
 can run in.

 Every step is one command line for the guest's shell, run as the guest's root
 through the QEMU guest agent. They are in the order they must run: packages
 first, then the toolchains, then the checkout, then the account the tests run
 as, then whatever the checkout needs built.

 Nothing here touches the host besides writing an archive into this package's
 cache directory.

 @module
 */

import { basename, } from 'node:path';

import type { ThrowawayMachine, } from './machine.ts';
import { GUEST_ACCOUNT, } from './machine.ts';
import {
  type ServedFiles,
} from './served-files.ts';
import {
  quoteForPosixShell,
  renderCommandLine,
  runLoggedCommand,
} from './shell-command.ts';
import {
  GUEST_CHECKOUT_DIRECTORY,
  GUEST_LOG_DIRECTORY,
  GUEST_MARKER_PATH,
} from './test-definitions.ts';

/**
 Packages the guest installs before anything else.

 `libatomic1` is what the Node archive links against, and `build-essential` is
 what linking a Rust binary needs.
 */
const GUEST_PACKAGES: readonly string[] = [
  'wireguard-tools',
  'nftables',
  'iproute2',
  'iputils-ping',
  'sudo',
  'curl',
  'ca-certificates',
  'jq',
  'python3',
  'libatomic1',
  'build-essential',
];

/**
 Where the Node archive is unpacked in the guest.
 */
const GUEST_NODE_DIRECTORY = '/opt/node';

/**
 Where the pnpm archive is unpacked in the guest.
 */
const GUEST_PNPM_DIRECTORY = '/opt/pnpm';

/**
 The account's uid, which the Ubuntu cloud image fixes at the first ordinary
 user.
 */
const GUEST_ACCOUNT_UID = '1000';

/**
 The account's commands that the runner needs a directory for.
 */
const GUEST_ARCHIVE_DIRECTORY = '/var/tmp';

/**
 What the guest must hold before the phases run.
 */
export type ProvisionedGuest = {
  /**
   Guest path of the crate's test executable, empty when the test needs none.
   */
  readonly testBinaryPath: string;
};

/**
 Runs the packages step.

 @param machine - the guest to provision

 @throws {@link GuestCommandFailedError} when the guest refuses the step
 */
async function installPackages({ machine, }: { readonly machine: ThrowawayMachine; },): Promise<void> {
  await machine.exec(renderCommandLine({ words: [
    'apt-get',
    'install',
    '--yes',
    '--no-install-recommends',
    ...GUEST_PACKAGES,
  ], },),);
}

/**
 Downloads one archive into the guest and answers its guest path.

 @param machine - the guest to provision

 @param served - the host's file server

 @param path - host path of the archive

 @returns guest path of the downloaded archive
 */
async function fetchIntoGuest(
  {
    machine,
    path,
    served,
  }: {
    readonly machine: ThrowawayMachine;
    readonly path: string;
    readonly served: ServedFiles;
  },
): Promise<string> {
  /**
   Guest path the archive is written to.
   */
  const guestPath = `${GUEST_ARCHIVE_DIRECTORY}/${basename(path,)}`;
  await machine.exec(renderCommandLine({ words: [
    'curl',
    '--silent',
    '--show-error',
    '--output',
    guestPath,
    served.urlOf(path,),
  ], },),);
  return guestPath;
}

/**
 Installs the Node toolchain the lockfile names.

 @param machine - the guest to provision

 @param archive - host path of the Node archive

 @param served - the host's file server
 */
async function installNode(
  {
    archive,
    machine,
    served,
  }: {
    readonly archive: string;
    readonly machine: ThrowawayMachine;
    readonly served: ServedFiles;
  },
): Promise<void> {
  /**
   The archive inside the guest.
   */
  const guestArchive = await fetchIntoGuest({
    machine,
    path: archive,
    served,
  },);
  await machine.exec([
    `mkdir --parents ${quoteForPosixShell(GUEST_NODE_DIRECTORY,)}`,
    `tar --extract --gzip --file ${quoteForPosixShell(guestArchive,)} --directory ${
      quoteForPosixShell(GUEST_NODE_DIRECTORY,)
    } --strip-components=1`,
    `ln --symbolic --force ${GUEST_NODE_DIRECTORY}/bin/node /usr/local/bin/node`,
    `ln --symbolic --force ${GUEST_NODE_DIRECTORY}/bin/npm /usr/local/bin/npm`,
    `ln --symbolic --force ${GUEST_NODE_DIRECTORY}/bin/npx /usr/local/bin/npx`,
  ].join(' && ',),);
}

/**
 Installs the pnpm the lockfile names.

 @param machine - the guest to provision

 @param archive - host path of the pnpm archive

 @param served - the host's file server
 */
async function installPnpm(
  {
    archive,
    machine,
    served,
  }: {
    readonly archive: string;
    readonly machine: ThrowawayMachine;
    readonly served: ServedFiles;
  },
): Promise<void> {
  /**
   The archive inside the guest.
   */
  const guestArchive = await fetchIntoGuest({
    machine,
    path: archive,
    served,
  },);
  await machine.exec([
    `mkdir --parents ${quoteForPosixShell(GUEST_PNPM_DIRECTORY,)}`,
    `tar --extract --gzip --file ${quoteForPosixShell(guestArchive,)} --directory ${
      quoteForPosixShell(GUEST_PNPM_DIRECTORY,)
    }`,
    `ln --symbolic --force ${GUEST_PNPM_DIRECTORY}/pnpm /usr/local/bin/pnpm`,
  ].join(' && ',),);
}

/**
 Unpacks the checkout into the guest and gives it to the account.

 @param machine - the guest to provision

 @param archive - host path of the payload archive

 @param served - the host's file server
 */
async function installCheckout(
  {
    archive,
    machine,
    served,
  }: {
    readonly archive: string;
    readonly machine: ThrowawayMachine;
    readonly served: ServedFiles;
  },
): Promise<void> {
  /**
   The payload archive inside the guest.
   */
  const guestArchive = await fetchIntoGuest({
    machine,
    path: archive,
    served,
  },);
  await machine.exec([
    `mkdir --parents ${quoteForPosixShell(GUEST_CHECKOUT_DIRECTORY,)}`,
    `tar --extract --gzip --file ${quoteForPosixShell(guestArchive,)} --directory ${
      quoteForPosixShell(GUEST_CHECKOUT_DIRECTORY,)
    }`,
    `mkdir --parents ${quoteForPosixShell(GUEST_LOG_DIRECTORY,)}`,
    `chown --recursive ${GUEST_ACCOUNT}:${GUEST_ACCOUNT} ${quoteForPosixShell(GUEST_CHECKOUT_DIRECTORY,)}`,
  ].join(' && ',),);
}

/**
 Gives the account a passwordless `sudo`, its user manager, and the marker the
 tests look for.

 The marker is created here and nowhere else: a test that does not find it
 refuses with the words that name the mise task which provides it.

 @param machine - the guest to provision
 */
async function prepareAccount({ machine, }: { readonly machine: ThrowawayMachine; },): Promise<void> {
  await machine.exec([
    `printf '%s\\n' ${quoteForPosixShell(`${GUEST_ACCOUNT} ALL=(ALL) NOPASSWD: ALL`,)} > ${quoteForPosixShell('/etc/sudoers.d/90-wg-test-in-a-virtual-machine',)}`,
    `chmod 0440 ${quoteForPosixShell('/etc/sudoers.d/90-wg-test-in-a-virtual-machine',)}`,
    `loginctl enable-linger ${GUEST_ACCOUNT}`,
    `systemctl start ${quoteForPosixShell(`user@${GUEST_ACCOUNT_UID}.service`,)}`,
    `mkdir --parents ${quoteForPosixShell(GUEST_MARKER_PATH,)}`,
  ].join(' && ',),);
}

/**
 Checks that the account really has the uid the test definitions name.

 @param machine - the guest to provision

 @throws when the image's account is not the one the definitions expect
 */
async function checkAccount({ machine, }: { readonly machine: ThrowawayMachine; },): Promise<void> {
  /**
   The account's uid as the guest reports it.
   */
  const uid = (await machine.exec(renderCommandLine({ words: [
    'id',
    '--user',
    GUEST_ACCOUNT,
  ], },),)).trim();
  if (uid !== GUEST_ACCOUNT_UID) {
    throw new Error(
      `the guest's ${GUEST_ACCOUNT} has uid ${uid}, and the tests' manager paths name uid ${GUEST_ACCOUNT_UID}.`,
    );
  }
}

/**
 Installs the checkout's Node dependencies from its lockfile, as the account.

 @param machine - the guest to provision
 */
async function installNodeWorkspace({ machine, }: { readonly machine: ThrowawayMachine; },): Promise<void> {
  await runLoggedStep({
    command: runLoggedCommand({
      environment: {},
      logPath: `${GUEST_LOG_DIRECTORY}/pnpm-install.log`,
      words: [
        ...accountPrefix(),
        'pnpm',
        '--dir',
        GUEST_CHECKOUT_DIRECTORY,
        'install',
        '--frozen-lockfile',
      ],
    },),
    logPath: `${GUEST_LOG_DIRECTORY}/pnpm-install.log`,
    machine,
  },);
}

/**
 Runs one provisioning command that writes its output to a guest log, and
 repeats what the guest said when it fails.

 @param command - command line for the guest's shell

 @param logPath - guest file the command's output goes to

 @param machine - the guest to provision

 @throws {@link Error} carrying the guest's own output

 @example
 ```ts
 await runLoggedStep({ command: 'true >> /work/logs/a.log 2>&1', logPath: '/work/logs/a.log', machine });
 ```
 */
async function runLoggedStep(
  {
    command,
    logPath,
    machine,
  }: {
    readonly command: string;
    readonly logPath: string;
    readonly machine: ThrowawayMachine;
  },
): Promise<void> {
  /**
   Status of the command and whatever the guest printed around it.
   */
  const result = await machine.execAllowingFailure(command,);
  if (result.status === 0)
    return;
  /**
   What the failing command wrote to its own log, when that file exists.
   */
  const written = await machine.execAllowingFailure(renderCommandLine({ words: [
    'cat',
    logPath,
  ], },),);
  throw new Error(
    `the guest command ended with status ${String(result.status,)}: ${command}\n${written.output.trim()}`,
  );
}

/**
 The words that run a command as the account without a login shell of its own.

 @returns the prefix for such a command
 */
function accountPrefix(): readonly string[] {
  return [
    'runuser',
    '--user',
    GUEST_ACCOUNT,
    '--',
  ];
}

/**
 Builds the crate and answers the path of its test executable.

 @param machine - the guest to provision

 @param crateDirectory - guest directory of the crate

 @returns guest path of the test executable

 @throws when cargo reports no test executable
 */
async function buildRustCrate(
  {
    crateDirectory,
    machine,
  }: {
    readonly crateDirectory: string;
    readonly machine: ThrowawayMachine;
  },
): Promise<string> {
  // rustup installs into the account's home, so the toolchain is the account's
  // own and the crate is built by the same caller that runs its unprivileged
  // tests.
  await runLoggedStep({
    command: runLoggedCommand({
      environment: {},
      logPath: `${GUEST_LOG_DIRECTORY}/rustup.log`,
      words: [
        ...accountPrefix(),
        'sh',
        '-lc',
        'curl --proto =https --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain stable',
      ],
    },),
    logPath: `${GUEST_LOG_DIRECTORY}/rustup.log`,
    machine,
  },);
  await runLoggedStep({
    command: runLoggedCommand({
      environment: {},
      logPath: `${GUEST_LOG_DIRECTORY}/cargo-build.log`,
      words: [
        ...accountPrefix(),
        'sh',
        '-lc',
        `cd ${quoteForPosixShell(crateDirectory,)} && cargo build`,
      ],
    },),
    logPath: `${GUEST_LOG_DIRECTORY}/cargo-build.log`,
    machine,
  },);
  await runLoggedStep({
    command: runLoggedCommand({
      environment: {},
      logPath: `${GUEST_LOG_DIRECTORY}/cargo-build-tests.log`,
      words: [
        ...accountPrefix(),
        'sh',
        '-lc',
        `cd ${quoteForPosixShell(crateDirectory,)} && cargo test --no-run --message-format=json > ${GUEST_LOG_DIRECTORY}/cargo-artifacts.json`,
      ],
    },),
    logPath: `${GUEST_LOG_DIRECTORY}/cargo-build-tests.log`,
    machine,
  },);
  /**
   Paths of every test executable cargo reported.
   */
  const reported = await machine.exec(renderCommandLine({ words: [
    'jq',
    '--raw-output',
    'select(.reason == "compiler-artifact" and .executable != null) | .executable',
    `${GUEST_LOG_DIRECTORY}/cargo-artifacts.json`,
  ], },),);
  /**
   The single test executable the crate builds.
   */
  const executable = reported.trim();
  if (executable === '') {
    throw new Error('cargo reported no test executable for the crate.',);
  }
  return executable;
}

/**
 Provisions a fresh guest for one of the tests.

 @param machine - the guest to provision

 @param nodeArchive - host path of the locked Node archive

 @param payloadPath - host path of the payload archive

 @param pnpmArchive - host path of the locked pnpm archive

 @param served - the host's file server

 @param crateDirectory - guest directory of the crate, when the test needs it

 @param needsNodeWorkspace - whether the checkout's Node dependencies are installed

 @returns what the phases need from the guest

 @throws when a step fails
 */
export async function provisionGuest(
  {
    crateDirectory,
    machine,
    needsNodeWorkspace,
    nodeArchive,
    payloadPath,
    pnpmArchive,
    served,
  }: {
    readonly crateDirectory: string | undefined;
    readonly machine: ThrowawayMachine;
    readonly needsNodeWorkspace: boolean;
    readonly nodeArchive: string;
    readonly payloadPath: string;
    readonly pnpmArchive: string;
    readonly served: ServedFiles;
  },
): Promise<ProvisionedGuest> {
  await installPackages({ machine, },);
  await installNode({
    archive: nodeArchive,
    machine,
    served,
  },);
  await installPnpm({
    archive: pnpmArchive,
    machine,
    served,
  },);
  await installCheckout({
    archive: payloadPath,
    machine,
    served,
  },);
  await prepareAccount({ machine, },);
  await checkAccount({ machine, },);
  if (needsNodeWorkspace) {
    await installNodeWorkspace({ machine, },);
  }
  /**
   Guest path of the crate's test executable, empty when there is no crate.
   */
  const testBinaryPath = crateDirectory === undefined
    ? ''
    : await buildRustCrate({
      crateDirectory,
      machine,
    },);
  return { testBinaryPath, };
}
