/**
 The two scripts the removal-lineage arm's guest runs, and the variables and pools they name.

 Both scripts are the arm's own commands, kept as lines a person reads rather than as strings built at the call site:
 the build unpacks the wrapper and compiles it, and the arm runs five nextest filtersets and then checks the log it
 wrote. Every pool, root and variable they name lives here beside them, because a name used by one script and not the
 other is still one name.
 */
import { shellQuote, } from './guest-driver-command-line.ts';

/**
 Where the wrapper's tree is unpacked and built inside the guest, which every account may traverse.
 */
export const CRATE_DIR = '/opt/removal-lineage-proof';

/**
 The log the arm's own test output is kept in, inside the guest's shared folder and beside the host's logs.
 */
export const ARM_LOG = 'removal-lineage-arm.log';

/**
 The word a test binary prints at the start of a line when it returned without running its arm.
 */
export const SKIP_WORD = 'SKIPPED';

/**
 The swap file the guest adds, in gibibytes, so one rustc at five gigabytes does not exhaust its memory.
 */
const SWAP_GIB = '6G';

/**
 The environment the arm reads, so it never opens the host's device.
 */
const GUEST_VARIABLE = 'CLI_GIT_ZFS_REMOVAL_LINEAGE_GUEST';

/**
 The pool step 14's arm builds and destroys.
 */
const POOL = 'zfsstep14';

/**
 The folder step 14's pool image and fixture repository live under.
 */
const FIXTURE_ROOT = '/home/zfs-step-14-fixture';

/**
 The environment step 15's destruction arm reads, so it never opens the host's device.
 */
const DESTRUCTION_GUEST_VARIABLE = 'CLI_GIT_ZFS_REMOVAL_DESTRUCTION_GUEST';

/**
 The pool step 15's destruction arm builds and destroys, so the two arms never share one.
 */
const DESTRUCTION_POOL = 'zfsstep15destruction';

/**
 The folder step 15's pool image and fixture repository live under.
 */
const DESTRUCTION_ROOT = '/home/zfs-step-15-destruction-fixture';

/**
 The environment step 16's whole-removal arm reads, so it never opens the host's device.
 */
const COMPLETION_GUEST_VARIABLE = 'CLI_GIT_ZFS_REMOVAL_COMPLETION_GUEST';

/**
 The pool step 16's whole-removal arm builds and destroys, so no two arms share one.
 */
const COMPLETION_POOL = 'zfsstep16completion';

/**
 The folder step 16's pool image and fixture repository live under.
 */
const COMPLETION_ROOT = '/home/zfs-step-16-completion-fixture';

/**
 The environment step 17's cancel arm reads, so it never opens the host's device.
 */
const CANCEL_GUEST_VARIABLE = 'CLI_GIT_ZFS_CANCEL_GUEST';

/**
 The pool step 17's cancel arm builds and destroys, so no two arms share one.
 */
const CANCEL_POOL = 'zfsstep17cancel';

/**
 The folder step 17's pool image and fixture repository live under.
 */
const CANCEL_ROOT = '/home/zfs-step-17-cancel-fixture';

/**
 The environment the finish pass's arm reads, so it never opens the host's device.
 */
const FINISH_GUEST_VARIABLE = 'CLI_GIT_ZFS_REMOVAL_FINISH_GUEST';

/**
 The pool the finish pass's arm builds and destroys, so no two arms share one.
 */
const FINISH_POOL = 'zfsstep16finish';

/**
 The folder the finish pass's pool image and fixture repository live under.
 */
const FINISH_ROOT = '/home/zfs-step-16-finish-fixture';

/**
 The environment the refusal arm reads, which drives the built wrapper's `git worktree remove`.
 */
const REFUSAL_GUEST_VARIABLE = 'CLI_GIT_ZFS_REMOVAL_REFUSAL_GUEST';

/**
 The pool the refusal arm builds its own datasets in, beside the proof arm's pool.
 */
const REFUSAL_POOL = 'removal-refusal';

/**
 The folder the refusal arm's file-backed pool image lives in.
 */
const REFUSAL_ROOT = '/home/removal-refusal-fixture';

/**
 The checkout the refusal arm registers a real worktree in.
 */
const REFUSAL_CHECKOUT = '/root/removal-refusal-checkout';

/**
 The OpenZFS 2.4.4 prefix the guest's recipe installs.
 */
const OPENZFS_PREFIX = '/opt/openzfs-2.4.4';

/**
 The `PATH` the guest's OpenZFS userspace needs.
 */
const OPENZFS_PATH = `${OPENZFS_PREFIX}/sbin:${OPENZFS_PREFIX}/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin`;

/**
 The lines a run must find in the guest's arm log, one per arm it performed.
 */
