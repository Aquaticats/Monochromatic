/**
 Tests for `displacement-probe` as an operator runs it: the built command in a
 child process that holds no provider key, over a throwaway runs directory and
 a throwaway corpus. The command asks no model, so it runs to its end here.

 The command's tagged logger stamps every line with the instant it was
 written, so each case reads the stamp as `<time>` and pins every other byte.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import {
  type ChildRun,
  runBuiltCommand,
} from '@monochromatic-dev/translation-repair-adjudication/ts/child-environment.test-fixture.ts';
import { BENCH_ENTRY, } from './built-entry.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import {
  corpusEnvOf,
  makeProbeCorpus,
} from '@monochromatic-dev/translation-repair-corpus-pass/ts/corpus-run/probes-b-built-command.test-fixture.ts';
import { settledArtifactText, } from '@monochromatic-dev/translation-repair-artifact/ts/corpus-run/settled-artifact.test-fixture.ts';

/**
 Exit code `reportingRefusals` sets for a stated refusal.
 */
const REFUSED_AS_STATED = 6;

/**
 Original with two sections.
 */
const SOURCE_PAGE = '## 第一节\n\n猫猫在窗台上睡觉。\n\n## 第二节\n\n猫猫有自己的碗。\n';

/**
 Translation of the same shape.
 */
const TARGET_PAGE = '## Section one\n\nThe cat sleeps on the sill.\n\n## Section two\n\nThe cat has a bowl.\n';

/**
 Commit name the fixture artifact claims until a case replaces it.
 */
const CLAIMED_COMMIT = 'b'.repeat(40,);

/**
 Reads the instant of every tagged log line as `<time>`, which is the one
 byte range two runs do not share; a line that is no tagged log line is kept
 as it is.

 @param text - what the command wrote

 @returns Same text with each stamp replaced

 @example
 ```ts
 const stable = withoutStamps({ text, },);
 ```
 */
function withoutStamps({ text, }: { readonly text: string; },): string {
  return text
    .split('\n',)
    .map(function withoutStamp(line,): string {
      if (!line.startsWith('[info] [',))
        return line;

      /**
       Where the stamp's own bracket opens, which is the second on the line.
       */
      const open = line.indexOf(
        '[',
        1,
      );

      /**
       Where it closes.
       */
      const close = line.indexOf(
        ']',
        open,
      );
      return `${line.slice(
        0,
        open + 1,
      )}<time>${line.slice(close,)}`;
    },)
    .join('\n',);
}

/**
 Runs the built command over a runs directory holding one settled artifact.

 @param commitSha - commit the artifact claims, the corpus's own or another

 @param args - arguments after the command

 @returns Exit code and both streams

 @example
 ```ts
 const run = await runOverOneArtifact({ claims: 'own', args: [], },);
 ```
 */
async function runOverOneArtifact(
  {
    claims,
    args,
  }: {
    readonly claims: 'own' | 'another';
    readonly args: readonly string[];
  },
): Promise<ChildRun> {
  await using corpus = await makeProbeCorpus({
    files: {
      'people/Mittens/page.md': SOURCE_PAGE,
      'people/Mittens/page.en.md': TARGET_PAGE,
    },
  },);
  await using runs = await scratchDir({ prefix: 'displacement-probe-runs-', },);
  await mkdir(
    join(
      runs.path,
      'artifacts',
    ),
    { recursive: true, },
  );
  await writeFile(
    join(
      runs.path,
      'artifacts',
      'Mittens.json',
    ),
    settledArtifactText({
      prepared: prepareDocumentPair({
        sourceText: SOURCE_PAGE,
        targetText: TARGET_PAGE,
      },),
      entryId: 'Mittens',
    },)
      .replaceAll(
      CLAIMED_COMMIT,
      (claims === 'own') ? corpus.commitSha : CLAIMED_COMMIT,
      ),
    'utf8',
  );
  return await runBuiltCommand({
    command: 'displacement-probe',
    packageEntry: BENCH_ENTRY,
    args,
    env: {
      ...corpusEnvOf({ corpus, },),
      TRANSLATION_REPAIR_RUNS_DIR: runs.path,
    },
  },);
}

