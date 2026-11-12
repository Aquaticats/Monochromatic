/**
 Test fixture for the daily update's package checks: a repository in memory, scripted package checks that answer
 differently before and after the update, and a recording stand-in for `gh`.

 Nothing here touches the network, git, the desktop or the real `gh`.

 @module
 */

import {
  type CheckEffects,
  type CheckOutcome,
  type DailyUpdateEffects,
  type GhRequest,
  type GhResult,
  type PackageCheck,
  type RenderedIssue,
  type UpdateCommand,
  firstErrorOf,
  NO_ERROR_IN_OUTPUT,
} from '../dist/final/node/testing.mjs';

/**
 Absolute path of the pretend repository.
 */
export const FIXTURE_REPOSITORY = '/repo';

/**
 Absolute path of the pretend machine-global lock.
 */
export const FIXTURE_GLOBAL_LOCK = '/config/toolchain.lock.json';

/**
 Milliseconds the pretend clock moves for each second a scripted check reports.
 */
const MILLISECONDS_PER_REPORTED_SECOND = 1_000;

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
 A commit made by the fixture, with the paths it holds.
 */
export type FixtureCommit = {
  /**
   Commit id.
   */
  readonly id: string;
  /**
   Commit message.
   */
  readonly message: string;
  /**
   Repository-relative paths committed.
   */
  readonly paths: readonly string[];
};

/**
 A notification the fixture received.
 */
export type FixtureNotification = {
  /**
   Title.
   */
  readonly title: string;
  /**
   Body.
   */
  readonly body: string;
};

/**
 A package check that ran.
 */
export type FixtureCheckRun = {
  /**
   Task name of the check, `//package/x/y:task`.
   */
  readonly key: string;
  /**
   Pass the check ran in.
   */
  readonly pass: 'after' | 'before';
};

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
   Commits made.
   */
  readonly commits: FixtureCommit[];
  /**
   Notifications sent.
   */
  readonly notifications: FixtureNotification[];
  /**
   Package checks that ran, in order, each with the pass it ran in.
   */
  readonly checksRun: FixtureCheckRun[];
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
   Texts of tracked files at past commits: commit id, then repository-relative path.
   */
  readonly history: Map<string, ReadonlyMap<string, string>>;
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
   Repository-relative paths that already have uncommitted changes when the run starts.
   */
  readonly dirty?: readonly string[];
};

/**
 What the fixture hands to a test.
 */
export type Fixture = {
  /**
   Effects of the update.
   */
  readonly effects: DailyUpdateEffects;
  /**
   Effects of the package checks.
   */
  readonly checkEffects: CheckEffects;
  /**
   What the effects recorded.
   */
  readonly world: FixtureWorld;
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
  if (request.args[1] === 'list') {
    return {
      exitCode: 0,
      stdout: '[]',
      stderr: '',
    };
  }
  if (request.args[1] === 'create') {
    return {
      exitCode: 0,
      stdout: 'https://github.com/example/repository/issues/501\n',
      stderr: '',
    };
  }
  return {
    exitCode: 0,
    stdout: '',
    stderr: '',
  };
}

/**
 Takes the text of every tracked file now.

 @param world - the world

 @param trackedPaths - repository-relative paths of the tracked files

 @returns repository-relative path to text

 @example
 ```ts
 const snapshot = snapshotOfTracked({ world, trackedPaths: ['a',], },);
 ```
 */
export function snapshotOfTracked({
  world,
  trackedPaths,
}: {
  readonly world: FixtureWorld;
  readonly trackedPaths: readonly string[];
},): ReadonlyMap<string, string> {
  return new Map(trackedPaths.map(function textOf(path,) {
    return [
      path,
      world.files
        .get(`${FIXTURE_REPOSITORY}/${path}`,)
        ?? '',
    ] as const;
  },),);
}

/**
 Builds the empty world: tracked files, the global lock and the first commit.

 @param options - the fixture options

 @returns the world

 @example
 ```ts
 const world = emptyWorld({ tracked: {}, checks: {}, },);
 ```
 */
export function emptyWorld(options: FixtureOptions,): FixtureWorld {
  /**
   The world with the tracked files in place.
   */
  const world: FixtureWorld = {
    ran: [],
    files: new Map(Object.keys(options.tracked,)
      .map(function absolute(path,) {
        return [
          `${FIXTURE_REPOSITORY}/${path}`,
          options.tracked[path] ?? '',
        ] as const;
      },),),
    commits: [],
    notifications: [],
    checksRun: [],
    ghCalls: [],
    pending: options.pending ?? [],
    maximumInFlight: 0,
    history: new Map(),
  };
  world.files
    .set(
    FIXTURE_GLOBAL_LOCK,
    '{"tools":{}}',
    );
  world.history
    .set(
    'c0',
    snapshotOfTracked({
      world,
      trackedPaths: Object.keys(options.tracked,),
    },),
    );
  return world;
}

/**
 Stands in for the journal: the fixture keeps none.

 @example
 ```ts
 keepNoJournal();
 ```
 */
export function keepNoJournal(): void {
  // The fixture keeps no journal.
}

/**
 Builds the effects of the update over a world.

 @param options - the fixture options

 @param world - the world

 @param state - which pass the run is in and the commit it is at

 @returns the effects

 @example
 ```ts
 const effects = updateEffects({ options, world, state: { updated: false, head: 'c0', }, },);
 ```
 */
