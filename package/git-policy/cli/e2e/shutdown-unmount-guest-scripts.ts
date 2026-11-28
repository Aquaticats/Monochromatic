/**
 The two scripts plan step 19's shutdown arm runs in its guest, and the variables and pool they name.

 Both scripts are the arm's own commands, kept as lines a person reads rather than as strings built at the call site: the
 build unpacks this worktree and compiles the crate's test targets, and the arm runs one `nextest` filterset and then
 refuses a log that lacks a claim it made. Every name used by one script and not the other lives here too, because a name
 is one name whichever script reads it.
 */
import {
  SWAP_GIBIBYTES,
} from '../bin/mvm-guest-bounds.ts';

/**
 Where the crate is unpacked and built inside the guest, which every account may traverse.
 */
export const CRATE_DIR = '/opt/shutdown-unmount-pass';

/**
 The log the build writes, inside the guest's shared folder.
 */
export const BUILD_LOG = 'shutdown-unmount-build.log';

/**
 The log the arm's own test output is kept in, inside the guest's shared folder.
 */
export const ARM_LOG = 'shutdown-unmount-arm.log';

/**
 The word a test binary prints at the start of a line when it returned without running its arm.
 */
export const SKIP_WORD = 'SKIPPED';

/**
 The variable the arm reads, so it never runs on a machine that is not its guest.
 */
export const GUEST_VARIABLE = 'CLI_GIT_ZFS_SHUTDOWN_UNMOUNT_GUEST';

/**
 The pool the driver's recipe prepared, which the sweep arms' variables already name.
 */
const POOL_VARIABLE = 'CLI_GIT_ZFS_SWEEP_POOL';

/**
 The OpenZFS userspace the arm's commands come from, in the same variable the sweep arms read.
 */
const SBIN_VARIABLE = 'CLI_GIT_ZFS_SWEEP_SBIN';

/**
 The checkout the arm builds its layout at, in the same variable the sweep arms read.
 */
const CHECKOUT_VARIABLE = 'CLI_GIT_ZFS_SWEEP_CHECKOUT';

/**
 The pool the arm's own clone container and checkout dataset live in.
 */
export const POOL = 'zfsshutdownunmount';

/**
 The OpenZFS 2.4.4 prefix the guest's recipe installs.
 */
export const OPENZFS_PREFIX = '/opt/openzfs-2.4.4';

/**
 The checkout the arm mounts its dataset at,
 which is a folder of its own: the pool's image lies beside it and must never be
 under a path a dataset is mounted at.
 */
export const CHECKOUT = '/home/shutdown-unmount/checkout';

/**
 The folder the pool's image lies in, beside the checkout and never below it.
 */
export const POOL_IMAGE_FOLDER = '/home/shutdown-unmount';

/**
 The `PATH` the guest's OpenZFS userspace needs.
 */
const OPENZFS_PATH = `${OPENZFS_PREFIX}/sbin:${OPENZFS_PREFIX}/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin`;

/**
 The lines a passing run must find in the arm log, one per claim the arm makes.
 */
export const EXPECTED_MARKERS = [
  'SHUTDOWN-UNMOUNT layout=two-mounts',
  'SHUTDOWN-UNMOUNT readiness=published',
  'SHUTDOWN-UNMOUNT idle=unmounted',
  'SHUTDOWN-UNMOUNT busy=held',
  'SHUTDOWN-UNMOUNT holder=named',
  'SHUTDOWN-UNMOUNT readiness=withdrawn',
  'SHUTDOWN-UNMOUNT intent=unchanged',
  'SHUTDOWN-UNMOUNT second-pass=proves-nothing',
  'SHUTDOWN-UNMOUNT done',
] as const;

/**
 The environment the arm reads, so every step names its own guest and its own pool.

 @param name - the guest's name, which the arm's own variable answers

 @returns the `export` lines the arm's script starts with, in the order the arm reads them

 @example
 ```ts
 armEnvironmentLines({ name: 'zfs-r7-shutdown-unmount' }); // ['export CLI_GIT_ZFS_SHUTDOWN_UNMOUNT_GUEST=zfs-r7-shutdown-unmount', ...]
 ```
 */
