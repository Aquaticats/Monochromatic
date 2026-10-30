import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { adversarialLines } from './adversarial-fixture.ts';
import { CrateReleaseError } from './error.ts';
import type { ReleaseOptions } from './release-options.ts';
import {
  runRelease,
  type ReleaseStepRunner,
} from './release-run.ts';
import type { ReleaseStep } from './release-steps.ts';

/**
 Crate manifest text of the fixture repository.
 */
const crateManifest = '[package]\nname = "monochromatic-git-policy-cli"\nversion = "0.4.2"\nedition = "2024"\n';

/**
 Fuzz manifest text of the fixture repository.
 */
const fuzzManifest = '[package]\nname = "fuzz"\nversion = "0.0.0"\n\n[dependencies]\n'
  + 'monochromatic-git-policy-cli = { path = "../cli", version = "=0.4.2" }\n';

/**
 Options of the fixture release.
 */
const options: ReleaseOptions = {
  root: '/repo',
  crateName: 'monochromatic-git-policy-cli',
  crateDirectory: 'package/git-policy/cli',
  scope: 'git-policy/cli',
  from: '0.4.2',
  to: '0.4.3',
  note: 'fix the thing',
  pinFiles: ['package/git-policy/cli.fuzz/Cargo.toml'],
  checkBin: 'cli-git-native',
  lockTasks: ['//package/git-policy/cli.fuzz:lock:members'],
  memoryMax: '12G',
  unitName: 'cli-git-publish-dry',
};

/**
 What a run left behind, for assertions.
 */
type Recording = {
  readonly status: number;
  readonly events: readonly string[];
  readonly files: ReadonlyMap<string, string>;
  readonly steps: readonly ReleaseStep[];
  readonly printed: readonly string[];
};

/**
 Run a release against an in-memory repository.

 @param releaseOptions - options of the run
 @param failing - label of the step that exits with the failure status, if any
 @param failureStatus - exit status of the failing step
 @param initialFiles - repository files by absolute path
 @returns the recorded events, files, steps and printed lines
 */
async function record({
  releaseOptions = options,
  failing,
  failureStatus = 101,
  initialFiles = new Map([
    ['/repo/package/git-policy/cli/Cargo.toml', crateManifest],
    ['/repo/package/git-policy/cli.fuzz/Cargo.toml', fuzzManifest],
  ]),
}: {
  readonly releaseOptions?: ReleaseOptions;
  readonly failing?: string;
  readonly failureStatus?: number;
  readonly initialFiles?: ReadonlyMap<string, string>;
} = {}): Promise<Recording> {
  const files = new Map(initialFiles);
  const events: string[] = [];
  const steps: ReleaseStep[] = [];
  const printed: string[] = [];
  const runStep: ReleaseStepRunner = async function runStep(step) {
    events.push(`step ${step.label}`);
    steps.push(step);
    return { status: step.label === failing ? failureStatus : 0 };
  };
  const status = await runRelease({
    options: releaseOptions,
    readText: async function readText(path) {
      const text = files.get(path);
      if (text === undefined)
        throw new CrateReleaseError(`missing fixture file ${path}`);
      return text;
    },
    writeText: async function writeText({ path, text }) {
      events.push(`write ${path}`);
      files.set(path, text);
    },
    runStep,
    print: function print(line) {
      printed.push(line);
    },
  });
  return { status, events, files, steps, printed };
}