export function updateEffects({
  options,
  world,
  state,
}: {
  readonly options: FixtureOptions;
  readonly world: FixtureWorld;
  readonly state: {
    updated: boolean;
    head: string
  };
},): DailyUpdateEffects {
  /**
   Tracked paths in the repository.
   */
  const trackedPaths = Object.keys(options.tracked,);
  /**
   Paths that became dirty because a command edited them.
   */
  const edited = new Set<string>(options.dirty,);
  return {
    repository: FIXTURE_REPOSITORY,
    runCommand: function runScripted(command: UpdateCommand,) {
      state.updated = true;
      world.ran
        .push(command.label,);
      for (const [path, text,] of Object.entries(options.edits?.[command.label] ?? {},)) {
        world.files
          .set(
          path.startsWith('/',) ? path : `${FIXTURE_REPOSITORY}/${path}`,
          text,
          );
        edited.add(path,);
      }
      return Promise.resolve({
        exitCode: 0,
        timedOut: false,
        outputTail: '',
      },);
    },
    dirtyPaths: function dirtyPaths() {
      return Promise.resolve([...edited,],);
    },
    trackedPaths: function listTracked() {
      return Promise.resolve(trackedPaths,);
    },
    readText: function readText(absolutePath,) {
      return Promise.resolve(world.files
        .get(absolutePath,)
        ?? '',);
    },
    writeText: function writeText({
      absolutePath,
      text,
    },) {
      world.files
        .set(
        absolutePath,
        text,
        );
      return Promise.resolve();
    },
    commit: function commit({
      message,
      paths,
    },) {
      state.head = `c${world.commits
        .length
        + 1}`;
      world.commits
        .push({
          id: state.head,
          message,
          paths,
        },);
      world.history
        .set(
        state.head,
        snapshotOfTracked({
          world,
          trackedPaths,
        },),
        );
      for (const path of paths) {
        edited.delete(path,);
      }
      return Promise.resolve();
    },
    headCommit: function headCommit() {
      return Promise.resolve(state.head,);
    },
    notify: function notify(request,) {
      world.notifications
        .push(request,);
      return Promise.resolve();
    },
    log: keepNoJournal,
  };
}

/**
 Builds the effects of the package checks over a world.

 @param options - the fixture options

 @param world - the world

 @param state - which pass the run is in

 @returns the effects

 @example
 ```ts
 const checkEffects = packageCheckEffects({ options, world, state: { updated: false, head: 'c0', }, },);
 ```
 */
export function packageCheckEffects({
  options,
  world,
  state,
}: {
  readonly options: FixtureOptions;
  readonly world: FixtureWorld;
  readonly state: {
    updated: boolean;
    head: string
  };
},): CheckEffects {
  /**
   Scripted behavior of a check by its task name.
   */
  const scripted = new Map(Object.entries(options.checks,),);
  /**
   Checks running now and the pretend clock, which moves by the seconds each check reports.
   */
  const running = {
    inFlight: 0,
    clock: 0,
  };
  return {
    listCheckTaskNames: function listCheckTaskNames() {
      return options.listingFails === true
        ? Promise.reject(new Error('mise tasks ls failed',),)
        : Promise.resolve(Object.keys(options.checks,),);
    },
    runCheck: async function runCheck(check: PackageCheck,): Promise<CheckOutcome> {
      /**
       Key the script is stored under.
       */
      const key = `//${check.package}:${check.task}`;
      /**
       Pass the check runs in.
       */
      const pass = state.updated ? 'after' : 'before';
      world.checksRun
        .push({
          key,
          pass,
        },);
      running.inFlight += 1;
      world.maximumInFlight = Math.max(
        world.maximumInFlight,
        running.inFlight,
      );
      // Yielding lets checks that start together overlap, so the pool bound is what the test observes.
      await Promise.resolve();
      await Promise.resolve();
      running.inFlight -= 1;
      /**
       What the script says for this pass.
       */
      const script = scripted.get(key,)?.[pass];
      /**
       Seconds the check reports.
       */
      const seconds = script?.seconds ?? 1;
      running.clock += seconds * MILLISECONDS_PER_REPORTED_SECOND;
      /**
       The error the scripted output starts, which the package's own reading of an output answers the same way.
       */
      const found = firstErrorOf(script?.output ?? '',);
      return {
        exitCode: script?.exitCode ?? 0,
        timedOut: script?.timedOut ?? false,
        seconds,
        ...(found === NO_ERROR_IN_OUTPUT ? {} : { firstError: found, }),
      };
    },
    fileAt: function fileAt({
      commit,
      path,
    },) {
      return Promise.resolve(world.history
        .get(commit,)
        ?.get(path,)
        ?? '',);
    },
    gh: function gh(request,) {
      world.ghCalls
        .push(request,);
      return Promise.resolve((options.gh ?? answerLikeAnEmptyTracker)(request,),);
    },
    readPendingIssues: function readPendingIssues() {
      return Promise.resolve(world.pending,);
    },
    writePendingIssues: function writePendingIssues(issues,) {
      world.pending = issues;
      return Promise.resolve();
    },
    now: function now() {
      return running.clock;
    },
  };
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
export function fixture(options: FixtureOptions,): Fixture {
  /**
   The world the effects act on.
   */
  const world = emptyWorld(options,);
  /**
   Whether any update command has run, which is what separates the passes, and the commit the repository is at.
   */
  const state = {
    updated: false,
    head: 'c0',
  };
  return {
    effects: updateEffects({
      options,
      world,
      state,
    },),
    checkEffects: packageCheckEffects({
      options,
      world,
      state,
    },),
    world,
  };
}
