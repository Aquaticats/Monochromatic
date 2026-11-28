/**
 The two scripts the forced removal's guest arms run, and the names they share.

 Both scripts are the arms' own commands, kept as lines a person reads rather than as strings built at the call
 site: the build unpacks the wrapper and compiles it, and the arm runs one nextest filterset and requires every line the
 arms it selected promised. Every folder and variable they name lives here beside them, because a name used by one script
 and not the other is still one name.
 */
import {
  SWAP_GIBIBYTES,
} from '../bin/mvm-guest-bounds.ts';
import {
  shellQuote,
} from '../bin/mvm-guest-command-line.ts';

/**
 Where the wrapper's tree is unpacked and built inside the guest, which every account may traverse.

 A build that runs again in the same guest unpacks over this tree and keeps its `target` folder, so a plant costs a
 rebuild of the crate and not of every dependency.
 */
export const CRATE_DIR = '/opt/removal-force-guest';

/**
 The log the arms' own test output is kept in, inside the guest's shared folder and beside the host's logs.
 */
export const ARM_LOG = 'removal-force-arm.log';

/**
 The word a test binary prints at the start of a line when it returned without running its arm.
 */
export const SKIP_WORD = 'SKIPPED';

/**
 The OpenZFS 2.4.4 prefix the guest's recipe installs.
 */
const OPENZFS_PREFIX = '/opt/openzfs-2.4.4';

/**
 The `PATH` the guest's OpenZFS userspace needs.
 */
const OPENZFS_PATH = `${OPENZFS_PREFIX}/sbin:${OPENZFS_PREFIX}/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin`;

/**
 One arm's pool and the folder under the account's home whose file is its only vdev.
 */
type ArmPool = {
  /**
   The pool's name, which the arm that makes it names.
   */
  readonly pool: string;
  /**
   The folder under `/root` the pool's image and its checkout live in.
   */
  readonly root: string;
};

/**
 The pools the arms make, each from its own file, so a run that died leaves the next arm a machine it cannot see.
 */
export const THE_POOLS: readonly ArmPool[] = [
  {
    pool: 'removalforce1',
    root: 'removal-force-arm-1',
  },
  {
    pool: 'removalforce2',
    root: 'removal-force-arm-2',
  },
  {
    pool: 'removalforce3',
    root: 'removal-force-arm-3',
  },
  {
    pool: 'removalforce4',
    root: 'removal-force-arm-4',
  },
  {
    pool: 'removalforce5',
    root: 'removal-force-arm-5',
  },
  {
    pool: 'removalforce6',
    root: 'removal-force-arm-6',
  },
  {
    pool: 'removalforce7',
    root: 'removal-force-arm-7',
  },
  {
    pool: 'removalforce8',
    root: 'removal-force-arm-8',
  },
  {
    pool: 'removalforce9',
    root: 'removal-force-arm-9',
  },
  {
    pool: 'removalforce10',
    root: 'removal-force-arm-10',
  },
  {
    pool: 'removalforce11',
    root: 'removal-force-arm-11',
  },
];

/**
 The script the guest runs to unpack the wrapper and build the library and its test binaries.

 @param fresh - whether the tree is thrown away first, which a first build of a guest needs and a rebuild does not

 @returns the script, its lines in the order the guest runs them

 @example
 ```ts
 const build = buildScript({ fresh: false });
 ```
 */
export function buildScript({ fresh, }: { readonly fresh: boolean; },): string {
  return [
    'set -eu',
    // The shared copy is the tree this run made; a guest reused by an earlier run still holds that run's tarball at
    // /root, so the shared copy always wins and a missing one refuses rather than building the older tree.
    'test -f /mnt/shared/subject.tar.gz',
    'cp /mnt/shared/subject.tar.gz /root/subject.tar.gz',
    ...(fresh
      ? [
        `rm --recursive --force ${CRATE_DIR}`,
        `mkdir --parents ${CRATE_DIR}`,
      ]
      : [`mkdir --parents ${CRATE_DIR}`]),
    `tar --extract --gzip --file /root/subject.tar.gz --directory ${CRATE_DIR}`,
    `cd ${CRATE_DIR}/package/git-policy/cli`,
    'export PATH=/root/.cargo/bin:$PATH',
    'df --human-readable / /mnt/shared',
    // `free` with no option, because the guest's procps has no `--mebytes` and `set -e` would end the build there.
    'free',
    // One rustc for a crate this size wants more than five gigabytes, and the guest has eight: the first runs compiled
    // two at a time, both together exhausted the guest's memory, and the kernel killed one of them, which looked like
    // a compile failure. One job and a swap file are what make the build fit.
    `if ! swapon --show | grep --quiet /swapfile; then fallocate --length ${SWAP_GIBIBYTES} /swapfile; chmod 600 /swapfile; mkswap /swapfile; swapon /swapfile; fi`,
    'free',
    // Only the executable and the one test target the arms run: `--tests` builds every target of this crate, which is
    // minutes of linking a rebuilt plant does not need.
    'cargo build --release --locked --features test-only-switches --jobs 1 --test native_binary',
  ].join('\n',);
}

/**
 The script the guest runs to perform the selected arms, require every line they promised, and leave no pool behind.

 @param name - the guest's name, which the arm's variable answers

 @param filterset - the nextest filterset selecting the arms this run names

 @param markers - the lines those arms print when they performed what they claim

 @returns the script, its lines in the order the guest runs them

 @example
 ```ts
 const arm = armScript({ name: 'guest-a', filterset: 'test(a_persons_lock_arm)', markers: ['REMOVAL-FORCE arm=5'] });
 ```
 */
export function armScript({
  name,
  filterset,
  markers,
}: {
  readonly name: string;
  readonly filterset: string;
  readonly markers: readonly string[];
},): string {
  return [
    'set -eu',
    `export PATH=${OPENZFS_PATH}:/root/.cargo/bin:$PATH`,
    `export CLI_GIT_ZFS_REMOVAL_FORCE_GUEST=${name}`,
    `export CLI_GIT_ZFS_SWEEP_SBIN=${OPENZFS_PREFIX}/sbin`,
    `cd ${CRATE_DIR}/package/git-policy/cli`,
    // A run that failed is the one whose log matters, so the log is copied to the shared folder before any assertion,
    // and the failure's own output is printed rather than the last lines of a passing run.
    `if cargo nextest run --release --locked --features test-only-switches --test native_binary --no-capture --test-threads=1 --status-level fail --filterset ${shellQuote(filterset,)} > /root/${ARM_LOG} 2>&1; then status=0; else status=$?; fi`,
    `cp /root/${ARM_LOG} /mnt/shared/${ARM_LOG} || true`,
    `tail --lines=200 /root/${ARM_LOG}`,
    'test "$status" = 0',
    // A returning arm is not a passing arm, which every guest driver of this package holds.
    `if grep --quiet '^${SKIP_WORD}' /root/${ARM_LOG}; then echo 'an arm returned without running'; exit 1; fi`,
    ...markers.map(function markerLine(marker) {
      return `grep --quiet --fixed-strings ${shellQuote(marker,)} /root/${ARM_LOG}`;
    },),
    `cp /root/${ARM_LOG} /mnt/shared/${ARM_LOG}`,
    // The pools go even when an arm left one behind, so a reused guest is ready for another run.
    ...THE_POOLS.map(function poolLine(pool) {
      return `zpool destroy -f ${pool.pool} 2>/dev/null || true`;
    },),
    ...THE_POOLS.map(function imageLine(pool) {
      return `rm --force /root/${pool.root}/pool.img 2>/dev/null || true`;
    },),
  ].join('\n',);
}
