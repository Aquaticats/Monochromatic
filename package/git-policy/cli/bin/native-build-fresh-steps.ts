/**
 The steps that build the native wrapper executable and leave, at the path this build names, the program compiled from
 these sources. `native-build-fresh.ts` is the command line; this file is the work.

 `native:build`, `native:build:release`, `build` and the candidate arm of `test:service-manager` all hand a person, or a
 control, a program to start by path. A bare `cargo build` does not make that true on this machine:

 - A worktree of this repository is a btrfs snapshot of the checkout it was added from, so it holds that checkout's
   `target/` before anything is built here, with the program of another checkout at the very path the task names
   (`doc/troubleshooting/cargo-fresh-build-leaves-another-checkouts-program.md`).
 - Cargo decides whether to compile by modification times, and a build directory can arrive from another checkout: the
   wrapper's worktree add fills a new worktree's build directory from the source checkout's, cargo-shadow keys build
   directories by toolchain and workspace path, and a worktree removed and added again at one path finds the build
   directory of its previous life. A unit whose state was written after these sources were is fresh, and Cargo then
   uplifts that state's program over the named path.

 So this step removes this crate's own units from the build directory Cargo resolves and the program at the named path
 before it builds: Cargo's own `clean --package` can only leave this crate's units gone, and the build after it can then
 only produce this crate's program. It refuses when Cargo resolves a target directory that is not this package's own,
 because then the program lands elsewhere and the named path keeps an older program, and it fails, naming the path and
 what to do, when no program stands there afterwards.

 Measured 2026-11-16 in a worktree of this repository with `mise exec -- cargo`: `clean --offline --package
 monochromatic-git-policy-cli` took 0.21 s and removed the program at `target/debug/`; the build after it took 22.06 s,
 because only this crate's units were gone and every dependency build stayed. A build of the same worktree from an empty
 build directory took 1 m 51 s.

 In TS you'd write (pseudocode):
 ```ts
 // metadata -> refuse a foreign target directory -> remove the old program -> clean this crate -> build -> prove it is there
 ```
 */
import {
  realpath,
  rm,
  stat,
} from 'node:fs/promises';
import { join } from 'node:path';
import { types } from 'node:util';

import { caughtValueText } from '@monochromatic-dev/caught-value-core/ts';

import { isMissingPathError } from './native-gate-paths.ts';
import { runGateCommand } from './native-gate-run.ts';
import {
  memberOf,
  workspaceOf,
} from './native-build-fresh-workspace.ts';

//region Bounds

/**
 Milliseconds one `cargo metadata` may take: the add's own timing records read 0.76 s over 35 resolutions on 2026-11-16,
 and a bound rather than a measurement leaves a loaded host room.
 */
export const metadataDeadlineMilliseconds = 60_000;

/**
 Milliseconds the clean may take. It removed 17 files and 434 MiB in 0.21 s on 2026-11-16, so this is a bound and not a
 measurement: a stopped file system must not hold the build.
 */
export const cleanDeadlineMilliseconds = 600_000;

/**
 Milliseconds the build may take: this crate alone, with every dependency build already there, took 22.06 s on
 2026-11-16, and a build from an empty build directory took 1 m 51 s, so half an hour covers a loaded host.
 */
export const buildDeadlineMilliseconds = 1_800_000;

/**
 The three bounds one build runs under.
 */
export type BuildDeadlines = {
  /**
   How long one `cargo metadata` may take.
   */
  readonly metadata: number;
  /**
   How long the clean may take.
   */
  readonly clean: number;
  /**
   How long the build may take.
   */
  readonly build: number;
};

/**
 The bounds every build runs under, each beside the measurement it rests on above.
 */
export const defaultBuildDeadlines: BuildDeadlines = {
  metadata: metadataDeadlineMilliseconds,
  clean: cleanDeadlineMilliseconds,
  build: buildDeadlineMilliseconds,
};

//endregion Bounds

//region The bin this builds and the profile it builds

/**
 The executable this package publishes, which is the program every task here starts by path.
 */
export const defaultBinName = 'monochromatic-git-policy-cli';

/**
 The two profiles a task of this package names, and the directory Cargo puts each profile's final artifacts in.
 */
export const profileDirectories = {
  dev: 'debug',
  release: 'release',
} as const;

/**
 One of the two profiles a task of this package names.
 */
export type Profile = keyof typeof profileDirectories;

//endregion The bin this builds and the profile it builds

/**
 A build step that could not leave this crate's program at the path it names, with the words a person reads.
 */
export class NativeBuildFreshError extends Error {
  /**
   The name a reader of the failure sees, which is the same for every refusal of this step.
   */
  override readonly name = 'NativeBuildFreshError';
}

/**
 What one build is asked for.
 */
