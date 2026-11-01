import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { allInInputOrder, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import {
  citedReferenceUrlsOf,
  EXA_API_KEY_VAR,
} from '@monochromatic-dev/translation-repair-reference/ts';
import { codePointLength, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  RenderingAuditInvariantError,
  runRenderingAudit,
} from '@monochromatic-dev/translation-repair-rendering-audit/ts';
import {
  askedAmong,
  RUN_MODELS,
  RUN_PER_CALL_TIMEOUT_MS,
} from '@monochromatic-dev/translation-repair-run/ts';
import type { PassReferenceReader, } from '@monochromatic-dev/translation-repair-corpus-pass/ts';
import { digestAuditedText, } from './rendering-audit-settled-digest.ts';
import type { SettledArtifactReading, } from './rendering-audit-settled-input.ts';
import {
  type SettledAuditRow,
  SETTLED_AUDIT_PROBE,
} from './rendering-audit-settled-row.ts';
import type {
  SettledAuditSubject,
  SettledReferences,
} from './rendering-audit-settled-subject.ts';
import type { StageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';
import type { ReadonlyEntry, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Settled audit buying
// What the settled rendering audit buys, in what order, what it prints before
// buying, and what one bought subject becomes.
//
// SPLIT OUT OF THE ENTRY so the built bundle can export these for its tests.
// An entry module runs its `main` under `import.meta.main`, and a module the
// barrel imports stops being an entry: the bundler folds it into a shared
// chunk, the entry file becomes a re-export of that chunk, and the guard inside
// the chunk reads false, so the command prints nothing and exits 0. Both
// settled-audit commands did exactly that for the length of one commit
// (`744890056`), which the boundary cases in
// `rendering-audit-settled-report.unit.test.ts` now catch. Nothing an entry
// module declares may be exported through a barrel; it moves here instead.

/**
 Audits one slice and keeps what the roster said, whole.

 Exported through the barrel so the built bundle's tests can hand it a
 scripted client; `runSettledAudit` (`rendering-audit-settled-drive.ts`) is
 its only caller.

 @internal

 @param subject - slice under audit, with the identity its producing run had

 @param references - what the pages its original links say, as
 `withCitedReferences` read them, or why none are shown; required so no
 caller audits without having asked

 @param client - roster client, built once per run by the caller so every
 subject counts into one seat tally

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns One row: provenance, plus the report uninterpreted

 @example
 ```ts
 const row = await auditOne({ subject, references, client, stageSettings, },);
 ```
 */
export async function auditOne(
  {
    subject,
    references,
    client,
    stageSettings,
  }: {
    readonly subject: SettledAuditSubject;
    readonly references: SettledReferences;
    readonly client: ModelClient;
    readonly stageSettings: StageSettings;
  },
): Promise<SettledAuditRow> {
  /**
   Logger tagged for this slice, so a long run's stream says where it is.
   */
  const l = tagged({ tag: `${SETTLED_AUDIT_PROBE}:${subject.entryId}:${String(subject.sliceIndex,)}`, },);

  /**
   Everything the reader put in front of the audit for this slice.
   */
  const {
    runSet,
    entryId,
    sliceIndex,
    deliveryKind,
    auditsArchiveText,
    pageRelation,
    artifactDigest,
    corpusSha,
    sourceText,
    candidateText,
    identity,
  } = subject;

  /**
   What the roster said about this rendering.

   The identity block goes in when the pair declared one: the producing judges
   had it, and an auditor without it has every reason to call a declared name
   a fabrication. The references go in for the same reason (ledger B29):
   the producing critics and panels read them, and an auditor without them
   calls a detail a cited page states an unsupported addition.
   */
  const report = await runRenderingAudit({
    client,
    subject: {
      sourceText,
      candidateText,
      ...((identity.kind === 'declared') ? { identityContext: identity.context, } : {}),
      ...((references.kind === 'cited') ? { referenceContext: references.context, } : {}),
    },
    modelIds: RUN_MODELS.checkerModelIds,
    signal: new AbortController().signal,
    perCallTimeoutMs: RUN_PER_CALL_TIMEOUT_MS,
    l,
    stageSettings,
  },);

  return {
    runSet,
    entryId,
    sliceIndex,
    deliveryKind,
    auditsArchiveText,
    pageRelation,
    artifactDigest,
    corpusSha,
    identityKind: identity.kind,
    referencesKind: references.kind,
    // Digested rather than kept, so a run file can say whether two rows saw one
    // text without carrying licensed material into a file that gets quoted.
    textIdentity: digestAuditedText({
      sourceText,
      candidateText,
      // Only what was shown: an unread row was shown none, so it keys as one.
      referenceContext: (references.kind === 'cited') ? references.context : '',
    },),
    report,
  };
}

/**
 One bought subject beside what the pages its original links say.

 @example
 ```ts
 const pair: CitedSubject = { subject, references: { kind: 'none', }, };
 ```
 */
export type CitedSubject = {
  /**
   Slice to audit.
   */
  readonly subject: SettledAuditSubject;

  /**
   What its page cites, or why the audit shows none.
   */
  readonly references: SettledReferences;
};

/**
 Reads what the pages each bought subject's original links say, once per
 page, and pairs every subject with its own page's.

 ONCE PER PAGE rather than once per subject: every slice of one page shares
 one original, so a second read could only cost a fetch or disagree with
 the first. Pages are keyed by their whole text, since two artifacts of one
 entry read at different commits may carry different originals.

 ONLY WHAT IS BOUGHT, so the wiring check that buys nothing reads nothing
 and spends nothing. The reader is required rather than defaulted to the
 run's, because a unit test that forgot it would read the environment's
 key and buy a real web read (ledger X19).

 Exported through the barrel for the built bundle's tests;
 `runSettledAudit` (`rendering-audit-settled-drive.ts`) is its only caller.

 @internal

 @param subjects - subjects the run will buy, in buying order

 @param reader - reads what one original's linked pages say

 @returns Every subject with its page's references, in buying order; a page
 that links pages and reads as nothing is `unread`, never `none`

 @throws {@link RenderingAuditInvariantError} when a subject's page was never
 read, which cannot happen while the pages read are these subjects' own

 @example
 ```ts
 const cited = await withCitedReferences({ subjects: buying, reader: outsideReads.references, },);
 ```
 */
export async function withCitedReferences(
  {
    subjects,
    reader,
  }: {
    readonly subjects: readonly SettledAuditSubject[];
    readonly reader: PassReferenceReader;
  },
): Promise<readonly CitedSubject[]> {
  /**
   Logger for the reads, so a run's stream says what each page cited.
   */
  const l = tagged({ tag: `${SETTLED_AUDIT_PROBE}:${withCitedReferences.name}`, },);

  /**
   Each distinct page once, in the order it is first bought.
   */
  const pages = [
    ...new Map(
      subjects.map(function byPage(subject,): readonly [
        string,
        string,
      ] {
        return [
          subject.pageSourceText,
          subject.entryId,
        ];
      },),
    ),
  ];

  /**
   Every page beside what it cites, read concurrently: pages are few, one
   per bought artifact at most.
   */
  const readPages = await allInInputOrder({
    members: pages.map(
      async function readPage([
        pageSourceText,
        entryId,
      ]: ReadonlyEntry<string, string>,): Promise<{
        readonly pageSourceText: string;
        readonly entryId: string;
        readonly references: SettledReferences;
      }> {
        /**
         What this page's links say, empty where there are none or none
         could be read.
         */
        const read = await reader({
          sourceText: pageSourceText,
          signal: new AbortController().signal,
          l,
        },);
        if (read !== '') {
          return {
            pageSourceText,
            entryId,
            references: {
              kind: 'cited',
              context: read,
            },
          };
        }

        /**
         Pages this original links, which tell a page citing nothing from
         a page nobody read.
         */
        const links = citedReferenceUrlsOf({ text: pageSourceText, },)
          .length;
        return {
          pageSourceText,
          entryId,
          references: (links === 0)
            ? { kind: 'none', }
            : {
              kind: 'unread',
              links,
            },
        };
      },
    ),
  },);

  // SAID IN BUYING ORDER, once every read has ended, so two runs over one
  // archive log the same lines; until 2026-10-10 each read logged as it
  // ended, in whichever order that was.
  for (
    const {
      entryId,
      references,
    } of readPages
  ) {
    if (references.kind === 'cited') {
      // IN CHARACTERS, counted as code points as the package counts a page's
      // characters (`codePointLength`); until 2026-10-10 the line printed the
      // text's UTF-16 length under the same name.
      l.info(`CITED REFERENCES entry=${entryId} cited, characters=${
        String(codePointLength({ text: references.context, },),)
      }`,);
    }
    if (references.kind === 'none')
      l.info(`CITED REFERENCES entry=${entryId} none: the page links nowhere`,);
    if (references.kind === 'unread') {
      l.warn(
        `CITED REFERENCES entry=${entryId} UNREAD: the page links ${String(references.links,)} page(s) and the read`
          + ` returned nothing, so its slices are audited without them and their rows record unread;`
          + ` the reference reader returns nothing for linked pages when ${EXA_API_KEY_VAR} is not set,`
          + ' so set it and audit again, or read these rows as audited without the references',
      );
    }
  }

  /**
   What each page cites, by its whole text.
   */
  const cited = new Map<string, SettledReferences>(readPages.map(function byPageText({
    pageSourceText,
    references,
  },): readonly [
    string,
    SettledReferences,
  ] {
    return [
      pageSourceText,
      references,
    ];
  },),);

  return subjects.map(function paired(subject,): CitedSubject {
    /**
     What this subject's page cites. The map is total over the subjects:
     it is built from this very list, one entry per page read, so a miss
     here is a defect and never a page that cites nothing.
     */
    const references = cited.get(subject.pageSourceText,);

    // Named rather than left to a bare non-null check, whose message says
    // only that a value was missing: the entry and the slice say which
    // purchase had no page read for it.
    if (references === undefined)
      throw new RenderingAuditInvariantError({
        invariant: `${subject.entryId} slice ${
          String(subject.sliceIndex,)
        } was bought and its page was never read for references`,
      },);
    return {
      subject,
      references,
    };
  },);
}

/**
 Every subject a run could buy, after the entry filter, in archive order.

 SEPARATE FROM THE CAP so the fraction a capped run reports is over what was
 SELECTABLE rather than over the whole archive. Reporting `5 of 40` where
 `--only` left 30 selectable overstates what was skipped and understates the
 coverage bought, and the line alone gives a reader no way to tell.

 Exported through the barrel for the built bundle's tests;
 `runSettledAudit` (`rendering-audit-settled-drive.ts`) is its only caller.

 @internal

 @param readings - every artifact the archive holds

 @param onlyIds - entries to keep, empty for all

 @returns Subjects the filter left

 @example
 ```ts
 const eligible = eligibleSubjects({ readings, onlyIds, },);
 ```
 */
export function eligibleSubjects(
  {
    readings,
    onlyIds,
  }: {
    readonly readings: readonly SettledArtifactReading[];
    readonly onlyIds: readonly string[];
  },
): readonly SettledAuditSubject[] {
  /**
   Entries the archive holds that the filter keeps.
   */
  const kept = new Set(askedAmong({
    asked: onlyIds,
    known: readings.map(function entryOf(reading,): string {
      return reading.entryId;
    },),
    source: '--only',
    within: 'the settled archive',
  },),);
  return readings
    .flatMap(function subjectsOf(reading,): readonly SettledAuditSubject[] {
      return reading.subjects;
    },)
    .filter(function isWanted(subject,): boolean {
      return kept.has(subject.entryId,);
    },);
}

/**
 Takes the prefix a cap allows.

 Exported through the barrel for the built bundle's tests;
 `runSettledAudit` (`rendering-audit-settled-drive.ts`) is its only caller. A
 negative cap reaches here only as the args module's own "every subject"
 sentinel, since `readAuditArguments` (`rendering-audit-settled-args.ts`)
 reads `--cap` through `wholeNumberFlag`, which refuses a typed sign.

 @internal

 @param eligible - subjects the filter left, in a stable order

 @param cap - how many to buy, negative for all

 @returns Subjects to audit

 @example
 ```ts
 const buying = capped({ eligible, cap, },);
 ```
 */
export function capped(
  {
    eligible,
    cap,
  }: {
    readonly eligible: readonly SettledAuditSubject[];
    readonly cap: number;
  },
): readonly SettledAuditSubject[] {
  if (cap < 0)
    return eligible;
  return eligible.slice(
    0,
    cap,
  );
}

/**
 Reports what the archive holds, before anything is bought.

 Exported through the barrel for the built bundle's tests;
 `runSettledAudit` (`rendering-audit-settled-drive.ts`) is its only caller.

 @internal

 @param readings - every artifact the archive holds

 @example
 ```ts
 printPopulation({ readings, },);
 ```
 */
export function printPopulation(
  { readings, }: { readonly readings: readonly SettledArtifactReading[]; },
): void {
  readings.forEach(function describe(reading,): void {
    /**
     Where this artifact came from, what it offers, and whether its recorded
     slicing still describes the pair.
     */
    const {
      runSet,
      artifactFile,
      subjects,
      verification,
    } = reading;

    /**
     Slices whose text is the archive's own wording.
     */
    const retained = subjects.filter(function isArchive(subject,): boolean {
      return subject.auditsArchiveText;
    },);

    /**
     Slices a later stage overruled, so the audit will read wording no
     reader of an assembled document would meet.

     PRINTED HERE because this whole reading is free, and knowing it before
     a roster is woken up is the reason this module is separate from the
     driver that spends quota.
     */
    const displaced = subjects.filter(function wasOverruled(subject,): boolean {
      /**
       How this subject relates to what a document would carry.
       */
      const { pageRelation, } = subject;
      return pageRelation.kind === 'displaced';
    },);

    /**
     Slices no stage has decided at all, which is the absence of a decision
     rather than one: the artifact records it as pending a human decision.
     */
    const undecided = subjects.filter(function wasNeverAsked(subject,): boolean {
      /**
       How this subject relates to what a document would carry.
       */
      const { pageRelation, } = subject;
      return pageRelation.kind === 'undecided';
    },);

    console.log(
      `${runSet}/${artifactFile}  subjects=${String(subjects.length,)} retained=${
        String(retained.length,)
      } replaced=${String(subjects.length - retained.length,)} displaced=${
        String(displaced.length,)
      } undecided=${String(undecided.length,)} verification=${verification.kind}`,
    );
    if (verification.kind === 'refused')
      console.log(`   REFUSED: ${verification.detail}`,);
    if (verification.kind === 'unverifiable') {
      /**
       Recipe halves the file lacks, which the rebuild had to guess.
       */
      const missing = verification.unrecorded
        .join(', ',);
      console.log(`   UNVERIFIABLE (records no ${missing}): ${verification.detail}`,);
    }
  },);
}

//endregion Settled audit buying
