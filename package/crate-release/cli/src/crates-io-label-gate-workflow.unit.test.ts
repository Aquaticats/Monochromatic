import { spawnSync } from 'node:child_process';
import {
  existsSync,
  writeFileSync,
} from 'node:fs';
import { readFile } from 'node:fs/promises';
import { join } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { defaultWorkflow } from './constants.ts';
import {
  loadCrateRegistry,
  registryCrateNames,
} from './crate-registry.ts';
import { labelCases } from './crates-io-version-label-cases.ts';
import {
  goesToCratesIo,
  releaseLabel,
} from './crates-io-version-label.ts';
import { findRepositoryRoot } from './repository-root.ts';
import {
  createStubFixture,
  type StubFixture,
} from './stub-tools-fixture.ts';
import {
  blockBelow,
  indentOf,
  type JobBlock,
  jobBlocks,
  runScripts,
  stepBlocks,
} from './workflow-text.ts';

/**
 Repository root, found from this package like the executables do.
 */
const root = findRepositoryRoot({
  startDirectory: import.meta.dirname,
  exists: existsSync,
});

/**
 Repository-relative folder of the action that decides whether a version goes to crates.io.
 */
const decisionActionFolder = '.github/actions/decide-whether-a-version-goes-to-crates-io';

/**
 Name of the job output, the step output and the environment variable that carry the decision.
 */
const decisionOutput = 'goes_to_crates_io';

/**
 The environment variable a script reads the decision from.
 */
const decisionVariable = 'GOES_TO_CRATES_IO';

/**
 Read a repository file as text.

 @param path - repository-relative path

 @returns the file's text
 */
async function readRepositoryFile(path: string): Promise<string> {
  return await readFile(
    join(
      root,
      path,
    ),
    'utf8',
  );
}

/**
 The registry this package ships.
 */
const registry = await loadCrateRegistry({
  packageDirectory: join(
    root,
    'package',
    'crate-release',
    'cli',
  ),
  readText: async function readText(path) {
    return await readFile(
      path,
      'utf8',
    );
  },
});

/**
 Lines of the publish workflow.
 */
const publishLines = (await readRepositoryFile(`.github/workflows/${defaultWorkflow}`)).split('\n');

/**
 Every job of the publish workflow.
 */
const jobs = jobBlocks(publishLines);

/**
 Lines of the decision action.
 */
const actionLines = (await readRepositoryFile(`${decisionActionFolder}/action.yml`)).split('\n');

/**
 The script of the decision action.
 */
const actionScript = (runScripts(actionLines)[0] ?? [])
  .join('\n');

/**
 The interpreter the decision action's step declares, read from the action instead of assumed, so a change there fails
 the invariant below rather than silently running the script some other way.
 */
const actionShell = (actionLines.find(function isShell(line) {
  return line.trim()
    .startsWith('shell:');
}) ?? '').trim()
  .slice('shell:'.length)
  .trim();

/**
 The shell a workflow step's script runs in when the step names none: the runner's default bash.
 */
const bashShell = 'bash {0}';

/**
 How long one step's script may run.

 Every spawn in this file used to wait without a bound, so a child that could not finish held the whole suite in an
 indefinite wait (`ep_poll` on the parent). A `node` that cannot start a thread under a full task ceiling sleeps for
 good instead of failing (`doc/troubleshooting/node-26-child-stuck-when-a-full-task-ceiling-refuses-its-threads.md`),
 and the shared per-user ceiling has been reached before. The file's peak is eight tasks and a green run of it takes
 about five seconds, so twenty seconds is several times any legitimate step and still ends the suite by itself.
 */
const stepTimeoutMilliseconds = 20_000;

/**
 What a finished step's script wrote and which status it exited with. A step that produced no status never becomes one
 of these, because that is a failure the caller has to see, not a value to compare.
 */
type StepResult = {
  readonly status: number;
  readonly stdout: string;
  readonly stderr: string;
};

/**
 Run a step's script the way GitHub runs it, with a bound on the wait.

 GitHub writes a step's `run:` body to an extensionless file and gives it to the interpreter the step's `shell:` names:
 `node {0}` for the JavaScript steps, and the default bash for a step that names none.

 @param options - the fixture directory, the script, its interpreter, the environment the step reads, and the working
 directory the step declares

 @returns the status, stdout and stderr of the finished step

 @throws Error when the step spawns nothing, is killed, or is still running when the bound expires

 @example
 ```ts
 runStep({ fixture, script, shell: 'node {0}', env: {} });
 ```
 */
