/**
 Test fixture for the daily update's package checks: a repository in memory, scripted package checks that answer
 differently before and after the update, and a recording stand-in for `gh`.

 Nothing here touches the network, git, the desktop or the real `gh`.

 @module
 */

import type {
  CheckEffects,
  DailyUpdateEffects,
  GhRequest,
  GhResult,
  PackageCheck,
  RenderedIssue,
  UpdateCommand,
} from './testing.ts';

/**
 Absolute path of the pretend repository.
 */
export const FIXTURE_REPOSITORY = '/repo';

/**
 Absolute path of the pretend machine-global lock.
 */
export const FIXTURE_GLOBAL_LOCK = '/config/toolchain.lock.json';

/**
 What a scripted check does in one pass.
 */
export type ScriptedPass = {
  /**
   Exit status the check ends with.
   */
  readonly exitCode: number;
  /**
   Whether the check is stopped for running past its time bound.
   */
  readonly timedOut?: boolean;
  /**
   Output the check printed.
   */
  readonly output?: string;
  /**
   Seconds the check took.
   */
  readonly seconds?: number;
};

/**
 What a scripted check does before and after the update.
 */
export type ScriptedCheck = {
  /**
   The pass before the update commits.
   */
  readonly before: ScriptedPass;
  /**
   The pass after the update commits.
   */
  readonly after: ScriptedPass;
};

/**
 A pass that succeeds.
 */
export const PASSES: ScriptedPass = { exitCode: 0, };

/**
 What the fixture recorded.
 */
export type FixtureWorld = {
  /**
   Labels of the update commands that ran, in order.
   */
  readonly ran: string[];
  /**
   File texts by absolute path.
   */
  readonly files: Map<string, string>;
  /**
   Commits made, with the text of every tracked file at that moment.
   */
  readonly commits: {
    readonly id: string;
    readonly message: string;
    readonly paths: readonly string[]
  }[];
  /**
   Notifications sent.
   */
  readonly notifications: { readonly title: string; readonly body: string; }[];
  /**
   Package checks that ran, in order, each with the pass it ran in.
   */
  readonly checksRun: { readonly key: string; readonly pass: 'after' | 'before'; }[];
  /**
   Requests the stand-in `gh` received.
   */
  readonly ghCalls: GhRequest[];
  /**
   Pending issues the run persisted.
   */
  pending: readonly RenderedIssue[];
  /**
   Most package checks that were running at one moment.
   */
  maximumInFlight: number;
  /**
   Texts of files at past commits: commit id, then repository-relative path.
   */
  readonly history: Map<string, Map<string, string>>;
};

/**
 Options of the fixture.
 */
export type FixtureOptions = {
  /**
   Tracked file texts by repository-relative path.
   */
  readonly tracked: Readonly<Record<string, string>>;
  /**
   Checks the repository offers, as task names `//package/x/y:task`, with how each behaves.
   */
  readonly checks: Readonly<Record<string, ScriptedCheck>>;
  /**
   File edits per update command label.
   */
  readonly edits?: Readonly<Record<string, Readonly<Record<string, string>>>>;
  /**
   Answers of the stand-in `gh`; the default answers an empty issue list and a created issue.
   */
  readonly gh?: (request: GhRequest,) => GhResult;
  /**
   Issues left pending by an earlier run.
   */
  readonly pending?: readonly RenderedIssue[];
  /**
   Whether listing the checks throws.
   */
  readonly listingFails?: boolean;
  /**
   Milliseconds the clock moves for every second a check reports.
   */
  readonly clockPerSecond?: number;
};

/**
 Answers a `gh` call the way a repository with no open issues does.

 @param request - the call

 @returns an empty list for `issue list`, a created-issue address for `issue create`, success for the rest

 @example
 ```ts
 answerLikeAnEmptyTracker({ args: ['issue', 'list',], stdin: '', },).stdout; // '[]'
 ```
 */
export function answerLikeAnEmptyTracker(request: GhRequest,): GhResult {
  if (request.args[1] === 'list')
    return { exitCode: 0, stdout: '[]', stderr: '', };
  if (request.args[1] === 'create') {
    return {
      exitCode: 0,
      stdout: 'https://github.com/example/repository/issues/501\n',
      stderr: '',
    };
  }
  return { exitCode: 0, stdout: '', stderr: '', };
}

/**
 Builds the world and both effect sets over it.

 @param options - tracked files, scripted checks, edits and `gh` answers

 @returns the effects of the update, the effects of the checks, and the world they act on

 @example
 ```ts
 const { effects, checkEffects, world, } = fixture({ tracked: {}, checks: {}, },);
 ```
 */
