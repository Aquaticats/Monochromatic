import { spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
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
      path
    ),
    'utf8'
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
    'cli'
  ),
  readText: async function readText(path) {
    return await readFile(
      path,
      'utf8'
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
 Jobs whose id is, or ends with, a suffix.

 @param suffix - end of the job id, after the crate prefix
 @returns the jobs in file order
 */
function jobsEnding(suffix: string): readonly JobBlock[] {
  return jobs.filter(function endsWithSuffix(job) {
    return (job.id === suffix) || job.id.endsWith(`-${suffix}`);
  });
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
 The crate prefix of a job id: `ms-` for `ms-detect`, nothing for `detect`.

 @param id - job id
 @param suffix - end of the id
 @returns the prefix with its hyphen
 */
function prefixOf({
  id,
  suffix,
}: {
  readonly id: string;
  readonly suffix: string;
}): string {
  return id.slice(
    0,
    id.length - suffix.length
  );
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

 @param step - lines of the step
 @param key - key without its colon
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
 Words that mark a step as one that touches crates.io.
 */
const cratesIoMarkers: readonly string[] = ['crates.io', 'cargo publish', 'cargo package', 'crates-io-auth'];

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
 Run the decision action's script on a manifest that holds a version.

 @param fixture - stub fixture whose directory is the working directory
 @param version - version to write into the manifest
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
    text: ''
  });
  const child = spawnSync(
    'bash',
    ['-c', actionScript],
    {
      cwd: fixture.directory,
      env: {
        ...fixture.environment(),
        MANIFEST_PATH: 'Cargo.toml',
        GITHUB_OUTPUT: join(
          fixture.directory,
          'out'
        ),
      },
      encoding: 'utf8',
    },
  );
  expect(child.status).toBe(0);
  return Object.fromEntries((await fixture.read('out')).split('\n')
    .filter(function nonBlank(line) {
      return line !== '';
    })
    .map(function pair(line): [string, string] {
      const equals = line.indexOf('=');
      return [line.slice(
        0,
        equals
      ), line.slice(equals + 1)];
    }));
}

/**
 Versions the workflow's own text can hold on one manifest line: the ones of the label table without a line break.
 */
const manifestLineCases = labelCases.filter(function holdsNoBreak({ version }) {
  return !version.includes('\n');
});

/**
 What one detect job shows the publish jobs, read from its text.
 */
type DetectFacts = {
  readonly id: string;
  readonly crate: string;
  readonly manifest: string;
  readonly script: string;
};

/**
 Read the single-quoted text that follows a marker.

 @param text - script text
 @param marker - text that ends just before the opening quote's content
 @param job - job id, for the message
 @returns the text up to the closing quote
 @throws Error when the marker or the closing quote is missing
 */
function quotedAfter({
  text,
  marker,
  job,
}: {
  readonly text: string;
  readonly marker: string;
  readonly job: string;
}): string {
  const start = text.indexOf(marker);
  if (start === (-1))
    throw new Error(`${job} does not hold ${marker}`);
  const from = start + marker.length;
  const end = text.indexOf(
    '\'',
    from
  );
  if (end === (-1))
    throw new Error(`${job} does not close the quote after ${marker}`);
  return text.slice(
    from,
    end
  );
}

/**
 Read the crate, the manifest and the script of a detect job.

 @param job - a `detect` job
 @returns the facts, or throws when the job lacks one
 */
function detectFacts(job: JobBlock): DetectFacts {
  const step = stepBlocks(job.lines)
    .find(function isDetect(candidate) {
      return stepValue({
        step: candidate,
        key: 'id'
      }) === 'detect';
    });
  if (step === undefined)
    throw new Error(`${job.id} has no step with the id detect`);
  const text = stepText(step);
  return {
    id: job.id,
    crate: quotedAfter({
      text,
      marker: 'INPUT_CRATE" = \'',
      job: job.id
    }),
    manifest: quotedAfter({
      text,
      marker: 'cargo_toml=\'',
      job: job.id
    }),
    script: (runScripts(step)[0] ?? []).join('\n'),
  };
}

/**
 Run the decision action and then the detect script of a job for a manual dispatch of its crate.

 @param facts - what the detect job names
 @param version - version the manifest holds
 @param event - workflow event name
 @returns the outputs of the detect script and of the decision, and the programs the stubs recorded
 */
async function runDetect({
  facts,
  version,
  event,
}: {
  readonly facts: DetectFacts;
  readonly version: string;
  readonly event: string;
}): Promise<{
  readonly detect: Readonly<Record<string, string>>;
  readonly decision: Readonly<Record<string, string>>;
  readonly programs: readonly string[];
}> {
  await using fixture = await createStubFixture();
  const manifestText = `[package]\nname = "${facts.crate}"\nversion = "${version}"\n`;
  await fixture.write({
    path: facts.manifest,
    text: manifestText
  });
  await fixture.write({
    path: 'Cargo.toml',
    text: manifestText
  });
  const decision = await decide({
    fixture,
    version
  });
  await fixture.write({
    path: 'detect-out',
    text: ''
  });
  const child = spawnSync(
    'bash',
    ['-c', facts.script],
    {
      cwd: fixture.directory,
      env: {
        ...fixture.environment(),
        EVENT_NAME: event,
        INPUT_CRATE: facts.crate,
        BEFORE_SHA: '',
        [decisionVariable]: decision[decisionOutput] ?? '',
        GITHUB_OUTPUT: join(
          fixture.directory,
          'detect-out'
        ),
      },
      encoding: 'utf8',
    },
  );
  expect(child.status).toBe(0);
  return {
    detect: Object.fromEntries((await fixture.read('detect-out')).split('\n')
      .filter(function nonBlank(line) {
        return line !== '';
      })
      .map(function pair(line): [string, string] {
        const equals = line.indexOf('=');
        return [line.slice(
          0,
          equals
        ), line.slice(equals + 1)];
      })),
    decision,
    programs: (await fixture.calls()).map(function programOf(call) {
      return call.program;
    }),
  };
}

/**
 The detect jobs of the workflow.
 */
const detectJobs = jobsEnding('detect');

/**
 The publish-crate jobs of the workflow.
 */
const publishJobs = jobsEnding('publish-crate');

/**
 The create-release jobs of the workflow.
 */
const releaseJobs = jobsEnding('create-release');

await describe({
  name: 'the crates.io label gate of the publish workflow',
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
                version
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
              text: '[package]\nname = "x"\n'
            });
            await fixture.write({
              path: 'out',
              text: ''
            });
            const child = spawnSync(
              'bash',
              ['-c', actionScript],
              {
                cwd: fixture.directory,
                env: {
                  ...fixture.environment(),
                  MANIFEST_PATH: 'Cargo.toml',
                  GITHUB_OUTPUT: join(
                    fixture.directory,
                    'out'
                  ),
                },
                encoding: 'utf8',
              },
            );
            expect(child.status).toBe(0);
            expect(await fixture.read('out')).toContain(`${decisionOutput}=false`);
          },
        }),
      ],
    }),
    describe({
      name: 'the jobs of the workflow',
      children: [
        it({
          name: 'has one detect job and one publish-crate job for every crate of the registry',
          fn: async () => {
            expect(detectJobs.length).toBe(registryCrateNames(registry).length);
            expect(publishJobs.length).toBe(registryCrateNames(registry).length);
          },
        }),
        it({
          name: 'has a detect job for each crate that decides from the crate manifest it also reads the version from, and exposes the decision',
          fn: async () => {
            for (const job of detectJobs) {
              const facts = detectFacts(job);
              const steps = stepBlocks(job.lines);
              const deciding = steps.filter(function usesAction(step) {
                return stepValue({
                  step,
                  key: 'uses'
                }) === `./${decisionActionFolder}`;
              });
              expect(deciding.map(function jobOf() {
                return job.id;
              })).toEqual([job.id]);
              const [step] = deciding;
              expect(stepText(step ?? [])).toContain(`manifest-path: '${facts.manifest}'`);
              expect(stepValue({
                step: step ?? [],
                key: 'id'
              })).toBe('label');
              expect(job.lines.join('\n')).toContain(`${decisionOutput}: \${{ steps.label.outputs.${decisionOutput} }}`);
              expect(steps.indexOf(step ?? [])).toBeLessThan(steps.findIndex(function isDetect(candidate) {
                return stepValue({
                  step: candidate,
                  key: 'id'
                }) === 'detect';
              }));
            }
          },
        }),
        it({
          name: 'covers every registry crate with the detect job that names it',
          fn: async () => {
            expect(detectJobs.map(function crateOf(job) {
              return detectFacts(job).crate;
            }).toSorted()).toEqual(registryCrateNames(registry).toSorted());
          },
        }),
        it({
          name: 'lets every step that touches crates.io, outside the decision itself, read the decision of its own crate',
          fn: async () => {
            const ungated: string[] = [];
            let gated = 0;
            for (const job of jobs)
              for (const step of stepBlocks(job.lines))
                if (touchesCratesIo(step) && (!stepText(step).includes(decisionActionFolder))) {
                  gated += 1;
                  if (!readsDecision(step))
                    ungated.push(`${job.id}: ${step[0] ?? ''}`);
                }
            expect(ungated).toEqual([]);
            expect(gated).toBeGreaterThan(registryCrateNames(registry).length);
          },
        }),
        it({
          name: 'gates each publish-crate step on the detect job of its own crate',
          fn: async () => {
            const ungated: string[] = [];
            for (const job of publishJobs) {
              const prefix = prefixOf({
                id: job.id,
                suffix: 'publish-crate'
              });
              for (const step of stepBlocks(job.lines))
                if (touchesCratesIo(step) && (!stepValue({
                  step,
                  key: 'if'
                }).includes(`needs.${prefix}detect.outputs.${decisionOutput} == 'true'`)))
                  ungated.push(`${job.id}: ${step[0] ?? ''}`);
            }
            expect(ungated).toEqual([]);
          },
        }),
        it({
          name: 'queries crates.io before every upload, so a re-run to rebuild the release binaries never fails on a duplicate upload',
          fn: async () => {
            const unguarded: string[] = [];
            let uploads = 0;
            for (const job of publishJobs)
              for (const step of stepBlocks(job.lines)) {
                const script = (runScripts(step)[0] ?? []).join('\n');
                const uploadsThisVersion = script.includes('cargo publish');
                if (!uploadsThisVersion)
                  continue;
                const onlyDryRun = script.includes('--dry-run');
                if (onlyDryRun)
                  continue;
                uploads += 1;
                const guarded = script.includes(cratesIoExistenceCheck) && script.includes(duplicateUploadNotice);
                if (!guarded)
                  unguarded.push(`${job.id}: ${script.split('\n')[0] ?? ''}`);
              }
            expect(unguarded).toEqual([]);
            expect(uploads).toBe(registryCrateNames(registry).length);
          },
        }),
        it({
          name: 'never gates a publish-crate job as a whole, because a skipped job would skip the GitHub release that needs it',
          fn: async () => {
            expect(publishJobs.filter(function gatesJob(job) {
              return (job.lines.find(function isJobCondition(line) {
                return line.startsWith('    if:');
              }) ?? '').includes(decisionOutput);
            }).map(function idOf(job) {
              return job.id;
            })).toEqual([]);
          },
        }),
        it({
          name: 'tells every GitHub release whether its crate is on crates.io, from the detect job of its own crate',
          fn: async () => {
            expect(releaseJobs.length).toBeGreaterThan(0);
            for (const job of releaseJobs) {
              const prefix = prefixOf({
                id: job.id,
                suffix: 'create-release'
              });
              const notes = stepBlocks(job.lines)
                .filter(function createsRelease(step) {
                  return stepText(step).includes('gh release create');
                });
              expect(notes.map(function jobOf() {
                return job.id;
              })).toEqual([job.id]);
              expect(stepText(notes[0] ?? [])).toContain(`${decisionVariable}: \${{ needs.${prefix}detect.outputs.${decisionOutput} }}`);
            }
          },
        }),
      ],
    }),
    describe({
      name: 'the detect jobs run for a manual dispatch of each crate',
      children: detectJobs.flatMap(function detectCases(job) {
        const facts = detectFacts(job);
        return [
          it({
            name: `${facts.crate}: an alpha version still publishes its GitHub release and skips crates.io`,
            fn: async () => {
              const result = await runDetect({
                facts,
                version: '0.2.7-alpha.1',
                event: 'workflow_dispatch'
              });
              expect(result.detect.should_publish).toBe('true');
              expect(result.detect.version).toBe('0.2.7-alpha.1');
              expect(result.decision[decisionOutput]).toBe('false');
            },
          }),
          it({
            name: `${facts.crate}: a beta version skips crates.io`,
            fn: async () => {
              const result = await runDetect({
                facts,
                version: '0.2.7-beta.3',
                event: 'workflow_dispatch'
              });
              expect(result.detect.should_publish).toBe('true');
              expect(result.decision[decisionOutput]).toBe('false');
            },
          }),
          it({
            name: `${facts.crate}: a release candidate and a plain number go to crates.io`,
            fn: async () => {
              const results = await Promise.all(['0.2.7-rc.1', '0.2.7'].map(async function detectVersion(version) {
                return await runDetect({
                  facts,
                  version,
                  event: 'workflow_dispatch'
                });
              }));
              for (const result of results) {
                expect(result.detect.should_publish).toBe('true');
                expect(result.decision[decisionOutput]).toBe('true');
              }
            },
          }),
          it({
            name: `${facts.crate}: a push of an alpha version asks no crates.io index`,
            fn: async () => {
              const alpha = await runDetect({
                facts,
                version: '0.2.7-alpha.1',
                event: 'push'
              });
              expect(alpha.programs).not.toContain('curl');
              expect(alpha.detect.should_publish).toBe('true');
            },
          }),
        ];
      }),
    }),
    describe({
      name: 'the forbidden-strings push check',
      children: [
        it({
          name: 'asks the crates.io index for a pushed plain number, which the alpha test above proves it skips for an alpha',
          fn: async () => {
            const facts = detectJobs.map(detectFacts)
              .find(function isScanner(candidate) {
                return candidate.crate === 'forbidden-strings';
              });
            expect(facts).not.toBe(undefined);
            if (facts === undefined)
              return;
            const result = await runDetect({
              facts,
              version: '0.2.7',
              event: 'push'
            });
            expect(result.programs).toContain('curl');
          },
        }),
      ],
    }),
  ],
});