function runStep({
  fixture,
  script,
  shell,
  env,
  cwd = fixture.directory,
}: {
  readonly fixture: StubFixture;
  readonly script: string;
  readonly shell: string;
  readonly env: Readonly<Record<string, string>>;
  readonly cwd?: string;
}): StepResult {
  /**
   Whether the step runs the file through Node, as the `node {0}` shell in the action's step declares. The launcher is
   extensionless because Actions writes one, and Node detects the module syntax of the body.
   */
  const runsNode = shell === 'node {0}';
  /**
   The command and its arguments: the interpreter with the script file, or bash with the script as its argument.
   */
  const command = runsNode ? process.execPath : 'bash';
  const args = runsNode ? ['step-script'] : ['-c', script];
  if (runsNode)
    writeFileSync(
      join(
        cwd,
        'step-script',
      ),
      script,
    );
  const child = spawnSync(
    command,
    args,
    {
      cwd,
      env: {
        ...fixture.environment(),
        ...env,
      },
      encoding: 'utf8',
      timeout: stepTimeoutMilliseconds,
      killSignal: 'SIGKILL',
    },
  );
  if (child.error !== undefined)
    throw new Error(
      `the ${shell} step did not finish within ${String(stepTimeoutMilliseconds)} ms`,
      { cause: child.error, },
    );
  if (child.status === null)
    throw new Error(`the ${shell} step produced no exit status, so it was killed by ${String(child.signal)}`);
  return {
    status: child.status,
    stdout: child.stdout,
    stderr: child.stderr,
  };
}

/**
 Text of a step without its comment lines.

 @param step - lines of the step

 @returns the step as text, so a word in a comment does not count
 */
function stepText(step: readonly string[]): string {
  return step.filter(function isCode(line) {
    return !line.trim()
      .startsWith('#');
  })
    .join('\n');
}

/**
 A step line without its indentation and without the dash that starts the step.

 @param line - one line of a step

 @returns the line as a key and value
 */
function withoutDash(line: string): string {
  const trimmed = line.trim();
  return trimmed.startsWith('- ') ? trimmed.slice('- '.length) : trimmed;
}

/**
 The value of a `key: value` line of a step, or an empty text when there is none.

 @param options - the step lines and the key without its colon

 @returns the value after the key
 */
function stepValue({
  step,
  key,
}: {
  readonly step: readonly string[];
  readonly key: string;
}): string {
  const line = step.map(withoutDash)
    .find(function hasKey(candidate) {
      return candidate.startsWith(`${key}:`);
    });
  return line === undefined ? '' : line.slice(key.length + 1)
    .trim();
}

/**
 The text of a block value of a step, dedented.

 @param options - the step lines and the key, which must hold a `|` block

 @returns the block's text without the indentation the workflow gives it

 @throws Error when the step has no such block
 */
function blockValue({
  step,
  key,
}: {
  readonly step: readonly string[];
  readonly key: string;
}): string {
  const at = step.findIndex(function isKey(line) {
    return withoutDash(line)
      .trim() === `${key}: |`;
  });
  if (at === (-1))
    throw new Error(`the step has no ${key}: | block`);
  /**
   The block's own indentation, two spaces below its key.
   */
  const blockIndent = indentOf(step[at] ?? '') + 2;
  return blockBelow({
    lines: step,
    at,
    stopsAt: blockIndent,
  })
    .map(function dedent(line) {
      return line.trim() === '' ? '' : line.slice(blockIndent);
    })
    .join('\n');
}

/**
 One descriptor of the workflow's crate list.
 */
type Descriptor = {
  readonly key: string;
  readonly crate: string;
  readonly manifest: string;
  readonly cache?: boolean;
  readonly system_libs?: boolean;
  readonly locked?: boolean;
};

/**
 The job with a given id.

 @param id - the job id

 @returns the job

 @throws Error when the workflow has no such job
 */
function jobWithId(id: string): JobBlock {
  const job = jobs.find(function isJob(candidate) {
    return candidate.id === id;
  });
  if (job === undefined)
    throw new Error(`the workflow has no job ${id}`);
  return job;
}

/**
 The job that decides every crate's version bump.
 */
const detectJob = jobWithId('detect');

/**
 The job that publishes the crates detect selected.
 */
const publishJob = jobWithId('publish-crate');

/**
 Name of the step that waits for the sibling releases a run publishes, which must run before the package step.
 */
const siblingWaitStepName = 'Wait for the sibling releases this run publishes';

/**
 Job ids the workflow defines.
 */
const jobIds = new Set(jobs.map(function idOf(job) {
  return job.id;
}));

/**
 The create-release jobs, which all read the label of the version they release.
 */
const releaseJobs = jobs.filter(function isRelease(job) {
  return (job.id === 'create-release') || job.id.endsWith('-create-release');
});

/**
 The step of a job with a given id.

 @param options - the job and the step id

 @returns the lines of the step

 @throws Error when the job has no such step
 */
function stepWithId({
  job,
  id,
}: {
  readonly job: JobBlock;
  readonly id: string;
}): readonly string[] {
  const step = stepBlocks(job.lines)
    .find(function isStep(candidate) {
      return stepValue({
        step: candidate,
        key: 'id',
      }) === id;
    });
  if (step === undefined)
    throw new Error(`the job ${job.id} has no step with the id ${id}`);
  return step;
}

/**
 The script of a job step, read from the workflow.

 @param options - the job and the step id

 @returns the script text

 @throws Error when the job or the step is missing, or the step holds no script
 */
function scriptOf({
  job,
  id,
}: {
  readonly job: JobBlock;
  readonly id: string;
}): string {
  const [script] = runScripts(stepWithId({
    job,
    id,
  }));
  if (script === undefined)
    throw new Error(`the step ${id} holds no script`);
  return script.join('\n');
}

