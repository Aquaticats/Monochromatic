import { existsSync } from 'node:fs';
import {
  glob,
  readFile,
} from 'node:fs/promises';
import { join } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  defaultReleaseHost,
  defaultWorkflow,
  lockFileName,
  manifestFileName,
  maximumShaLength,
  registryFileName,
  releaseHosts,
  releaseWorkflow,
  releaseWorkflowInputs,
} from './constants.ts';
import {
  loadCrateRegistry,
  registryCrateNames,
} from './crate-registry.ts';
import { replaceDependencyPin } from './manifest-edit.ts';
import { commitPaths } from './release-steps.ts';
import { findRepositoryRoot } from './repository-root.ts';

/**
 Repository root, found from this package like the executables do.
 */
const root = findRepositoryRoot({
  startDirectory: import.meta.dirname,
  exists: existsSync,
});

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
 Repository-relative paths of every file a pattern matches, outside dependency and build directories.

 @param pattern - glob relative to the repository root
 @returns the matching paths in sorted order, so a comparison does not depend on directory order
 */
async function repositoryFiles(pattern: string): Promise<string[]> {
  const found = await Array.fromAsync(glob(pattern, {
    cwd: root,
    exclude: ['**/node_modules/**', '**/target/**'],
  }));
  return found.toSorted();
}

/**
 Whether a manifest line names a crate as a dependency: an inline table, `<name> = { ... }`, or the header of a table of
 its own, `[dependencies.<name>]` (also `dev-` and `build-`).

 @param line - one line of a manifest
 @param name - crate the line may name
 @returns true for the two dependency forms the pin edit supports, so the scan asks the edit only about those lines
 */