export type NativeBuildRequest = {
  /**
   The package directory to build in, which is its workspace root here.
   */
  readonly packageDirectory: string;
  /**
   The `cargo` to run, from `--cargo` or the search path's own.
   */
  readonly cargoProgram: string;
  /**
   The executable to build, from `--bin`.
   */
  readonly binName: string;
  /**
   The profile to build, from `--release` or its absence.
   */
  readonly profile: Profile;
  /**
   The environment the build runs with, which is the process's own unless a control hands another over.
   */
  readonly environment?: NodeJS.ProcessEnv;
  /**
   The bounds it runs under, which a control shortens so it proves a stopped step without waiting the real ones.
   */
  readonly deadlines?: BuildDeadlines;
};

/**
 What one build came to.
 */
export type NativeBuildResult = {
  /**
   The program this build left at the path it names.
   */
  readonly artifactPath: string;
  /**
   The directory Cargo resolves for final artifacts, which this step required to be this package's own `target/`.
   */
  readonly targetDirectory: string;
  /**
   The directory Cargo resolves for intermediate build state, when this Cargo names one.
   */
  readonly buildDirectory?: string;
  /**
   The package name this crate has in its own workspace, which is what the clean removes.
   */
  readonly member: string;
  /**
   Cargo's own standard output of the build, printed by the caller so a person reads it as before.
   */
  readonly buildStdout: string;
  /**
   Cargo's own standard error of the build, printed by the caller so a person reads it as before.
   */
  readonly buildStderr: string;
};

/**
 Refuses a target directory that is not this package's own, because the program would land outside the path a task names.

 @param packageDirectory - the package a task builds

 @param targetDirectory - the directory Cargo resolves for final artifacts

 @param movedBy - what moved that directory, in the words the failure uses

 @throws NativeBuildFreshError when that directory is not `<packageDirectory>/target`

 @example
 ```ts
 refuseAForeignTargetDirectory({ packageDirectory: '/repo/package/git-policy/cli', targetDirectory: '/elsewhere/target', movedBy: 'CARGO_TARGET_DIR names /elsewhere/target' });
 ```
 */
export function refuseAForeignTargetDirectory({
  packageDirectory,
  targetDirectory,
  movedBy,
}: {
  readonly packageDirectory: string;
  readonly targetDirectory: string;
  readonly movedBy: string;
}): void {
  /**
   The directory a person reads a task's path under.
   */
  const own = join(
    packageDirectory,
    'target',
  );
  if (targetDirectory === own) return;
  throw new NativeBuildFreshError(
    `Cargo puts final artifacts in ${targetDirectory}, which is not this package's own ${own} (${movedBy}), so a build here would leave the program a task names at ${own} untouched; unset CARGO_TARGET_DIR and build again`,
  );
}

/**
 Tells whether a path holds a regular file.

 @param path - the path to ask about

 @returns whether a regular file is there

 @example
 ```ts
 const there = await isRegularFile({ path: '/repo/package/git-policy/cli/target/debug/program' });
 ```
 */
async function isRegularFile({ path }: { readonly path: string; }): Promise<boolean> {
  try {
    /**
     What the path holds, which is a file or not.
     */
    const seen = await stat(path,);
    return seen.isFile();
  }
  catch (caught) {
    // Absent is the answer this asks for; a path that cannot be read for any other reason is nobody's "no program".
    if (!isMissingPathError({ error: caught })) throw caught;
    return false;
  }
}

/**
 Runs Cargo and answers with what it printed, refusing with Cargo's own words when it did not end successfully.

 @param run - the program, its arguments, the directory, the bound and the environment

 @returns standard output and standard error of the run

 @throws NativeBuildFreshError when the run failed, ran past its bound or never started

 @example
 ```ts
 const printed = await cargoOrRefuse({ run: { program: 'cargo', args: ['clean'], workingDirectory: '/repo/package/git-policy/cli', deadlineMilliseconds: cleanDeadlineMilliseconds, environment: process.env } });
 ```
 */
async function cargoOrRefuse({ run }: {
  readonly run: {
    readonly program: string;
    readonly args: readonly string[];
    readonly workingDirectory: string;
    readonly deadlineMilliseconds: number;
    readonly environment?: NodeJS.ProcessEnv;
  };
}): Promise<{
  readonly stdout: string;
  readonly stderr: string
}> {
  /**
   What the program answered.
   */
  const outcome = await runGateCommand({
    program: run.program,
    args: run.args,
    deadlineMilliseconds: run.deadlineMilliseconds,
    environment: run.environment ?? process.env,
    workingDirectory: run.workingDirectory,
  },);
  /**
   The command as a person would type it, which every failure of this step quotes.
   */
  const typed = [
    run.program,
    ...run.args
  ].join(' ',);
  if (outcome.timedOut) {
    throw new NativeBuildFreshError(
      `${typed} ran past the ${String(run.deadlineMilliseconds,)} ms this step allows it and was stopped; nothing was built, and the path a task names holds no program of this build`,
    );
  }
  if (outcome.status !== 0) {
    /**
     What the program said, which is the reason a person reads.
     */
    const said = `${outcome.stderr}${outcome.stdout}`.trim();
    /**
     How the outcome names a failure that never reached an exit, when it carries one.
     */
    const problem = (outcome.problem === undefined) ? '' : ` (${outcome.problem})`;
    /**
     What the program printed, when it printed anything.
     */
    const printed = (said === '') ? '' : `:\n${said}`;
    throw new NativeBuildFreshError(
      `${typed} ended with status ${String(outcome.status,)}${problem}${printed}`,
    );
  }
  return {
    stdout: outcome.stdout,
    stderr: outcome.stderr,
  };
}