/**
 The interpreter a step declares, or the runner's default bash.

 @param step - lines of the step

 @returns the `shell:` value, or the default bash for a step that names none
 */
function shellOf(step: readonly string[]): string {
  const declared = stepValue({
    step,
    key: 'shell',
  });
  return declared === '' ? bashShell : declared;
}

/**
 The step of the detect job that decides every crate's bump.

 @returns the lines of the step

 @throws Error when the workflow has no such step
 */
function detectStep(): readonly string[] {
  return stepWithId({
    job: detectJob,
    id: 'detect',
  });
}

/**
 The step of the publish job that waits for the sibling releases this run publishes.

 @returns the lines of the step

 @throws Error when the workflow has no such step
 */
function waitStep(): readonly string[] {
  const step = stepBlocks(publishJob.lines)
    .find(function isWait(candidate) {
      return stepValue({
        step: candidate,
        key: 'name',
      }) === siblingWaitStepName;
    });
  if (step === undefined)
    throw new Error('the publish job has no sibling wait step');
  return step;
}

/**
 The crate list the workflow carries: one descriptor per published crate.

 @returns the descriptors, in the order the workflow lists them
 */
function descriptors(): readonly Descriptor[] {
  return JSON.parse(blockValue({
    step: detectStep(),
    key: 'CRATES',
  })) as Descriptor[];
}

/**
 The crates the workflow describes, by their short key.
 */
const crateList = descriptors();

/**
 The characters a crate key is written in.
 */
const lowerCaseLetters = 'abcdefghijklmnopqrstuvwxyz';

/**
 The ids the per-crate phases of one crate would have: its build, release-state and release jobs.

 The three jobs of forbidden-strings carry no prefix, because its phases came first in the workflow.

 @param key - the crate's short key

 @returns the job id of each phase, whether or not the workflow defines it
 */
function phaseJobIdsOf(key: string): readonly string[] {
  const prefix = key === 'fs' ? '' : `${key}-`;
  return ['build-binary', 'release-state', 'create-release']
    .map(function idOf(phase) {
      return `${prefix}${phase}`;
    });
}

/**
 Whether one crate has a per-crate phase job, which is what makes the detect job's outputs for it read.

 @param key - the crate's short key

 @returns true when at least one phase job of the crate exists
 */
function hasPerCratePhase(key: string): boolean {
  return phaseJobIdsOf(key)
    .some(function isDefined(id) {
      return jobIds.has(id);
    });
}

/**
 The outputs the detect job declares, as keys.

 @returns the declared output names
 */
function declaredOutputs(): Readonly<Record<string, string>> {
  /**
   Lines of the job from its `outputs:` key to its `steps:` key.
   */
  const { lines } = detectJob;
  const outputsAt = lines.findIndex(function isOutputs(line) {
    return line.trim() === 'outputs:';
  });
  const stepsAt = lines.findIndex(function isSteps(line) {
    return line.trim() === 'steps:';
  });
  return Object.fromEntries(lines.slice(outputsAt + 1, stepsAt)
    .filter(function isEntry(line) {
      /**
       The line without the six spaces an output key starts at.
       */
      const name = line.slice(6);
      return (name !== '')
        && (!name.startsWith(' '))
        && (!name.startsWith('-'))
        && (!name.startsWith('#'))
        && (name.indexOf(':') > 0);
    })
    .map(function entry(line): [string, string] {
      return [
        line.slice(
          6,
          line.indexOf(':'),
        ), '',
      ];
    }));
}

/**
 Every output name that follows a marker on a line.

 @param options - the line and the marker whose text ends just before the name

 @returns one entry per occurrence, in line order
 */
function namesAfter({
  line,
  marker,
}: {
  readonly line: string;
  readonly marker: string;
}): readonly string[] {
  /**
   The characters an output names is written in.
   */
  const nameCharacters = 'abcdefghijklmnopqrstuvwxyz0123456789_';
  const names: string[] = [];
  let from = line.indexOf(marker);
  while (from !== (-1)) {
    const start = from + marker.length;
    let end = start;
    while ((end < line.length) && nameCharacters.includes(line[end] ?? ''))
      end += 1;
    if (end > start)
      names.push(line.slice(start, end));
    from = line.indexOf(marker, end);
  }
  return names;
}

/**
 Versions the workflow's own text can hold on one manifest line: the ones of the label table without a line break.
 */
const manifestLineCases = labelCases.filter(function holdsNoBreak({ version }) {
  return !version.includes('\n');
});

/**
 Read a file GitHub's `$GITHUB_OUTPUT` holds as key and value pairs.

 @param text - the file's text

 @returns one entry per written key
 */
function outputsOf(text: string): Readonly<Record<string, string>> {
  return Object.fromEntries(text.split('\n')
    .filter(function nonBlank(line) {
      return line !== '';
    })
    .map(function pair(line): [string, string] {
      const equals = line.indexOf('=');
      return [
        line.slice(
          0,
          equals,
        ), line.slice(equals + 1),
      ];
    }));
}

/**
 Run the decision action's script on a manifest that holds a version.

 @param options - the fixture whose directory is the working directory, and the version to write

 @returns what the script wrote to its output file, as key and value pairs
 */
