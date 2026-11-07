/**
 The fixture stages of `prepare`: the plan's F1 to F8 on two throwaway loop images, run before anything real is
 touched. Every command names the fixture folder through `$FX`, which the stage before them wrote.

 @module
 */

import type {
  GuidedConfig,
  Stage,
} from './guided-types.ts';

/**
 The argument list of the copy tool against the fixture's filesystems.

 @param fx - the fixture folder
 
 @returns the shared arguments
 */
function fixtureArguments(fx: string,): string {
  return `--main=${fx}/source/main --snapshots=${fx}/snap --received=${fx}/dest/recv --final=${fx}/dest/final --state=${fx}/state --inventory=${fx}/inventory.txt --inventory-main=main --inventory-prefix=main/=${fx}/source/main/ --deadline-seconds=600 --min-free-gib=1`;
}

/**
 The fixture stages.

 @param c - the configuration
 
 @param fx - the fixture folder; a placeholder in a dry run, before the folder exists
 
 @returns the stages in order
 */
export function fixtureStages(
  {
    c,
    fx,
  }: {
    readonly c: GuidedConfig;
    readonly fx: string;
  },
): readonly Stage[] {
  /**
   The arguments shared by the commands of this step.
   */
  const args = fixtureArguments(fx,);
  /**
   The copy tool command, run as the user.
   */
  const copy = `${c.nodePath} ${c.copyTool}`;
  /**
   The copy tool command, run through sudo with the absolute path of node.
   */
  const rootCopy = `sudo ${c.nodePath} ${c.copyTool}`;
  /**
   The fixture step deadline in seconds.
   */
  const timeout = c.fixtureStepTimeoutSeconds;
  return [
    {
      id: 'f1a-folder',
      title: 'Fixture: folder',
      says: `Fixture F1a: a fresh private folder under ${c.fixtureParent}. Only what this run creates is ever touched.`,
      command: `mkdir --parents ${c.fixtureParent} && mktemp --directory ${c.fixtureParent}/send-fixture.XXXXXX | tee ${c.wiz}/fixture-path`,
      check: `test -d "$(cat ${c.wiz}/fixture-path)"`,
      checkText: 'the fixture folder exists',
      rollback: 'the folder is empty; its removal is printed in the last fixture stage',
      root: 'no',
      unprivileged: 'not needed: the folder is made as the user',
    },
    {
      id: 'f1-images',
      title: 'Fixture: loop images',
      says: 'Fixture F1: two 3G sparse images, a btrfs on each, mounted by loop. No real disk is touched.',
      command: `set -o noclobber
: > ${fx}/source.img
: > ${fx}/dest.img
truncate --no-create --size=3G ${fx}/source.img ${fx}/dest.img
sudo mkfs.btrfs --quiet ${fx}/source.img
sudo mkfs.btrfs --quiet ${fx}/dest.img
sudo mkdir ${fx}/source ${fx}/dest
sudo mount -o loop ${fx}/source.img ${fx}/source
sudo mount -o loop ${fx}/dest.img ${fx}/dest
sudo mkdir ${fx}/dest/recv
mkdir ${fx}/snap ${fx}/state
findmnt --noheadings --output SOURCE,TARGET ${fx}/source ${fx}/dest`,
      check: `test "$(findmnt --noheadings --output SOURCE,TARGET ${fx}/source ${fx}/dest | grep --count '^/dev/loop')" = 2`,
      checkText: 'both mount lines are loop devices (a missing line stops the test)',
      rollback: `sudo umount ${fx}/source ${fx}/dest (whichever is mounted); the images are deleted from the list printed in the last fixture stage`,
      timeoutSeconds: timeout,
      root: 'yes',
      unprivileged: 'mkfs.btrfs on the image file works as the user; the loop mount needs CAP_SYS_ADMIN and btrfs cannot be mounted in a user namespace, so root stays',
    },
    {
      id: 'f2-populate',
      title: 'Fixture: source tree',
      says: 'Fixture F2: a main checkout that is a subvolume with a Git repository and two worktree subvolumes sharing one 300 MiB extent by reflink; the filesystem list is taken with UUIDs (list -u).',
      command: `sudo btrfs subvolume create ${fx}/source/main
sudo chown ${c.userName}:${c.groupName} ${fx}/source/main
/usr/bin/git -C ${fx}/source/main init --quiet
head --bytes=300M /dev/urandom > ${fx}/source/main/big
/usr/bin/git -C ${fx}/source/main add big
/usr/bin/git -C ${fx}/source/main -c user.name=t -c user.email=t@t commit --quiet --message base
mkdir --parents ${fx}/source/main/.claude/worktrees
sudo btrfs subvolume create ${fx}/source/main/.claude/worktrees/a
sudo btrfs subvolume create ${fx}/source/main/.claude/worktrees/b
sudo chown ${c.userName}:${c.groupName} ${fx}/source/main/.claude/worktrees/a ${fx}/source/main/.claude/worktrees/b
/usr/bin/git -C ${fx}/source/main worktree add --force --quiet ${fx}/source/main/.claude/worktrees/a -b a
/usr/bin/git -C ${fx}/source/main worktree add --force --quiet ${fx}/source/main/.claude/worktrees/b -b b
cp --reflink=always ${fx}/source/main/big ${fx}/source/main/.claude/worktrees/a/big.copy
cp --reflink=always ${fx}/source/main/big ${fx}/source/main/.claude/worktrees/b/big.copy
sudo btrfs filesystem du --summarize ${fx}/source/main | tee ${fx}/source-baseline.txt
sudo btrfs subvolume list -u ${fx}/source > ${fx}/inventory.txt`,
      check: `test "$(grep --count -E ' uuid [0-9a-f-]+ path (main|.*/.claude/worktrees/[ab])$' ${fx}/inventory.txt)" = 3`,
      checkText: 'the filesystem list with UUIDs shows main and its two worktree subvolumes',
      rollback: 'nothing outside the fixture changed',
      timeoutSeconds: timeout,
      root: 'yes',
      unprivileged: 'after the first chown the subvolume creates work as the user (btrfs allows unprivileged subvolume create on a folder the user owns); btrfs filesystem du and subvolume list are root-only',
    },
    {
      id: 'f3a-print',
      title: 'Fixture F3a: print mode',
      says: 'Fixture F3a: the copy tool in print mode against the fixture. It runs nothing; it proves the inventory path rebuild.',
      command: `${copy} --pass=first ${args} > ${fx}/print-first.txt`,
      check: `test "$(grep --count '^btrfs subvolume snapshot -r' ${fx}/print-first.txt)" = 3 && grep --fixed-strings 'inventory agrees: 2 nested subvolumes' "$OUTPUT_FILE"`,
      checkText: 'three snapshots printed and the inventory agrees with Git (main, a, b)',
      rollback: 'nothing ran',
      root: 'no',
      unprivileged: 'not needed: print mode runs as the user',
    },
    {
      id: 'f3b-first',
      title: 'Fixture F3b: first pass',
      says: 'Fixture F3b: the first pass for real, as root, on the loop images.',
      command: `${rootCopy} --run --pass=first ${args}`,
      check: `grep --fixed-strings 'pass first finished' "$OUTPUT_FILE" && ls ${fx}/state/s000.received.json ${fx}/state/s001.received.json ${fx}/state/s002.received.json`,
      checkText: "'pass first finished' and a received record for s000, s001, s002",
      rollback: 'the images are throwaway; nothing real changed',
      timeoutSeconds: timeout,
      root: 'yes',
      unprivileged: 'none for send and receive (they need CAP_SYS_ADMIN); the plan\'s method M2 (unprivileged copy plus per-worktree snapshots) loses extent sharing and is the fallback if the root fixture shows sending does not work',
    },
    {
      id: 'f3c-change',
      title: 'Fixture F3c: change',
      says: 'Fixture F3c: change content after the first pass (a new file in worktree b, an empty commit in main).',
      command: `echo changed >> ${fx}/source/main/.claude/worktrees/b/extra.txt
/usr/bin/git -C ${fx}/source/main -c user.name=t -c user.email=t@t commit --allow-empty --quiet --message after`,
      check: `test -s ${fx}/source/main/.claude/worktrees/b/extra.txt`,
      checkText: 'the change exists',
      rollback: 'throwaway fixture',
      root: 'no',
      unprivileged: 'not needed: the fixture belongs to the user',
    },
    {
      id: 'f3d-freeze',
      title: 'Fixture F3d: freeze',
      says: 'Fixture F3d: the freeze as its own step, as the cut-over runs it: ro=true on every fixture subvolume and a manifest of what was frozen. It runs as the user, because the plan measured that btrfs property set -ts ro works as the subvolume\'s owner without root, and the fixture subvolumes were chowned to the user.',
      command: `${copy} --run --pass=freeze ${args}`,
      check: `grep --fixed-strings 'freeze finished' "$OUTPUT_FILE" && test -s ${fx}/state/freeze-manifest.tsv`,
      checkText: "'freeze finished' and a freeze manifest",
      rollback: `${copy} --run --pass=thaw ${args}`,
      timeoutSeconds: timeout,
      root: 'no',
      unprivileged: 'this is the unprivileged route: the owner of a subvolume sets ro on it without root (plan, review item 4 measurement); a subvolume the user does not own makes the tool stop and name it',
    },
    {
      id: 'f3d-final',
      title: 'Fixture F3d: final pass',
      says: 'Fixture F3d: the final pass after the freeze, as the cut-over will run it.',
      command: `${rootCopy} --run --pass=final ${args}`,
      check: `grep --fixed-strings 'pass final finished' "$OUTPUT_FILE" && ls ${fx}/state/f000.received.json ${fx}/state/f001.received.json ${fx}/state/f002.received.json ${fx}/state/f000.placed.json ${fx}/state/f001.placed.json ${fx}/state/f002.placed.json`,
      checkText: "'pass final finished' and a received and a placed record for f000, f001, f002",
      rollback: 'throwaway fixture',
      timeoutSeconds: timeout,
      root: 'yes',
      unprivileged: 'none for send and receive; placement (snapshot of a received copy, rmdir of the stub) works as the user but is the last step of a root run',
    },
    {
      id: 'f4-compare',
      title: 'Fixture F4: compare',
      says: 'Fixture F4: compare every subvolume, check the received state, and check that reflink sharing survived (a copy that lost sharing is larger than the source).',
      command: `for path in main main/.claude/worktrees/a main/.claude/worktrees/b; do
  sudo rsync --archive --checksum --dry-run --itemize-changes --delete --one-file-system --omit-dir-times ${fx}/source/$path/ ${fx}/dest/final/\${path#main}/
done > ${fx}/rsync-compare.txt
sudo btrfs property get -ts ${fx}/dest/recv/f001 ro
sudo btrfs subvolume show ${fx}/dest/recv/f001 | awk '$1 == "Received" && $2 == "UUID:" { print "received " $3 }'
sudo btrfs subvolume show ${fx}/snap/f001 | awk '$1 == "UUID:" { print "sent " $2 }'
sudo btrfs filesystem du --summarize --raw ${fx}/dest/final | tee ${fx}/dest-du.txt
sudo btrfs filesystem du --summarize --raw ${fx}/source/main | tee ${fx}/source-du.txt
df --block-size=1M ${fx}/source ${fx}/dest`,
      check: `test ! -s ${fx}/rsync-compare.txt && grep --fixed-strings 'ro=true' "$OUTPUT_FILE" && test "$(grep '^received ' "$OUTPUT_FILE" | cut --delimiter=' ' --fields=2)" = "$(grep '^sent ' "$OUTPUT_FILE" | cut --delimiter=' ' --fields=2)" && test $(( $(awk 'END { print $2 + $3 }' ${fx}/dest-du.txt) * 100 )) -le $(( $(awk 'END { print $2 + $3 }' ${fx}/source-du.txt) * ${String(c.sharingLimitPercent,)} ))`,
      checkText: `rsync prints nothing for all three, ro=true, received UUID equals sent UUID, copy footprint within ${String(c.sharingLimitPercent,)} percent of the source (sharing kept)`,
      rollback: 'read-only checks; nothing to undo. If sharing was lost the method is wrong for this repository: stop and report',
      timeoutSeconds: timeout,
      root: 'yes',
      unprivileged: 'rsync and df work as the user (the fixture is user-owned after chown); btrfs subvolume show and filesystem du are root-only',
    },
    {
      id: 'f5a-deadline',
      title: 'Fixture F5a: deadline',
      says: 'Fixture F5a: a first pass with a one-second per-transfer deadline on fresh folders. It is expected to stop; a copy that finishes inside a second is also accepted, because F5b makes the interrupted case deterministically.',
      command: `sudo mkdir ${fx}/dest/recv-f5a
mkdir ${fx}/snap-f5a ${fx}/state-f5a
${rootCopy} --run --pass=first --main=${fx}/source/main --snapshots=${fx}/snap-f5a --received=${fx}/dest/recv-f5a --final=${fx}/dest/final-f5a --state=${fx}/state-f5a --inventory=${fx}/inventory.txt --inventory-main=main --inventory-prefix=main/=${fx}/source/main/ --deadline-seconds=1 --min-free-gib=1; echo "script exit status: $?"`,
      check: `grep --extended-regexp 'script exit status: [01]' "$OUTPUT_FILE"`,
      checkText: 'the run ended with status 0 (finished) or 1 (stopped at the deadline)',
      rollback: `a partial copy in ${fx}/dest/recv-f5a, if any, goes with the images`,
      timeoutSeconds: timeout,
      root: 'yes',
      unprivileged: 'none for send and receive',
    },
    {
      id: 'f5b-leftover',
      title: 'Fixture F5b: partial receive',
      says: 'Fixture F5b: a leftover partial receive (a subvolume named s000 with no completion record). The tool must refuse and name the exact delete for you to run.',
      command: `sudo mkdir ${fx}/dest/recv-f5b
sudo btrfs subvolume create ${fx}/dest/recv-f5b/s000
mkdir ${fx}/snap-f5b ${fx}/state-f5b
${rootCopy} --run --pass=first --main=${fx}/source/main --snapshots=${fx}/snap-f5b --received=${fx}/dest/recv-f5b --final=${fx}/dest/final-f5b --state=${fx}/state-f5b --inventory=${fx}/inventory.txt --inventory-main=main --inventory-prefix=main/=${fx}/source/main/ --deadline-seconds=600 --min-free-gib=1`,
      okCodes: [1,],
      check: `grep --fixed-strings 'exists without a completion record' "$OUTPUT_FILE" && grep --fixed-strings 'btrfs subvolume delete' "$OUTPUT_FILE"`,
      checkText: "exit status 1 with 'exists without a completion record' and the printed 'btrfs subvolume delete'",
      rollback: 'the leftover is inside the throwaway image',
      timeoutSeconds: timeout,
      root: 'yes',
      unprivileged: 'none for the root-made receive folder; the tool itself needs root only for send and receive',
    },
    {
      id: 'f5c-corrupt',
      title: 'Fixture F5c: corrupted stream',
      says: 'Fixture F5c: a stream with one flipped byte must be refused by btrfs receive (a checksum error).',
      command: `sudo bash -c 'btrfs send ${fx}/snap/s000 > ${fx}/stream.bin' && sudo chown ${c.userName}:${c.groupName} ${fx}/stream.bin
cp ${fx}/stream.bin ${fx}/stream.orig
python3 -I -c "import sys; p=sys.argv[1]; f=open(p,'r+b'); f.seek(100000); b=f.read(1); f.seek(100000); f.write(bytes([b[0]^255]))" ${fx}/stream.bin
if cmp --silent ${fx}/stream.orig ${fx}/stream.bin; then echo 'the flip changed nothing'; exit 1; fi
sudo mkdir ${fx}/dest/recv-f5c
if sudo btrfs receive ${fx}/dest/recv-f5c < ${fx}/stream.bin; then echo 'UNEXPECTED: the corrupted stream was accepted'; exit 1; else echo 'receive refused the corrupted stream, as required'; fi`,
      check: `grep --fixed-strings 'receive refused the corrupted stream' "$OUTPUT_FILE"`,
      checkText: 'btrfs receive refused the flipped stream',
      rollback: `a partial subvolume under ${fx}/dest/recv-f5c, if any, goes with the images`,
      timeoutSeconds: timeout,
      root: 'yes',
      unprivileged: 'none: btrfs send and receive need root',
    },
    {
      id: 'f6-staged',
      title: 'Fixture F6: staged verification',
      says: 'Fixture F6: the staged-verification pattern. A private mount namespace binds the copy over the source path; Git then reads the copy. Git runs as you, never as root.',
      command: `sudo unshare --mount --propagation private bash -euo pipefail -c "
  mount --bind ${fx}/dest/final ${fx}/source/main
  runuser -u ${c.userName} -- env HOME=${c.homeDir} /usr/bin/git --no-optional-locks -C ${fx}/source/main worktree list
  runuser -u ${c.userName} -- env HOME=${c.homeDir} /usr/bin/git --no-optional-locks -C ${fx}/source/main fsck --full
  runuser -u ${c.userName} -- env HOME=${c.homeDir} ${c.nodePath} ${c.scanTool} ${fx}/source/main ${fx}/staged.tsv 60
"`,
      check: `test "$(wc --lines < ${fx}/staged.tsv)" = 4 && grep --fixed-strings ${fx}/source/main/.claude/worktrees/a "$OUTPUT_FILE"`,
      checkText: 'worktree list shows the source paths and staged.tsv is the table of the three staged worktrees',
      rollback: 'the bind exists only inside the private namespace and vanishes when the command ends',
      timeoutSeconds: timeout,
      root: 'yes',
      unprivileged: 'none: a bind mount of a btrfs subvolume needs CAP_SYS_ADMIN in the initial namespace (a user namespace could only bind what it can already see, which was measured in the plan, state tests)',
    },
    {
      id: 'f7-immutable',
      title: 'Fixture F7: mount over immutable',
      says: 'Fixture F7: can a mount cover an immutable directory? Unmounts the two fixture mounts, then mounts the destination image on a +i folder.',
      command: `sudo umount ${fx}/source ${fx}/dest
sudo mkdir ${fx}/mp
sudo chattr +i ${fx}/mp
sudo mount -o loop ${fx}/dest.img ${fx}/mp
echo "while mounted: $(lsattr -d ${fx}/mp)"
sudo umount ${fx}/mp
echo "after unmount: $(lsattr -d ${fx}/mp)"
sudo chattr -i ${fx}/mp`,
      check: `grep --extended-regexp '^after unmount: .{4}i' "$OUTPUT_FILE" && ! grep --extended-regexp '^while mounted: .{4}i' "$OUTPUT_FILE"`,
      checkText: 'the mount succeeded; while mounted the directory shows no i flag; after unmount it shows i again',
      rollback: `sudo umount ${fx}/mp if still mounted; sudo chattr -i ${fx}/mp`,
      timeoutSeconds: timeout,
      root: 'yes',
      unprivileged: 'none: umount, chattr +i and a loop mount need root',
    },
    {
      id: 'f8-cleanup-printed',
      title: 'Fixture F8: removals printed',
      says: 'Fixture F8: cleanup. This runner never deletes. The next command only prints the removals for you to run when you are done with the fixture.',
      command: `printf '%s\\n' "sudo rmdir ${fx}/mp ${fx}/source ${fx}/dest" "rm -- ${fx}/source.img ${fx}/dest.img" "(then remove ${fx} and its remaining files yourself when you no longer need the logs)"`,
      check: `grep --count 'rmdir' "$OUTPUT_FILE"`,
      checkText: 'the removals are printed',
      rollback: 'nothing to undo',
      root: 'no',
      unprivileged: 'not needed: it only prints',
    },
  ];
}
