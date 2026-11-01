import { join, } from 'node:path';

import { allInInputOrder, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  DECLINED_DIR,
  declinedEntryIds,
  digestPipeline,
} from '@monochromatic-dev/translation-repair-glossary/ts';
import {
  type EntryAgreement,
  pairPublishedPages,
  publishedEntryIds,
  settledEntryIds,
  whatThereIsToVerify,
} from '@monochromatic-dev/translation-repair-page-check/ts';
import {
  type JudgedEntry,
  judgePublishedEntry,
} from './verify-published-entry.ts';

//region Verify published run
// A run's PUBLISHED TREE read back against the artifacts that produced it, and
// the report of what was found. Moved out of `verify-published.ts`, which keeps
// the wiring: the runs directory and the directory of the built pipeline come in
// as arguments, and the exit code goes back to the entry.
//
// A RUN ALWAYS SHIPS (the owner, 2026-09-27; ledger A16b). Every finding is
// printed and the exit is 0; only a run that could not be read at all exits
// otherwise, since that run was never examined.

/**
 Exit code a run that could not be checked at all leaves behind.

 THE ONE NONZERO EXIT. A finding says what the run ships and what the next
 pass repairs, and a run always ships; this says the run was never examined,
 which no printed finding can stand in for.
 */
const NOTHING_WAS_VERIFIED = 2;

/**
 Exit code a run leaves behind when it was read to its end, whatever it found.
 */
const READ_TO_THE_END = 0;

/**
 Judges every matched entry side by side and hands back what each came to,
 in the order the entries were listed.

 A JUDGEMENT CAN STILL THROW. `judgePublishedEntry` answers every failure to
 read or parse an entry as a refused line, but it judges what it parsed
 through `pageAgreement`, outside that catch, and `wouldShipTextPerSlice`
 raises `UnansweredContestSliceError`, naming the slice, where a slice's
 lanes differ and the contest never answered it. Two entries that both raise
 would otherwise report whichever judgement ended first.

 @param entryIds - matched entries, in the order the run lists them, which a
 failure is reported in

 @param judge - judges one entry, passed in so a case scripts which of two
 judgements ends first

 @returns Each entry's verdict and lines, in the order given

 @throws Whatever the first entry in the order given, whose judging failed,
 was refused with

 @example
 ```ts
 const judged = await judgePublishedEntries({ entryIds: matched, judge: ({ entryId, },) => judgePublishedEntry({ runsDir, entryId, thisBuild, },), },);
 ```
 */
export async function judgePublishedEntries(
  {
    entryIds,
    judge,
  }: {
    readonly entryIds: readonly string[];
    readonly judge: (input: { readonly entryId: string; },) => Promise<JudgedEntry>;
  },
): Promise<readonly JudgedEntry[]> {
  return await allInInputOrder({
    members: entryIds.map(function judgeOne(entryId,): Promise<JudgedEntry> {
      return judge({ entryId, },);
    },),
  },);
}

/**
 Reads a run's published tree back and reports whether it agrees with its
 artifacts.

 The report on stdout and the returned exit code ARE the output.

 @param runsDir - run directory whose artifacts and published tree are read

 @param pipelineDir - directory of the built pipeline that does the reading,
 ordinarily the entry's own `import.meta.dirname`, whose digest names the
 build a disagreement over another build's artifact is read by

 @returns The exit code the command leaves: 0 for a run read to its end,
 whatever it found, and 2 for a run that left nothing to read

 @example
 ```ts
 process.exitCode = await verifyPublishedRun({ runsDir, pipelineDir: import.meta.dirname, },);
 ```
 */
