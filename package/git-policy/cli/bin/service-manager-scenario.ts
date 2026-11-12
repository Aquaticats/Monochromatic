/**
 The steps every arm of the reload proof runs: one commit through one release's wrapper, in one worktree, with the state of
 the desktop stand-in read after each of them.

 A release change and a second worktree are what the old template made a change of the text the manager read, and four
 callers at once is what the incident's manager had when one of its reloads ended the session.
 */
import {
  askedOrNothing,
  inside,
  type Machine,
  releasePaths,
  run,
  worktreePaths,
} from './service-manager-machine.ts';
import {
  describe,
  desktop,
  isAlive,
  reloads,
  startDesktop,
} from './service-manager-desktop.ts';
import {
  makeWorktrees,
  starter,
} from './service-manager-worktrees.ts';

/**
 How many lines of a command's answer a refused step reports, which is enough to say what the wrapper answered.
 */
const reportedTailLines = 6;

/**
 What one arm did, which is what the judgement reads.
 */
export type ArmReport = {
  /**
   The arm's name, which its report leads with.
   */
  readonly name: string;
  /**
   The manager's own record of every reload it ran during the arm's steps.
   */
  readonly reloads: readonly string[];
  /**
   One line per step, in the order they ran.
   */
  readonly steps: readonly string[];
  /**
   The commits that did not land.
   */
  readonly failures: readonly string[];
  /**
   The steps after which the unrelated desktop service and its target were gone.
   */
  readonly deaths: readonly string[];
};

/**
 What:
  The release a machine is seeded with before an arm runs, or the fact that it is not seeded at all.

 Why:
  The control arm runs a fresh machine, and the migration arm runs one that the release from `main` provisioned first; a
  tagged value says which without an absent property, which this repository's exact optional types refuse to assign.
 */
export type Seed = { readonly kind: 'none'; } | {
  readonly kind: 'release';
  /**
   The executable of the release from `main`.
   */
  readonly wrapper: string;
};

/**
 What an arm keeps while it runs: its machine, its name, and the report it is filling.
 */
type ArmState = {
  /**
   The machine this arm owns, which its steps name.
   */
  readonly machine: Machine;
  /**
   The arm's name, which its report leads with.
   */
  readonly name: string;
  /**
   One line per step, in the order they ran.
   */
  readonly steps: string[];
  /**
   The commits that did not land.
   */
  readonly failures: string[];
  /**
   The steps after which the unrelated desktop service and its target were gone.
   */
  readonly deaths: string[];
  /**
   How many reloads the machine's manager had run before this arm's steps.
   */
  before: number;
  /**
   The reloads the manager ran during this arm's steps, read at the end of them.
   */
  reloads: readonly string[];
};

/**
 One step of the scenario: a commit through one release in one worktree.
 */
type Step = {
  /**
   The step's name, which its line in the report leads with.
   */
  readonly label: string;
  /**
   The release's directory.
   */
  readonly release: string;
  /**
   The worktree the commit is made in.
   */
  readonly worktree: string;
  /**
   The text the commit adds.
   */
  readonly change: string;
};

/**
 One arm of the proof: what it does before its steps, and the steps themselves.
 */
export type Arm = {
  /**
   Puts the arm's release at both of its paths, and seeds the machine when this arm is the migration.
   */
  readonly prepare: (input: {
    readonly wrapper: string;
    readonly seed: Seed
  }) => Promise<void>;
  /**
   Runs the scenario's steps and answers what the arm did.
   */
  readonly run: (input: { readonly probeDeadline: string; }) => Promise<ArmReport>;
};

/**
 One commit through one release's wrapper, in one worktree.

 @param machine - the machine the commit runs on

 @param release - the directory holding the release's `git`

 @param worktree - the worktree the commit is made in

 @param change - the text the commit adds to `a.txt`

 @returns whether the commit landed, and its last lines of output

 @example
 ```ts
 await commit({ machine, release: '/opt/release-a', worktree: '/work/one', change: 'one' });
 ```
 */
async function commit({
  machine,
  release,
  worktree,
  change,
}: {
  readonly machine: Machine;
  readonly release: string;
  readonly worktree: string;
  readonly change: string;
}): Promise<{
  readonly landed: boolean;
  readonly output: string;
}> {
  /**
   What the commit answered; its own status is written as the last line so a refused commit is read and not thrown.
   */
  const answer = await askedOrNothing({
    machine,
    script: [
      `printf 'base\\n${change}\\n' > ${worktree}/a.txt`,
      `cd ${worktree}`,
      'set +e',
      `${release}/git add a.txt`,
      `${release}/git commit --message '${change}' -- a.txt`,
      'echo "status=$?"',
    ].join('\n',),
  });
  /**
   The last lines of the answer, which say what a refused commit was refused for.
   */
  const tail = answer.split('\n',)
    .slice(-reportedTailLines,)
    .join(' | ',);
  return {
    // The status the script wrote after `status=`, as a number, where an unreadable one counts as a commit that did not land.
    landed: Math.trunc(
      Number(answer.slice(answer.lastIndexOf('status=',) + 'status='.length,),),
    ) === 0,
    output: tail,
  };
}

