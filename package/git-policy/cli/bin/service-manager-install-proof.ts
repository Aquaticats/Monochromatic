#!/usr/bin/env node
/**
 What:
  The proof that an install of one release over the one before it leaves no wrapper unit failing a start, and that two
  commits of one worktree made at once across an upgrade each land once.

 Why:
  On 2026-11-13 the install of `0.6.1-alpha.20` left one worktree's instance starting from
  `<runtime>/systemd/user/<notes>/service-program`, a link into the release the install had just removed: five starts
  failed at the EXEC step and the manager left the unit failed. An hour later the install of `0.6.1-alpha.21` left the
  main checkout's unit failed at the start limit instead, because a client of another release retired each service the
  socket activated. Both are what this run asserts are gone.

 How it runs:
  One container with its own system manager, session bus and user manager (`service-manager-machine.ts`), two worktrees,
  the release before this one at `/opt/release-old` and this build at `/opt/release-a`. Both worktrees are seeded by the
  older release, so both instances are loaded under its unit text; the older release is then removed, as an install
  removes the release it replaces; the installing command runs in the first worktree only, and the second worktree's next
  commit is the start that used to fail. Everything this run asserts is read from the container's own manager and its
  journal.

 Usage:
  `node bin/service-manager-install-proof.ts --image <tag> --older-wrapper <path>`
  where `--older-wrapper` is a build of the release the account had before this one.
 */
import { join, } from 'node:path';

import { NativeVerificationError, } from './native-verification-process.ts';
import {
  countsOfJournalLines,
  nonEmptyLines,
  statusOf,
} from './service-manager-install-proof-answers.ts';
import { options, } from './service-manager-install-proof-options.ts';
import {
  containerRuntimeDirectory,
  inside,
  type Machine,
  releasePaths,
  removeKeptMachines,
  run,
  startMachine,
  worktreePaths,
} from './service-manager-machine.ts';
import { makeWorktrees, } from './service-manager-worktrees.ts';

/**
 The folder the release before this one is put in inside the container.
 */
const olderRelease: string = '/opt/release-old';

/**
 The notes folder the wrapper keeps inside the runtime unit directory.
 */
const notesFolder: string = `${containerRuntimeDirectory}/systemd/user/monochromatic-git-policy-cli-worktree-notes`;

/**
 How many transient units the machine holds, so a reload of its manager costs what a loaded host's costs.
 */
const machineUnits = 200;

/**
 What a background commit answered once it ended.
 */
type WaitedCommit = {
  /**
   The status the commit wrote for itself.
   */
  readonly status: number;
  /**
   The commit's last lines, joined onto one line.
   */
  readonly printed: string;
};

/**
 One commit through one release's wrapper in one worktree.

 @param machine - the machine the commit runs in

 @param release - the folder holding the release's `git`

 @param worktree - the worktree the commit is made in

 @param change - the text the commit writes into its own file

 @param file - the file it changes, so two commits of one worktree do not collide

 @param background - whether to leave it running, writing its status when it ends

 @returns the status line of one that ran to its end, or the line that started the other
 */
function commit({
  machine,
  release,
  worktree,
  change,
  file,
  background,
}: {
  readonly machine: Machine;
  readonly release: string;
  readonly worktree: string;
  readonly change: string;
  readonly file: string;
  readonly background: boolean;
}): Promise<string> {
  return inside({
    machine,
    script: [
      `cd ${worktree}`,
      `printf 'base\\n${change}\\n' > ${file}`,
      `${release}/git add ${file}`,
      background
        ? `{ ${release}/git commit --message '${change}' -- ${file}; echo "status=$?"; } > ${containerRuntimeDirectory}/${change}.out 2>&1 & echo started`
        : `set +e; ${release}/git commit --message '${change}' -- ${file} 2>&1 | tail -n 2; echo "status=\${PIPESTATUS[0]}"`,
    ].join('\n',),
  },);
}

/**
 Seeds one worktree with one commit through the older release, so its commit service loads that release's unit text.

 @param machine - the machine the seeding commit runs in

 @param worktree - the worktree the commit is made in

 @throws NativeVerificationError when the commit did not land, naming what the machine printed
 */
