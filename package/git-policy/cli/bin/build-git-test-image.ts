#!/usr/bin/env node
/**
 Build the gate's base image: the newest CachyOS image on its `latest` tag, the Rust toolchain and the audited Git 2.56.0.

 `native:test:container` does the same before it builds the gate image, so this runner exists to build the base ahead of
 time, to rebuild it after a CachyOS update with chosen limits, and to prove what it holds. It pulls the `latest` tag and
 builds only when the pull or `native-test-base.Containerfile` is new to this host.
 */
import {
  mkdir,
  mkdtemp,
  writeFile,
} from 'node:fs/promises';
import {
  join,
  resolve,
} from 'node:path';

import {
  parseContainerLimitOptions,
  runLimitArguments,
} from './container-limits.ts';
import { ensureTestBase, } from './native-test-base.ts';
import {
  NativeVerificationError,
  runCommand,
} from './native-verification-process.ts';

/**
 Spaces of indentation in the evidence files.
 */
const jsonIndent = 2;

/**
 The version line the base's Git must print: the release the gate's results are evidence about.
 */
const expectedGitVersion = 'git version 2.56.0';

/**
 Container limits from the `--memory`, `--cpus` and `--pids-limit` options, and any extra arguments, which are a usage error.
 */
const {
  limits: containerLimits,
  rest: extra,
} = parseContainerLimitOptions({
  options: process.argv
    .slice(2),
  acceptPidsLimit: true,
});
if (extra.length > 0)
  throw new NativeVerificationError(`Only --memory <size>, --cpus <count> and --pids-limit <count> are accepted, got ${extra.join(' ')}.`);
/**
 Directory under the package's `target/` that keeps the evidence of every image build.
 */
const evidenceRoot = resolve('target/verification');
await mkdir(
  evidenceRoot,
  { recursive: true },
);
/**
 Evidence directory of this build.
 */
const evidence = await mkdtemp(join(
  evidenceRoot,
  'git-image-',
));
/**
 The base image, built now when the registry's current CachyOS image or its Containerfile is new to this host.
 */
const base = await ensureTestBase({ limits: containerLimits, });
/**
 Result of asking the base's Git for its version, in a container without network.
 */
const reported = await runCommand({
  command: 'podman',
  args: [
    'run',
    '--rm',
    '--init',
    '--network=none',
    ...runLimitArguments({
      limits: containerLimits,
      withPidsLimit: true,
    }),
    base.id,
    '/usr/bin/git',
    '--version',
  ],
  capture: true,
});
/**
 The version line the base's Git printed, which must be the audited release.
 */
const version = reported.stdout
  .trim();
if (version !== expectedGitVersion)
  throw new NativeVerificationError(`The base image returned an unexpected Git version: ${version}`);
await writeFile(
  join(
    evidence,
    'result.json',
  ),
  `${JSON.stringify(
    {
      ...base,
      version,
    },
    null,
    jsonIndent,
  )}\n`,
);
console.log(`${base.built ? 'Built' : 'Found'} the gate base ${base.tag} (${base.id}) on ${base.cachyosImage} (${base.cachyosImageId}); ${version}; evidence ${evidence}`);