async function decide({
  fixture,
  version,
}: {
  readonly fixture: StubFixture;
  readonly version: string;
}): Promise<Readonly<Record<string, string>>> {
  await fixture.write({
    path: 'Cargo.toml',
    text: `[package]\nname = "x"\nversion = "${version}"\n`,
  });
  await fixture.write({
    path: 'out',
    text: '',
  });
  const child = runStep({
    fixture,
    script: actionScript,
    shell: actionShell,
    env: {
      MANIFEST_PATH: 'Cargo.toml',
      GITHUB_OUTPUT: join(
        fixture.directory,
        'out',
      ),
    },
  });
  expect(child.status).toBe(0);
  return outputsOf(await fixture.read('out'));
}

/**
 What the detect step decided in one run.
 */
type DetectRun = {
  readonly status: number;
  readonly outputs: Readonly<Record<string, string>>;
  readonly publishing: readonly Descriptor[];
  readonly programs: readonly string[];
  readonly stdout: string;
  readonly stderr: string;
};

/**
 Run the workflow's detect script over the crate list, once, as the job does.

 @param options - the version the pre-push commit held, the crates whose manifest holds another version, the event, and
 what the stubs answer

 @returns what the run wrote, which crates it selected, and the programs it started

 @example
 ```ts
 await runDetect({ event: 'push', previous: '0.1.0', versions: { nws: '0.1.1' } });
 ```
 */
async function runDetect({
  previous = '0.1.0',
  versions = {},
  event = 'push',
  dispatchCrate = 'forbidden-strings',
  beforeSha = 'a'.repeat(40),
  indexVersion = '',
  gitShowAvailable = true,
  gitCatFileAvailable = true,
  fetchStatus = 0,
  missingVersionIn = '',
}: {
  readonly previous?: string;
  readonly versions?: Readonly<Record<string, string>>;
  readonly event?: string;
  readonly dispatchCrate?: string;
  readonly beforeSha?: string;
  readonly indexVersion?: string;
  readonly gitShowAvailable?: boolean;
  readonly gitCatFileAvailable?: boolean;
  readonly fetchStatus?: number;
  readonly missingVersionIn?: string;
} = {}): Promise<DetectRun> {
  await using fixture = await createStubFixture();
  await Promise.all(crateList.map(async function writeManifest(crate) {
    const held = versions[crate.key] ?? previous;
    await fixture.write({
      path: crate.manifest,
      text: crate.key === missingVersionIn
        ? `[package]\nname = "${crate.crate}"\n`
        : `[package]\nname = "${crate.crate}"\nversion = "${held}"\n`,
    });
  }));
  await fixture.write({
    path: 'detect-out',
    text: '',
  });
  await Promise.all([
    fixture.script({
      key: 'git.show',
      stdout: `[package]\nname = "x"\nversion = "${previous}"\n`,
      status: gitShowAvailable ? 0 : 128,
    }),
    fixture.script({
      key: 'git.cat-file',
      stdout: '',
      status: gitCatFileAvailable ? 0 : 128,
    }),
    fixture.script({
      key: 'git.fetch',
      stdout: '',
      status: fetchStatus,
    }),
    fixture.script({
      key: 'curl',
      stdout: indexVersion === '' ? '' : `{"name":"x","vers":"${indexVersion}"}\n`,
      status: indexVersion === '' ? 22 : 0,
    }),
  ]);
  const step = detectStep();
  const child = runStep({
    fixture,
    script: scriptOf({
      job: detectJob,
      id: 'detect',
    }),
    shell: shellOf(step),
    env: {
      BEFORE_SHA: beforeSha,
      EVENT_NAME: event,
      INPUT_CRATE: dispatchCrate,
      CRATES: blockValue({
        step,
        key: 'CRATES',
      }),
      GITHUB_OUTPUT: join(
        fixture.directory,
        'detect-out',
      ),
    },
  });
  const outputs = outputsOf(await fixture.read('detect-out'));
  return {
    status: child.status,
    outputs,
    publishing: JSON.parse(outputs.publishing ?? '[]') as Descriptor[],
    programs: (await fixture.calls()).map(function programOf(call) {
      return call.program;
    }),
    stdout: child.stdout,
    stderr: child.stderr,
  };
}

/**
 What one run of the sibling wait script did.
 */
type WaitRun = {
  readonly status: number;
  readonly stdout: string;
  readonly stderr: string;
  readonly programs: readonly string[];
};

/**
 Run the publish job's sibling wait script on a manifest of the fixture's own package directory.

 @param options - the crate, the dependency line its manifest holds, the crates this run publishes, what the index
 answers, and how many attempts the wait gets

 @returns the status, what the script printed, and the programs it started
 */