export function armEnvironmentLines({ name, }: { readonly name: string; },): readonly string[] {
  return [
    `export ${GUEST_VARIABLE}=${name}`,
    `export ${POOL_VARIABLE}=${POOL}`,
    `export ${SBIN_VARIABLE}=${OPENZFS_PREFIX}/sbin`,
    `export ${CHECKOUT_VARIABLE}=${CHECKOUT}`,
  ];
}

/**
 The script the guest runs to unpack this worktree and build the library and the test target the arm runs.

 The pool is made here, from a file, so nothing of the guest's root disk is a vdev, and its mount point is none so no
 dataset of it stands where a slot would.

 @returns the script, its lines in the order the guest runs them

 @example
 ```ts
 const build = buildScript();
 ```
 */
export function buildScript(): string {
  return [
    'set -eu',
    // The shared copy is the tree this run made; a guest reused by an earlier run still holds that run's tarball at
    // /root, so the shared copy always wins and a missing one refuses rather than building the older tree.
    'test -f /mnt/shared/subject.tar.gz',
    'cp /mnt/shared/subject.tar.gz /root/subject.tar.gz',
    `rm --recursive --force ${CRATE_DIR}`,
    `mkdir --parents ${CRATE_DIR}`,
    `tar --extract --gzip --file /root/subject.tar.gz --directory ${CRATE_DIR}`,
    `mkdir --parents ${POOL_IMAGE_FOLDER}`,
    `if ! ${OPENZFS_PREFIX}/sbin/zpool list -H -o name ${POOL} > /dev/null 2>&1; then`,
    `  truncate --size 1G ${POOL_IMAGE_FOLDER}/pool.img`,
    `  ${OPENZFS_PREFIX}/sbin/zpool create -f -O mountpoint=none ${POOL} ${POOL_IMAGE_FOLDER}/pool.img`,
    'fi',
    `cd ${CRATE_DIR}/package/git-policy/cli`,
    'export PATH=/root/.cargo/bin:$PATH',
    'df --human-readable / /mnt/shared',
    // `free` with no option, because the guest's procps has no `--mebytes` and `set -e` would end the build there.
    'free',
    // One rustc for a crate this size wants more than five gigabytes, and the guest has eight: a second compile beside
    // it exhausted the guest's memory and the kernel killed one of them, which looked like a compile failure. One job
    // and a swap file are what make the build fit.
    `if ! swapon --show | grep --quiet /swapfile; then fallocate --length ${SWAP_GIBIBYTES} /swapfile; chmod 600 /swapfile; mkswap /swapfile; swapon /swapfile; fi`,
    'free',
    'cargo build --release --tests --locked --features test-only-switches --jobs 1',
  ]
    .join('\n',);
}

/**
 The script the guest runs to perform the arm and refuse a log that lacks a claim it made.

 @param name - the guest's name, which the arm's variable answers

 @returns the script, its lines in the order the guest runs them

 @example
 ```ts
 const arm = armScript({ name: 'zfs-r7-shutdown-unmount' });
 ```
 */
export function armScript({ name, }: { readonly name: string; },): string {
  return [
    'set -eu',
    `export PATH=${OPENZFS_PATH}:/root/.cargo/bin:$PATH`,
    ...armEnvironmentLines({ name, },),
    `cd ${CRATE_DIR}/package/git-policy/cli`,
    // A run that failed is the one whose log matters, so the log is copied to the shared folder before any assertion.
    'if cargo nextest run --release --locked --features test-only-switches --test native_binary --no-capture --status-level fail ' +
      `--filterset "test(the_shutdown_unmount_guest_arm)" > /root/${ARM_LOG} 2>&1; then status=0; else status=$?; fi`,
    `cp /root/${ARM_LOG} /mnt/shared/${ARM_LOG} || true`,
    `tail --lines=120 /root/${ARM_LOG}`,
    'test "$status" = 0',
    // A returning arm is not a passing arm.
    `if grep --quiet '^${SKIP_WORD}' /root/${ARM_LOG}; then echo 'the arm returned without running'; exit 1; fi`,
  ]
    .join('\n',);
}
