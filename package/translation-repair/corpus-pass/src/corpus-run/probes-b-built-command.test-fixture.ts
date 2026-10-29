/**
 A throwaway corpus repository for the as-built suites of the probes to read,
 and what those suites share around it.

 TEST SUPPORT, NOT PACKAGE SOURCE. A child is started by the shared keyless
 fixture (`child-environment.test-fixture.ts`) and by nothing here.

 Fixtures are invented and cat-themed.

 @module
 */

import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';

import { resolveRealGit as resolveGit, } from '@monochromatic-dev/git-executable-core/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider';
import type { CorpusPin, } from '@monochromatic-dev/translation-repair-corpus-source';
import { namingFixtureGit, } from '../archive-naming.test-fixture.ts';
import {
  type ChildRun,
  runBuiltCommand,
} from '@monochromatic-dev/translation-repair-adjudication/ts/child-environment.test-fixture.ts';
import { criticClient, } from '@monochromatic-dev/translation-repair-provider/ts/critic-scripted-client.test-fixture.ts';
import {
  scratchDir,
  scratchDirWith,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

/**
 Native executable, never the current repository's command-policy wrapper.
 */
const REAL_GIT = await resolveGit();

/**
 A throwaway corpus repository and the commit a run reads it at.

 @example
 ```ts
 await using corpus = await makeProbeCorpus({ files: { 'people/mittens/page.md': 'Mittens naps.\n', }, },);
 ```
 */
type ProbeCorpus = AsyncDisposable & {
  /**
   Directory of the repository.
   */
  readonly cloneDir: string;

  /**
   Full name of the one commit holding every file.
   */
  readonly commitSha: string;

  /**
   Pin reading that commit through the native executable.
   */
  readonly pin: CorpusPin;
};

/**
 Builds a repository of invented files committed once, for a run to read as
 its corpus.

 @param files - file text by repository-relative path

 @returns Repository and its commit, removed on scope exit

 @example
 ```ts
 await using corpus = await makeProbeCorpus({ files: { 'people/mittens/page.md': 'Mittens naps.\n', }, },);
 ```
 */
export async function makeProbeCorpus(
  { files, }: { readonly files: Readonly<Record<string, string>>; },
): Promise<ProbeCorpus> {
  return await scratchDirWith({
    prefix: 'probes-b-corpus-',
    setup: async function seeded({ path: cloneDir, },): Promise<{
      readonly cloneDir: string;
      readonly commitSha: string;
      readonly pin: CorpusPin;
    }> {
      await namingFixtureGit({
        cloneDir,
        args: ['init',],
      },);
      for (
        const [
          relPath,
          text,
        ] of Object.entries(files,)
      ) {
        /* oxlint-disable no-await-in-loop -- one git index, so each file is written and added in turn */
        await mkdir(
          dirname(join(
            cloneDir,
            relPath,
          ),),
          { recursive: true, },
        );
        await writeFile(
          join(
            cloneDir,
            relPath,
          ),
          text,
          'utf8',
        );
        await namingFixtureGit({
          cloneDir,
          args: [
            'add',
            '--',
            relPath,
          ],
        },);
        /* oxlint-enable no-await-in-loop */
      }
      await namingFixtureGit({
        cloneDir,
        args: [
          'commit',
          '--message',
          'record invented corpus',
        ],
      },);
      /**
       The one commit.
       */
      const commitSha = await namingFixtureGit({
        cloneDir,
        args: [
          'rev-parse',
          'HEAD',
        ],
      },);
      return {
        cloneDir,
        commitSha,
        pin: {
          cloneDir,
          commitSha,
          gitPath: REAL_GIT,
        },
      };
    },
  },);
}

/**
 Environment pointing a run at a throwaway corpus.

 @param corpus - repository and commit the run reads

 @returns Variables to hand `runBuiltWithoutKeys` as its `env`

 @example
 ```ts
 const env = corpusEnvOf({ corpus, },);
 ```
 */
export function corpusEnvOf(
  { corpus, }: { readonly corpus: Pick<ProbeCorpus, 'cloneDir' | 'commitSha'>; },
): Readonly<Record<string, string>> {
  return {
    TRANSLATION_REPAIR_CORPUS_CLONE_DIR: corpus.cloneDir,
    TRANSLATION_REPAIR_CORPUS_COMMIT: corpus.commitSha,
  };
}

/**
 A corpus of one entry whose page and translation both hold one line, for an
 as-built case that needs an entry to name and nothing more.
 */
export const ONE_ENTRY_CORPUS: Readonly<Record<string, string>> = {
  'people/Mittens/page.md': 'Mittens naps.\n',
  'people/Mittens/page.en.md': 'Mittens naps.\n',
};

/**
 Runs a built probe against a throwaway corpus of the files given, with a
 runs directory of its own and no provider key, for the probes' as-built
 suites; until 2026-10-10 three of them kept a copy each.

 @param command - built probe to run

 @param packageEntry - URL of the built index of the package that owns the probe, as `import.meta.resolve` gives it
 for that package's name

 @param files - corpus files by repository-relative path

 @param args - arguments after the command

 @returns Exit code and both streams

 @example
 ```ts
 const run = await runProbeOver({
   command: 'coverage-probe',
   packageEntry: import.meta.resolve('@monochromatic-dev/translation-repair-coverage-census'),
   files: { 'people/Mittens/page.md': 'Mittens naps.\n', },
   args: ['--cap', '2',],
 },);
 ```
 */
export async function runProbeOver(
  {
    command,
    packageEntry,
    files,
    args,
  }: {
    readonly command: string;
    readonly packageEntry: string;
    readonly files: Readonly<Record<string, string>>;
    readonly args: readonly string[];
  },
): Promise<ChildRun> {
  /**
   Throwaway corpus the probe reads.
   */
  await using corpus = await makeProbeCorpus({ files, },);
  /**
   Throwaway runs directory the probe may write under.
   */
  await using runs = await scratchDir({ prefix: `${command}-runs-`, },);
  return await runBuiltCommand({
    command,
    packageEntry,
    args,
    env: {
      ...corpusEnvOf({ corpus, },),
      TRANSLATION_REPAIR_RUNS_DIR: runs.path,
    },
  },);
}

/**
 Runs something that may refuse and says how it ended.

 @param run - call under test

 @returns Refusal as `String(error)` reads it, empty when the call ran to
 its end

 @example
 ```ts
 const refusal = await refusalOf({ run: async function walk(): Promise<void> { await probe(); }, },);
 ```
 */
export async function refusalOf(
  { run, }: { readonly run: () => Promise<void>; },
): Promise<string> {
  try {
    await run();
    return '';
  }
  catch (error) {
    return String(error,);
  }
}

/**
 The report an inert client gives every critic.

 @returns Report raising nothing

 @example
 ```ts
 const report = noIssues();
 ```
 */
function noIssues(): unknown {
  return { issues: [], };
}

/**
 Builds a client no case calls, and keeps it, so a case can say how many were
 built and whether one was handed on.

 @param clients - list every client built is added to

 @returns Client answering every critic with nothing

 @example
 ```ts
 const client = keptInertClient({ clients, },);
 ```
 */
export function keptInertClient({ clients, }: { readonly clients: ModelClient[]; },): ModelClient {
  /**
   Client answering every critic with nothing.
   */
  const client = criticClient({ reportFor: noIssues, },);
  clients.push(client,);
  return client;
}