async function runWait({
  crate,
  dependency,
  publishing,
  indexVersion = '',
  attempts = 1,
}: {
  readonly crate: string;
  readonly dependency: string;
  readonly publishing: readonly { readonly crate: string; readonly version: string }[];
  readonly indexVersion?: string;
  readonly attempts?: number;
}): Promise<WaitRun> {
  await using fixture = await createStubFixture();
  await fixture.write({
    path: 'package/dir/Cargo.toml',
    text: `[package]\nname = "${crate}"\nversion = "0.1.0"\n\n[dependencies]\n${dependency}`,
  });
  await fixture.script({
    key: 'curl',
    stdout: indexVersion === '' ? '' : `{"name":"x","vers":"${indexVersion}"}\n`,
    status: indexVersion === '' ? 22 : 0,
  });
  const child = runStep({
    fixture,
    script: scriptOf({
      job: publishJob,
      id: 'wait',
    }),
    shell: shellOf(waitStep()),
    cwd: join(
      fixture.directory,
      'package',
      'dir',
    ),
    env: {
      CRATE: crate,
      PUBLISHING: JSON.stringify(publishing),
      WAIT_ATTEMPTS: String(attempts),
      WAIT_SECONDS: '0',
    },
  });
  return {
    status: child.status,
    stdout: child.stdout,
    stderr: child.stderr,
    programs: (await fixture.calls()).map(function programOf(call) {
      return call.program;
    }),
  };
}

/**
 Text every upload step holds before it publishes: the crates.io existence check that answers whether the version is
 already published, and the notice it logs instead of uploading a duplicate.
 */
const cratesIoExistenceCheck = 'crates.io/api/v1/crates/';

/**
 Text an upload step logs when the version is already on crates.io.
 */
const duplicateUploadNotice = 'already on crates.io';

/**
 Words that mark a step as one that touches crates.io.
 */
const cratesIoMarkers: readonly string[] = [
  'crates.io',
  'cargo publish',
  'cargo package',
  'crates-io-auth',
];

/**
 Whether a step touches crates.io: it names crates.io, publishes, packages or logs in to the registry.

 @param step - lines of the step

 @returns true for a step that must not run for a version that never goes to crates.io
 */
function touchesCratesIo(step: readonly string[]): boolean {
  return cratesIoMarkers.some(function appearsInStep(marker) {
    return stepText(step)
      .includes(marker);
  });
}

/**
 Whether a step reads the decision: it names the output, the variable, or the action that makes it.

 @param step - lines of the step

 @returns true when the step is the decision or reads it
 */
function readsDecision(step: readonly string[]): boolean {
  const text = stepText(step);
  return text.includes(decisionOutput) || text.includes(decisionVariable) || text.includes(decisionActionFolder);
}

/**
 The steps whose script asks a registry question before any decision exists, and so cannot read one: the detect step,
 which asks the index which bumped versions it already holds, and the decision action's own steps.

 @param step - lines of the step

 @returns true for a step that is exempt from reading the decision
 */
function isBeforeAnyDecision(step: readonly string[]): boolean {
  const id = stepValue({
    step,
    key: 'id',
  });
  return (id === 'detect') || (id === 'label');
}

