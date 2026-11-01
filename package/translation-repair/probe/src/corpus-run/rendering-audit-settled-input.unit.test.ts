/**
 Tests for turning an archive of settled artifacts into audit subjects.

 What matters here is not that the reader parses. It is that every claim a
 persisted audit row will later make about its own provenance is true: that
 the text audited is the text the judges saw, that the corpus commit read is
 the artifact's own rather than whatever the pin says today, that a retained
 slice is marked as the archive's wording rather than a fresh rendering, and
 that a preparation which no longer matches is REPORTED rather than thrown.

 Fixtures are cat-themed invention on a throwaway git repository. No corpus
 content appears here.

 @module
 */

import {
  mkdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import {
  preparationIdentity,
  readCorpusFile,
} from '@monochromatic-dev/translation-repair-corpus-source';
import { listArtifactFiles, } from '@monochromatic-dev/translation-repair-artifact-pool';
import {
  readArchiveSubjects,
  readArtifactSubjects,
} from '../../dist/final/node/index.mjs';
import { refusalOrder, } from '@monochromatic-dev/translation-repair-test-fixture/ts/refusal-order.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { restampedAsGenerationFourteen, } from '@monochromatic-dev/translation-repair-document-structure/ts/corpus-run/generation-restamp.test-fixture.ts';
import { pageReadsRefusingLastFor, } from '@monochromatic-dev/translation-repair-corpus-source/ts/corpus-run/ordered-page-reads.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/rejection-of.test-fixture.ts';
import {
  BARE_SOURCE_PAGE,
  BARE_TARGET_PAGE,
  commitEntry,
  ENTRY_ID,
  makeCorpus,
  SOURCE_PAGE,
  TARGET_PAGE,
  writeArtifact,
} from '@monochromatic-dev/translation-repair-artifact/ts/corpus-run/settled-archive.test-fixture.ts';

/**
 Characters in a SHA-1 object id.
 */
const OBJECT_ID_LENGTH = 40;

/**
 Commit the artifacts of the refusal cases name and no clone carries.
 */
const LACKED_COMMIT = 'b'.repeat(OBJECT_ID_LENGTH,);

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: readArtifactSubjects.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'reads the audited text OUT OF THE ARTIFACT rather than out of a fresh slicing, since '
            + 'auditing re-sliced text would audit a different input than the one the judges saw',
          fn: async () => {
            await using corpus = await makeCorpus();
            await using archive = await scratchDir({ prefix: 'settled-audit-archive-', },);

            /**
             Preparation the fixture artifact was written over.
             */
            const prepared = prepareDocumentPair({
              sourceText: SOURCE_PAGE,
              targetText: TARGET_PAGE,
            },);
            await writeArtifact({
              archiveDir: archive.path,
              runSet: 'first',
              prepared,
              corpusSha: corpus.commitSha,
              entryId: ENTRY_ID,
            },);

            /**
             Artifact as the reader returns it.
             */
            const reading = await readArtifactSubjects({
              archiveDir: archive.path,
              runSetDir: 'first',
              runSet: 'first',
              artifactFile: `${ENTRY_ID}.json`,
              cloneDir: corpus.cloneDir,
              readFile: readCorpusFile,
            },);

            expect(reading.entryId,).toBe(ENTRY_ID,);
            expect(reading.subjects.length,).toBe(prepared.slices.length,);

            /**
             First slice, which the lane replaced.
             */
            const [first,] = reading.subjects;
            expect(first?.candidateText,).toContain('It purrs.',);
            expect(first?.sourceText,).toBe(prepared.slices[0]?.source
              .text,);
            // THE WHOLE PAGE at the artifact's commit, since the producing run
            // read the links of the page and not of one slice (ledger B29).
            /**
             Whether every subject carries the page the corpus holds.
             */
            const carriesPage = reading.subjects.every(function isWholePage(subject,): boolean {
              return subject.pageSourceText === SOURCE_PAGE;
            },);
            expect(carriesPage,).toBe(true,);
          },
        },),

        it({
          name: 'REPORTS A REFUSED VERIFICATION when a recorded measurement disagrees with what the '
            + 'preparation measures, rather than auditing numbers nobody checked',
          fn: async () => {
            await using corpus = await makeCorpus();
            await using archive = await scratchDir({ prefix: 'settled-audit-archive-', },);

            /**
             Preparation the fixture artifact was written over.
             */
            const prepared = prepareDocumentPair({
              sourceText: SOURCE_PAGE,
              targetText: TARGET_PAGE,
            },);
            await writeArtifact({
              archiveDir: archive.path,
              runSet: 'first',
              prepared,
              corpusSha: corpus.commitSha,
              entryId: ENTRY_ID,
            },);

            /**
             Artifact file as written, with one measurement bumped.
             */
            const artifactPath = join(
              archive.path,
              'first',
              `${ENTRY_ID}.json`,
            );
            /**
             Artifact as parsed, retampered at its recorded measurements.
             */
            const tampered = JSON.parse(await readFile(artifactPath, 'utf8',),) as {
              preparation: { sourceChars: number; };
            };
            tampered.preparation.sourceChars += 1;
            await writeFile(artifactPath, JSON.stringify(tampered,),);

            /**
             Reading of the tampered artifact.
             */
            const reading = await readArtifactSubjects({
              archiveDir: archive.path,
              runSetDir: 'first',
              runSet: 'first',
              artifactFile: `${ENTRY_ID}.json`,
              cloneDir: corpus.cloneDir,
              readFile: readCorpusFile,
            },);
            expect(reading.verification,).toEqual({
              kind: 'refused',
              detail: 'artifact parse failed at mittens.preparation.sourceChars: expected 104, which is what this '
                + 'preparation measures, rather than 105.',
            },);
          },
        },),

        it({
          name: 'SEPARATES a slice carrying the archive\'s own wording from one carrying a fresh '
            + 'rendering, because the instrument was built for output with no BEFORE text and reading '
            + 'both in one denominator would blur its first real measurement',
          fn: async () => {
            await using corpus = await makeCorpus();
            await using archive = await scratchDir({ prefix: 'settled-audit-archive-', },);

            await writeArtifact({
              archiveDir: archive.path,
              runSet: 'first',
              prepared: prepareDocumentPair({
                sourceText: SOURCE_PAGE,
                targetText: TARGET_PAGE,
              },),
              corpusSha: corpus.commitSha,
              entryId: ENTRY_ID,
            },);

            /**
             Subjects the artifact offers.
             */
            const { subjects, } = await readArtifactSubjects({
              archiveDir: archive.path,
              runSetDir: 'first',
              runSet: 'first',
              artifactFile: `${ENTRY_ID}.json`,
              cloneDir: corpus.cloneDir,
              readFile: readCorpusFile,
            },);

            expect(subjects[0]?.deliveryKind,).toBe('replacement-shipped',);
            expect(subjects[0]?.auditsArchiveText,).toBe(false,);

            /**
             Every slice after the first, all of which kept the archive.
             */
            const retained = subjects.slice(1,);
            expect(retained.length > 0,).toBe(true,);

            /**
             Whether every one of them is marked as the archive's own wording.
             */
            const allArchive = retained.every(function keptArchive(subject,): boolean {
              return subject.auditsArchiveText;
            },);
            expect(allArchive,).toBe(true,);
          },
        },),

        it({
          name: 'CARRIES the declared names the producing judges had, so an auditor is not shown a '
            + 'rendering whose name it cannot derive from the source and left to call it a fabrication',
          fn: async () => {
            await using corpus = await makeCorpus();
            await using archive = await scratchDir({ prefix: 'settled-audit-archive-', },);

            await writeArtifact({
              archiveDir: archive.path,
              runSet: 'first',
              prepared: prepareDocumentPair({
                sourceText: SOURCE_PAGE,
                targetText: TARGET_PAGE,
              },),
              corpusSha: corpus.commitSha,
              entryId: ENTRY_ID,
            },);

            /**
             Subjects the artifact offers.
             */
            const { subjects, } = await readArtifactSubjects({
              archiveDir: archive.path,
              runSetDir: 'first',
              runSet: 'first',
              artifactFile: `${ENTRY_ID}.json`,
              cloneDir: corpus.cloneDir,
              readFile: readCorpusFile,
            },);

            /**
             Identity block the first subject carries.
             */
            const identity = subjects[0]?.identity;
            expect(identity?.kind,).toBe('declared',);
            expect(identity?.kind === 'declared' ? identity.context : '',).toContain('Mittens',);
          },
        },),

        it({
          name: 'SHOWS THE AUDITORS THE DATE OF DEATH the run\'s models saw, read from the entry\'s info.yml at the '
            + 'ARTIFACT\'S OWN COMMIT rather than at whatever the clone now holds',
          fn: async () => {
            await using corpus = await makeCorpus({ infoYml: 'info:\n  died: 2025-02-26\n', },);
            await using archive = await scratchDir({ prefix: 'settled-audit-archive-', },);

            /**
             Preparation the run made, whose identity block carries the recorded death.
             */
            const prepared = prepareDocumentPair({
              sourceText: SOURCE_PAGE,
              targetText: TARGET_PAGE,
              sourceEntryInfo: { info: { died: '2025-02-26', }, },
            },);
            await writeArtifact({
              archiveDir: archive.path,
              runSet: 'first',
              prepared,
              corpusSha: corpus.commitSha,
              entryId: ENTRY_ID,
            },);
            // The record is edited AFTER the artifact settled, so a reader that followed the clone would show another date.
            await commitEntry({
              cloneDir: corpus.cloneDir,
              entryId: ENTRY_ID,
              sourcePage: SOURCE_PAGE,
              targetPage: TARGET_PAGE,
              infoYml: 'info:\n  died: 2031-01-01\n',
            },);

            /**
             Subjects the artifact offers.
             */
            const { subjects, } = await readArtifactSubjects({
              archiveDir: archive.path,
              runSetDir: 'first',
              runSet: 'first',
              artifactFile: `${ENTRY_ID}.json`,
              cloneDir: corpus.cloneDir,
              readFile: readCorpusFile,
            },);
            expect(subjects[0]?.identity,).toEqual({
              kind: 'declared',
              context: prepared.identityContext,
            },);
            expect(prepared.identityContext?.includes('"2025-02-26"',),).toBe(true,);
          },
        },),

        it({
          name: 'LEAVES THE DATE OF DEATH OUT of the replay of an artifact of generation fourteen, whose run never read '
            + 'an info.yml and told its models no date, and reads no info.yml for it, though the entry now records one',
          fn: async () => {
            // A DAMAGED info.yml: a replay that read it for this artifact would be refused.
            await using corpus = await makeCorpus({ infoYml: 'info: [\n', },);
            await using archive = await scratchDir({ prefix: 'settled-audit-archive-', },);

            /**
             Preparation the run made, which was told no death.
             */
            const prepared = prepareDocumentPair({
              sourceText: SOURCE_PAGE,
              targetText: TARGET_PAGE,
            },);
            await writeArtifact({
              archiveDir: archive.path,
              runSet: 'first',
              prepared,
              corpusSha: corpus.commitSha,
              entryId: ENTRY_ID,
            },);
            /**
             Where the artifact sits in the archive.
             */
            const artifactPath = join(
              archive.path,
              'first',
              `${ENTRY_ID}.json`,
            );
            // Restamped as the generation before the one that tells the death.
            await writeFile(
              artifactPath,
              restampedAsGenerationFourteen({ text: await readFile(artifactPath, 'utf8',), },),
            );

            /**
             Subjects the artifact offers.
             */
            const { subjects, } = await readArtifactSubjects({
              archiveDir: archive.path,
              runSetDir: 'first',
              runSet: 'first',
              artifactFile: `${ENTRY_ID}.json`,
              cloneDir: corpus.cloneDir,
              readFile: readCorpusFile,
            },);
            expect(subjects[0]?.identity,).toEqual({
              kind: 'declared',
              context: prepared.identityContext,
            },);
            expect(prepared.identityContext?.includes('died',),).toBe(false,);
          },
        },),

        it({
          name: 'OFFERS THE ARTIFACT of the generation that reads the info.yml although that file is damaged at its own commit, '
            + 'with the identity block of a run that recorded no death for the entry, instead of refusing the replay',
          fn: async () => {
            await using corpus = await makeCorpus({ infoYml: 'info: [\n', },);
            await using archive = await scratchDir({ prefix: 'settled-audit-archive-', },);

            /**
             Preparation the run made, which recorded no death for the entry.
             */
            const prepared = prepareDocumentPair({
              sourceText: SOURCE_PAGE,
              targetText: TARGET_PAGE,
            },);
            await writeArtifact({
              archiveDir: archive.path,
              runSet: 'first',
              prepared,
              corpusSha: corpus.commitSha,
              entryId: ENTRY_ID,
            },);

            /**
             Subjects the artifact offers.
             */
            const { subjects, } = await readArtifactSubjects({
              archiveDir: archive.path,
              runSetDir: 'first',
              runSet: 'first',
              artifactFile: `${ENTRY_ID}.json`,
              cloneDir: corpus.cloneDir,
              readFile: readCorpusFile,
            },);
            expect(subjects[0]?.identity,).toEqual({
              kind: 'declared',
              context: prepared.identityContext,
            },);
          },
        },),

        it({
          name: 'reports NO DECLARED NAMES as a positive answer when the pair declares none, rather '
            + 'than as a missing field, so a later reader can tell "this pair declared nothing" from '
            + '"nobody recorded whether it did"',
          fn: async () => {
            await using corpus = await makeCorpus();
            await using archive = await scratchDir({ prefix: 'settled-audit-archive-', },);

            /**
             A second entry whose pages carry no front matter, committed after the
             first.
             */
            const bareSha = await commitEntry({
              cloneDir: corpus.cloneDir,
              entryId: 'tabby',
              sourcePage: BARE_SOURCE_PAGE,
              targetPage: BARE_TARGET_PAGE,
            },);
            await writeArtifact({
              archiveDir: archive.path,
              runSet: 'first',
              prepared: prepareDocumentPair({
                sourceText: BARE_SOURCE_PAGE,
                targetText: BARE_TARGET_PAGE,
              },),
              corpusSha: bareSha,
              entryId: 'tabby',
            },);

            /**
             Subjects that artifact offers.
             */
            const { subjects, } = await readArtifactSubjects({
              archiveDir: archive.path,
              runSetDir: 'first',
              runSet: 'first',
              artifactFile: 'tabby.json',
              cloneDir: corpus.cloneDir,
              readFile: readCorpusFile,
            },);

            expect(subjects[0]?.identity
              .kind,).toBe('none',);
          },
        },),

        it({
          name: 'reads the corpus at the ARTIFACT\'S OWN COMMIT rather than at whatever the clone now '
            + 'points to, so a settled file keeps answering for itself after the pair is edited',
          fn: async () => {
            await using corpus = await makeCorpus();
            await using archive = await scratchDir({ prefix: 'settled-audit-archive-', },);

            await writeArtifact({
              archiveDir: archive.path,
              runSet: 'first',
              prepared: prepareDocumentPair({
                sourceText: SOURCE_PAGE,
                targetText: TARGET_PAGE,
              },),
              corpusSha: corpus.commitSha,
              entryId: ENTRY_ID,
            },);

            // The pair is edited AFTER the artifact settled, which is the whole
            // point: a reader that followed the clone would now prepare a different
            // identity block and verify against the wrong documents.
            await commitEntry({
              cloneDir: corpus.cloneDir,
              entryId: ENTRY_ID,
              sourcePage: SOURCE_PAGE.split('毛毛',)
                .join('大毛',),
              targetPage: TARGET_PAGE.split('Mittens',)
                .join('Bigpaw',),
            },);

            /**
             Artifact read after the clone moved on.
             */
            const reading = await readArtifactSubjects({
              archiveDir: archive.path,
              runSetDir: 'first',
              runSet: 'first',
              artifactFile: `${ENTRY_ID}.json`,
              cloneDir: corpus.cloneDir,
              readFile: readCorpusFile,
            },);

            expect(reading.verification
              .kind,).toBe('verified',);

            /**
             Identity block, which must still name the cat the run licensed.
             */
            const identity = reading.subjects[0]?.identity;
            expect(identity?.kind === 'declared' ? identity.context : '',).toContain('Mittens',);
            expect(identity?.kind === 'declared' ? identity.context : '',).not
              .toContain('Bigpaw',);
          },
        },),

        it({
          name: 'REPORTS a preparation that no longer describes the pair instead of throwing, because a '
            + 'slicing that moved under a settled artifact is a finding about that artifact and not a '
            + 'reason to refuse to read the rows it settled',
          fn: async () => {
            await using corpus = await makeCorpus();
            await using archive = await scratchDir({ prefix: 'settled-audit-archive-', },);

            // Written over a DIFFERENT pair than the one the entry id resolves to,
            // which is what a moved slicing looks like from the reader's side.
            // WITH A COMPLETE RECIPE: the block rounds were asked and agreed
            // nothing, so the file records both halves and a mismatch is a
            // refusal rather than a gap.
            await writeArtifact({
              archiveDir: archive.path,
              runSet: 'first',
              prepared: prepareDocumentPair({
                sourceText: BARE_SOURCE_PAGE,
                targetText: BARE_TARGET_PAGE,
                blockPairings: new Map(),
              },),
              corpusSha: corpus.commitSha,
              entryId: ENTRY_ID,
            },);

            /**
             Artifact read against a preparation it does not describe.
             */
            const reading = await readArtifactSubjects({
              archiveDir: archive.path,
              runSetDir: 'first',
              runSet: 'first',
              artifactFile: `${ENTRY_ID}.json`,
              cloneDir: corpus.cloneDir,
              readFile: readCorpusFile,
            },);

            expect(reading.verification
              .kind,).toBe('refused',);
            // The rows still arrive. They are what the run actually judged.
            expect(reading.subjects.length > 0,).toBe(true,);
          },
        },),
        it({
          name: 'VERIFIES an artifact whose rows the rebuild reproduces though its recorded identity differs, '
            + 'since the identity also hashes the declared names as the run\'s build worded them (ledger A12: '
            + 'every settled artifact read REFUSED)',
          fn: async () => {
            await using corpus = await makeCorpus();
            await using archive = await scratchDir({ prefix: 'settled-audit-archive-', },);

            /**
             The run's carve of the pair the entry id resolves to.
             */
            const prepared = prepareDocumentPair({
              sourceText: SOURCE_PAGE,
              targetText: TARGET_PAGE,
            },);
            await writeArtifact({
              archiveDir: archive.path,
              runSet: 'first',
              // The same carve, its declared names worded as an older build
              // worded them.
              prepared: {
                ...prepared,
                identityContext: `${prepared.identityContext ?? ''}\n- an older wording of the cat's names`,
              },
              corpusSha: corpus.commitSha,
              entryId: ENTRY_ID,
            },);

            /**
             Artifact read against today's rebuild of that carve.
             */
            const reading = await readArtifactSubjects({
              archiveDir: archive.path,
              runSetDir: 'first',
              runSet: 'first',
              artifactFile: `${ENTRY_ID}.json`,
              cloneDir: corpus.cloneDir,
              readFile: readCorpusFile,
            },);

            expect(reading.verification
              .kind,).toBe('verified',);
          },
        },),
        it({
          name:
            'VERIFIES an artifact built over a roster-paired preparation, by rebuilding with the recipe it '
              + 'records: the bare carve disagrees with it, and until the rebuild every such artifact read '
              + 'refused, on exactly the artifacts the check exists for',
          fn: async () => {
            await using corpus = await makeCorpus();
            await using archive = await scratchDir({ prefix: 'settled-audit-archive-', },);

            /**
             How a roster run carved it: sections crossed, block rounds asked.
             */
            const paired = prepareDocumentPair({
              sourceText: SOURCE_PAGE,
              targetText: TARGET_PAGE,
              sectionPairing: [{
                source: 0,
                target: 1,
              },],
              blockPairings: new Map(),
            },);

            // POSITIVE CONTROL: the bare carve has to disagree, or this would be
            // verifying an artifact the deterministic rebuild reproduces anyway.
            expect(preparationIdentity({ prepared: paired, },),).not
              .toBe(preparationIdentity({
                prepared: prepareDocumentPair({
                  sourceText: SOURCE_PAGE,
                  targetText: TARGET_PAGE,
                },),
              },),);
            await writeArtifact({
              archiveDir: archive.path,
              runSet: 'first',
              prepared: paired,
              corpusSha: corpus.commitSha,
              entryId: ENTRY_ID,
            },);

            /**
             Artifact read against the corpus with its own recipe.
             */
            const reading = await readArtifactSubjects({
              archiveDir: archive.path,
              runSetDir: 'first',
              runSet: 'first',
              artifactFile: `${ENTRY_ID}.json`,
              cloneDir: corpus.cloneDir,
              readFile: readCorpusFile,
            },);
            expect(reading.verification
              .kind,).toBe('verified',);
          },
        },),
        it({
          name:
            'REPORTS a mismatch beside a missing recipe half as unverifiable rather than refused, naming the '
              + 'halves the file lacks: the rebuild guessed those halves, so the disagreement may be the guess '
              + 'rather than a moved slicing, which is the honest verdict for every artifact settled before '
              + 'the recipe was recorded',
          fn: async () => {
            await using corpus = await makeCorpus();
            await using archive = await scratchDir({ prefix: 'settled-audit-archive-', },);

            // Written over a DIFFERENT pair, as an old file: no recipe at all.
            await writeArtifact({
              archiveDir: archive.path,
              runSet: 'first',
              prepared: prepareDocumentPair({
                sourceText: BARE_SOURCE_PAGE,
                targetText: BARE_TARGET_PAGE,
              },),
              corpusSha: corpus.commitSha,
              entryId: ENTRY_ID,
              strip: [
                'sectionPairing',
                'blockPairing',
              ],
            },);

            /**
             Artifact read against a pair it does not describe, with no recipe.
             */
            const reading = await readArtifactSubjects({
              archiveDir: archive.path,
              runSetDir: 'first',
              runSet: 'first',
              artifactFile: `${ENTRY_ID}.json`,
              cloneDir: corpus.cloneDir,
              readFile: readCorpusFile,
            },);

            /**
             Verdict, narrowed for its fields.
             */
            const { verification, } = reading;
            expect(verification.kind,).toBe('unverifiable',);
            if (verification.kind !== 'unverifiable')
              throw new Error('unreachable: the kind was checked',);
            expect(verification.unrecorded,).toEqual([
              'sectionPairing',
              'blockPairing',
            ],);
            // The first departure from the recorded rows, which is what the verdict
            // now reads rather than the identity (ledger A12b).
            expect(verification.detail,).toContain('slice',);
          },
        },),

        it({
          name: 'REFUSES with the page.md read when both page reads of a commit the clone lacks are refused and the '
            + 'page.en.md read is refused first',
          fn: async () => {
            await using archive = await scratchDir({ prefix: 'settled-audit-archive-', },);
            await writeArtifact({
              archiveDir: archive.path,
              runSet: 'first',
              prepared: prepareDocumentPair({
                sourceText: SOURCE_PAGE,
                targetText: TARGET_PAGE,
              },),
              corpusSha: LACKED_COMMIT,
              entryId: ENTRY_ID,
            },);

            /**
             What the reading refused with.
             */
            const refusal = await rejectionOf({
              promise: readArtifactSubjects({
                archiveDir: archive.path,
                runSetDir: 'first',
                runSet: 'first',
                artifactFile: `${ENTRY_ID}.json`,
                cloneDir: '/nonexistent/clone',
                readFile: pageReadsRefusingLastFor({
                  endsLast: '/page.md',
                  commit: 'lacking',
                },),
              },),
            },);

            expect(String(refusal,),).toBe(
              `CorpusReadError: corpus read failed for ${LACKED_COMMIT}:people/${ENTRY_ID}/page.md (missing-commit); `
                + 'the clone holds no commit by that revision: fetch it, or pin a commit the clone holds.',
            );
          },
        },),
      ],
    },),

    describe({
      name: readArchiveSubjects.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'reads every run set in a stable order, so a capped run always buys the same prefix and '
            + 'two invocations can be compared row against row',
          fn: async () => {
            await using corpus = await makeCorpus();
            await using archive = await scratchDir({ prefix: 'settled-audit-archive-', },);

            /**
             Preparation both run sets were settled over.
             */
            const prepared = prepareDocumentPair({
              sourceText: SOURCE_PAGE,
              targetText: TARGET_PAGE,
            },);

            // Written second-first, so a reader that trusted directory order rather
            // than sorting would return them the other way round.
            await writeArtifact({
              archiveDir: archive.path,
              runSet: 'second-run',
              prepared,
              corpusSha: corpus.commitSha,
              entryId: ENTRY_ID,
            },);
            await writeArtifact({
              archiveDir: archive.path,
              runSet: 'first-run',
              prepared,
              corpusSha: corpus.commitSha,
              entryId: ENTRY_ID,
            },);

            /**
             Everything the archive holds.
             */
            const readings = await readArchiveSubjects({
              archiveDir: archive.path,
              cloneDir: corpus.cloneDir,
              readFile: readCorpusFile,
              listFiles: listArtifactFiles,
            },);

            expect(readings.length,).toBe(2,);
            expect(readings[0]?.runSet,).toBe('first-run',);
            expect(readings[1]?.runSet,).toBe('second-run',);

            // Two runs of one entry write the same file name, so the run set is the
            // only thing telling their rows apart.
            expect(readings[0]?.artifactFile,).toBe(readings[1]?.artifactFile,);
          },
        },),

        it({
          name: 'IGNORES anything that is not an artifact, since a run directory collects logs and notes '
            + 'beside its artifacts and reading one as JSON would end the census at that file',
          fn: async () => {
            await using corpus = await makeCorpus();
            await using archive = await scratchDir({ prefix: 'settled-audit-archive-', },);

            await writeArtifact({
              archiveDir: archive.path,
              runSet: 'first-run',
              prepared: prepareDocumentPair({
                sourceText: SOURCE_PAGE,
                targetText: TARGET_PAGE,
              },),
              corpusSha: corpus.commitSha,
              entryId: ENTRY_ID,
            },);
            await writeFile(
              join(
                archive.path,
                'first-run',
                'run.log',
              ),
              'the cat sat on the log\n',
              'utf8',
            );

            /**
             Everything the archive holds.
             */
            const readings = await readArchiveSubjects({
              archiveDir: archive.path,
              cloneDir: corpus.cloneDir,
              readFile: readCorpusFile,
              listFiles: listArtifactFiles,
            },);

            expect(readings.length,).toBe(1,);
            expect(readings[0]?.entryId,).toBe(ENTRY_ID,);
          },
        },),

        it({
          name: 'IGNORES A DIRECTORY NAMED LIKE AN ARTIFACT inside a run set, as the loose layout, the census and the '
            + 'scheduler do: only a regular file is an artifact (ledger B64)',
          fn: async () => {
            await using corpus = await makeCorpus();
            await using archive = await scratchDir({ prefix: 'settled-audit-archive-', },);

            await writeArtifact({
              archiveDir: archive.path,
              runSet: 'first-run',
              prepared: prepareDocumentPair({
                sourceText: SOURCE_PAGE,
                targetText: TARGET_PAGE,
              },),
              corpusSha: corpus.commitSha,
              entryId: ENTRY_ID,
            },);
            await mkdir(
              join(
                archive.path,
                'first-run',
                'saved.json',
              ),
              { recursive: true, },
            );

            /**
             Everything the archive holds.
             */
            const readings = await readArchiveSubjects({
              archiveDir: archive.path,
              cloneDir: corpus.cloneDir,
              readFile: readCorpusFile,
              listFiles: listArtifactFiles,
            },);

            expect(readings.length,).toBe(1,);
            expect(readings[0]?.entryId,).toBe(ENTRY_ID,);
          },
        },),

        it({
          name: 'READS THE FLAT LAYOUT A CORPUS RUN ACTUALLY WRITES, artifacts straight under the directory '
            + 'with no run-set subdirectory. corpus-pass produces one settlement so it invents no '
            + 'subdirectory for it, and before this the audit refused such a directory with "no '
            + 'artifacts under", which reads like an empty pass rather than like a layout it cannot see',
          fn: async () => {
            await using corpus = await makeCorpus();
            await using archive = await scratchDir({ prefix: 'settled-audit-archive-', },);

            /**
             Pair both sides describe.
             */
            const prepared = prepareDocumentPair({
              sourceText: SOURCE_PAGE,
              targetText: TARGET_PAGE,
            },);

            // Written with an EMPTY run set, which puts the file at the archive
            // root exactly as a pass writes it.
            await writeArtifact({
              archiveDir: archive.path,
              runSet: '',
              prepared,
              corpusSha: corpus.commitSha,
              entryId: ENTRY_ID,
            },);

            /**
             Everything the flat directory holds.
             */
            const readings = await readArchiveSubjects({
              archiveDir: archive.path,
              cloneDir: corpus.cloneDir,
              readFile: readCorpusFile,
              listFiles: listArtifactFiles,
            },);

            expect(readings.length,).toBe(1,);
            expect(readings[0]?.entryId,).toBe(ENTRY_ID,);

            // The directory names the settlement, since there is exactly one and a
            // reader tracing a row back wants the directory it came from.
            expect(readings[0]?.runSet.length,).toBeGreaterThan(0,);
          },
        },),

        it({
          name: 'REFUSES A DIRECTORY CARRYING BOTH LAYOUTS rather than choosing one, because artifacts '
            + 'at the root AND in subdirectories are either two populations or a half-finished move, '
            + 'and reading one while ignoring the other would report a population smaller than the '
            + 'archive holds without saying so',
          fn: async () => {
            await using corpus = await makeCorpus();
            await using archive = await scratchDir({ prefix: 'settled-audit-archive-', },);

            /**
             Pair both sides describe.
             */
            const prepared = prepareDocumentPair({
              sourceText: SOURCE_PAGE,
              targetText: TARGET_PAGE,
            },);

            await writeArtifact({
              archiveDir: archive.path,
              runSet: '',
              prepared,
              corpusSha: corpus.commitSha,
              entryId: ENTRY_ID,
            },);
            await writeArtifact({
              archiveDir: archive.path,
              runSet: 'nested-run',
              prepared,
              corpusSha: corpus.commitSha,
              entryId: ENTRY_ID,
            },);

            /**
             What the reader says about the mixture.
             */
            const refusal = await readArchiveSubjects({
              archiveDir: archive.path,
              cloneDir: corpus.cloneDir,
              readFile: readCorpusFile,
              listFiles: listArtifactFiles,
            },).then(
              function unexpected(): string {
                return 'no refusal';
              },
              String,
            );

            expect(refusal.includes('at its root AND in subdirectories',),).toBe(true,);
            // STATED, so the CLI boundary prints the line and exits 6 rather than
            // printing frames for a bug that is not one.
            expect(refusal.startsWith('StatedRefusalError:',),).toBe(true,);
          },
        },),

        it({
          name: 'REFUSES with the first artifact\'s page read when two artifacts of one run set are refused and the '
            + 'second artifact\'s reads are refused first',
          fn: async () => {
            await using archive = await scratchDir({ prefix: 'settled-audit-archive-', },);
            /**
             Preparation both artifacts were settled over.
             */
            const prepared = prepareDocumentPair({
              sourceText: SOURCE_PAGE,
              targetText: TARGET_PAGE,
            },);
            await writeArtifact({
              archiveDir: archive.path,
              runSet: 'first-run',
              prepared,
              corpusSha: LACKED_COMMIT,
              entryId: 'cat-alpha',
            },);
            await writeArtifact({
              archiveDir: archive.path,
              runSet: 'first-run',
              prepared,
              corpusSha: LACKED_COMMIT,
              entryId: 'cat-beta',
            },);

            /**
             What the archive reading refused with.
             */
            const refusal = await rejectionOf({
              promise: readArchiveSubjects({
                archiveDir: archive.path,
                cloneDir: '/nonexistent/clone',
                readFile: pageReadsRefusingLastFor({
                  endsLast: 'people/cat-alpha/',
                  commit: 'lacking',
                },),
                listFiles: listArtifactFiles,
              },),
            },);

            expect(String(refusal,),).toBe(
              `CorpusReadError: corpus read failed for ${LACKED_COMMIT}:people/cat-alpha/page.md (missing-commit); `
                + 'the clone holds no commit by that revision: fetch it, or pin a commit the clone holds.',
            );
          },
        },),

        it({
          name: 'REFUSES with the first run set\'s listing when two run sets cannot be listed and the second '
            + 'run set\'s listing is refused first',
          fn: async () => {
            await using archive = await scratchDir({ prefix: 'settled-audit-archive-', },);
            /**
             Preparation both run sets were settled over.
             */
            const prepared = prepareDocumentPair({
              sourceText: SOURCE_PAGE,
              targetText: TARGET_PAGE,
            },);
            await writeArtifact({
              archiveDir: archive.path,
              runSet: 'first-run',
              prepared,
              corpusSha: LACKED_COMMIT,
              entryId: ENTRY_ID,
            },);
            await writeArtifact({
              archiveDir: archive.path,
              runSet: 'second-run',
              prepared,
              corpusSha: LACKED_COMMIT,
              entryId: ENTRY_ID,
            },);
            /**
             The two refusals the listings end in.
             */
            const { refuseAtOnce, refuseAfterThat, } = refusalOrder();

            /**
             What the archive reading refused with.
             */
            const refusal = await rejectionOf({
              promise: readArchiveSubjects({
                archiveDir: archive.path,
                cloneDir: '/nonexistent/clone',
                readFile: pageReadsRefusingLastFor({
                  endsLast: 'no such page',
                  commit: 'lacking',
                },),
                listFiles: async function listsRunSetsOutOfOrder(
                  { artifactsDir, }: Parameters<typeof listArtifactFiles>[0],
                ): Promise<Awaited<ReturnType<typeof listArtifactFiles>>> {
                  if (artifactsDir.endsWith('first-run',))
                    return await refuseAfterThat(new Error('the first run set cannot be listed',),);
                  if (artifactsDir.endsWith('second-run',))
                    return await refuseAtOnce(new Error('the second run set cannot be listed',),);
                  return [];
                },
              },),
            },);

            expect(String(refusal,),).toBe('Error: the first run set cannot be listed',);
          },
        },),
      ],
    },),
  ],
},);