export async function verifyPublishedRun(
  {
    runsDir,
    pipelineDir,
  }: {
    readonly runsDir: string;
    readonly pipelineDir: string;
  },
): Promise<number> {
  /**
   What the run left on disk, or why each half could not be read.

   Only the published half can still throw (`carriesPage` rethrows a failure
   that is no absence); the settled half answers every listing failure as
   data. Read by input order all the same, so a throw added to the settled
   half later cannot make the report depend on which listing ended first.
   */
  const [
    settled,
    published,
  ] = await allInInputOrder({
    members: [
      settledEntryIds({ runsDir, },),
      publishedEntryIds({ runsDir, },),
    ],
  },);

  /**
   Ids to check, or why this run leaves nothing to check.
   */
  const toVerify = whatThereIsToVerify({
    settled,
    published,
  },);

  if (toVerify.kind === 'nothing-verified') {
    console.log(
      `verify-published: NOTHING VERIFIED, ${toVerify.why}. No page was read and no artifact was `
        + 'compared, so this is not a clean run',
    );
    return NOTHING_WAS_VERIFIED;
  }

  // Reported only once there are settled entries to report it ABOUT: an absent
  // tree beside no artifacts at all says nothing, and phrasing it as "every
  // settled entry" when none was settled is a claim about an empty set that
  // reads as a finding.
  /**
   Entries the run settled, which is what an absent tree loses all of.
   */
  const settledCount = toVerify
    .settled
    .length;

  if (published.kind === 'unreadable')
    console.log(
      `verify-published: NO PUBLISHED TREE (${published.reason}). `
        + `${String(settledCount,)} settled ${
          wordForCount({
            count: settledCount,
            one: 'entry',
            many: 'entries',
          },)
        } ${
          wordForCount({
            count: settledCount,
            one: 'is',
            many: 'are',
          },)
        } unpublished, and the next pass `
        + 'started in this runs directory writes each from its artifact',
    );

  /**
   Which entries were settled and which were published.
   */
  const {
    matched,
    unpublished,
    unsettled,
  } = pairPublishedPages({
    settled: toVerify.settled,
    published: toVerify.published,
  },);

  /**
   Entries the pipeline declined to repair (the archive's note says the page
   is the author's own English), which carry no artifact and must carry no
   page: the archive stands.
   */
  const declined = await declinedEntryIds({
    declinedDir: join(
      runsDir,
      DECLINED_DIR,
    ),
  },);

  console.log(
    `verify-published: matched=${String(matched.length,)} `
      + `settledWithNoPage=${String(unpublished.length,)} `
      + `pageWithNoArtifact=${String(unsettled.length,)} `
      + `declined=${String(declined.size,)}`,
  );

  for (const id of unpublished) {
    console.log(
      `  SETTLED AND NEVER PUBLISHED: ${id}. The archive ships for it until the next pass started in `
        + 'this runs directory writes it from its artifact',
    );
  }
  for (const id of unsettled) {
    console.log(
      declined.has(id,)
        ? `  DECLINED AND PUBLISHED ANYWAY: ${id}. The archive's note says the page is the author's own `
          + 'English, so no page should stand here; the next pass started in this runs directory removes it'
        : `  PUBLISHED AND NOT SETTLED: ${id}. It ships as it stands, with no artifact to check it against, `
          + 'until a pass settles the entry again and overwrites it',
    );
  }

  /**
   This build, which reads every artifact this check compares; an artifact another build
   settled can read differently here (ledger A16b).
   */
  const { digest: thisBuild, } = await digestPipeline({ dir: pipelineDir, },);

  /**
   What every matched entry came to, in the order the entries were listed.

   EVERY ENTRY IS READ before the verdict, rather than stopping at the first
   disagreement, because the report is what an operator reads to know what
   ships and what the next pass rewrites, and a partial answer tells neither.
   The reads run together; what each printed waits for the loop that prints them, since
   a line printed as its read finished put the entries in whatever order the
   disk answered.
   */
  const judged = await judgePublishedEntries({
    entryIds: matched,
    judge: function judgeOne({ entryId, },): Promise<JudgedEntry> {
      return judgePublishedEntry({
        runsDir,
        entryId,
        thisBuild,
      },);
    },
  },);
  for (const { lines, } of judged) {
    for (const line of lines)
      console.log(line,);
  }

  /**
   Whether each matched entry's page carried what its artifact promised.
   */
  const agreements = judged.map(function agreementOf({ agreement, },): EntryAgreement {
    return agreement;
  },);

  /**
   Entries whose page disagreed with their artifact, or could not be read.
   */
  const disagreed = agreements
    .filter(function isBad(agreed,): boolean {
      return agreed === 'disagreed';
    },)
    .length;

  /**
   Entries whose page carried every wording but could not be weighed, because
   the artifact predates the stored archive text. Counted apart so the closing
   line claims a length check only for the pages that had one.
   */
  const unweighed = agreements
    .filter(function wasUnweighed(agreed,): boolean {
      return agreed === 'agreed-unweighed';
    },)
    .length;

  /**
   Pages that carried every wording, weighed or not.
   */
  const agreed = agreements.length - disagreed;

  /**
   Pages that carried every wording and weighed what the artifact implies.
   */
  const weighed = agreed - unweighed;

  console.log(
    `verify-published: ${String(agreed,)} of ${String(agreements.length,)} `
      + `${
        wordForCount({
          count: agreements.length,
          one: 'page',
          many: 'pages',
        },)
      } ${
        wordForCount({
          count: agreed,
          one: 'carries every wording its artifact promised',
          many: 'carry every wording their artifacts promised',
        },)
      }; ${String(weighed,)} of those at the length `
      + `it implies, ${String(unweighed,)} UNWEIGHED because the artifact predates the stored archive text`,
  );

  return READ_TO_THE_END;
}

//endregion Verify published run