async function seedWorktree({
  machine,
  worktree,
}: {
  readonly machine: Machine;
  readonly worktree: string;
}): Promise<void> {
  /**
   The status line the seeding commit printed.
   */
  const seeded: string = await commit({
    machine,
    release: olderRelease,
    worktree,
    change: 'seed',
    file: 'a.txt',
    background: false,
  },);
  if (!seeded.includes('status=0',)) throw new NativeVerificationError(`the older release did not seed ${worktree}: ${seeded}`);
}

/**
 The status a background commit ended with, once it has ended.

 @param machine - the machine the commit ran in

 @param change - the text that names it

 @returns its status and its last lines
 */
async function waited({
  machine,
  change,
}: {
  readonly machine: Machine;
  readonly change: string;
}): Promise<WaitedCommit> {
  /**
   What the machine printed when it read the commit's own output file.
   */
  const answer: string = await inside({
    machine,
    script: [
      'set +e',
      'for _ in $(seq 1 2400); do',
      `  grep --quiet '^status=' ${containerRuntimeDirectory}/${change}.out 2>/dev/null && break`,
      '  sleep 0.05',
      'done',
      `tail -n 3 ${containerRuntimeDirectory}/${change}.out 2>/dev/null || true`,
    ].join('\n',),
  },);
  return {
    status: statusOf({ printed: answer, },),
    printed: answer.replaceAll(
      '\n',
      ' | ',
    ),
  };
}

/**
 What:
  The units the manager lists as failed,
  which is what a person sees in `systemctl --user --failed`.

 @param machine - the machine to ask

 @returns the unit names, empty when it lists none
 */
async function failedUnits({ machine, }: { readonly machine: Machine; }): Promise<readonly string[]> {
  /**
   What the manager listed.
   */
  const listed: string = await inside({
    machine,
    script: "systemctl --user list-units --state=failed --all --plain --no-legend --no-pager 2>/dev/null | awk '{print $1}'",
  },);
  return nonEmptyLines({ listed, },);
}

/**
 Puts one wrapper in the container as `<folder>/git` and makes it executable.

 @param machine - the machine to put it in

 @param wrapper - the executable to copy

 @param folder - the folder it goes in
 */
async function putWrapper({
  machine,
  wrapper,
  folder,
}: {
  readonly machine: Machine;
  readonly wrapper: string;
  readonly folder: string;
}): Promise<void> {
  await inside({
    machine,
    script: `mkdir --parents ${folder}`,
  },);
  await run({ args: [
    'podman',
    'cp',
    wrapper,
    `${machine.container}:${folder}/git`,
  ], },);
  await inside({
    machine,
    script: `chmod +x ${folder}/git`,
  },);
}

/**
 What:
  Runs the two steps this proof is about and judges them.

 Why:
  The machine is removed when this function ends however it ends, so a run that throws leaves no manager holding a
  container on this host.

 @param machine - the machine the steps run on

 @param candidate - the executable this worktree built, which the installing command runs

 @param olderReleaseWrapper - the executable of the release the account had before this one

 @returns everything this run found wrong, in the order the steps found it
 */
