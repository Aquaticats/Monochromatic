/**
 The stages of `cutover`: the window (freeze, final pass, verification, mountpoint guard, swap, identity record,
 release). It starts only when the coordinator has said no agent works in any worktree, and it has a stated maximum
 duration that it does not exceed.

 @module
 */

import { containerLimitArguments, } from '@monochromatic-dev/container-limit-core/ts';

import { EMPTY_SUBVOLUME_PLACEHOLDER_INODE, } from './constants.ts';
import { realArguments, } from './guided-prepare-stages.data.ts';
import type {
  GuidedConfig,
  Stage,
} from './guided-types.ts';
import {
  hookHoldersBefore,
  hookHoldersFrozen,
  hookSelectVerify,
  hookWindowMaximum,
} from './guided-stage-hooks.ts';
import { externalNames, } from './guided-holders.ts';

/**
 Bounds of the container that checks the bind through the link: one `test -d` command, so 256 MiB, one CPU and 64
 processes, with the lowest disk weight added by the shared helper.
 */
const CONTAINER_BIND_LIMITS = {
  memory: '256m',
  cpus: 1,
  pidsLimit: 64,
};

/**
 The limit options of that container as the shell command spells them, the lowest disk weight included.
 */
const CONTAINER_BIND_LIMIT_OPTIONS = containerLimitArguments(CONTAINER_BIND_LIMITS,)
  .join(' ',);

/**
 The unit name patterns of the commit services, quoted for the shell: the earlier `cli-git@` units, whose sockets
 stay listening after the rename until their helpers are replaced, and the renamed `monochromatic-git-policy-cli@`
 units. The window stops both and checks that neither is active, so no helper writes into the tree while it freezes.
 */
const COMMIT_SERVICE_UNIT_PATTERNS = [
  "'cli-git@*'",
  "'monochromatic-git-policy-cli@*'",
].join(' ',);

/**
 The stages of `cutover`.

 @param c - the configuration
 
 @param stamp - the timestamp that names the aside folders of the swap
 
 @returns the stages in order

 @example
 ```ts
 import { cutoverStages, } from '\@monochromatic-dev/repository-move-cli';
 
 const result = await cutoverStages({ c, stamp, },);
 ```
 */