export function fixture(options: FixtureOptions,): {
  readonly effects: DailyUpdateEffects;
  readonly checkEffects: CheckEffects;
  readonly world: FixtureWorld;
} {
  /**
   Tracked paths in the repository.
   */
  const trackedPaths = Object.keys(options.tracked,);
  /**
   The world the effects act on.
   */
  const world: FixtureWorld = {
    ran: [],
    files: new Map(Object.entries(options.tracked,).map(([path, text,],) =>
      [
        `${FIXTURE_REPOSITORY}/${path}`,
        text,
      ] as const
    ),),
    commits: [],
    notifications: [],
    checksRun: [],
    ghCalls: [],
    pending: options.pending ?? [],
    maximumInFlight: 0,
    history: new Map(),
  };
  world.files.set(FIXTURE_GLOBAL_LOCK, '{"tools":{}}',);
  world.history.set('c0', snapshotOfTracked(world,),);
  /**
   Whether any update command has run, which is what separates the passes.
   */
  const state = { updated: false, head: 'c0', clock: 0, inFlight: 0, };
  /**
   Paths that became dirty because a command edited them.
   */
  const edited = new Set<string>();
  /**
   Takes the text of every tracked file now.

   @param snapshotWorld - the world

   @returns repository-relative path to text
   */
  function snapshotOfTracked(snapshotWorld: FixtureWorld,): Map<string, string> {
    return new Map(trackedPaths.map((path,) => [path, snapshotWorld.files.get(`${FIXTURE_REPOSITORY}/${path}`,) ?? '',] as const),);
  }
  const effects: DailyUpdateEffects = {
    repository: FIXTURE_REPOSITORY,
    runCommand: (command: UpdateCommand,) => {
      state.updated = true;
      world.ran.push(command.label,);
      for (const [path, text,] of Object.entries(options.edits?.[command.label] ?? {},)) {
        world.files.set(path.startsWith('/',) ? path : `${FIXTURE_REPOSITORY}/${path}`, text,);
        edited.add(path,);
      }
      return Promise.resolve({ exitCode: 0, timedOut: false, outputTail: '', },);
    },
    dirtyPaths: () => Promise.resolve([...edited,],),
    trackedPaths: () => Promise.resolve(trackedPaths,),
    readText: (absolutePath,) => Promise.resolve(world.files.get(absolutePath,) ?? '',),
    writeText: ({ absolutePath, text, },) => {
      world.files.set(absolutePath, text,);
      return Promise.resolve();
    },
    commit: ({ message, paths, },) => {
      state.head = `c${world.commits.length + 1}`;
      world.commits.push({ id: state.head, message, paths, },);
      world.history.set(state.head, snapshotOfTracked(world,),);
      for (const path of paths) {
        edited.delete(path,);
      }
      return Promise.resolve();
    },
    headCommit: () => Promise.resolve(state.head,),
    notify: (request,) => {
      world.notifications.push(request,);
      return Promise.resolve();
    },
    log: () => undefined,
  };
  /**
   Scripted behavior of a check by its task name.
   */
  const scripted = new Map(Object.entries(options.checks,),);
  /**
   Task names the listing returns.
   */
  const taskNames = Object.keys(options.checks,);
  const checkEffects: CheckEffects = {
    listCheckTaskNames: () => options.listingFails === true
      ? Promise.reject(new Error('mise tasks ls failed',),)
      : Promise.resolve(taskNames,),
    runCheck: async (check: PackageCheck,) => {
      /**
       Key the script is stored under.
       */
      const key = `//${check.package}:${check.task}`;
      /**
       Pass the check runs in.
       */
      const pass = state.updated ? 'after' : 'before';
      world.checksRun.push({ key, pass, },);
      state.inFlight += 1;
      world.maximumInFlight = Math.max(world.maximumInFlight, state.inFlight,);
      // Yielding lets checks that start together overlap, so the pool bound is what the test observes.
      await Promise.resolve();
      await Promise.resolve();
      state.inFlight -= 1;
      /**
       What the script says for this pass.
       */
      const script = scripted.get(key,)?.[pass];
      const seconds = script?.seconds ?? 1;
      state.clock += seconds * (options.clockPerSecond ?? 1_000);
      return {
        exitCode: script?.exitCode ?? 0,
        timedOut: script?.timedOut ?? false,
        seconds,
        outputHead: script?.output ?? '',
      };
    },
    fileAt: ({ commit, path, },) => Promise.resolve(world.history.get(commit,)?.get(path,) ?? '',),
    gh: (request,) => {
      world.ghCalls.push(request,);
      return Promise.resolve((options.gh ?? answerLikeAnEmptyTracker)(request,),);
    },
    readPendingIssues: () => Promise.resolve(world.pending,),
    writePendingIssues: (issues,) => {
      world.pending = issues;
      return Promise.resolve();
    },
    now: () => state.clock,
  };
  return { effects, checkEffects, world, };
}