export const EXPECTED_MARKERS = [
  'REMOVAL healthy=entered',
  'REMOVAL promoted=refused',
  'REMOVAL reverse-promotion=proved',
  'REMOVAL bookmark-live=passed',
  'REMOVAL bookmark-stale=refused',
  'REMOVAL grants=lineage-changing-absent',
  'REMOVAL replacement=refused',
  'REMOVAL done',
  'REMOVAL-ARMS healthy=proved wrote=nothing',
  'REMOVAL-ARMS promoted=refused',
  'REMOVAL-ARMS inherited-tuple=measured',
  'REMOVAL-ARMS reverse-promotion=proved',
  'REMOVAL-ARMS bookmark-live=printed',
  'REMOVAL-ARMS bookmark-stale=refused',
  'REMOVAL-ARMS replacement=refused',
  'REMOVAL-ARMS done',
  'COMPLETION mounted=refused',
  'COMPLETION healthy=destroyed',
  'COMPLETION blocked=origin',
  'COMPLETION resume=finished',
  'COMPLETION done',
  'CANCEL destroyed=',
  'CANCEL already-cancelled=reported',
  'CANCEL done',
  'FINISH left=head-gone',
  'FINISH settled=registration',
  'FINISH done',
  'DESTRUCTION healthy=destroyed',
  'DESTRUCTION look=refused',
  'DESTRUCTION first-check=refused',
  'DESTRUCTION head-check=refused',
  'DESTRUCTION replacement=refused',
  'DESTRUCTION done',
] as const;

/**
 The variables the arm exports before it runs, one per arm, so each arm builds its own pools and reads none on this host.

 @param name - the guest's name, which every guest variable answers

 @returns the `export` lines the arm's script starts with, in the order the arms read them

 @example
 ```ts
 armEnvironmentLines({ name: 'guest-a' }); // ['export CLI_GIT_ZFS_REMOVAL_LINEAGE_GUEST=guest-a', ...]
 ```
 */
function armEnvironmentLines({ name, }: { readonly name: string; },): readonly string[] {
  return [
    `export ${GUEST_VARIABLE}=${name}`,
    `export CLI_GIT_ZFS_REMOVAL_LINEAGE_POOL=${POOL}`,
    `export CLI_GIT_ZFS_REMOVAL_LINEAGE_PREFIX=${OPENZFS_PREFIX}`,
    `export CLI_GIT_ZFS_REMOVAL_LINEAGE_ROOT=${FIXTURE_ROOT}`,
    `export ${DESTRUCTION_GUEST_VARIABLE}=${name}`,
    `export CLI_GIT_ZFS_REMOVAL_DESTRUCTION_POOL=${DESTRUCTION_POOL}`,
    `export CLI_GIT_ZFS_REMOVAL_DESTRUCTION_PREFIX=${OPENZFS_PREFIX}`,
    `export CLI_GIT_ZFS_REMOVAL_DESTRUCTION_ROOT=${DESTRUCTION_ROOT}`,
    `export ${COMPLETION_GUEST_VARIABLE}=${name}`,
    `export CLI_GIT_ZFS_REMOVAL_COMPLETION_POOL=${COMPLETION_POOL}`,
    `export CLI_GIT_ZFS_REMOVAL_COMPLETION_PREFIX=${OPENZFS_PREFIX}`,
    `export CLI_GIT_ZFS_REMOVAL_COMPLETION_ROOT=${COMPLETION_ROOT}`,
    `export ${CANCEL_GUEST_VARIABLE}=${name}`,
    `export CLI_GIT_ZFS_CANCEL_POOL=${CANCEL_POOL}`,
    `export CLI_GIT_ZFS_CANCEL_PREFIX=${OPENZFS_PREFIX}`,
    `export CLI_GIT_ZFS_CANCEL_ROOT=${CANCEL_ROOT}`,
    `export ${FINISH_GUEST_VARIABLE}=${name}`,
    `export CLI_GIT_ZFS_REMOVAL_FINISH_POOL=${FINISH_POOL}`,
    `export CLI_GIT_ZFS_REMOVAL_FINISH_PREFIX=${OPENZFS_PREFIX}`,
    `export CLI_GIT_ZFS_REMOVAL_FINISH_ROOT=${FINISH_ROOT}`,
    `export ${REFUSAL_GUEST_VARIABLE}=${name}`,
    `export CLI_GIT_ZFS_SWEEP_POOL=${REFUSAL_POOL}`,
    `export CLI_GIT_ZFS_SWEEP_SBIN=${OPENZFS_PREFIX}/sbin`,
    `export CLI_GIT_ZFS_SWEEP_CHECKOUT=${REFUSAL_CHECKOUT}`,
  ];
}

