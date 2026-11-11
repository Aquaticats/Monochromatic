import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  cratesIoApiBase,
  releaseWorkflow,
  releaseWorkflowInputs,
} from './constants.ts';
import { CrateReleaseError } from './error.ts';
import {
  buildCratesIoArguments,
  buildDispatchedRunListArguments,
  buildGraphqlInputArguments,
  buildManifestContentsArguments,
  buildReleaseDispatchArguments,
  buildReleaseViewArguments,
  buildRunListArguments,
  buildRunViewArguments,
  cratesIoUrl,
} from './service-arguments.ts';

/**
 Run a builder that must throw and return what it threw.

 @param action - builder call under test
 @returns the thrown value
 */
function thrownBy(action: () => unknown): unknown {
  try {
    action();
  }
  catch (error) {
    return error;
  }
  return undefined;
}

/**
 A manifest path and a commit hash that the contents builder accepts.
 */
const manifestPath = 'package/forbidden-strings/cli/Cargo.toml';
const sha = 'b0d0b14a01'.padEnd(40, '0');

await describe({
  name: '',
  children: [
    describe({
      name: buildManifestContentsArguments.name,
      children: [
        it({
          name: 'reads the file at the commit as raw text, leaving the repository to gh when none is named',
          fn: async () => {
            const args = buildManifestContentsArguments({ manifestPath, sha });
            expect(args[0]).toBe('api');
            expect(args[1]).toBe(`repos/{owner}/{repo}/contents/${manifestPath}?ref=${sha}`);
            expect(args[2]).toBe('--header=Accept: application/vnd.github.raw+json');
            expect(args).toHaveLength(3);
          },
        }),
        it({
          name: 'writes a named repository into the endpoint, because gh api has no repository option',
          fn: async () => {
            const args = buildManifestContentsArguments({ manifestPath, sha, repo: 'o/n' });
            expect(args[1]).toBe(`repos/o/n/contents/${manifestPath}?ref=${sha}`);
            expect(args.some(function namesRepoOption(arg) {
              return arg.startsWith('--repo');
            })).toBe(false);
          },
        }),
        it({
          name: 'lower-cases the commit as the API writes it',
          fn: async () => {
            expect(buildManifestContentsArguments({ manifestPath, sha: sha.toUpperCase() })[1]).toContain(`?ref=${sha}`);
          },
        }),
        it({
          name: 'percent-encodes a path segment so it cannot add a query parameter or a fragment',
          fn: async () => {
            const args = buildManifestContentsArguments({ manifestPath: 'package/a?b#c d/Cargo.toml', sha });
            expect(args[1]).toBe(`repos/{owner}/{repo}/contents/package/a%3Fb%23c%20d/Cargo.toml?ref=${sha}`);
          },
        }),
        ...[
          `${sha}&per_page=1`,
          `${sha}\n`,
          `${sha}?x=1`,
          '../x',
          'main',
          '',
          'abc',
          `${sha}0`,
        ].map(function rejectSha(badSha) {
          return it({
            name: `rejects the commit ${JSON.stringify(badSha)}`,
            fn: async () => {
              expect(thrownBy(() => buildManifestContentsArguments({ manifestPath, sha: badSha }))).toBeInstanceOf(CrateReleaseError);
            },
          });
        }),
        ...[
          '',
          '../Cargo.toml',
          '/Cargo.toml',
          'package/a/../Cargo.toml',
          'package/a/README.md',
          String.raw`package/a\Cargo.toml`,
        ].map(function rejectPath(badPath) {
          return it({
            name: `rejects the manifest path ${JSON.stringify(badPath)}`,
            fn: async () => {
              expect(thrownBy(() => buildManifestContentsArguments({ manifestPath: badPath, sha }))).toBeInstanceOf(CrateReleaseError);
            },
          });
        }),
        ...['o', 'o/n/x', '-o/n', 'o/n?x=1', '../n', 'o /n'].map(function rejectRepo(repo) {
          return it({
            name: `rejects the repository ${JSON.stringify(repo)}`,
            fn: async () => {
              expect(thrownBy(() => buildManifestContentsArguments({ manifestPath, sha, repo }))).toBeInstanceOf(CrateReleaseError);
            },
          });
        }),
      ],
    }),
    describe({
      name: buildRunListArguments.name,
      children: [
        it({
          name: 'lists the latest runs of the workflow as JSON without any query text',
          fn: async () => {
            expect(buildRunListArguments({ workflow: 'cargo-publish.yml', limit: 10 })).toEqual([
              'run',
              'list',
              '--workflow=cargo-publish.yml',
              '--limit=10',
              '--json=databaseId,headSha',
            ]);
          },
        }),
        it({
          name: 'adds the repository when one is named',
          fn: async () => {
            expect(buildRunListArguments({ workflow: 'w.yml', limit: 5, repo: 'owner/name' })).toContain('--repo=owner/name');
          },
        }),
        it({
          name: 'keeps a workflow name that starts with a hyphen inside its option',
          fn: async () => {
            const args = buildRunListArguments({ workflow: '-x', limit: 1 });
            expect(args).toContain('--workflow=-x');
            expect(args).not.toContain('-x');
          },
        }),
        it({
          name: 'holds no jq program, so no commit hash is ever interpolated into one',
          fn: async () => {
            const joined = buildRunListArguments({ workflow: 'w.yml', limit: 5 }).join(' ');
            expect(joined).not.toContain('jq');
            expect(joined).not.toContain('startswith');
          },
        }),
      ],
    }),
    describe({
      name: buildReleaseDispatchArguments.name,
      children: [
        it({
          name: 'starts the release workflow with the four inputs, each as one raw-field argument',
          fn: async () => {
            expect(buildReleaseDispatchArguments({
              crate: 'forbidden-strings',
              from: '0.6.0',
              to: '0.6.1',
              note: 'Skip the generated rule sources.',
              dryRun: false,
            })).toEqual([
              'workflow',
              'run',
              'crate-release.yml',
              '--raw-field=crate=forbidden-strings',
              '--raw-field=from=0.6.0',
              '--raw-field=to=0.6.1',
              '--raw-field=note=Skip the generated rule sources.',
            ]);
          },
        }),
        it({
          name: 'adds the dry-run input only for a dry run, so a real release sends exactly the four inputs',
          fn: async () => {
            const input = { crate: 'c', from: '0.1.0', to: '0.1.1', note: 'n' };
            expect(buildReleaseDispatchArguments({ ...input, dryRun: true })).toContain(`--raw-field=${releaseWorkflowInputs.dryRun}=true`);
            expect(buildReleaseDispatchArguments({ ...input, dryRun: false }).join(' ')).not.toContain('dry-run');
          },
        }),
        it({
          name: 'adds the repository when one is named',
          fn: async () => {
            const args = buildReleaseDispatchArguments({ crate: 'c', from: '0.1.0', to: '0.1.1', note: 'n', dryRun: false, repo: 'o/r' });
            expect(args.at(-1)).toBe('--repo=o/r');
          },
        }),
        it({
          name: 'keeps a note that starts with a hyphen or holds shell syntax inside its own argument',
          fn: async () => {
            for (const note of ['-x', '--looks-like-an-option', "it's \"q\"; $(id) && `id`", 'a=b=c']) {
              const args = buildReleaseDispatchArguments({ crate: 'c', from: '0.1.0', to: '0.1.1', note, dryRun: false });
              expect(args).toContain(`--raw-field=note=${note}`);
              expect(args).not.toContain(note);
              expect(args).toHaveLength(7);
            }
          },
        }),
      ],
    }),
    describe({
      name: buildRunViewArguments.name,
      children: [
        it({
          name: 'reads the creation time, the status, the conclusion and every job of the run as JSON',
          fn: async () => {
            expect(buildRunViewArguments({ runId: '123' })).toEqual([
              'run',
              'view',
              '123',
              '--json=createdAt,status,conclusion,jobs',
            ]);
          },
        }),
        it({
          name: 'adds the repository when one is named',
          fn: async () => {
            expect(buildRunViewArguments({ runId: '1', repo: 'o/r' })).toContain('--repo=o/r');
          },
        }),
        ...['', 'abc', '12 3', '-1', '1;2', '--json'].map(function badRun(runId) {
          return it({
            name: `rejects the run id ${JSON.stringify(runId)}`,
            fn: async () => {
              expect(thrownBy(() => buildRunViewArguments({ runId }))).toBeInstanceOf(CrateReleaseError);
            },
          });
        }),
      ],
    }),
    describe({
      name: buildDispatchedRunListArguments.name,
      children: [
        it({
          name: 'lists the latest manually started runs of the workflow as JSON with their creation times',
          fn: async () => {
            expect(buildDispatchedRunListArguments({ workflow: 'cargo-publish.yml', limit: 10 })).toEqual([
              'run',
              'list',
              '--workflow=cargo-publish.yml',
              '--event=workflow_dispatch',
              '--limit=10',
              '--json=databaseId,headSha,createdAt',
            ]);
          },
        }),
        it({
          name: 'adds the repository when one is named, and keeps a workflow name that starts with a hyphen inside its option',
          fn: async () => {
            const args = buildDispatchedRunListArguments({ workflow: '-x', limit: 1, repo: 'o/r' });
            expect(args).toContain('--repo=o/r');
            expect(args).toContain('--workflow=-x');
            expect(args).not.toContain('-x');
          },
        }),
      ],
    }),
    describe({
      name: buildGraphqlInputArguments.name,
      children: [
        it({
          name: 'sends the request body kept in the file, with the path inside its option',
          fn: async () => {
            expect(buildGraphqlInputArguments('/runner/temp/request.json')).toEqual([
              'api',
              'graphql',
              '--input=/runner/temp/request.json',
            ]);
          },
        }),
        ...['', 'relative.json', '-x', '/a\0b'].map(function badPath(path) {
          return it({
            name: `rejects the request file ${JSON.stringify(path)}`,
            fn: async () => {
              expect(thrownBy(() => buildGraphqlInputArguments(path))).toBeInstanceOf(CrateReleaseError);
            },
          });
        }),
      ],
    }),
    describe({
      name: buildReleaseViewArguments.name,
      children: [
        it({
          name: 'views the tag and asks for the asset list as JSON',
          fn: async () => {
            expect(buildReleaseViewArguments({ tag: 'monochromatic-git-policy-cli-v0.4.10' })).toEqual([
              'release',
              'view',
              'monochromatic-git-policy-cli-v0.4.10',
              '--json=assets',
            ]);
          },
        }),
        it({
          name: 'adds the repository when one is named',
          fn: async () => {
            expect(buildReleaseViewArguments({ tag: 't', repo: 'o/n' })).toContain('--repo=o/n');
          },
        }),
        it({
          name: 'passes a tag with shell syntax as one argument',
          fn: async () => {
            const args = buildReleaseViewArguments({ tag: `a b;'$(x)` });
            expect(args[2]).toBe(`a b;'$(x)`);
          },
        }),
        ...['', '-x', '--json', 'a\0b'].map(function rejectCase(tag) {
          return it({
            name: `rejects the tag ${JSON.stringify(tag)}`,
            fn: async () => {
              expect(thrownBy(() => buildReleaseViewArguments({ tag }))).toBeInstanceOf(CrateReleaseError);
            },
          });
        }),
      ],
    }),
    describe({
      name: cratesIoUrl.name,
      children: [
        it({
          name: 'addresses one version of one crate',
          fn: async () => {
            expect(cratesIoUrl({ crateName: 'monochromatic-git-policy-cli', version: '0.4.10' }))
              .toBe(`${cratesIoApiBase}/monochromatic-git-policy-cli/0.4.10`);
          },
        }),
        it({
          name: 'encodes the plus sign of a build version',
          fn: async () => {
            expect(cratesIoUrl({ crateName: 'c', version: '1.0.0+build.5' })).toBe(`${cratesIoApiBase}/c/1.0.0%2Bbuild.5`);
          },
        }),
        ...['../x', 'a?b', 'a#b', 'a b', 'a/b', '', '1.0'].map(function rejectVersion(version) {
          return it({
            name: `rejects the version ${JSON.stringify(version)}`,
            fn: async () => {
              expect(thrownBy(() => cratesIoUrl({ crateName: 'c', version }))).toBeInstanceOf(CrateReleaseError);
            },
          });
        }),
        ...['../x', 'a?b', 'a#b', 'a b', 'a/b', ''].map(function rejectName(crateName) {
          return it({
            name: `rejects the crate name ${JSON.stringify(crateName)}`,
            fn: async () => {
              expect(thrownBy(() => cratesIoUrl({ crateName, version: '1.0.0' }))).toBeInstanceOf(CrateReleaseError);
            },
          });
        }),
      ],
    }),
    describe({
      name: buildCratesIoArguments.name,
      children: [
        it({
          name: 'sends a quiet bounded request that names the caller, as crates.io requires',
          fn: async () => {
            expect(buildCratesIoArguments({
              crateName: 'c',
              version: '1.0.0',
              userAgent: 'crate-release-check',
              maxTimeSeconds: 30,
            })).toEqual([
              '--silent',
              '--show-error',
              '--max-time',
              '30',
              '--user-agent',
              'crate-release-check',
              `${cratesIoApiBase}/c/1.0.0`,
            ]);
          },
        }),
        it({
          name: 'passes a user agent with shell syntax as one argument',
          fn: async () => {
            const args = buildCratesIoArguments({
              crateName: 'c',
              version: '1.0.0',
              userAgent: `a b;'$(x)`,
              maxTimeSeconds: 1,
            });
            expect(args[5]).toBe(`a b;'$(x)`);
          },
        }),
      ],
    }),
  ],
});
