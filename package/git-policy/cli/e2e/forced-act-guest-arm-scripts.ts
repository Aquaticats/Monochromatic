/**
 The two guest scripts the forced act's driver starts detached: the crate's build, and the arm itself.

 Both outlive one `mvm exec`: the build takes tens of minutes under this host's load, and the arm's seven earlier phases
 plus the locked holder's stop take minutes of their own, so a plain `mvm exec` loses their result when the guest agent
 misses one status request (`doc/troubleshooting/mvm-guest-agent-forgets-a-long-commands-result.md`).
 They live in a file of their own so the driver's file stays inside the linter's line budget, and so the arm can be run
 again without its build.

 The guest's name check is `none` here: the arm itself refuses a machine whose host name is not the guest's
 (`Fixture::from_environment`), and the build only ever touches the guest's own tree.
 */

/**
 The folder the wrapper's tree is unpacked and built in, which every account may traverse.
 */
export const CRATE_DIR = '/opt/forced-act';

/**
 The pool the arm builds and destroys.
 */
export const POOL = 'forcedact';

/**
 The folder the pool's image lives under.
 */
export const FIXTURE_ROOT = '/home/forced-act-fixture';

/**
 The OpenZFS 2.4.4 prefix the guest's recipe installs.
 */
export const OPENZFS_PREFIX = '/opt/openzfs-2.4.4';

/**
 The `PATH` the guest's OpenZFS userspace needs.
 */
const OPENZFS_PATH = `${OPENZFS_PREFIX}/sbin:${OPENZFS_PREFIX}/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin`;

/**
 The environment variable naming the guest the arm may run in.
 */
const GUEST_VARIABLE = 'CLI_GIT_ZFS_FORCED_ACT_GUEST';

/**
 The arm's own log inside the guest, which the run pulls back as its evidence.
 */
export const ARM_LOG = '/root/forced-act-arm.log';

/**
 The guest's own script: unpack the pushed archive and build this crate's library and its tests.

 The dependency build outlives one run, so it is moved out of the tree this run replaces and back again: a second run
 then compiles this crate's own test binary alone. The swap file is made before the build because one `rustc` of this
 crate's tests takes 6.7 to 6.9 GiB of the guest's 8 GiB
 (`doc/planning/mvm-guest-memory-held-for-a-rounds-life.md`).

 @returns the script, as the shell lines the guest runs

 @example
 ```ts
 await startDetachedGuestScript({ chosen, script: buildScript(), log: 'build.log', timeoutMilliseconds: 2_400_000, nameCheck: 'none' });
 ```
 */
export function buildScript(): string {
  return [
    'set -eu',
    // `mvm push` lands the tarball on the shared folder, so the guest copies it where the build reads it, over whatever
    // an earlier run left there: a kept copy from an earlier run is a stale tree, not this run's.
    'cp --force /mnt/shared/subject.tar.gz /root/subject.tar.gz',
    `if test -d ${CRATE_DIR}/package/git-policy/cli/target; then rm --recursive --force /root/target-kept; mv ${CRATE_DIR}/package/git-policy/cli/target /root/target-kept; fi`,
    `rm --recursive --force ${CRATE_DIR}`,
    `mkdir --parents ${CRATE_DIR}`,
    `tar --extract --gzip --file /root/subject.tar.gz --directory ${CRATE_DIR}`,
    `if test -d /root/target-kept; then mv /root/target-kept ${CRATE_DIR}/package/git-policy/cli/target; fi`,
    `cd ${CRATE_DIR}/package/git-policy/cli`,
    'export PATH=/root/.cargo/bin:$PATH',
    // A swap file traded the guest's memory for the build's speed and lost: with 6 GiB of swap the crate's own compile
    // ran past the 90-minute bound on 2026-11-28T03:36 while the guest thrashed, and the three builds that finished in
    // 29m54s each on 2026-11-27 ran without one. So any swap an earlier run left is turned off here.
    'swapoff /swapfile 2>/dev/null || true',
    'rm --force /swapfile 2>/dev/null || true',
    // One job, for the same reason the deferred-origin driver runs one: this build is the library's test binary, and
    // one `rustc` of it takes the guest's whole memory. Measured 2026-11-27 in the guest of issue 6612's round at two
    // jobs, the build's own samples read `AnonPages` 7,249,568 kB of the guest's 8,131,780 kB with one `rustc` at
    // 7,418,336 kB resident and `MemAvailable` at 398,408 kB at its lowest.
    'cargo test --release --lib --no-run --locked -j 1 > /root/forced-act-build.log 2>&1',
    'tail --lines=3 /root/forced-act-build.log',
  ]
    .join('\n',);
}