export async function cutoverStages(
  {
    c,
    stamp,
  }: {
    readonly c: GuidedConfig;
    readonly stamp: string;
  },
): Promise<readonly Stage[]> {
  /**
   The external worktree folder names.
   */
  const names = (await externalNames(c,)).join(' ',);
  /**
   The inventory taken at the start of the cut-over.
   */
  const finalInventory = `${c.wiz}/inventory-final.txt`;
  /**
   The arguments shared by the commands of this step.
   */
  const args = realArguments({
    c,
    inventory: finalInventory,
  },);
  /**
   The copy tool command, run as the user.
   */
  const copy = `${c.nodePath} ${c.copyTool}`;
  /**
   The copy tool command, run through sudo with the absolute path of node.
   */
  const rootCopy = `sudo ${c.nodePath} ${c.copyTool}`;
  /**
   The scan tool command.
   */
  const scan = `${c.nodePath} ${c.scanTool}`;
  /**
   The installed link check command.
   */
  const link = `${c.nodePath} ${c.linkBinFolder}`;
  /**
   The prefix that runs a command as the user.
   */
  const userRun = `runuser -u ${c.userName} -- env HOME=${c.homeDir}`;
  /**
   Where the old checkout is renamed to at the swap.
   */
  const oldRepo = `${c.repo}.nvme-before-860-${stamp}`;
  /**
   Where the old external worktrees folder is renamed to at the swap.
   */
  const oldExternal = `${c.externalWorktrees}.nvme-before-860-${stamp}`;
  /**
   Seconds between the terminate signal and the kill of a timed-out step.
   */
  const killGrace = String(c.killGraceSeconds,);
  /**
   The deadline of the whole scan, in seconds.
   */
  const scanTime = String(c.scanTimeoutSeconds,);
  /**
   The deadline of one worktree scan, in seconds.
   */
  const perScan = String(c.scanPerWorktreeSeconds,);
  /**
   The deadline of one footprint measurement, in seconds.
   */
  const measure = String(c.measureTimeoutSeconds,);
  /**
   The sharing limit in percent.
   */
  const sharing = String(c.sharingLimitPercent,);
  /**
   The mount point below `/var`, as the hidden path sees it.
   */
  const underVar = c.drive
    .startsWith('/var/',) ? c.drive
      .slice('/var'.length,) : c.drive;
  /**
   The rollback text shared by the window stages.
   */
  const thawRollback = 'run the thaw command (unfreezes everything and restarts the stopped units)';
  /**
   The repository as kopia's source names it: kopia recorded the source as `/home/user/Monochromatic`
   (plan item W12), the other spelling of the same folder.
   */
  const kopiaSource = c.repo
    .startsWith('/var/home/',) ? c.repo
      .slice('/var'.length,) : c.repo;
  /**
   The fresh worktree the pnpm check installs into.
   */
  const freshWorktree = `${c.repo}/.claude/worktrees/move-check-${stamp}`;
  /**
   The seconds the checks after the cut-over may take: the author's bound, the transfer deadline's order.
   */
  const afterSeconds = c.fixtureStepTimeoutSeconds;
  return [
    {
      id: 'sudo-cutover',
      title: 'Root access',
      says: 'The root steps use sudo, with the absolute path of node. This checks that sudo works without a prompt.',
      command: 'sudo --non-interactive true',
      check: 'sudo --non-interactive true',
      checkText: 'sudo works without a prompt',
      noMark: true,
      root: 'yes',
      unprivileged: 'none: this stage only proves the root route',
    },
    {
      id: 'c00-checks',
      title: 'Pre-window checks',
      says: 'Pre-window checks, nothing changes: the checkout is a real writable folder, the 860 is mounted, and no half-done cut-over left objects that would collide.',
      command: `test -d ${c.repo} && test ! -L ${c.repo}
test "$(btrfs property get -ts ${c.repo} ro)" = ro=false
mountpoint --quiet ${c.drive}
test ! -e ${c.finalPath}
test -z "$(find ${c.snapshots} ${c.received} -maxdepth 1 -name 'f[0-9]*' -print)"
echo checks passed`,
      check: `grep --fixed-strings 'checks passed' "$OUTPUT_FILE"`,
      checkText: 'all checks passed',
      rollback: 'read-only. A leftover from an aborted attempt (f### snapshots, received copies or the final folder) must be deleted by you first: the thaw command prints the list',
      root: 'no',
      unprivileged: 'not needed: property get and the folder tests run as the user',
    },
    {
      id: 'c01-pre-freeze-table',
      title: 'Table before the freeze',
      says: 'Table of every worktree before the freeze, timed. Any row with an unfinished merge, rebase, cherry-pick, revert, bisect or a held index.lock must be settled by its owner before the freeze (plan, section Quiesce and freeze). The time measured here feeds the window maximum.',
      command: `systemd-run ${c.userScope} timeout --kill-after=${killGrace} ${scanTime} ${scan} ${c.repo} ${c.wiz}/pre-freeze.tsv ${perScan}
/usr/bin/git --no-optional-locks -C ${c.repo} worktree list --porcelain > ${c.wiz}/porcelain-start.txt`,
      check: `awk -F'\\t' 'NR > 1 && ($2 != "ok" || $5 != "" || $6 == "true") { print "settle first: " $1 " (" $2 " " $5 " lock=" $6 ")"; bad = 1 } END { exit bad }' ${c.wiz}/pre-freeze.tsv`,
      checkText: 'no worktree is missing, failing, mid-operation or holding index.lock',
      rollback: 'read-only',
      timer: {
        file: 'scan-seconds',
        mode: 'set',
      },
      root: 'no',
      unprivileged: 'not needed: git status of the user\'s own worktrees runs as the user',
    },
    {
      id: 'c02-window-maximum',
      title: 'Window maximum',
      says: 'The stated maximum of the cut-over window, from numbers this runner measured: first pass + a content-check allowance (one more first-pass duration, the author\'s choice) + 2 x scan + 2 x footprint. You may shorten it with MOVE_WINDOW_MAX_SECONDS, never lengthen it.',
      command: 'true',
      internal: hookWindowMaximum,
      check: `test -s ${c.wiz}/window-max-seconds`,
      checkText: 'the maximum is recorded',
      rollback: 'nothing started; the clock starts only at the step that stops the units',
      root: 'no',
      unprivileged: 'not needed: arithmetic on files',
    },
    {
      id: 'c03-holders-before',
      title: 'Holder scan before the freeze',
      says: `Holder scan before the freeze: any process whose working folder or open file lies in the repository, in ${c.externalWorktrees}, or in a worktree elsewhere. Run as root so rootless containers are seen. Nothing is signalled.`,
      command: hookHoldersBefore,
      okCodes: [
        0,
        1,
      ],
      check: `echo "scanner messages: $(wc --lines < ${c.wiz}/holders-before.err)"; test ! -s ${c.wiz}/holders-before.txt || { echo 'holders found:'; cat ${c.wiz}/holders-before.txt; exit 1; }`,
      checkText: `no holder (find may exit 1 for processes that vanish while scanning; ${c.wiz}/holders-before.err keeps its messages)`,
      rollback: 'nothing changed. Ask each session to finish, then run the cut-over again',
      root: 'yes',
      unprivileged: 'a scan as the user sees only the user\'s own processes; rootless containers and other users\' processes need root, which is why this scan is root',
    },
    {
      id: 'c04-wrapper-state',
      title: 'Wrapper state',
      says: 'Wrapper state: no commit in progress (busy/) and no planned push (push-retries/) for any commit service instance, in the state folder of the earlier cli-git releases and in that of the renamed wrapper; the pending transactions in .git are listed (they move unchanged with .git).',
      command: `for folder in ${c.earlierWrapperState} ${c.wrapperState}; do if test -d "$folder"; then /usr/bin/find "$folder" \\( -path '*/busy/*' -o -path '*/push-retries/*' \\) -type f -print; fi; done > ${c.wiz}/wrapper-busy.txt
compgen -G '${c.repo}/.git/cli-git-transactions/*.pending' || echo 'no pending transactions'`,
      check: `test ! -s ${c.wiz}/wrapper-busy.txt || { cat ${c.wiz}/wrapper-busy.txt; echo 'push the branches of these plans by hand, then run again'; exit 1; }`,
      checkText: 'busy/ and push-retries/ are empty everywhere',
      rollback: 'read-only',
      root: 'no',
      unprivileged: 'not needed: the state folders belong to the user',
    },
    {
      id: 'c05-fresh-inventory',
      title: 'Fresh inventory and freeze list',
      says: "R0 again, because worktrees come and go since the first inventory. The copy tool's print mode then reconciles it with Git and prints the freeze lines it will run.",
      command: `sudo btrfs subvolume list -u -q /var/home > ${finalInventory}
${copy} --pass=freeze ${args} > ${c.wiz}/freeze-print.txt
${copy} --pass=final ${args} > ${c.wiz}/final-pass-print.txt
echo "subvolumes to freeze: $(grep --count '^btrfs property set -ts ' ${c.wiz}/freeze-print.txt)"`,
      check: `grep --extended-regexp 'inventory agrees' "$OUTPUT_FILE" && test "$(grep --count '^btrfs property set -ts ' ${c.wiz}/freeze-print.txt)" -ge 1 && test "$(grep --count '^btrfs property set -ts ' ${c.wiz}/freeze-print.txt)" = "$(grep 'btrfs send' ${c.wiz}/final-pass-print.txt | grep --count --invert-match --fixed-strings 'btrfs send --no-data')"`,
      checkText: 'the inventory agrees with Git and one freeze line exists per subvolume to send',
      rollback: 'read-only. On a refusal give the named subvolumes a written disposition and start the cut-over again',
      root: 'yes',
      unprivileged: 'none for the list (root-only); the two print runs are unprivileged',
    },
    {
      id: 'c06-stop-units',
      title: 'Window starts: stop the user units',
      says: 'THE WINDOW STARTS AT THIS STEP. It stops the repository\'s user units, so commits fail from here. Freeze, final pass, verification and swap must finish inside the stated maximum or the runner stops and tells you to thaw.',
      assertion: 'No agent is working in any worktree, and commits may be refused until the window ends.',
      windowStart: true,
      command: `${ // A rerun after a partial stop merges into the list instead of replacing it, so release and thaw still restart what the first attempt stopped.
        ''}touch ${c.wiz}/stopped-units.txt
{ cat ${c.wiz}/stopped-units.txt; systemctl --user list-units --plain --no-legend --all ${COMMIT_SERVICE_UNIT_PATTERNS} | awk '{ print $1 }'; if systemctl --user is-active --quiet claude-cpu-watch.service; then echo claude-cpu-watch.service; fi; } | sort --unique > ${c.wiz}/stopped-units.txt.new
mv ${c.wiz}/stopped-units.txt.new ${c.wiz}/stopped-units.txt
${ // One call per unit, because systemctl refuses a whole list when any unit in it is not loaded. A listed unit that is no longer loaded fails its own stop; the check decides whether anything still runs.
        ''}xargs --arg-file=${c.wiz}/stopped-units.txt --no-run-if-empty --max-args=1 systemctl --user stop || echo 'a listed unit could not be stopped (one no longer loaded fails this way); the check reads what still runs'`,
      check: `${ // Each query's own failure fails the check, so a manager that cannot be reached never reads as "stopped".
        ''}running="$(systemctl --user list-units --plain --no-legend --state=active,activating,deactivating,reloading ${COMMIT_SERVICE_UNIT_PATTERNS})" && test -z "$running" && watch="$(systemctl --user show --property=ActiveState --value claude-cpu-watch.service)" && test "$watch" != active && test "$watch" != activating`,
      checkText: 'no commit service is active, starting or stopping, and claude-cpu-watch.service is stopped',
      rollback: 'the thaw command restarts claude-cpu-watch.service; commit services start again on the next commit',
      root: 'no',
      unprivileged: 'not needed: user units stop as the user',
    },
    {
      id: 'c07-freeze',
      title: 'Freeze',
      window: true,
      says: `FREEZE: ro=true on the checkout and every nested subvolume (the tool's freeze pass, run as the user, with its manifest), then a read-only bind mount over each external worktree (root). A write through a descriptor opened before this still succeeds (measured in the plan), which is why the next stage scans again.`,
      command: `${copy} --run --pass=freeze ${args} --total-deadline-seconds=@REMAINING@
for name in ${names}; do
  grep --fixed-strings --line-regexp --quiet ${c.externalWorktrees}/$name ${c.wiz}/ext-binds.txt 2>/dev/null || echo ${c.externalWorktrees}/$name >> ${c.wiz}/ext-binds.txt
  mountpoint --quiet ${c.externalWorktrees}/$name || { sudo mount --bind ${c.externalWorktrees}/$name ${c.externalWorktrees}/$name && sudo mount -o remount,ro,bind ${c.externalWorktrees}/$name; }
done`,
      check: `grep --fixed-strings 'freeze finished' "$OUTPUT_FILE" && touch ${c.wiz}/ext-binds.txt && while read -r p; do findmnt --noheadings --output OPTIONS --mountpoint "$p" | grep --quiet -E '(^|,)ro(,|$)' || { echo "$p not read-only"; exit 1; }; done < ${c.wiz}/ext-binds.txt`,
      checkText: 'every subvolume was frozen and read back ro=true by the tool, and every external folder is a read-only mount',
      rollback: thawRollback,
      root: 'yes',
      unprivileged: 'the subvolume freeze runs as the user (the owner of a subvolume sets ro on it without root, measured in the plan); only the read-only bind mounts over the external worktrees need root, and a subvolume the user does not own makes the freeze stop and name it',
    },
    {
      id: 'c08-holders-frozen',
      title: 'Holder scan after the freeze',
      window: true,
      says: 'Holder scan after the freeze. A holder now has a descriptor open that the freeze cannot stop: thaw and report it.',
      command: hookHoldersFrozen,
      okCodes: [
        0,
        1,
      ],
      check: `echo "scanner messages: $(wc --lines < ${c.wiz}/holders-frozen.err)"; test ! -s ${c.wiz}/holders-frozen.txt || { echo 'holders found:'; cat ${c.wiz}/holders-frozen.txt; echo 'run the thaw command'; exit 1; }`,
      checkText: 'no holder',
      rollback: thawRollback,
      root: 'yes',
      unprivileged: 'see the scan before the freeze: only root sees every process',
    },
    {
      id: 'c09-baseline',
      title: 'Baseline of the frozen tree',
      window: true,
      says: 'Baseline of the frozen tree: the table of every worktree (registration, HEAD, index hash, change counts, unfinished operations) and the hash of all refs. Taken after the freeze, because values taken earlier legitimately differ.',
      command: `systemd-run ${c.userScope} timeout --kill-after=${killGrace} ${scanTime} ${scan} ${c.repo} ${c.wiz}/baseline-frozen.tsv ${perScan}
/usr/bin/git --no-optional-locks -C ${c.repo} for-each-ref | sha256sum | cut --delimiter=' ' --fields=1 > ${c.wiz}/frozen-refs.sha
/usr/bin/git --no-optional-locks -C ${c.repo} worktree list --porcelain > ${c.wiz}/porcelain-frozen.txt`,
      check: `awk -F'\\t' 'NR > 1 && ($2 != "ok" || $5 != "" || $6 == "true") { print "unsettled: " $1; bad = 1 } END { exit bad }' ${c.wiz}/baseline-frozen.tsv && test -s ${c.wiz}/frozen-refs.sha`,
      checkText: 'no unsettled row and the refs hash exists',
      rollback: thawRollback,
      root: 'no',
      unprivileged: 'not needed: read-only Git as the user',
    },
    {
      id: 'c10-final-pass',
      title: 'Final pass',
      window: true,
      says: `FINAL PASS: new read-only snapshots, incremental sends against the first-pass copies, and each copy placed as a writable subvolume at its final path. The tool refuses to run unless every subvolume it sends is in the freeze manifest and still reads ro=true. Deadline per transfer: the smaller of ${String(c.transferDeadlineSeconds,)} s and what remains of the window; total deadline: what remains of the window.`,
      command: `${rootCopy} --run --pass=final ${args} --deadline-seconds=@DEADLINE@ --total-deadline-seconds=@REMAINING@`,
      check: `grep --fixed-strings 'pass final finished' "$OUTPUT_FILE" && test "$(ls ${c.moveState} | grep --count '^f[0-9]*\\.placed\\.json$')" = "$(ls ${c.moveState} | grep --count '^f[0-9]*\\.snapshot\\.json$')" && test "$(btrfs property get -ts ${c.finalPath} ro)" = ro=false`,
      checkText: "'pass final finished', a placed record for every final snapshot, and the placed checkout is writable",
      rollback: `${thawRollback}. The old tree is untouched; the partial 860 copy stays for you to delete (thaw prints the list)`,
      root: 'yes',
      unprivileged: 'none for send and receive',
    },
    {
      id: 'c10b-verify-final',
      title: 'Verify the final copies that changed',
      window: true,
      says: 'Verification of the final pass: each final copy against its read-only snapshot by path, type, size, time and mode (rsync --dry-run, no data read), only for the copies btrfs found changed since the first pass (its own incremental comparison of the two snapshots, recorded by the final pass) and for any copy that has no first-pass copy. The nested subvolume folders a snapshot shows empty are the only excused differences.',
      command: `${rootCopy} --run --pass=verify-final ${args} --deadline-seconds=@DEADLINE@ --total-deadline-seconds=@REMAINING@`,
      check: `grep --fixed-strings 'verify-final finished' "$OUTPUT_FILE"`,
      checkText: "'verify-final finished': every changed final copy matches its snapshot",
      rollback: thawRollback,
      root: 'yes',
      unprivileged: 'none: the comparison reads files only root can read (root-owned logs inside some worktrees)',
    },
    {
      id: 'c11-external-final',
      title: 'External worktrees: pass two',
      window: true,
      says: 'External worktrees, pass two: the same rsync again, now from the read-only mounts. It deletes nothing from the copy; a dry-run rsync --delete then reports every file that was deleted at the source after pass one (saved per folder under the state folder as external-<name>.deleted.txt), and the copy keeps those files.',
      command: `${rootCopy} --run --pass=external-final ${args} --external-source=${c.externalWorktrees} --external-final=${c.externalFinal} --external-deadline-seconds=@DEADLINE@ --total-deadline-seconds=@REMAINING@
for f in ${c.moveState}/external-*.deleted.txt; do [ -e "$f" ] && echo "$f: $(wc --lines < "$f") files deleted at the source after pass one"; done || true`,
      check: `grep --fixed-strings 'pass external-final finished' "$OUTPUT_FILE"`,
      checkText: 'the second pass finished and wrote its deletion reports',
      rollback: thawRollback,
      root: 'yes',
      unprivileged: 'as for pass one: a user rsync works for what the user can read; root reads the rest',
    },
    {
      id: 'c12-verify-staged',
      title: 'Verify 1-3: fsck, table, refs',
      window: true,
      says: 'Verification 1 to 3, in a private mount namespace that binds the copy over the real paths, so every .git file and gitdir file resolves into the copy. Git and the scan run as you. 1: git fsck --full. 2: the table equals the frozen baseline byte for byte. 3: the refs hash equals the frozen one.',
      command: `sudo unshare --mount --propagation private bash --noprofile --norc -euo pipefail -c "
  mount --bind ${c.finalPath} ${c.repo}
  mount --bind ${c.externalFinal} ${c.externalWorktrees}
  ${userRun} /usr/bin/git --no-optional-locks -C ${c.repo} fsck --full
  ${userRun} ${scan} ${c.repo} ${c.wiz}/staged.tsv ${perScan}
  ${userRun} /usr/bin/git --no-optional-locks -C ${c.repo} for-each-ref | sha256sum | cut --delimiter=' ' --fields=1 > ${c.wiz}/staged-refs.sha
  ${userRun} /usr/bin/git --no-optional-locks -C ${c.repo} worktree list --porcelain > ${c.wiz}/porcelain-staged.txt
"`,
      check: `cmp ${c.wiz}/baseline-frozen.tsv ${c.wiz}/staged.tsv && cmp ${c.wiz}/frozen-refs.sha ${c.wiz}/staged-refs.sha && cmp ${c.wiz}/porcelain-frozen.txt ${c.wiz}/porcelain-staged.txt`,
      checkText: 'staged table, refs hash and worktree list are byte for byte equal to the frozen ones',
      rollback: `${thawRollback} (the namespace vanished with the command)`,
      root: 'yes',
      unprivileged: 'none: a bind mount in the initial mount namespace needs root; a user namespace cannot bind a btrfs subvolume it does not own the mount of (the plan measured the user-level states)',
    },
    {
      id: 'c13a-select',
      title: 'Verify 4: choose subvolumes',
      window: true,
      says: "Choose the subvolumes whose content is compared byte by byte: the checkout, every worktree with a non-clean baseline row, and the six largest and six smallest by the size table. The plan says the rest 'if time allows'; this runner checks the rest by file count in the count stage.",
      command: 'true',
      internal: hookSelectVerify,
      check: `test -s ${c.wiz}/verify-set.tsv`,
      checkText: 'the selection is not empty',
      rollback: 'read-only',
      root: 'no',
      unprivileged: 'not needed: reads files this runner wrote',
    },
    {
      id: 'c13b-rsync',
      title: 'Verify 4: content compare',
      window: true,
      says: 'Verification 4: rsync --checksum --dry-run of each chosen source snapshot against its placed copy must print nothing (--one-file-system keeps the comparison at the subvolume boundary). An empty placeholder folder the snapshot shows for a subvolume (inode 2) is not compared when the live tree holds the same placeholder at its path or nothing, since a send never carries one and nothing can be stored in one; each is named in the output. One where the live tree holds a real folder or subvolume is compared, so a subvolume the inventory missed shows as a difference.',
      command: `while IFS=$'\\t' read -r name path; do
  target=$(sed -n 's/.*"target":"\\([^"]*\\)".*/\\1/p' ${c.moveState}/$name.placed.json)
  sudo find ${c.snapshots}/$name/ -xdev -inum ${String(EMPTY_SUBVOLUME_PLACEHOLDER_INODE,)} -type d -printf '%P\\0' > ${c.wiz}/placeholders-$name.found
  : > ${c.wiz}/placeholders-$name.bin
  while IFS= read -r -d '' folder; do
    if sudo test -e "$path/$folder" || sudo test -L "$path/$folder"; then live=$(sudo stat --format=%i -- "$path/$folder"); else live=gone; fi
    if [ "$live" = ${String(EMPTY_SUBVOLUME_PLACEHOLDER_INODE,)} ] || [ "$live" = gone ]; then
      echo "$name: not compared, an empty placeholder folder of a subvolume: $folder" >&2
      ${ // One NUL-ended rsync pattern per excused folder; rsync reads a backslash as an escape only in a pattern with a wildcard character, so only such a pattern is escaped.
  ''}printf '/%s/\\0' "$folder" | sed --null-data '/[*?[]/ s/[][*?\\\\]/\\\\&/g' >> ${c.wiz}/placeholders-$name.bin
    fi
  done < ${c.wiz}/placeholders-$name.found
  sudo rsync --archive --checksum --dry-run --itemize-changes --delete --one-file-system --omit-dir-times --from0 --exclude-from=${c.wiz}/placeholders-$name.bin ${c.snapshots}/$name/ $target/
done < ${c.wiz}/verify-set.tsv > ${c.wiz}/rsync-compare.txt`,
      check: `test ! -s ${c.wiz}/rsync-compare.txt || { head --lines=20 ${c.wiz}/rsync-compare.txt; exit 1; }`,
      checkText: 'rsync printed nothing for every chosen subvolume',
      rollback: thawRollback,
      root: 'yes',
      unprivileged: 'a user rsync reads everything the user can; root-owned files (the plan found root-owned logs) need root to compare',
    },
    {
      id: 'c14-sharing',
      title: 'Verify 5: sharing kept',
      window: true,
      says: `Verification 5: sharing kept. The copy's footprint (exclusive plus set shared) must not exceed ${sharing} percent of the frozen source's, the plan's quarter margin. A larger copy means extents were written again and the swap does not happen.`,
      command: `sudo systemd-run ${c.rootScope} timeout --kill-after=${killGrace} ${measure} btrfs filesystem du --summarize --raw ${c.finalPath} | tee ${c.wiz}/final-du.txt
sudo systemd-run ${c.rootScope} timeout --kill-after=${killGrace} ${measure} btrfs filesystem du --summarize --raw ${c.repo} | tee ${c.wiz}/source-du.txt`,
      check: `test $(( $(awk 'END { print $2 + $3 }' ${c.wiz}/final-du.txt) * 100 )) -le $(( $(awk 'END { print $2 + $3 }' ${c.wiz}/source-du.txt) * ${sharing} ))`,
      checkText: `copy footprint within ${sharing} percent of the source`,
      rollback: thawRollback,
      root: 'yes',
      unprivileged: 'none: btrfs filesystem du needs root',
    },
    {
      id: 'c15-counts',
      title: 'Verify 6: file counts',
      window: true,
      says: 'Verification 6: file counts equal for every subvolume (source snapshot against placed copy, as root, within each subvolume).',
      command: `while IFS=$'\\t' read -r name path; do
  target=$(sed -n 's/.*"target":"\\([^"]*\\)".*/\\1/p' ${c.moveState}/$name.placed.json)
  echo "$name $(sudo find ${c.snapshots}/$name -xdev -type f | wc --lines) $(sudo find $target -xdev -type f | wc --lines)"
done < ${c.wiz}/final-map.tsv | tee ${c.wiz}/file-counts.txt`,
      check: `awk '$2 != $3 { print "differs: " $0; bad = 1 } END { exit bad }' ${c.wiz}/file-counts.txt`,
      checkText: 'every subvolume has equal counts on both sides',
      rollback: thawRollback,
      root: 'yes',
      unprivileged: 'a user find sees what the user can read; root-owned files need root',
    },
    {
      id: 'c16-root-owned',
      title: 'Verify 7: root-owned files',
      window: true,
      says: 'Verification 7: root-owned files survived (the plan found 126 root-owned log files that an unprivileged copy cannot read). Counted on the frozen source and on the copy.',
      command: `echo "source $(sudo find ${c.repo} -user root -name '*.log.jsonl' | wc --lines)"
echo "copy $(sudo find ${c.finalPath} -user root -name '*.log.jsonl' | wc --lines)"`,
      check: `test "$(grep '^source ' "$OUTPUT_FILE" | cut --delimiter=' ' --fields=2)" = "$(grep '^copy ' "$OUTPUT_FILE" | cut --delimiter=' ' --fields=2)"`,
      checkText: "equal counts (zero on both sides means none exist now; the plan's 126 may be gone)",
      rollback: thawRollback,
      root: 'yes',
      unprivileged: 'none: finding root-owned files in unreadable folders needs root',
    },
    {
      id: 'c17-externals',
      title: 'Verify 8: external worktrees',
      window: true,
      says: "Verification 8: each external worktree's read-only source against its copy. A dry-run rsync --delete prints every difference. A line starting *deleting is a file deleted at the source after pass one: the copy keeps it on purpose and pass two reported it, so it is listed and does not fail the stage. Any other difference fails it.",
      command: `for name in ${names}; do
  sudo rsync --archive --checksum --dry-run --itemize-changes --delete --one-file-system --omit-dir-times ${c.externalWorktrees}/$name/ ${c.externalFinal}/$name/
done > ${c.wiz}/external-compare.txt`,
      check: `${ // grep --invert-match exits 1 when it selects no lines, which an empty compare and one of only *deleting lines give; the group keeps that from failing a stage that has no difference to report.
        ''}{ grep --invert-match --fixed-strings '*deleting' ${c.wiz}/external-compare.txt || true; } | head --lines=20 > ${c.wiz}/external-differences.txt
echo "kept files deleted at the source after pass one: $(grep --count --fixed-strings '*deleting' ${c.wiz}/external-compare.txt || true)"
test ! -s ${c.wiz}/external-differences.txt || { cat ${c.wiz}/external-differences.txt; exit 1; }`,
      checkText: 'no difference other than files deleted at the source after pass one',
      rollback: `${thawRollback}. A real difference means pass two missed a change: run this stage again after the cause is understood`,
      root: 'yes',
      unprivileged: 'a user rsync works for what the user can read',
    },
    {
      id: 'c18-guard',
      title: 'Mountpoint guard',
      window: true,
      says: `MOUNTPOINT GUARD (plan R1 and R2) before the swap: in a private namespace, bind the NVMe /var, require that the hidden ${c.drive} folder holds no entries, then make it root-owned, mode 0755 and immutable. Without it a program that creates the resolved path while the 860 is away writes to the NVMe. It stops if the alias is not the NVMe or the folder holds entries.`,
      command: `sudo unshare --mount --propagation private bash --noprofile --norc -euo pipefail -c '
  alias_dir=$(mktemp --directory /run/underlying-var.XXXXXX)
  mount --bind /var "$alias_dir"
  hidden="$alias_dir${underVar}"
  [ "$(findmnt --noheadings --output UUID --target "$alias_dir")" = "$(findmnt --noheadings --output UUID --target /var)" ]
  [ "$(findmnt --noheadings --output UUID --target "$alias_dir")" != "$(findmnt --noheadings --output UUID --target ${c.drive})" ]
  stat --format="%U:%G %a %n" "$hidden"
  lsattr -d "$hidden"
  entries=$(ls -A "$hidden" | wc --lines)
  echo "entries under the hidden mountpoint: $entries"
  [ "$entries" -eq 0 ]
  chown root:root "$hidden"
  chmod 0755 "$hidden"
  chattr +i "$hidden"
  echo "after guard: $(lsattr -d "$hidden")"
  umount "$alias_dir"
  rmdir "$alias_dir"
'
echo "mounted drive: $(lsattr -d ${c.drive})"`,
      check: `grep --extended-regexp '^after guard: .{4}i' "$OUTPUT_FILE" && ! grep --extended-regexp '^mounted drive: .{4}i' "$OUTPUT_FILE"`,
      checkText: 'the hidden folder shows the i flag and the mounted 860 does not',
      rollback: 'the guard is harmless to keep. To undo: the same namespace pattern with chattr -i on the hidden folder. If entries were found, read and move them yourself, then run this stage again',
      root: 'yes',
      unprivileged: 'none: a bind mount of the underlying /var and chattr +i need root; no user-level route to the hidden folder exists (measured in the plan)',
    },
    {
      id: 'c19-swap',
      title: 'Swap the old paths to links',
      window: true,
      says: 'SWAP: rename the old checkout and the old external-worktrees folder aside (renames, no copy, no delete), then point the old paths at the copies with symbolic links. The old trees stay frozen and read-only as cold storage.',
      command: `test ! -e ${oldRepo} && test ! -e ${oldExternal}
test -d ${c.finalPath}/.git && test -d ${c.externalFinal}
mv --no-clobber --no-target-directory ${c.repo} ${oldRepo}
ln --symbolic ${c.finalPath} ${c.repo}
mv --no-clobber --no-target-directory ${c.externalWorktrees} ${oldExternal}
ln --symbolic ${c.externalFinal} ${c.externalWorktrees}
ls -ld ${c.repo} ${c.externalWorktrees}`,
      check: `test -L ${c.repo} && test "$(readlink ${c.repo})" = ${c.finalPath} && test -L ${c.externalWorktrees} && test "$(readlink ${c.externalWorktrees})" = ${c.externalFinal} && test -d ${c.repo}/.git && test -d ${oldRepo}`,
      checkText: "both old paths are links to the 860 copies, the checkout's .git resolves through the link, the old tree sits beside it",
      rollback: 'the rollback command (fast path, valid only until the release stage)',
      root: 'no',
      unprivileged: `not needed: the home folder is user-owned mode 0750 (measured in the plan), so the renames and links need no root. The old tree stays frozen read-only while renamed (a rename of a read-only subvolume's folder within the home folder works as the user)`,
    },
    {
      id: 'c20-through-link',
      title: 'Verify 9: through the link',
      window: true,
      says: 'Verification 9: the same fsck, table and worktree list, now through the real link. Git prints the main worktree under its resolved path, so the through-link table and list spell that one line as the 860 path where the frozen baseline spells the link path: the comparison allows exactly that spelling on that one line, and a difference on any other line still fails the stage.',
      command: `/usr/bin/git --no-optional-locks -C ${c.repo} fsck --full
systemd-run ${c.userScope} timeout --kill-after=${killGrace} ${scanTime} ${scan} ${c.repo} ${c.wiz}/through-link.tsv ${perScan}
/usr/bin/git --no-optional-locks -C ${c.repo} worktree list --porcelain > ${c.wiz}/porcelain-through-link.txt`,
      check: `${ // Git prints the main worktree under its resolved path, so one line of each file spells it as the 860 path where the baseline spells the link path. Only the line where that path stands alone is rewritten, on both sides, so any other difference still differs.
        ''}main_spelling() {
  sed --expression 's|^worktree ${c.repo}\$|worktree <the main worktree>|' --expression 's|^worktree ${c.finalPath}\$|worktree <the main worktree>|' --expression 's|^${c.repo}\t|<the main worktree>\t|' --expression 's|^${c.finalPath}\t|<the main worktree>\t|' "$1"
}
main_spelling ${c.wiz}/baseline-frozen.tsv > ${c.wiz}/baseline-spelling.tsv
main_spelling ${c.wiz}/through-link.tsv > ${c.wiz}/through-link-spelling.tsv
main_spelling ${c.wiz}/porcelain-frozen.txt > ${c.wiz}/porcelain-frozen-spelling.txt
main_spelling ${c.wiz}/porcelain-through-link.txt > ${c.wiz}/porcelain-through-link-spelling.txt
grep --quiet --extended-regexp "^${c.repo}[[:space:]]" ${c.wiz}/baseline-frozen.tsv
grep --quiet --extended-regexp "^${c.finalPath}[[:space:]]" ${c.wiz}/through-link.tsv
grep --quiet --line-regexp "worktree ${c.repo}" ${c.wiz}/porcelain-frozen.txt
grep --quiet --line-regexp "worktree ${c.finalPath}" ${c.wiz}/porcelain-through-link.txt
cmp --silent ${c.wiz}/baseline-spelling.tsv ${c.wiz}/through-link-spelling.tsv || { diff --unified=0 ${c.wiz}/baseline-spelling.tsv ${c.wiz}/through-link-spelling.tsv; exit 1; }
cmp --silent ${c.wiz}/porcelain-frozen-spelling.txt ${c.wiz}/porcelain-through-link-spelling.txt || { diff --unified=0 ${c.wiz}/porcelain-frozen-spelling.txt ${c.wiz}/porcelain-through-link-spelling.txt; exit 1; }`,
      checkText: 'the table and the worktree list through the link equal the frozen baseline, the main worktree spelled as the 860 path there and as the link path in the baseline, and nothing else differs',
      rollback: 'the rollback command',
      root: 'no',
      unprivileged: 'not needed: read-only Git as the user',
    },
    {
      id: 'c20b-record-after',
      title: 'Record the repository link identity, phase after',
      window: true,
      says: 'Record the identity the check enforces now that the old path is a link (phase after): the link path, the exact link text, the approved canonical root, the 860 filesystem UUID, and the first commit of main, which is also written to .git/monochromatic-root-commit. Run as the user, never with sudo. Then the installed check must say ready.',
      command: `${link}/record.mjs --phase=after --link-path=${c.repo} --mount-point=${c.drive}
${link}/check.mjs`,
      check: `grep --fixed-strings 'ready (on the 860)' "$OUTPUT_FILE"`,
      checkText: 'the installed check says ready (on the 860)',
      rollback: 'the rollback command renames the identity file aside and records phase before again',
      root: 'no',
      unprivileged: 'not needed: written as the user',
    },
    {
      id: 'c20c-refresh-checkouts',
      title: 'Refresh the moved checkouts',
      window: true,
      says: `The move gave every file a new inode, so each checkout's index keeps the stat data of the files as they were before it: a commit there is stopped at the wrapper's 3-second scratch ceiling, on every retry, until a status has rewritten the index (measured after the move). The relative links of node_modules reach the pnpm store through the checkout's physical path, so they point nowhere after the move, and the main checkout's dangling links broke pi; mise exec -- pnpm install --offline --frozen-lockfile repaired that checkout in 7.6 s. This stage runs a status in every registered checkout, which rewrites the index and changes no file (a --no-optional-locks status does not rewrite it, measured on 2026-11-08), and repairs every node_modules that holds a dangling link. It runs after the through-the-link checks, which compare the index hashes with the frozen baseline, and before the release, because commits resume there. A checkout with no node_modules is left as it is and named.`,
      command: `/usr/bin/git --no-optional-locks -C ${c.repo} worktree list --porcelain > ${c.wiz}/checkouts-moved.txt
awk '/^worktree /{ print substr($0, 10) }' ${c.wiz}/checkouts-moved.txt > ${c.wiz}/checkout-paths.txt
: > ${c.wiz}/checkout-indexes.txt
: > ${c.wiz}/checkout-index-refusals.txt
: > ${c.wiz}/checkout-installs.txt
: > ${c.wiz}/checkout-links-left.txt
: > ${c.wiz}/checkout-without-node-modules.txt
: > ${c.wiz}/checkout-install.log
while IFS= read -r path; do
  if /usr/bin/git -C "$path" status --porcelain > /dev/null 2> ${c.wiz}/checkout-status.err; then
    printf 'index rewritten: %s\\n' "$path" >> ${c.wiz}/checkout-indexes.txt
  else
    printf 'index not rewritten: %s: %s\\n' "$path" "$(head --lines=1 ${c.wiz}/checkout-status.err)" >> ${c.wiz}/checkout-index-refusals.txt
  fi
  if [ ! -d "$path/node_modules" ]; then
    printf 'no node_modules: %s\\n' "$path" >> ${c.wiz}/checkout-without-node-modules.txt
    continue
  fi
  links=$(find "$path/node_modules" -xtype l -print | wc --lines)
  if [ "$links" -eq 0 ]; then
    continue
  fi
  printf 'dangling links before the install: %s %s\\n' "$links" "$path" >> ${c.wiz}/checkout-installs.txt
  if ( cd "$path" && mise exec -- pnpm install --offline --frozen-lockfile ) >> ${c.wiz}/checkout-install.log 2>&1; then
    printf 'repaired: %s\\n' "$path" >> ${c.wiz}/checkout-installs.txt
  else
    printf 'the install failed: %s\\n' "$path" >> ${c.wiz}/checkout-installs.txt
  fi
  left=$(find "$path/node_modules" -xtype l -print | wc --lines)
  if [ "$left" -ne 0 ]; then
    printf '%s still holds %s dangling node_modules links after the install\\n' "$path" "$left" >> ${c.wiz}/checkout-links-left.txt
  fi
done < ${c.wiz}/checkout-paths.txt
printf 'checkouts: %s, indexes rewritten: %s, node_modules repaired: %s, without node_modules: %s\\n' "$(wc --lines < ${c.wiz}/checkout-paths.txt)" "$(wc --lines < ${c.wiz}/checkout-indexes.txt)" "$(grep --count '^repaired: ' ${c.wiz}/checkout-installs.txt || true)" "$(wc --lines < ${c.wiz}/checkout-without-node-modules.txt)"`,
      check: `test -s ${c.wiz}/checkout-paths.txt && test ! -s ${c.wiz}/checkout-index-refusals.txt && test ! -s ${c.wiz}/checkout-links-left.txt`,
      checkText: "every checkout's index was rewritten and no node_modules link is left dangling",
      rollback: 'the indexes and node_modules folders it wrote belong to the working trees; the rollback command still applies (it renames the links aside)',
      root: 'no',
      unprivileged: 'not needed: Git and pnpm run as the user, and the copy is writable after the final pass',
    },
    {
      id: 'c21-release',
      title: 'Release',
      window: true,
      says: 'RELEASE: start the stopped commit sockets and claude-cpu-watch.service again and close the window. After this the fast rollback no longer applies (writes will reach the 860 tree); a later rollback is the full reverse migration in the plan, section Rollback, which this runner does not run.',
      assertion: 'Release now: this is the last chance for the fast rollback.',
      command: `{ grep '\\.socket$' ${c.wiz}/stopped-units.txt || true; } | xargs --no-run-if-empty systemctl --user start || echo 'WARNING: a commit socket did not restart; the lines below name each one'
{ grep '\\.socket$' ${c.wiz}/stopped-units.txt || true; } | while read -r unit; do systemctl --user is-active --quiet "$unit" || echo "not listening again: $unit"; done
if grep --fixed-strings --line-regexp --quiet claude-cpu-watch.service ${c.wiz}/stopped-units.txt; then systemctl --user start claude-cpu-watch.service; fi
date +%s > ${c.wiz}/window-end
echo "window length: $(( $(cat ${c.wiz}/window-end) - $(cat ${c.wiz}/window-start) )) s"`,
      check: `! grep --fixed-strings --line-regexp --quiet claude-cpu-watch.service ${c.wiz}/stopped-units.txt || systemctl --user is-active --quiet claude-cpu-watch.service`,
      checkText: 'claude-cpu-watch.service is active again (if the cut-over stopped it)',
      rollback: 'none after this stage; see the plan, section Rollback, full path',
      root: 'no',
      unprivileged: 'not needed: user units start as the user',
    },
    {
      id: 'c22-verify-final-checksum',
      title: 'Verify the final copies that changed, by checksum',
      says: `The same comparison as the window's, reading every byte, for the same final copies, after the cut-over and outside the window. The old tree ${oldRepo} is kept for the rollback: do not delete it until this stage is recorded done (the status command says so).`,
      command: `${rootCopy} --run --pass=verify-final --checksum ${args} --deadline-seconds=${String(c.transferDeadlineSeconds,)} --total-deadline-seconds=${String(c.firstPassTotalSeconds,)}`,
      check: `grep --fixed-strings 'verify-final finished' "$OUTPUT_FILE"`,
      checkText: "'verify-final finished' at checksum depth",
      rollback: 'read-only comparison; nothing changed. A copy that differs is reported with its name before the old tree is deleted',
      timeoutSeconds: c.firstPassTotalSeconds + (c.killGraceSeconds
        * 2),
      root: 'yes',
      unprivileged: 'none: the comparison reads files only root can read',
    },
    {
      id: 'c23-link-check',
      title: 'After the cut-over: the link check',
      says: 'W15: the shared check says the repository link is ready, now that the window is closed and writers are back. The Git wrapper\'s doctor prints the same line (git monochromatic-git-policy-cli doctor).',
      command: `${link}/check.mjs`,
      check: `grep --fixed-strings 'ready (on the 860)' "$OUTPUT_FILE"`,
      checkText: 'the installed check says ready (on the 860)',
      rollback: 'read-only',
      root: 'no',
      unprivileged: 'not needed: the check only reads',
    },
    {
      id: 'c24-mise-trust',
      title: 'After the cut-over: mise trust',
      says: 'W14: mise keys trust by the canonical path, so the 860 path starts untrusted. Trust the repository through mise itself (nothing is written into mise state by hand), then show the trust of the root, of one package and of one worktree.',
      command: `cd ${c.repo} && mise trust --all
cd ${c.repo} && mise trust --show
cd ${c.repo}/package/repository-move/cli && mise trust --show
worktree=$(awk '/^worktree /{ print $2 }' ${c.wiz}/porcelain-through-link.txt | sed --quiet 2p)
if [ -n "$worktree" ]; then cd "$worktree" && mise trust --show; fi`,
      check: `test "$(grep --count ': trusted' "$OUTPUT_FILE")" -ge 3 && ! grep --fixed-strings 'untrusted' "$OUTPUT_FILE"`,
      checkText: 'the root, a package and a worktree show trusted and nothing shows untrusted',
      rollback: 'mise trust --untrust (the trust links of the old path stay as stale files)',
      timeoutSeconds: afterSeconds,
      root: 'no',
      unprivileged: 'not needed: mise state is the user\'s',
    },
    {
      id: 'c25-pnpm-links',
      title: 'After the cut-over: pnpm links in a fresh worktree',
      says: `W14: install the dependencies in a fresh worktree (${freshWorktree}) and list every link of node_modules that points nowhere; there must be none. The worktree and its branch stay for you to remove (the rollback text names the commands).`,
      command: `/usr/bin/git -C ${c.repo} worktree add --quiet ${freshWorktree} -b move-check-${stamp}
cd ${freshWorktree} && timeout --kill-after=${killGrace} ${String(afterSeconds,)} pnpm install --frozen-lockfile --offline
cd ${freshWorktree} && find node_modules -xtype l > ${c.wiz}/broken-links.txt
echo "broken links: $(wc --lines < ${c.wiz}/broken-links.txt)"`,
      check: `test -d ${freshWorktree}/node_modules && test ! -s ${c.wiz}/broken-links.txt`,
      checkText: 'node_modules exists and find -xtype l printed nothing',
      rollback: `/usr/bin/git -C ${c.repo} worktree remove --force ${freshWorktree} && /usr/bin/git -C ${c.repo} branch --delete --force move-check-${stamp} (yours to run; this runner deletes nothing)`,
      timeoutSeconds: afterSeconds + c.killGraceSeconds,
      root: 'no',
      unprivileged: 'not needed: the worktree and the install are the user\'s',
    },
    {
      id: 'c26-kopia-snapshot',
      title: 'After the cut-over: first kopia snapshot',
      says: `W12: kopia resolves the link and records the source path as given, so the source is unchanged (${kopiaSource}). Make one snapshot now and read the size of the last snapshots: btrfs send and receive keep modification times, so this one should upload little. The snapshot goes to the same repository the hourly job uses.`,
      command: `kopia snapshot create ${kopiaSource}
kopia snapshot list ${kopiaSource} | tail --lines=6`,
      check: `kopia snapshot list ${kopiaSource} > /dev/null`,
      checkText: 'kopia lists snapshots of the source (the first run created one)',
      rollback: 'read-only after the snapshot; a snapshot is removed with kopia snapshot delete (yours to run)',
      timeoutSeconds: c.firstPassTotalSeconds,
      root: 'no',
      unprivileged: 'not needed: kopia runs as the user',
    },
    {
      id: 'c27-container-bind',
      title: 'After the cut-over: a container bind through the link',
      says: 'W13: a container with the repository as a read-only bind source, spelled through the link, starts and sees the Git folder. It uses the first image already on this host and pulls nothing; its memory, processor and process counts are capped.',
      // awk reads the whole listing: a reader that stops at the first line ends the writer with SIGPIPE, which pipefail reports as a failure.
      command: `image=$(podman images --noheading --format '{{.Repository}}:{{.Tag}}' | grep --invert-match '<none>' | awk 'NR == 1')
test -n "$image"
podman run --rm --pull=never ${CONTAINER_BIND_LIMIT_OPTIONS} --mount type=bind,source=${c.repo},target=/repository,ro --entrypoint test "$image" -d /repository/.git
echo 'container bind through the link: ok'`,
      check: `grep --fixed-strings 'container bind through the link: ok' "$OUTPUT_FILE"`,
      checkText: 'the container saw /repository/.git',
      rollback: 'the container is removed when it exits (--rm)',
      timeoutSeconds: afterSeconds,
      root: 'no',
      unprivileged: 'not needed: rootless podman',
    },
    {
      id: 'c28-ide-through-link',
      title: 'After the cut-over: the IDE through the link',
      says: `W13: open ${c.repo} in the IDE you use (JetBrains, Helium, Lapce, Codex, Emdash, Polypane, t3code) through the link spelling and confirm the file tree and one search read it. Editors only read, and keep whichever spelling they were given. This runner cannot see an IDE, so it asks you to say you did.`,
      assertion: 'I opened the repository in the IDE through the link and it read the files.',
      command: `echo "open ${c.repo} in the IDE, then pass --assert=c28-ide-through-link"`,
      check: 'true',
      checkText: 'you said you did',
      rollback: 'nothing changed',
      root: 'no',
      unprivileged: 'not needed: nothing is run',
    },
  ];
}