/**
 Holds one arm's machine and the report it is filling.

 @param machine - the machine this arm owns

 @param name - the arm's name

 @returns the arm, whose two calls run its steps

 @example
 ```ts
 const arm = newArm({ machine, name: 'main' });
 ```
 */
export function newArm({
  machine,
  name,
}: {
  readonly machine: Machine;
  readonly name: string
}): Arm {
  /**
   What this arm keeps while it runs.
   */
  const state: ArmState = {
    machine,
    name,
    steps: [],
    failures: [],
    deaths: [],
    before: 0,
    reloads: [],
  };
  return Object.freeze({
    prepare(input: {
      readonly wrapper: string;
      readonly seed: Seed
    }): Promise<void> {
      return prepareArm({
        state,
        wrapper: input.wrapper,
        seed: input.seed,
      });
    },
    run(input: { readonly probeDeadline: string; }): Promise<ArmReport> {
      return runArm({
        state,
        probeDeadline: input.probeDeadline,
      });
    },
  });
}

/**
 What:
  Puts the arm's release at both of its paths, and seeds the machine with the release from `main` when this arm is the
  migration.

 Why:
  A machine that has that release's template installed is the migration this change has to survive, and the seeding runs
  before the stand-in for a desktop starts, so the seeding's own reload cannot be mistaken for what this arm did.

 @param state - the arm's machine and report

 @param wrapper - the executable of the release this arm runs

 @param seed - the release from `main`, which provisions the machine first when this arm is the migration

 @example
 ```ts
 await prepareArm({ state, wrapper: '/tmp/git', seed: { kind: 'none' } });
 ```
 */
async function prepareArm({
  state,
  wrapper,
  seed,
}: {
  readonly state: ArmState;
  readonly wrapper: string;
  readonly seed: Seed;
}): Promise<void> {
  await run({
    args: [
      'podman',
      'cp',
      wrapper,
      `${state.machine
        .container}:${releasePaths.first}/git`,
    ],
  });
  await inside({
    machine: state.machine,
    script: `cp ${releasePaths.first}/git ${releasePaths.second}/git && chmod +x ${releasePaths.first}/git ${releasePaths.second}/git`,
  });
  await makeWorktrees({ machine: state.machine, });
  if (seed.kind === 'none') return;
  await run({
    args: [
      'podman',
      'cp',
      seed.wrapper,
      `${state.machine
        .container}:${releasePaths.seed}/git`,
    ],
  });
  /**
   What the seeding commit printed.
   */
  const seeded = await commit({
    machine: state.machine,
    release: releasePaths.seed,
    worktree: worktreePaths.first,
    change: 'seed',
  });
  state.steps
    .push(`the machine was seeded by the release from main, as one that has its template installed: ${seeded.output}`);
  /**
   The `ExecStart` line the seeding left, which is the earlier release's absolute path.
   */
  const left = await inside({
    machine: state.machine,
    script: 'grep --max-count=1 "^ExecStart=" /run/user/0/systemd/user/monochromatic-git-policy-cli@.service 2>/dev/null || echo none',
  });
  state.steps
    .push(`the template the seeding left: ${left}`);
}

/**
 Reads the desktop's state and records it, which is the measurement every step is judged on.

 @param state - the arm's machine and report

 @param label - the step this reading belongs to

 @example
 ```ts
 await recordStep({ state, label: 'first provisioning and its start' });
 ```
 */
async function recordStep({
  state,
  label,
}: {
  readonly state: ArmState;
  readonly label: string
}): Promise<void> {
  /**
   The desktop's state now.
   */
  const read = await desktop({ machine: state.machine, });
  state.steps
    .push(`${label}: ${describe({ state: read, })}`);
  if (!isAlive(read,)) state.deaths
    .push(`${label}: ${describe({ state: read, })}`);
}

/**
 Runs one commit and records what it did to the desktop.

 @param state - the arm's machine and report

 @param step - the commit's name, release, worktree and change

 @example
 ```ts
 await attemptCommit({ state, step: { label: 'an upgrade', release: '/opt/release-b', worktree: '/work/one', change: 'b1' } });
 ```
 */