await describe({
  name: 'displacement-probe as built',
  children: [
    it({
      name: 'RUNS TO ITS END over a runs directory with no artifact, printing an empty rows document and the zero totals, and exits 0',
      fn: async () => {
        await using corpus = await makeProbeCorpus({ files: { 'people/Mittens/page.md': SOURCE_PAGE, }, },);
        await using runs = await scratchDir({ prefix: 'displacement-probe-runs-', },);
        const run = await runBuiltCommand({
          command: 'displacement-probe',
          packageEntry: BENCH_ENTRY,
          args: [],
          env: {
            ...corpusEnvOf({ corpus, },),
            TRANSLATION_REPAIR_RUNS_DIR: runs.path,
          },
        },);

        expect({
          code: run.code,
          stdout: withoutStamps({ text: run.stdout, },),
          stderr: run.stderr,
        },).toEqual({
          code: 0,
          stdout: '{\n  "rows": []\n}\n'
            + '[info] [<time>] [displacement-probe] settled entries carved: 0 of 0 artifacts\n'
            + '[info] [<time>] [displacement-probe]   with a defaulted recipe half: 0\n'
            + '[info] [<time>] [displacement-probe] slices read: 0\n'
            + '[info] [<time>] [displacement-probe] entries falling back to the corpus baseline: 0\n'
            + '[info] [<time>] [displacement-probe] relocation candidates: 0\n'
            + '[info] [<time>] [displacement-probe]   of which a transcription would also explain: 0\n'
            + '[info] [<time>] [displacement-probe] untranslated slices: 0\n'
            + '[info] [<time>] [displacement-probe] target-only slices: 0\n'
            + '[info] [<time>] [displacement-probe] other imbalances: 0\n',
          stderr: '',
        },);
      },
    },),
    it({
      name: 'READS one settled artifact through its recipe, printing its row, the defaulted half and the totals, and exits 0',
      fn: async () => {
        const run = await runOverOneArtifact({
          claims: 'own',
          args: [],
        },);

        expect({
          code: run.code,
          stdout: withoutStamps({ text: run.stdout, },),
          stderr: run.stderr,
        },).toEqual({
          code: 0,
          stdout: '{\n  "rows": [\n    {\n      "entryId": "Mittens",\n      "sliceCount": 2,\n'
            + '      "baseline": 2.86,\n      "baselineFrom": "corpus-reference",\n      "untranslated": [],\n'
            + '      "targetOnly": [],\n      "relocationCandidates": [],\n      "transcriptionSuspects": [],\n'
            + '      "markupDonors": [],\n      "otherImbalances": []\n    }\n  ]\n}\n'
            + '[info] [<time>] [displacement-probe] settled entries carved: 1 of 1 artifact\n'
            + '[info] [<time>] [displacement-probe]   with a defaulted recipe half: 1\n'
            + '[info] [<time>] [displacement-probe]   Mittens: deterministic default for blockPairing\n'
            + '[info] [<time>] [displacement-probe] slices read: 2\n'
            + '[info] [<time>] [displacement-probe] entries falling back to the corpus baseline: 1\n'
            + '[info] [<time>] [displacement-probe] relocation candidates: 0\n'
            + '[info] [<time>] [displacement-probe]   of which a transcription would also explain: 0\n'
            + '[info] [<time>] [displacement-probe] untranslated slices: 0\n'
            + '[info] [<time>] [displacement-probe] target-only slices: 0\n'
            + '[info] [<time>] [displacement-probe] other imbalances: 0\n',
          stderr: '',
        },);
      },
    },),
    it({
      name: 'REFUSES as stated and exits 6 when an artifact names a commit the clone lacks',
      fn: async () => {
        const run = await runOverOneArtifact({
          claims: 'another',
          args: [],
        },);

        expect(run,).toEqual({
          code: REFUSED_AS_STATED,
          stdout: '',
          stderr: `displacement-probe: corpus read failed for ${CLAIMED_COMMIT}:people/Mittens/page.md `
            + '(missing-commit); the clone holds no commit by that revision: fetch it, or pin '
            + 'a commit the clone holds.\n',
        },);
      },
    },),
    it({
      name: 'REFUSES a flag it does not read, and exits 6 with its usage line',
      fn: async () => {
        const run = await runOverOneArtifact({
          claims: 'own',
          args: ['--bogus',],
        },);

        expect(run,).toEqual({
          code: REFUSED_AS_STATED,
          stdout: '',
          stderr: 'displacement-probe: --bogus is not a flag this command reads. Usage: displacement-probe\n',
        },);
      },
    },),
  ],
},);