async function prove({
  machine,
  candidate,
  olderReleaseWrapper,
}: {
  readonly machine: Machine;
  readonly candidate: string;
  readonly olderReleaseWrapper: string;
}): Promise<readonly string[]> {
  /**
   The removal of this run's containers, made when the steps end by `await using`.
   */
  await using removal = {
    async [Symbol.asyncDispose](): Promise<void> {
      await removeKeptMachines();
    },
  };
  /**
   Everything that makes this run stop proving anything.
   */
  const problems: string[] = [];
  await putWrapper({
    machine,
    wrapper: olderReleaseWrapper,
    folder: olderRelease,
  },);
  await putWrapper({
    machine,
    wrapper: candidate,
    folder: releasePaths.first,
  },);
  await makeWorktrees({ machine, },);
  await seedWorktree({
    machine,
    worktree: worktreePaths.first,
  },);
  await seedWorktree({
    machine,
    worktree: worktreePaths.second,
  },);
  /**
   What the notes folder's program pointer names, which is the path an instance loaded under the older release starts.
   */
  const pointer: string = await inside({
    machine,
    script: `readlink ${notesFolder}/service-program 2>/dev/null || echo 'no program pointer'`,
  },);
  console.log(`the program pointer the older release's texts name: ${pointer.trim()}`);
  // What: The install of a release removes the release before it, which is what leaves those texts naming a program that
  //       is gone.
  // Why:  `mise run toolchain:install` does exactly this on a machine, and the proof is about the state it leaves.
  await inside({
    machine,
    script: `rm -rf ${olderRelease}`,
  },);
  /**
   The journal's own starting point, so nothing this run reads was written before it.
   */
  const since: string = (await inside({
    machine,
    script: 'date +%s',
  },)).trim();
  // The installing command: the first commit of this release, in the first worktree only.
  /**
   What the installing command printed.
   */
  const installing: string = await commit({
    machine,
    release: releasePaths.first,
    worktree: worktreePaths.first,
    change: 'install',
    file: 'a.txt',
    background: false,
  },);
  console.log(`the installing command: ${installing.replaceAll(
    '\n',
    ' | ',
  )}`);
  if (!installing.includes('status=0',)) problems.push(`the installing command did not land: ${installing}`);
  // What: Every instance the manager still holds at the text the older release loaded is started again, which is what a
  //       manager does at such an instance's next start; the socket-activated start would reach it only when the manager
  //       had not collected the unit first, which the install on this machine did not leave to chance.
  // Why:  That start reads the program the loaded text names, and the text names a link into the release the install
  //       removed; what happens then is the whole of the first finding.
  /**
   What the manager said when every instance it holds was started again.
   */
  const restarted: string = await inside({
    machine,
    script: `systemctl --user restart 'monochromatic-git-policy-cli@*' 2>&1 | tail -n 3; sleep 2; systemctl --user is-active 'monochromatic-git-policy-cli@*' 2>&1 | sort | uniq -c`,
  },);
  console.log(`every instance started again by the manager: ${restarted.trim()
    .replaceAll(
      '\n',
      '; '
    )}`);
  // The second worktree's next commit, through this release, which is a start of its own.
  /**
   What the second worktree's next commit printed.
   */
  const second: string = await commit({
    machine,
    release: releasePaths.first,
    worktree: worktreePaths.second,
    change: 'second-worktree',
    file: 'a.txt',
    background: false,
  },);
  console.log(`the second worktree's next commit: ${second.replaceAll(
    '\n',
    ' | ',
  )}`);
  if (!second.includes('status=0',)) problems.push(`the second worktree's commit did not land: ${second}`);
  // What: Every wrapper unit's own record, from this container's system journal, from the install step onwards.
  // Why:  The user manager writes its lines to this journal, so a read of it is the manager's own answer about a start.
  /**
   What the journal holds about every wrapper unit, from the install step onwards.
   */
  const journal: string = await inside({
    machine,
    script: `journalctl --no-pager --output=short-iso --since=@${since} 2>/dev/null | grep -E 'monochromatic-git-policy-cli@|started release=' || true`,
  },);
  /**
   The journal's lines counted by what each one says.
   */
  const counts = countsOfJournalLines({ journal, },);
  console.log(`starts of a wrapper service this run made: ${String(counts.starts)}`);
  console.log(`starts that failed at the EXEC step: ${String(counts.execFailures)}`);
  console.log(`units that reached the start limit: ${String(counts.limits)}`);
  // What: A journal that holds no start of a wrapper service is not a pass.
  // Why:  A read that reached nothing proves nothing about the starts it was meant to show.
  if (counts.starts === 0) throw new NativeVerificationError('the journal this run read holds no start of a wrapper service');
  if (counts.execFailures > 0) problems.push(`the manager failed ${String(counts.execFailures)} start(s) at the EXEC step`);
  if (counts.limits > 0) problems.push('a wrapper unit reached the start limit');
  /**
   The units the manager lists as failed once the install step and the restarted instances have run.
   */
  const failed: readonly string[] = await failedUnits({ machine, },);
  console.log(`units the manager left failed: ${failed.length === 0 ? 'none' : failed.join('; ')}`);
  if (failed.length > 0) problems.push(`the manager left ${failed.join('; ')} failed`);
  // What: Two commits of one worktree at once, across the upgrade, each landing once.
  // Why:  That is the window the unload's look and stop leave open, and the pair is what shows it is not cut.
  await inside({
    machine,
    script: `rm -f ${notesFolder}/*.units-the-manager-read`,
  },);
  /**
   How many commits the first worktree had before the pair.
   */
  const before: string = await inside({
    machine,
    script: `cd ${worktreePaths.first} && /usr/bin/git rev-list --count HEAD`,
  },);
  await commit({
    machine,
    release: releasePaths.first,
    worktree: worktreePaths.first,
    change: 'window-a',
    file: 'a.txt',
    background: true,
  },);
  await commit({
    machine,
    release: releasePaths.first,
    worktree: worktreePaths.first,
    change: 'window-b',
    file: 'b.txt',
    background: true,
  },);
  /**
   What the first commit of the pair answered once it ended.
   */
  const first: WaitedCommit = await waited({
    machine,
    change: 'window-a',
  },);
  /**
   What the second commit of the pair answered once it ended.
   */
  const secondOfThePair: WaitedCommit = await waited({
    machine,
    change: 'window-b',
  },);
  /**
   How many commits the first worktree had after the pair.
   */
  const after: string = await inside({
    machine,
    script: `cd ${worktreePaths.first} && /usr/bin/git rev-list --count HEAD`,
  },);
  /**
   How many commits the pair added.
   */
  const added: number = Number(after.trim(),) - Number(before.trim(),);
  console.log(`the pair of commits at once: ${String(first.status)} and ${String(secondOfThePair.status)}; commits ${before.trim()} to ${after.trim()} (${String(added)} added)`);
  if (first.status !== 0) problems.push(`the concurrent commit window-a did not land: ${first.printed}`);
  if (secondOfThePair.status !== 0) problems.push(`the concurrent commit window-b did not land: ${secondOfThePair.printed}`);
  if (added !== 2) problems.push(`the pair added ${String(added)} commits, not two`);
  /**
   The units the manager lists as failed after the pair.
   */
  const failedAfter: readonly string[] = await failedUnits({ machine, },);
  if (failedAfter.length > 0) problems.push(`after the pair the manager left ${failedAfter.join('; ')} failed`);
  // What: What every unit did, so a run that fails says which start it was.
  /**
   What every wrapper unit's own journal lines say, from the install step onwards.
   */
  const summary: string = await inside({
    machine,
    script: `journalctl --no-pager --output=short-iso --since=@${since} --unit 'monochromatic-git-policy-cli@*' 2>/dev/null | grep -E 'started release=|retire requested|203/EXEC|start-limit|exiting reason' | tail -n 30`,
  },);
  console.log(`every unit's starts and endings:\n${summary}`);
  return problems;
}

/**
 What this run was asked to do, read before any machine exists.
 */
const {
  image,
  olderWrapper,
} = options({ argv: process.argv
  .slice(2,), });

/**
 The release built from this worktree, which the caller builds before it runs this file.
 */
const candidate: string = process.env
  .GPC_CANDIDATE_WRAPPER
  ?? join(
    import.meta.dirname,
    '..',
    'target',
    'debug',
    'monochromatic-git-policy-cli',
  );

/**
 The machine this proof runs on, whose removal waits for the end of the run's containers.
 */
await using running = await startMachine({
  image,
  units: machineUnits,
},);

/**
 Everything this run found wrong, in the order the steps found it.
 */
const problems: readonly string[] = await prove({
  machine: running.machine,
  candidate,
  olderReleaseWrapper: olderWrapper,
},);
if (problems.length > 0) throw new NativeVerificationError(`THE PROOF FAILED: ${problems.join('; ')}`);
console.log('THE PROOF PASSED: the instance the older release loaded started as this release, and the pair of commits each landed once');