async function attemptCommit({
  state,
  step,
}: {
  readonly state: ArmState;
  readonly step: Step
}): Promise<void> {
  /**
   What the commit answered.
   */
  const result = await commit({
    machine: state.machine,
    release: step.release,
    worktree: step.worktree,
    change: step.change,
  });
  if (!result.landed) state.failures
    .push(`${step.label}: the commit did not land: ${result.output}`);
  await recordStep({
    state,
    label: step.label,
  });
}

/**
 Runs four callers at once, in both worktrees and through both releases, as the incident's manager had.

 @param state - the arm's machine and report

 @example
 ```ts
 await callersStep({ state });
 ```
 */
async function callersStep({ state, }: { readonly state: ArmState; }): Promise<void> {
  /**
   One line of shell per caller, each backgrounded.
   */
  const lines = [
    starter({
      release: releasePaths.first,
      worktree: worktreePaths.first,
      change: 'p1',
    }),
    starter({
      release: releasePaths.second,
      worktree: worktreePaths.first,
      change: 'p2',
    }),
    starter({
      release: releasePaths.first,
      worktree: worktreePaths.second,
      change: 'p3',
    }),
    starter({
      release: releasePaths.second,
      worktree: worktreePaths.second,
      change: 'p4',
    }),
  ];
  /**
   What the four callers printed, one line per caller as it ends.
   */
  const parallel = await inside({
    machine: state.machine,
    script: `${lines.join('\n',)}\nwait`,
  });
  /**
   The callers' lines, one per caller as it ended.
   */
  const reported = parallel.split('\n',);
  for (const line of reported) {
    if (line === '') continue;
    if (!line.endsWith(' 0',)) state.failures
      .push(`a caller at once: ${line}`);
  }
  await recordStep({
    state,
    label: 'several callers at once',
  });
}

/**
 What:
  Runs the scenario's commits one after another, because each one reads the state the one before it left.

 Why:
  A loop that awaited inside itself would read as steps that could run at once, and these cannot: the second commit of a
  worktree lands on the branch the first one moved.

 @param state - the arm's machine and report

 @param steps - the commits left to run

 @param index - which of them runs next

 @example
 ```ts
 await commitSteps({ state, steps, index: 0 });
 ```
 */
async function commitSteps({
  state,
  steps,
  index,
}: {
  readonly state: ArmState;
  readonly steps: readonly Step[];
  readonly index: number;
}): Promise<void> {
  /**
   The step to run now, absent once every step has run.
   */
  const step = steps[index];
  if (step === undefined) return;
  await attemptCommit({
    state,
    step,
  });
  await commitSteps({
    state,
    steps,
    index: index + 1,
  });
}

/**
 What:
  Runs the scenario's steps on one arm's machine and answers what it did.

 Why:
  Every step reads the desktop stand-in, so an arm that ended it says at which step, and one that kept it says so for every
  step.

 @param state - the arm's machine and report

 @param probeDeadline - how long the desktop stand-in's own start may take

 @returns this arm's report

 @example
 ```ts
 const report = await runArm({ state, probeDeadline: '200ms' });
 ```
 */
async function runArm({
  state,
  probeDeadline,
}: {
  readonly state: ArmState;
  readonly probeDeadline: string
}): Promise<ArmReport> {
  await startDesktop({
    machine: state.machine,
    probeDeadline,
  });
  state.before = (await reloads({ machine: state.machine, })).length;
  await recordStep({
    state,
    label: 'first provisioning and its start',
  });
  await commitSteps({
    state,
    steps: [
      {
        label: 'the first commit of release A',
        release: releasePaths.first,
        worktree: worktreePaths.first,
        change: 'a1',
      },
      {
        label: 'an upgrade to release B',
        release: releasePaths.second,
        worktree: worktreePaths.first,
        change: 'b1',
      },
      {
        label: 'a downgrade to release A',
        release: releasePaths.first,
        worktree: worktreePaths.first,
        change: 'a2',
      },
      {
        label: 'release B again',
        release: releasePaths.second,
        worktree: worktreePaths.first,
        change: 'b2',
      },
      {
        label: 'release A in a second worktree',
        release: releasePaths.first,
        worktree: worktreePaths.second,
        change: 'a3',
      },
      {
        label: 'release B in the second worktree',
        release: releasePaths.second,
        worktree: worktreePaths.second,
        change: 'b3',
      },
    ],
    index: 0,
  });
  await callersStep({ state, });
  /**
   Every reload this machine's manager ran, of which this arm's steps are the ones after the seeding.
   */
  const asked = await reloads({ machine: state.machine, });
  state.steps
    .push(`the manager was asked to re-read its units ${String(asked.length - state.before)} time(s) during this arm's steps, ${String(asked.length)} in this machine's whole life`);
  state.reloads = asked.slice(state.before,);
  return {
    name: state.name,
    reloads: state.reloads,
    steps: state.steps,
    failures: state.failures,
    deaths: state.deaths,
  };
}