/**
 Builds this crate in this package directory, and leaves at the path it names the program compiled from these sources.

 @param request - what to build and where

 @returns the path of the program and where Cargo put it

 @throws NativeBuildFreshError when Cargo moved the target directory, when the clean or the build failed, or when no
   program stands at the path afterwards

 @example
 ```ts
 const built = await buildFresh({
   packageDirectory: '/repo/package/git-policy/cli',
   cargoProgram: 'cargo',
   binName: defaultBinName,
   profile: 'dev',
 });
 ```
 */
export async function buildFresh(request: NativeBuildRequest): Promise<NativeBuildResult> {
  /**
   The package directory as Cargo names paths, so the manifest it reports is the one this step looks for.
   */
  const packageDirectory = await realpath(request.packageDirectory,);
  /**
   What Cargo says about this workspace, read instead of a configuration file this step would have to parse.
   */
  const described = await cargoOrRefuse({
    run: {
      program: request.cargoProgram,
      args: [
        'metadata',
        '--locked',
        '--offline',
        '--no-deps',
        '--format-version',
        '1',
      ],
      workingDirectory: packageDirectory,
      deadlineMilliseconds: (request.deadlines ?? defaultBuildDeadlines).metadata,
      ...(request.environment === undefined ? {} : { environment: request.environment }),
    },
  },);
  /**
   The four facts this step uses out of that answer.
   */
  const seen = workspaceOf(described.stdout,);
  /**
   The member this crate is, which the clean takes.
   */
  const member = memberOf({
    seen,
    packageDirectory,
  },);
  /**
   The variable that moves Cargo's target directory, which the refusal names when the environment holds one.
   */
  const movedVariable = request.environment
    ?.CARGO_TARGET_DIR;
  /**
   What moved the target directory, in the words the refusal uses.
   */
  const movedBy = ((movedVariable === undefined) || (movedVariable === ''))
    ? 'Cargo resolves it from its own configuration'
    : `CARGO_TARGET_DIR names ${movedVariable}`;
  refuseAForeignTargetDirectory({
    packageDirectory,
    targetDirectory: seen.targetDirectory,
    movedBy,
  },);
  /**
   The one path this build promises: the program of this profile, under the directory Cargo resolved.
   */
  const artifactPath = join(
    seen.targetDirectory,
    profileDirectories[request.profile],
    request.binName,
  );
  // A program standing here belongs to an earlier build or to the checkout this worktree was copied from, and this step
  //  never leaves one of those where a person reads this build's own.
  await rm(
    artifactPath,
    { force: true, },
  );
  /**
   The arguments every Cargo run of this step carries: the same ones a person's task carried before this step existed.
   */
  const common = {
    program: request.cargoProgram,
    workingDirectory: packageDirectory,
    ...(request.environment === undefined ? {} : { environment: request.environment }),
  };
  await cargoOrRefuse({
    run: {
      ...common,
      args: [
        'clean',
        '--offline',
        '--package',
        member,
      ],
      deadlineMilliseconds: (request.deadlines ?? defaultBuildDeadlines).clean,
    },
  },);
  /**
   The build's own output, which the caller prints as Cargo printed it.
   */
  const built = await cargoOrRefuse({
    run: {
      ...common,
      args: [
        'build',
        '--locked',
        '--offline',
        ...(request.profile === 'release' ? ['--release'] : []),
        '--bin',
        request.binName,
      ],
      deadlineMilliseconds: (request.deadlines ?? defaultBuildDeadlines).build,
    },
  },);
  if (!(await isRegularFile({ path: artifactPath, }))) {
    /**
     Where Cargo keeps intermediate state, which a person reads to say why no program appeared.
     */
    const buildDirectory = (seen.buildDirectory === undefined)
      ? 'Cargo named no build directory'
      : `Cargo's build directory is ${seen.buildDirectory}`;
    throw new NativeBuildFreshError(
      `the build ended successfully and ${artifactPath} holds no program; ${buildDirectory}, and the target directory is ${seen.targetDirectory}. Run \`cargo clean --package ${member}\` in ${packageDirectory} and build again, and report this if the program is still absent`,
    );
  }
  return {
    artifactPath,
    targetDirectory: seen.targetDirectory,
    ...(seen.buildDirectory === undefined ? {} : { buildDirectory: seen.buildDirectory }),
    member,
    buildStdout: built.stdout,
    buildStderr: built.stderr,
  };
}

/**
 Reports a caught value as the words a person reads.

 @param caught - what was thrown

 @returns the failure's own message, or the value's text when it is not one of this step's refusals

 @example
 ```ts
 const said = refusalText(new NativeBuildFreshError('no program'));
 ```
 */
export function refusalText(caught: unknown): string {
  return (Error.isError(caught,)) ? caught.message : caughtValueText(caught,);
}