/**
 The guest's own script: run the arm, refuse a returned one, and copy its log to the shared folder.

 The account's user manager is started before the arm and stopped after it, because the locked holder's phase stops a
 unit of that manager and a guest's account is never logged in.

 @param name - the guest's name, which the arm's own environment names

 @param markers - the `FORCEDACT` lines the run must find in the arm's log, one per phase

 @returns the script, as the shell lines the guest runs

 @example
 ```ts
 await startDetachedGuestScript({ chosen, script: armScript({ name: 'forced-act', markers }), log: 'arm.log', timeoutMilliseconds: 900_000, nameCheck: 'none' });
 ```
 */
export function armScript({
  name,
  markers,
}: {
  readonly name: string;
  readonly markers: readonly string[];
},): string {
  return [
    'set -eu',
    `export PATH=${OPENZFS_PATH}:/root/.cargo/bin:$PATH`,
    `export ${GUEST_VARIABLE}=${name}`,
    `export CLI_GIT_ZFS_FORCED_ACT_POOL=${POOL}`,
    `export CLI_GIT_ZFS_FORCED_ACT_PREFIX=${OPENZFS_PREFIX}`,
    `export CLI_GIT_ZFS_FORCED_ACT_ROOT=${FIXTURE_ROOT}`,
    // A run that was killed leaves its pool and the checkout mount this arm's phase makes in the guest's root
    // namespace, and a pool that exists refuses the next run's own.
    `nsenter --target 1 --mount -- umount ${FIXTURE_ROOT}/checkout 2>/dev/null || true`,
    `zpool destroy -f ${POOL} 2>/dev/null || true`,
    'loginctl enable-linger root',
    'for look in $(seq 1 120); do if systemctl is-active --quiet user@0.service; then break; fi; sleep 0.5; done',
    'systemctl is-active user@0.service',
    `cd ${CRATE_DIR}/package/git-policy/cli`,
    `if cargo nextest run --release --locked --lib --run-ignored all --no-capture --status-level all --filterset "test(the_forced_act_guest_arm_tests)" > ${ARM_LOG} 2>&1; then status=0; else status=$?; fi`,
    `tail --lines=60 ${ARM_LOG}`,
    'test "$status" = 0',
    // A returning arm is not a passing arm, which the binding's own live recipe also holds.
    `if grep --quiet '^SKIPPED' ${ARM_LOG}; then echo 'the arm returned without running'; exit 1; fi`,
    ...markers
      .map(function markerLine(marker: string,): string {
        return `grep --quiet --fixed-strings ${shellQuote(marker,)} ${ARM_LOG}`;
      },),
    `cp ${ARM_LOG} /mnt/shared/forced-act-arm.log`,
    'loginctl disable-linger root',
    // The pool goes even when the arm left it behind, so the guest is ready for another run.
    `zpool destroy -f ${POOL} 2>/dev/null || true`,
    `rm --force ${FIXTURE_ROOT}/pool.img 2>/dev/null || true`,
  ]
    .join('\n',);
}

/**
 Quote one word for the guest's shell.

 A marker holds spaces and `=`, so the word the guest's `grep` reads must be one word.

 @param word - the text to quote

 @returns the word, quoted for `sh`

 @example
 ```ts
 shellWord('FORCEDACT phase=arm-finished'); // 'FORCEDACT phase=arm-finished'
 ```
 */
function shellQuote(word: string,): string {
  return `'${word.replaceAll(
    "'",
    `'\\''`,
  )}'`;
}