/**
 The script the guest runs to unpack the wrapper and build the library and its test binaries.

 The refusal arm's own pools are made here, from files, so nothing of the guest's root disk is a vdev.

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
    // The refusal arm's own pool, made from a file so nothing of the guest's root disk is a vdev.
    `mkdir --parents ${REFUSAL_ROOT}`,
    `mkdir --parents ${CANCEL_ROOT}`,
    `if ! ${OPENZFS_PREFIX}/sbin/zpool list -H -o name ${CANCEL_POOL} > /dev/null 2>&1; then`,
    `  truncate --size 1G ${CANCEL_ROOT}/pool.img`,
    `  ${OPENZFS_PREFIX}/sbin/zpool create -f -O mountpoint=none ${CANCEL_POOL} ${CANCEL_ROOT}/pool.img`,
    'fi',
    `mkdir --parents ${REFUSAL_CHECKOUT}`,
    `if ! ${OPENZFS_PREFIX}/sbin/zpool list -H -o name ${REFUSAL_POOL} > /dev/null 2>&1; then`,
    `  truncate --size 1G ${REFUSAL_ROOT}/pool.img`,
    `  ${OPENZFS_PREFIX}/sbin/zpool create -f -O mountpoint=none ${REFUSAL_POOL} ${REFUSAL_ROOT}/pool.img`,
    'fi',
    `cd ${CRATE_DIR}/package/git-policy/cli`,
    'export PATH=/root/.cargo/bin:$PATH',
    'df --human-readable / /mnt/shared',
    // `free` with no option, because the guest's procps has no `--mebytes` and `set -e` would end the build there.
    'free',
    // One rustc for a crate this size wants more than five gigabytes, and the guest has eight: the first runs compiled
    // two at a time, both together exhausted the guest's memory, and the kernel killed one of them, which looked like
    // a compile failure. One job and a swap file are what make the build fit.
    `if ! swapon --show | grep --quiet /swapfile; then fallocate --length ${SWAP_GIB} /swapfile; chmod 600 /swapfile; mkswap /swapfile; swapon /swapfile; fi`,
    'free',
    'cargo build --release --tests --locked --features test-only-switches --jobs 1',
  ].join('\n',);
}

/**
 The script the guest runs to perform every arm, refuse a log that lacks a claim it made, and leave no pool behind.

 @param name - the guest's name, which every guest variable answers

 @returns the script, its lines in the order the guest runs them

 @example
 ```ts
 const arm = armScript({ name: 'guest-a' });
 ```
 */
export function armScript({ name, }: { readonly name: string; },): string {
  return [
    'set -eu',
    `export PATH=${OPENZFS_PATH}:/root/.cargo/bin:$PATH`,
    ...armEnvironmentLines({ name, },),
    `cd ${CRATE_DIR}/package/git-policy/cli`,
    // A run that failed is the one whose log matters, so the log is copied to the shared folder before any assertion,
    // and the failure's own output is printed rather than the last forty lines of a passing run.
    'if cargo nextest run --release --locked --features test-only-switches --lib --run-ignored all --no-capture --status-level fail ' +
      '--filterset "test(the_removal_lineage_guest_arms) | test(the_removal_destruction_guest_arms) | test(the_removal_completion_guest_arms) | test(the_cancel_command_guest_arms) | test(the_removal_finish_pass_guest_arms)" '
      +
      `> /root/${ARM_LOG} 2>&1; then status=0; else status=$?; fi`,
    `cp /root/${ARM_LOG} /mnt/shared/${ARM_LOG} || true`,
    `tail --lines=120 /root/${ARM_LOG}`,
    // The refusal arms run through the built wrapper's own `git worktree remove`, in the same log.
    'if cargo nextest run --release --locked --features test-only-switches --test native_binary --no-capture --status-level fail ' +
      `--filterset "test(zfs_removal_refusal_guest)" >> /root/${ARM_LOG} 2>&1; then refusal=0; else refusal=$?; fi`,
    `cp /root/${ARM_LOG} /mnt/shared/${ARM_LOG} || true`,
    `tail --lines=60 /root/${ARM_LOG}`,
    'test "$status" = 0',
    'test "$refusal" = 0',
    // A returning arm is not a passing arm, which the binding's own live recipe also holds.
    `if grep --quiet '^${SKIP_WORD}' /root/${ARM_LOG}; then echo 'the arm returned without running'; exit 1; fi`,
    ...EXPECTED_MARKERS.map(function markerLine(marker) {
      return `grep --quiet --fixed-strings ${shellQuote({ script: marker, },)} /root/${ARM_LOG}`;
    },),
    `cp /root/${ARM_LOG} /mnt/shared/${ARM_LOG}`,
    // The pool goes even when the arm left it behind, so the guest is ready for another run.
    `zpool destroy -f ${POOL} 2>/dev/null || true`,
    `rm --force ${FIXTURE_ROOT}/pool.img 2>/dev/null || true`,
    `zpool destroy -f ${DESTRUCTION_POOL} 2>/dev/null || true`,
    `rm --force ${DESTRUCTION_ROOT}/pool.img 2>/dev/null || true`,
    `zpool destroy -f ${COMPLETION_POOL} 2>/dev/null || true`,
    `rm --force ${COMPLETION_ROOT}/pool.img 2>/dev/null || true`,
    `rmdir ${COMPLETION_ROOT} 2>/dev/null || true`,
    `zpool destroy -f ${CANCEL_POOL} 2>/dev/null || true`,
    `rm --force ${CANCEL_ROOT}/pool.img 2>/dev/null || true`,
    `zpool destroy -f ${FINISH_POOL} 2>/dev/null || true`,
    `rm --force ${FINISH_ROOT}/pool.img 2>/dev/null || true`,
    `rmdir ${FINISH_ROOT} 2>/dev/null || true`,
  ].join('\n',);
}