function namesDependency({ line, name }: { line: string; name: string }): boolean {
  const header = line.trim();
  if (['dependencies', 'dev-dependencies', 'build-dependencies'].some(function isHeaderOf(kind) {
    return header.startsWith(`[${kind}.${name}]`);
  }))
    return true;
  if (!line.startsWith(name))
    return false;
  const afterName = line.slice(name.length).trimStart();
  return afterName.startsWith('=') && afterName.slice('='.length).trimStart().startsWith('{');
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
 Lines of the release workflow.
 */
const releaseLines = (await readRepositoryFile(`.github/workflows/${releaseWorkflow}`)).split('\n');

/**
 Lines of the publish workflow.
 */
const publishLines = (await readRepositoryFile(`.github/workflows/${defaultWorkflow}`)).split('\n');

/**
 The host of the release workflow: the one that is not a developer machine.
 */
const runnerHost = releaseHosts.find(function isNotLocal(host) {
  return host !== defaultReleaseHost;
}) ?? defaultReleaseHost;

/**
 Indentation of a line: the count of leading spaces.

 @param line - one line of a workflow file
 @returns how many spaces start the line
 */
function indentOf(line: string): number {
  return line.length - line.trimStart().length;
}

/**
 Take the lines below a line that belong to it: the ones before the first non-blank line indented less than a bound.

 @param lines - all lines of the file
 @param at - index of the line whose block is wanted
 @param stopsAt - the shallowest indentation a line of the block may have; a non-blank line shallower than this ends it
 @returns the lines of the block
 */
function blockBelow({
  lines,
  at,
  stopsAt,
}: {
  readonly lines: readonly string[];
  readonly at: number;
  readonly stopsAt: number;
}): readonly string[] {
  const body = lines.slice(at + 1);
  const end = body.findIndex(function isOutside(line) {
    return (line.trim() !== '') && (indentOf(line) < stopsAt);
  });
  return end === (-1) ? body : body.slice(
    0,
    end
  );
}

/**
 Remove one pair of surrounding quotes from a YAML scalar.

 @param scalar - the scalar as written
 @returns the scalar without its quotes
 */
function unquoted(scalar: string): string {
  const quote = scalar.at(0);
  return ((quote === '\'') || (quote === '"')) && scalar.endsWith(quote) && (scalar.length > 1) ? scalar.slice(
    1,
    -1
  ) : scalar;
}

/**
 List the `options` of a `choice` input of the `workflow_dispatch` trigger.

 The workflow files are written in block style, so the reader follows indentation: the input's key, then its `options:`
 key, then the `- value` lines that follow it, which may sit at the indentation of the key.

 @param lines - workflow file lines
 @param input - input name
 @returns the listed options, or an empty list when the input has none
 */
function choiceOptions({
  lines,
  input,
}: {
  readonly lines: readonly string[];
  readonly input: string;
}): readonly string[] {
  const dispatch = lines.findIndex(function isDispatch(line) {
    return line.trim() === 'workflow_dispatch:';
  });
  const inputAt = lines.findIndex(function isInput(line, index) {
    return (index > dispatch) && (line.trim() === `${input}:`);
  });
  if ((dispatch === (-1)) || (inputAt === (-1)))
    return [];
  const inputBlock = blockBelow({
    lines,
    at: inputAt,
    stopsAt: indentOf(lines[inputAt] ?? '') + 1
  });
  const optionsAt = inputBlock.findIndex(function isOptions(line) {
    return line.trim() === 'options:';
  });
  if (optionsAt === (-1))
    return [];
  const below = blockBelow({
    lines: inputBlock,
    at: optionsAt,
    stopsAt: indentOf(inputBlock[optionsAt] ?? '')
  });
  const firstOther = below.findIndex(function isNotItem(line) {
    return !line.trim()
      .startsWith('- ');
  });
  return (firstOther === (-1) ? below : below.slice(
    0,
    firstOther
  ))
    .map(function value(line) {
      return unquoted(line.trim()
        .slice(2));
    });
}

/**
 Collect the scripts of the `run:` steps of a workflow file, each as its non-blank lines.

 @param lines - workflow file lines
 @returns one entry per `run:` key, holding the script's lines
 */
function runScripts(lines: readonly string[]): readonly (readonly string[])[] {
  return lines.flatMap(function scriptAt(line, index): (readonly string[])[] {
    const trimmed = line.trim();
    if (!trimmed.startsWith('run:'))
      return [];
    const inline = trimmed.slice('run:'.length)
      .trim();
    if ((inline !== '') && (inline !== '|'))
      return [[inline]];
    return [
      blockBelow({
        lines,
        at: index,
        stopsAt: indentOf(line) + 1
      })
        .map(function trim(entry) {
          return entry.trim();
        })
        .filter(function nonBlank(entry) {
          return entry !== '';
        }),
    ];
  });
}

/**
 Read the first `version = "..."` of a manifest, which is the one the publish workflow reads.

 @param manifest - manifest text
 @returns the version
 */
function versionOf(manifest: string): string {
  const line = manifest.split('\n')
    .find(function isVersion(candidate) {
      return candidate.startsWith('version = "');
    });
  if (line === undefined)
    throw new Error('the manifest holds no version');
  return line.slice(
    'version = "'.length,
    line.lastIndexOf('"')
  );
}

/**
 Count the lines of a file that hold exactly some text once trimmed.

 @param lines - file lines
 @param text - text a trimmed line must equal
 @returns how many lines match
 */
function countLines({
  lines,
  text,
}: {
  readonly lines: readonly string[];
  readonly text: string;
}): number {
  return lines.filter(function isText(line) {
    return line.trim() === text;
  }).length;
}

await describe({
  name: 'the release workflow and the registry',
  children: [
    describe({
      name: choiceOptions.name,
      children: [
        it({
          name: 'lists the options of a choice input in order and stops at the next key',
          fn: async () => {
            const lines = [
              'on:',
              '  workflow_dispatch:',
              '    inputs:',
              '      crate:',
              '        type: choice',
              '        options:',
              '        - a',
              '        - \'b\'',
              '        default: a',
              '      other:',
              '        options:',
              '        - z',
            ];
            expect(choiceOptions({ lines, input: 'crate' })).toEqual(['a', 'b']);
            expect(choiceOptions({ lines, input: 'other' })).toEqual(['z']);
          },
        }),
        it({
          name: 'lists nothing for an input that is missing or has no options',
          fn: async () => {
            const lines = ['on:', '  workflow_dispatch:', '    inputs:', '      crate:', '        type: string'];
            expect(choiceOptions({ lines, input: 'crate' })).toEqual([]);
            expect(choiceOptions({ lines, input: 'nope' })).toEqual([]);
            expect(choiceOptions({ lines: ['name: x'], input: 'crate' })).toEqual([]);
          },
        }),
        it({
          name: 'reads the crate choices of the real publish workflow, so the other tests compare something',
          fn: async () => {
            const options = choiceOptions({ lines: publishLines, input: 'crate' });
            expect(options.length).toBeGreaterThan(5);
            expect(options).toContain(registryCrateNames(registry)[0] ?? '');
          },
        }),
      ],
    }),
    describe({
      name: runScripts.name,
      children: [
        it({
          name: 'collects block scripts and one-line scripts and stops at the next key',
          fn: async () => {
            const lines = [
              'steps:',
              '- name: a',
              '  run: |',
              '    one',
              '',
              '    two',
              '  env:',
              '    X: 1',
              '- name: b',
              '  run: three',
            ];
            expect(runScripts(lines)).toEqual([['one', 'two'], ['three']]);
          },
        }),
      ],
    }),
    describe({
      name: 'crate choices',
      children: [
        it({
          name: 'the release workflow offers exactly the crates of the registry, in its order',
          fn: async () => {
            expect(choiceOptions({ lines: releaseLines, input: releaseWorkflowInputs.crate })).toEqual(
              registryCrateNames(registry),
            );
          },
        }),
        it({
          name: 'the publish workflow can be dispatched for every crate of the registry',
          fn: async () => {
            const publishable = choiceOptions({ lines: publishLines, input: releaseWorkflowInputs.crate });
            for (const name of registryCrateNames(registry))
              expect(publishable).toContain(name);
          },
        }),
      ],
    }),
    describe({
      name: 'registry entries against the repository',
      children: registryCrateNames(registry)
        .map(function entryCase(name) {
          const entry = registry.get(name);
          if (entry === undefined)
            throw new Error(`${name} vanished from the registry`);
          const manifestPath = `${entry.crateDirectory}/${manifestFileName}`;
          return describe({
            name,
            children: [
              it({
                name: `${registryFileName} names the crate's own manifest`,
                fn: async () => {
                  const lines = (await readRepositoryFile(manifestPath)).split('\n');
                  expect(lines).toContain(`name = "${name}"`);
                },
              }),
              it({
                name: 'the pin files are exactly the manifests that pin the crate at the version the crate holds now',
                fn: async () => {
                  const version = versionOf(await readRepositoryFile(manifestPath));
                  const manifests = await repositoryFiles(`package/**/${manifestFileName}`);
                  // The crate's own manifest is part of the scan, so a scan that reached no manifest cannot pass
                  // for a crate that nothing pins.
                  expect(manifests).toContain(manifestPath);
                  const pinning = (await Promise.all(manifests.map(async function pinsTheCrate(manifest) {
                    const text = await readRepositoryFile(manifest);
                    const namesTheCrate = text.split('\n').some(function isDependencyLine(line) {
                      return namesDependency({ line, name });
                    });
                    if (!namesTheCrate)
                      return [];
                    const edit = replaceDependencyPin({
                      text,
                      crateName: name,
                      from: version,
                      to: '99.99.99',
                    });
                    return edit.changed ? [manifest] : [];
                  }))).flat();
                  expect(pinning).toEqual(entry.pinFiles.toSorted());
                },
              }),
              it({
                name: 'every path the release commits exists, because a commit names its paths',
                fn: async () => {
                  const version = versionOf(await readRepositoryFile(manifestPath));
                  const paths = commitPaths({
                    root,
                    crateName: name,
                    crateDirectory: entry.crateDirectory,
                    scope: entry.scope,
                    from: version,
                    to: '99.99.99',
                    note: 'n',
                    pinFiles: entry.pinFiles,
                    checkBins: entry.checkBins,
                    lockDirectories: entry.lockDirectories,
                    host: 'runner',
                    memoryMax: '1G',
                    unitName: 'u',
                  });
                  for (const path of paths)
                    expect(
                      existsSync(join(
                        root,
                        path
                      )),
                    ).toBe(true);
                },
              }),
              it({
                name: 'every lock directory holds a manifest and a lockfile, and every check binary is a binary of the crate',
                fn: async () => {
                  for (const directory of entry.lockDirectories) {
                    expect(
                      existsSync(join(
                        root,
                        directory,
                        manifestFileName
                      )),
                    ).toBe(true);
                    expect(
                      existsSync(join(
                        root,
                        directory,
                        lockFileName
                      )),
                    ).toBe(true);
                  }
                  const lines = (await readRepositoryFile(manifestPath)).split('\n');
                  for (const bin of entry.checkBins)
                    expect(lines).toContain(`name = "${bin}"`);
                },
              }),
            ],
          });
        }),
    }),
    describe({
      name: 'the release workflow file',
      children: [
        it({
          name: 'is started by hand only: workflow_dispatch and no other trigger',
          fn: async () => {
            const start = releaseLines.indexOf('on:');
            const end = releaseLines.indexOf('permissions: {}');
            expect(start).toBeGreaterThan(-1);
            expect(end).toBeGreaterThan(start);
            const triggers = releaseLines.slice(
              start,
              end
            )
              .join('\n');
            expect(triggers).toContain('workflow_dispatch:');
            for (const other of ['push:', 'pull_request', 'schedule:', 'workflow_call:', 'workflow_run:'])
              expect(triggers).not.toContain(other);
          },
        }),
        it({
          name: 'declares every input the dispatch command sends, the dry run switch included',
          fn: async () => {
            for (const name of Object.values(releaseWorkflowInputs))
              expect(releaseLines).toContain(`      ${name}:`);
          },
        }),
        it({
          name: 'gives the token no permission by default and write access to contents or to actions in one job each',
          fn: async () => {
            expect(releaseLines).toContain('permissions: {}');
            expect(countLines({ lines: releaseLines, text: 'contents: write' })).toBe(1);
            expect(countLines({ lines: releaseLines, text: 'actions: write' })).toBe(1);
          },
        }),
        it({
          name: 'serializes releases in one concurrency group that lets a running release finish',
          fn: async () => {
            const at = releaseLines.indexOf('concurrency:');
            expect(at).toBeGreaterThan(-1);
            expect(releaseLines.slice(
              at,
              at + 3
            )).toEqual(['concurrency:', '  group: crate-release', '  cancel-in-progress: false']);
          },
        }),
        it({
          name: 'pins every external action to a full commit hash',
          fn: async () => {
            const targets = releaseLines.map(function trimmed(line) {
              return line.trim();
            })
              .filter(function isUses(line) {
                return line.startsWith('uses: ');
              })
              .map(function target(line) {
                return line.slice('uses: '.length);
              });
            expect(targets.length).toBeGreaterThan(0);
            for (const target of targets) {
              const hash = target.slice(target.indexOf('@') + 1);
              expect(target.startsWith('./') || ((hash.length === maximumShaLength) && (target.includes('@')))).toBe(true);
            }
          },
        }),
        it({
          name: 'never writes an expression into a script, so no input can become shell syntax',
          fn: async () => {
            const scripts = runScripts(releaseLines);
            expect(scripts.length).toBeGreaterThan(3);
            for (const script of scripts)
              for (const line of script)
                expect(line).not.toContain('${{');
          },
        }),
        it({
          name: 'keeps every script within the workflow shell budget: two lines, no pipe, chain, semicolon or substitution',
          fn: async () => {
            for (const script of runScripts(releaseLines)) {
              expect(script.length).toBeLessThanOrEqual(2);
              for (const line of script)
                for (const banned of ['|', ' && ', ';', '$(', '`'])
                  expect(line).not.toContain(banned);
            }
          },
        }),
        it({
          name: 'runs the release command for the chosen crate on the runner host, with the note after a double hyphen',
          fn: async () => {
            expect(releaseLines.map(function trimmed(line) {
              return line.trim();
            })).toContain(
              `run: node package/crate-release/cli/bin/release.ts --crate "$CRATE" --host ${runnerHost} -- "$FROM" "$TO" "$NOTE"`,
            );
          },
        }),
        it({
          name: 'pushes the bump through the API push command, not with git push, and only when the run is not a dry run',
          fn: async () => {
            const pushAt = releaseLines.indexOf('    - name: Push the bump to main as a signed commit');
            expect(pushAt).toBeGreaterThan(-1);
            expect(releaseLines[pushAt + 1]).toBe(`      if: \${{ !inputs.${releaseWorkflowInputs.dryRun} }}`);
            expect(releaseLines.slice(
              pushAt,
              pushAt + 8
            )).toContain(`        CRATE: \${{ inputs.${releaseWorkflowInputs.crate} }}`);
            expect(releaseLines.join('\n')).toContain(
              'run: node package/crate-release/cli/bin/push-release.ts --crate "$CRATE" --repo "$GITHUB_REPOSITORY" --branch main --scratch-dir "$RUNNER_TEMP"',
            );
            for (const script of runScripts(releaseLines))
              for (const line of script)
                expect(line).not.toContain('git push');
          },
        }),
        it({
          name: 'starts the publish workflow on main for the chosen crate with its dry-run input off, after the release job',
          fn: async () => {
            const publishAt = releaseLines.indexOf('  publish:');
            expect(publishAt).toBeGreaterThan(-1);
            expect(releaseLines.slice(
              publishAt + 1,
              publishAt + 3
            )).toEqual(['    needs: release', `    if: \${{ !inputs.${releaseWorkflowInputs.dryRun} }}`]);
            expect(releaseLines.map(function trimmed(line) {
              return line.trim();
            })).toContain(
              `run: gh workflow run ${defaultWorkflow} --repo "$GITHUB_REPOSITORY" --ref main --raw-field "${releaseWorkflowInputs.crate}=$CRATE" --raw-field ${releaseWorkflowInputs.dryRun}=false`,
            );
          },
        }),
        it({
          name: 'targets a dry-run input and a crate input that the publish workflow declares',
          fn: async () => {
            expect(publishLines).toContain(`      ${releaseWorkflowInputs.dryRun}:`);
            expect(publishLines).toContain(`      ${releaseWorkflowInputs.crate}:`);
          },
        }),
      ],
    }),
  ],
});