await describe({
  name: '',
  children: [
    describe({
      name: runRelease.name,
      children: [
        it({
          name: 'bumps both manifests, then runs the four steps in order and reports the commit',
          fn: async () => {
            const result = await record();
            expect(result.status).toBe(0);
            expect(result.events).toEqual([
              'write /repo/package/git-policy/cli/Cargo.toml',
              'write /repo/package/git-policy/cli.fuzz/Cargo.toml',
              'step cargo check',
              'step lock members',
              'step publish dry run',
              'step commit',
            ]);
            expect(result.files.get('/repo/package/git-policy/cli/Cargo.toml')).toContain('version = "0.4.3"');
            expect(result.files.get('/repo/package/git-policy/cli.fuzz/Cargo.toml')).toContain('version = "=0.4.3"');
            expect(result.files.get('/repo/package/git-policy/cli.fuzz/Cargo.toml')).toContain('version = "0.0.0"');
            expect(result.printed.at(-1)).toBe('RELEASE COMMITTED');
          },
        }),
        it({
          name: 'prints a labeled copyable command line for every step',
          fn: async () => {
            const result = await record();
            expect(result.printed).toContain('== cargo check');
            expect(result.printed).toContain('$ cargo check --offline --bin cli-git-native');
            expect(result.printed).toContain(
              `$ git commit --message 'chore(git-policy/cli): release 0.4.3: fix the thing' -- `
                + 'package/git-policy/cli/Cargo.toml package/git-policy/cli/Cargo.lock '
                + 'package/git-policy/cli.fuzz/Cargo.toml package/git-policy/cli.fuzz/Cargo.lock',
            );
          },
        }),
        ...adversarialLines.map(function noteCase(note) {
          return it({
            name: `passes the note ${JSON.stringify(note)} to the commit as one argument`,
            fn: async () => {
              const result = await record({ releaseOptions: { ...options, note } });
              const commit = result.steps.find(function isCommit(step) {
                return step.label === 'commit';
              });
              expect(commit?.args[2]).toBe(`chore(git-policy/cli): release 0.4.3: ${note}`);
              expect(commit?.args).toHaveLength(8);
            },
          });
        }),
        it({
          name: 'does not rewrite a manifest that already holds the target and says so',
          fn: async () => {
            const result = await record({
              releaseOptions: { ...options, from: '0.4.1', to: '0.4.2' },
            });
            expect(result.status).toBe(0);
            expect(result.events.filter(function isWrite(event) {
              return event.startsWith('write ');
            })).toEqual([]);
            expect(result.printed.some(function mentions(line) {
              return line.includes('already') && line.includes('Cargo.toml');
            })).toBe(true);
          },
        }),
        it({
          name: 'rewrites only the manifest that still holds the current version',
          fn: async () => {
            const files = new Map([
              ['/repo/package/git-policy/cli/Cargo.toml', crateManifest.replace('0.4.2', '0.4.3')],
              ['/repo/package/git-policy/cli.fuzz/Cargo.toml', fuzzManifest],
            ]);
            const result = await record({ initialFiles: files });
            expect(result.events.filter(function isWrite(event) {
              return event.startsWith('write ');
            })).toEqual(['write /repo/package/git-policy/cli.fuzz/Cargo.toml']);
          },
        }),
        it({
          name: 'stops at the first failing step, names it and its status, and runs nothing after it',
          fn: async () => {
            const result = await record({ failing: 'lock members', failureStatus: 7 });
            expect(result.status).toBe(1);
            expect(result.printed).toContain('FAILED lock members (7)');
            expect(result.events).toContain('step lock members');
            expect(result.events).not.toContain('step publish dry run');
            expect(result.events).not.toContain('step commit');
            expect(result.printed).not.toContain('RELEASE COMMITTED');
          },
        }),
        it({
          name: 'writes nothing and runs nothing when the pin file is not an exact pin of the current version',
          fn: async () => {
            const files = new Map([
              ['/repo/package/git-policy/cli/Cargo.toml', crateManifest],
              ['/repo/package/git-policy/cli.fuzz/Cargo.toml', fuzzManifest.replace('"=0.4.2"', '"0.4.2"')],
            ]);
            let caught: unknown;
            try {
              await record({ initialFiles: files });
            }
            catch (error) {
              caught = error;
            }
            expect(caught).toBeInstanceOf(CrateReleaseError);
          },
        }),
        it({
          name: 'leaves the crate manifest unwritten when a later manifest is refused',
          fn: async () => {
            const files = new Map([
              ['/repo/package/git-policy/cli/Cargo.toml', crateManifest],
              ['/repo/package/git-policy/cli.fuzz/Cargo.toml', fuzzManifest.replace('"=0.4.2"', '"=0.3.0"')],
            ]);
            const written: string[] = [];
            let caught: unknown;
            try {
              await runRelease({
                options,
                readText: async function readText(path) {
                  return files.get(path) ?? '';
                },
                writeText: async function writeText({ path }) {
                  written.push(path);
                },
                runStep: async function runStep() {
                  return { status: 0 };
                },
                print: function print() {},
              });
            }
            catch (error) {
              caught = error;
            }
            expect(caught).toBeInstanceOf(CrateReleaseError);
            expect(written).toEqual([]);
          },
        }),
        it({
          name: 'skips pin handling when the release has no pin file',
          fn: async () => {
            const result = await record({ releaseOptions: { ...options, pinFiles: [], lockTasks: [] } });
            expect(result.status).toBe(0);
            expect(result.events).toEqual([
              'write /repo/package/git-policy/cli/Cargo.toml',
              'step cargo check',
              'step publish dry run',
              'step commit',
            ]);
          },
        }),
      ],
    }),
  ],
});