await describe({
  name: 'the crate list and the jobs that read it in the publish workflow',
  children: [
    describe({
      name: 'the decision action',
      children: [
        it({
          name: 'has a script to run',
          fn: async () => {
            expect(actionScript).toContain(decisionOutput);
          },
        }),
        it({
          name: 'runs that script as a Node step, which is how this file runs it, so the two cannot drift apart',
          fn: async () => {
            expect(actionShell).toBe('node {0}');
          },
        }),
        ...manifestLineCases.map(function actionCase({
          version,
          label,
        }) {
          return it({
            name: `decides ${JSON.stringify(version)} as the TypeScript decision does: ${label}, ${goesToCratesIo(version) ? 'to crates.io' : 'never to crates.io'}`,
            fn: async () => {
              await using fixture = await createStubFixture();
              const outputs = await decide({
                fixture,
                version,
              });
              expect(outputs[decisionOutput]).toBe(
                String(goesToCratesIo(version)),
              );
              expect(outputs.label).toBe(releaseLabel(version));
            },
          });
        }),
        it({
          name: 'says unlabelled and never to crates.io for a manifest without a version line',
          fn: async () => {
            await using fixture = await createStubFixture();
            await fixture.write({
              path: 'Cargo.toml',
              text: '[package]\nname = "x"\n',
            });
            await fixture.write({
              path: 'out',
              text: '',
            });
            const child = runStep({
              fixture,
              script: actionScript,
              shell: actionShell,
              env: {
                MANIFEST_PATH: 'Cargo.toml',
                GITHUB_OUTPUT: join(
                  fixture.directory,
                  'out',
                ),
              },
            });
            expect(child.status).toBe(0);
            expect(await fixture.read('out')).toContain(`${decisionOutput}=false`);
          },
        }),
      ],
    }),
    describe({
      name: 'the crate list',
      children: [
        it({
          name: 'names exactly the crates of the registry, so every crate that publishes is described once',
          fn: async () => {
            expect(crateList.map(function crateOf(entry) {
              return entry.crate;
            }).toSorted()).toEqual(
              registryCrateNames(registry)
                .toSorted(),
            );
          },
        }),
        it({
          name: 'points each descriptor at a manifest the repository holds, and one the push trigger watches',
          fn: async () => {
            for (const crate of crateList) {
              expect(
                existsSync(
                  join(
                    root,
                    crate.manifest,
                  ),
                ),
              ).toBe(true);
              expect(publishLines).toContain(`    - '${crate.manifest}'`);
            }
          },
        }),
        it({
          name: 'gives every crate one key, which names its outputs and its cache entry',
          fn: async () => {
            const keys = crateList.map(function keyOf(entry) {
              return entry.key;
            });
            expect(new Set(keys).size).toBe(keys.length);
            for (const key of keys)
              for (const character of key)
                expect(lowerCaseLetters).toContain(character);
          },
        }),
      ],
    }),
    describe({
      name: 'the detect job',
      children: [
        it({
          name: 'exists once, and the workflow has no per-crate copy of it',
          fn: async () => {
            expect(jobs.filter(function isDetectJob(job) {
              return job.id === 'detect';
            }).length).toBe(1);
            expect(jobs.filter(function isCopy(job) {
              return job.id.endsWith('-detect');
            })).toEqual([]);
          },
        }),
        it({
          name: 'declares an output the jobs read for every crate that has a build or release job',
          fn: async () => {
            const declared = Object.keys(declaredOutputs());
            const cratesWithPhases = crateList.filter(function hasPhase(crate) {
              return hasPerCratePhase(crate.key);
            });
            expect(cratesWithPhases.length).toBeGreaterThan(0);
            for (const crate of cratesWithPhases) {
              expect(declared).toContain(`should_publish_${crate.key}`);
              expect(declared).toContain(`version_${crate.key}`);
            }
          },
        }),
        it({
          name: 'declares every output a job reads through the needs context, and declares no other',
          fn: async () => {
            /**
             The outputs the job graph reads from the detect job.
             */
            const declaredNames = Object.keys(declaredOutputs());
            const declared = new Set(declaredNames);
            const read = new Set(publishLines.flatMap(function namesOfLine(line) {
              return namesAfter({
                line,
                marker: 'needs.detect.outputs.',
              });
            }));
            expect(read.size).toBeGreaterThan(crateList.length);
            expect([...read].filter(function isUndeclared(name) {
              return !declared.has(name);
            })).toEqual([]);
            expect([...declared].filter(function isUnread(name) {
              return !read.has(name);
            })).toEqual([]);
          },
        }),
        it({
          name: 'asks the crates.io index about a pushed version before anything builds',
          fn: async () => {
            const script = scriptOf({
              job: detectJob,
              id: 'detect',
            });
            expect(script).toContain('index.crates.io');
            expect(script).toContain('can be published only once');
          },
        }),
      ],
    }),
    describe({
      name: 'the detect script',
      children: [
        it({
          name: 'publishes only the crate a manual dispatch names',
          fn: async () => {
            const run = await runDetect({
              event: 'workflow_dispatch',
              dispatchCrate: 'monochromatic-lint',
            });
            expect(run.status).toBe(0);
            expect(run.publishing.map(function keyOf(entry) {
              return entry.key;
            })).toEqual(['ml']);
            for (const crate of crateList)
              expect(run.outputs[`should_publish_${crate.key}`]).toBe(String(crate.key === 'ml'));
          },
        }),
        it({
          name: 'publishes the one crate whose pushed version changed, and nothing else',
          fn: async () => {
            const run = await runDetect({
              event: 'push',
              versions: { nws: '0.2.7' },
              previous: '0.2.6',
            });
            expect(run.status).toBe(0);
            expect(run.publishing.map(function keyOf(entry) {
              return entry.key;
            })).toEqual(['nws']);
            expect(run.outputs.version_nws).toBe('0.2.7');
            expect(run.outputs.should_publish_nws).toBe('true');
          },
        }),
        it({
          name: 'publishes nothing when a push leaves every version unchanged, and asks no index',
          fn: async () => {
            const run = await runDetect({
              event: 'push',
              previous: '0.2.6',
            });
            expect(run.status).toBe(0);
            expect(run.outputs.publishing).toBe('[]');
            expect(run.programs).not.toContain('curl');
            for (const crate of crateList)
              expect(run.outputs[`should_publish_${crate.key}`]).toBe('false');
          },
        }),
        it({
          name: 'skips a pushed version the crates.io index already lists, which frees the build and release jobs',
          fn: async () => {
            const run = await runDetect({
              event: 'push',
              versions: { fs: '0.6.1' },
              previous: '0.6.0',
              indexVersion: '0.6.1',
            });
            expect(run.status).toBe(0);
            expect(run.programs).toContain('curl');
            expect(run.outputs.publishing).toBe('[]');
            expect(run.stdout).toContain('already on crates.io; nothing to publish');
          },
        }),
        it({
          name: 'asks a manual dispatch no index, because a dispatch rebuilds the release of a published version',
          fn: async () => {
            const run = await runDetect({
              event: 'workflow_dispatch',
              dispatchCrate: 'forbidden-strings',
              indexVersion: '0.1.0',
            });
            expect(run.status).toBe(0);
            expect(run.programs).not.toContain('curl');
            expect(run.outputs.should_publish_fs).toBe('true');
          },
        }),
        it({
          name: 'publishes an alpha push, which the index can never list, after asking it',
          fn: async () => {
            const run = await runDetect({
              event: 'push',
              versions: { fs: '0.2.7-alpha.1' },
              previous: '0.2.7-alpha.0',
            });
            expect(run.status).toBe(0);
            expect(run.programs).toContain('curl');
            expect(run.outputs.should_publish_fs).toBe('true');
          },
        }),
        it({
          name: 'reads a manifest the pre-push commit did not hold as a bump, which a package move is',
          fn: async () => {
            const run = await runDetect({
              event: 'push',
              previous: '0.1.0',
              gitShowAvailable: false,
            });
            expect(run.status).toBe(0);
            expect(run.outputs.should_publish_fs).toBe('true');
          },
        }),
        it({
          name: 'fetches the pre-push commit when the checkout does not hold it, and refuses when the fetch fails',
          fn: async () => {
            const fetched = await runDetect({
              event: 'push',
              versions: { fs: '0.2.0' },
              gitCatFileAvailable: false,
            });
            expect(fetched.programs).toContain('git');
            expect(fetched.status).toBe(0);
            const refused = await runDetect({
              event: 'push',
              versions: { fs: '0.2.0' },
              gitCatFileAvailable: false,
              fetchStatus: 128,
            });
            expect(refused.status).toBe(1);
            expect(refused.stderr).toContain('refusing to publish');
            expect(Object.keys(refused.outputs)).toEqual([]);
          },
        }),
        it({
          name: 'refuses a manifest without a version line instead of publishing an empty version',
          fn: async () => {
            const run = await runDetect({
              event: 'push',
              versions: { ml: '0.2.0' },
              missingVersionIn: 'ml',
            });
            expect(run.status).toBe(1);
            expect(run.stderr).toContain('holds no version line; refusing to publish');
          },
        }),
        it({
          name: 'hands the publish matrix its own directory, crate name, version and flags',
          fn: async () => {
            const run = await runDetect({
              event: 'workflow_dispatch',
              dispatchCrate: 'monochromatic-nested-wayland-session',
            });
            expect(run.publishing).toEqual([
              {
                key: 'nws',
                crate: 'monochromatic-nested-wayland-session',
                manifest: 'package/nested-wayland-session/cli/Cargo.toml',
                dir: 'package/nested-wayland-session/cli',
                version: '0.1.0',
                cache: true,
                system_libs: true,
                locked: false,
              },
            ]);
          },
        }),
        it({
          name: 'marks the one crate that packages and publishes with --locked',
          fn: async () => {
            const run = await runDetect({
              event: 'workflow_dispatch',
              dispatchCrate: 'monochromatic-git-policy-cli',
            });
            expect(run.publishing.map(function lockedOf(entry) {
              return entry.locked;
            })).toEqual([true]);
          },
        }),
      ],
    }),
    describe({
      name: 'the publish job',
      children: [
        it({
          name: 'exists once, and the workflow has no per-crate copy of it',
          fn: async () => {
            expect(jobs.filter(function isPublishJob(job) {
              return job.id === 'publish-crate';
            }).length).toBe(1);
            expect(jobs.filter(function isCopy(job) {
              return job.id.endsWith('-publish-crate');
            })).toEqual([]);
          },
        }),
        it({
          name: 'skips itself instead of expanding an empty matrix, which GitHub refuses to run',
          fn: async () => {
            /**
             The job as text, which is what an expression in it is read from.
             */
            const text = publishJob.lines.join('\n');
            expect(text).toContain(`if: \${{ needs.detect.outputs.publishing != '[]' }}`);
            expect(text).toContain(`include: \${{ fromJSON(needs.detect.outputs.publishing) }}`);
          },
        }),
        it({
          name: 'has a label step per matrix entry, which is how a step reads its own crate',
          fn: async () => {
            /**
             The label step of one matrix entry.
             */
            const step = stepWithId({
              job: publishJob,
              id: 'label',
            });
            expect(stepValue({
              step,
              key: 'uses',
            })).toBe(`./${decisionActionFolder}`);
            expect(stepValue({
              step,
              key: 'manifest-path',
            })).toBe(`\${{ matrix.manifest }}`);
          },
        }),
        it({
          name: 'lets every step that touches crates.io read the decision, except the ones made before it exists',
          fn: async () => {
            const ungated: string[] = [];
            let gated = 0;
            for (const job of jobs)
              for (const step of stepBlocks(job.lines))
                if (touchesCratesIo(step) && (!isBeforeAnyDecision(step))) {
                  gated += 1;
                  if (!readsDecision(step))
                    ungated.push(`${job.id}: ${step[0] ?? ''}`);
                }
            expect(ungated).toEqual([]);
            expect(gated).toBeGreaterThanOrEqual(releaseJobs.length);
          },
        }),
        it({
          name: 'has exactly one upload in the whole workflow, and it queries crates.io before it publishes',
          fn: async () => {
            const uploads: string[] = [];
            const unguarded: string[] = [];
            for (const job of jobs)
              for (const step of stepBlocks(job.lines)) {
                const script = (runScripts(step)[0] ?? []).join('\n');
                if (!script.includes('cargo publish'))
                  continue;
                if (script.includes('--dry-run'))
                  continue;
                uploads.push(`${job.id}: ${step[0] ?? ''}`);
                if (!(script.includes(cratesIoExistenceCheck) && script.includes(duplicateUploadNotice)))
                  unguarded.push(`${job.id}: ${script.split('\n')[0] ?? ''}`);
              }
            expect(unguarded).toEqual([]);
            expect(uploads.length).toBe(1);
          },
        }),
        it({
          name: 'runs the sibling wait before it packages, because a package resolves its pins against crates.io',
          fn: async () => {
            const steps = stepBlocks(publishJob.lines);
            const waitAt = steps.findIndex(function isWait(step) {
              return stepValue({
                step,
                key: 'name',
              }) === siblingWaitStepName;
            });
            const packageAt = steps.findIndex(function isPackage(step) {
              return stepText(step)
                .includes('cargo package');
            });
            expect(waitAt).toBeGreaterThan(-1);
            expect(packageAt).toBeGreaterThan(-1);
            expect(waitAt).toBeLessThan(packageAt);
            /**
             The wait step as text, which is where its inputs are read from.
             */
            const waitText = stepText(waitStep());
            expect(waitText).toContain(`PUBLISHING: \${{ needs.detect.outputs.publishing }}`);
          },
        }),
      ],
    }),
    describe({
      name: 'the sibling wait script',
      children: [
        it({
          name: 'waits for a pin this run publishes until the index lists it',
          fn: async () => {
            const run = await runWait({
              crate: 'monochromatic-pnpm-shadow',
              dependency: 'monochromatic-jsonc-edit = { path = "../../jsonc-edit/core-rust", version = "=0.1.3" }\n',
              publishing: [
                {
                  crate: 'monochromatic-jsonc-edit',
                  version: '0.1.3',
                },
              ],
              indexVersion: '0.1.3',
            });
            expect(run.status).toBe(0);
            expect(run.programs).toContain('curl');
            expect(run.stdout).toContain('is in the crates.io index');
          },
        }),
        it({
          name: 'asks nothing when the pinned version is not one this run publishes, which must already be indexed',
          fn: async () => {
            const run = await runWait({
              crate: 'monochromatic-pnpm-shadow',
              dependency: 'monochromatic-jsonc-edit = { path = "../../jsonc-edit/core-rust", version = "=0.1.2" }\n',
              publishing: [
                {
                  crate: 'monochromatic-jsonc-edit',
                  version: '0.1.3',
                },
              ],
            });
            expect(run.status).toBe(0);
            expect(run.programs).not.toContain('curl');
          },
        }),
        it({
          name: 'fails at its bound instead of waiting forever, naming the pin that never appeared',
          fn: async () => {
            const run = await runWait({
              crate: 'monochromatic-pnpm-shadow',
              dependency: 'monochromatic-jsonc-edit = { path = "../../jsonc-edit/core-rust", version = "=0.1.3" }\n',
              publishing: [
                {
                  crate: 'monochromatic-jsonc-edit',
                  version: '0.1.3',
                },
              ],
              attempts: 2,
            });
            expect(run.status).toBe(1);
            expect(run.stderr).toContain('monochromatic-jsonc-edit 0.1.3 did not appear in the crates.io index');
          },
        }),
        it({
          name: 'ignores a pin the manifest writes without an exact version, which a published manifest drops',
          fn: async () => {
            const run = await runWait({
              crate: 'monochromatic-pnpm-shadow',
              dependency: 'monochromatic-jsonc-edit = { path = "../../jsonc-edit/core-rust" }\n',
              publishing: [
                {
                  crate: 'monochromatic-jsonc-edit',
                  version: '0.1.3',
                },
              ],
            });
            expect(run.status).toBe(0);
            expect(run.programs).not.toContain('curl');
          },
        }),
      ],
    }),
    describe({
      name: 'the release jobs',
      children: [
        it({
          name: 'reads the label of its own crate for the note it writes, from the action instead of a detect job',
          fn: async () => {
            expect(releaseJobs.length).toBeGreaterThan(0);
            for (const job of releaseJobs) {
              const notes = stepBlocks(job.lines)
                .filter(function createsRelease(step) {
                  return stepText(step)
                    .includes('gh release create');
                });
              expect(notes.map(function jobOf() {
                return job.id;
              })).toEqual([job.id]);
              expect(stepText(notes[0] ?? [])).toContain(`GOES_TO_CRATES_IO: \${{ steps.label.outputs.${decisionOutput} }}`);
              /**
               The label step of this release job.
               */
              const label = stepWithId({
                job,
                id: 'label',
              });
              expect(stepValue({
                step: label,
                key: 'uses',
              })).toBe(`./${decisionActionFolder}`);
              /**
               The manifest the release job reads its label from.
               */
              const manifest = stepValue({
                step: label,
                key: 'manifest-path',
              });
              expect(manifest.startsWith('package/')).toBe(true);
              expect(manifest.endsWith('/Cargo.toml')).toBe(true);
            }
          },
        }),
        it({
          name: 'never gates a release job as a whole on the crates.io label, because a skipped job would skip its release',
          fn: async () => {
            expect(jobs.filter(function gatesJob(job) {
              return (job.lines.find(function isJobCondition(line) {
                return line.startsWith('    if:');
              }) ?? '').includes(decisionOutput);
            }).map(function idOf(job) {
              return job.id;
            })).toEqual([]);
          },
        }),
      ],
    }),
  ],
});
