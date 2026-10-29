import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import { reviewAbsoluteNaturalness, } from './absolute-naturalness-review-stage.ts';
import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { StageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Archive block naturalness

/**
 Converts located naturalness findings to continuation evidence.

 @param findings - reviewer findings

 @returns Prompt-safe evidence without model identity
 */
function describeNaturalnessFindings(
  { findings, }: { readonly findings: readonly {
    readonly paragraph: number;
    readonly problem: string
  }[]; },
): readonly string[] {
  return findings.map(function describe(finding,): string {
    return `archive naturalness paragraph ${String(finding.paragraph,)}: ${finding.problem}`;
  },);
}

/**
 Records defect discovery and distinct acceptance challenge for a retained block.

 Verdicts are evidence, never withholding authority:
 a rejection or an unheard review roster becomes located findings on the
 settlement while the block ships,
 because reviewer opinion after a completed review round must not pause an
 entry that a producing stage already settled.

 @param client - provider client

 @param modelIds - independent review roster

 @param sourceText - aligned source context

 @param blockText - exact retained English wording

 @param identityContext - declared names the block's reviewers read, so a
 name in its declared form is not read as awkward English (ledger B28)

 @param signal - caller cancellation

 @param exchangeTimeoutMs - per-call bound

 @param l - stage logger

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Findings from both responsibilities, located evidence on rejection

 @example
 ```ts
 const findings = await recordArchiveBlockNaturalness(input);
 ```
 */
export async function recordArchiveBlockNaturalness(
  {
    client,
    modelIds,
    sourceText,
    blockText,
    identityContext,
    signal,
    exchangeTimeoutMs,
    l,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly modelIds: readonly RosterModelId[];
    readonly sourceText: string;
    readonly blockText: string;
    readonly identityContext?: string;
    readonly signal: AbortSignal;
    readonly exchangeTimeoutMs: number;
    readonly l: Logger;
    readonly stageSettings: StageSettings;
  }>,
): Promise<readonly string[]> {
  /**
   Exact candidate and aligned context shared by distinct reviews.
   */
  const subject = {
    sourceText,
    candidateText: blockText,
    paragraphs: [blockText,],
    // An unclaimed archive block is page prose no line rule governs, so the
    // reviewer reads it as the page renders it (ledger H6).
    lineStructured: false,
    ...((identityContext === undefined) ? {} : { identityContext, }),
  };
  /**
   First responsibility searches for defects.
   */
  const discovery = await reviewAbsoluteNaturalness({
    client,
    modelIds,
    subject,
    perspective: 'defect-discovery',
    signal,
    exchangeTimeoutMs,
    l,
    stageSettings,
  },);
  /**
   First-round located evidence.
   */
  const discoveryFindings = describeNaturalnessFindings({ findings: discovery.findings, });
  /**
   Distinct responsibility challenges prior acceptance.
   */
  const challenge = await reviewAbsoluteNaturalness({
    client,
    modelIds,
    subject,
    perspective: 'acceptance-challenge',
    signal,
    exchangeTimeoutMs,
    l,
    stageSettings,
  },);
  /**
   Second-round located evidence.
   */
  const challengeFindings = describeNaturalnessFindings({ findings: challenge.findings, });
  if ((discovery.verdict === 'acceptable') && (challenge.verdict === 'acceptable'))
    return ['archive block absolute naturalness accepted and challenged',];
  return [
    ...discoveryFindings,
    ...challengeFindings,
    ...(discovery.verdict === 'quorum-not-met'
      ? ['archive naturalness defect-discovery review quorum not met',]
      : []),
    ...(challenge.verdict === 'quorum-not-met'
      ? ['archive naturalness acceptance-challenge review quorum not met',]
      : []),
    'archive block retained with naturalness findings recorded',
  ];
}

//endregion Archive block naturalness
