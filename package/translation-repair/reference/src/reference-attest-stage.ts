import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import { withoutTranslatorsNotes, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  buildReferenceAttestConfirmMessages,
  confirmedDetails,
  isReferenceAttestConfirmWire,
  REFERENCE_ATTEST_CONFIRM_RESPONSE_FORMAT,
  type ReferenceAttestConfirmWire,
} from './reference-attest-confirm-wire.ts';
import {
  type AttestedDetail,
  attestedDetailLines,
  mergedAttestations,
  type VerifiedAttestation,
} from './reference-attest-match.ts';
import {
  type AttestationVerdict,
  attestationVerdictLine,
  attestationVerdicts,
  keptAttestations,
} from './reference-attest-verdict.ts';
import {
  buildReferenceAttestMessages,
  isReferenceAttestWire,
  REFERENCE_ATTEST_RESPONSE_FORMAT,
  type ReferenceAttestWire,
} from './reference-attest-wire.ts';
import {
  type FanOutMode,
  gatherStageVoices,
  MIN_STAGE_VOICES,
  rosterQuorumSize,
  type StageSettings,
} from '@monochromatic-dev/translation-repair-stage/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Reference attestation stage
// Asks the bench once per entry which archive details a cited reference
// states, checks every quote word for word, and keeps the details enough
// voices gave (class thirty-seven, 2026-09-16). What comes out is both the
// ATTESTED lines every sheet carries under the references and the list the
// repair lane screens addition claims against before the panel.

/**
 What the attestation produced.

 @example
 ```ts
 const { details, lines, findings, } = await attestCitedReferences({ ... },);
 ```
 */
export type ReferenceAttestation = {
  /**
   Details attested by enough voices, in archive order.
   */
  readonly details: readonly AttestedDetail[];
  /**
   Lines the sheets carry for them, one each.
   */
  readonly lines: readonly string[];
  /**
   Quorum and verification findings in scorecard-stable wording.
   */
  readonly findings: readonly string[];
};

/**
 Distinct voices a detail needs: half the voices heard, rounded up, and never
 fewer than the two every stage keeps (ledger B29). Sized on the heard bench
 alone, one voice heard needed one, so a single model attested or confirmed
 a detail the repair lane then screens addition claims against.

 @param heard - voices the round heard

 @returns Voices that must give or confirm a detail

 @example
 ```ts
 attestationVotesNeeded({ heard: 1, },); // 2
 attestationVotesNeeded({ heard: 5, },); // 3
 ```
 */
function attestationVotesNeeded({ heard, }: { readonly heard: number; },): number {
  return Math.max(
    MIN_STAGE_VOICES,
    rosterQuorumSize({ rosterSize: heard, },),
  );
}

/**
 Puts every verified detail to the bench as a yes-or-no candidate and keeps
 those enough voices confirm.

 EXTRACTION PROPOSES, CONFIRMATION DISPOSES. A voice that answered the open
 question with an empty list never said the detail is false; it said it
 found none, which on Mio26 four of five voices did in a handful of tokens.
 Shown the quote pair, the same voice reads it. When nothing was extracted
 there is nothing to ask, and when nobody answers the confirmation the
 extraction's own quorum stands, so a silent hour cannot lose a detail the
 open question already carried.

 @param client - provider client

 @param modelIds - bench asked, the extraction's

 @param sourceText - original document

 @param archiveText - archive rendering as inherited

 @param referenceContext - reference lines, one per page

 @param candidates - verified details from any voice, numbered in this order

 @param extracted - details the extraction quorum alone kept, the fallback

 @param signal - entry's abort

 @param exchangeTimeoutMs - per-call timeout

 @param l - stage logger

 @param fanOut - seats a round asks: the window of quorum plus one by
 default, or the whole bench a fixture scripting every seat asks for

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Details to attest and the findings of the round

 @example
 ```ts
 const { details, confirmFindings, } = await confirmCandidates({ client, modelIds, sourceText, archiveText, referenceContext, candidates, extracted, signal, exchangeTimeoutMs, l, stageSettings, },);
 ```

 @internal
 */
async function confirmCandidates(
  {
    client,
    modelIds,
    sourceText,
    archiveText,
    referenceContext,
    candidates,
    extracted,
    signal,
    exchangeTimeoutMs,
    l,
    fanOut,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly modelIds: readonly RosterModelId[];
    readonly sourceText: string;
    readonly archiveText: string;
    readonly referenceContext: string;
    readonly candidates: readonly AttestedDetail[];
    readonly extracted: readonly AttestedDetail[];
    readonly signal: AbortSignal;
    readonly exchangeTimeoutMs: number;
    readonly l: Logger;
    readonly fanOut?: FanOutMode;
    readonly stageSettings: StageSettings;
  }>,
): Promise<{
  readonly details: readonly AttestedDetail[];
  readonly confirmFindings: readonly string[];
}> {
  if (candidates.length === 0) {
    return {
      details: extracted,
      confirmFindings: [],
    };
  }
  /**
   Quorum-bounded confirmation replies.
   */
  const gather = await gatherStageVoices<ReferenceAttestConfirmWire>({
    client,
    modelIds,
    messages: buildReferenceAttestConfirmMessages({
      sourceText,
      archiveText,
      referenceContext,
      candidates,
    },),
    signal,
    exchangeTimeoutMs,
    responseFormat: REFERENCE_ATTEST_CONFIRM_RESPONSE_FORMAT,
    validate: isReferenceAttestConfirmWire,
    stage: 'reference-attest-confirm',
    l,
    ...((fanOut === undefined) ? {} : { fanOut, }),
    graceMs: stageSettings
      .grace
      .roundMs,
  },);
  /**
   Voices heard on the confirmation.
   */
  const heard = gather.voices
    .length;
  if (heard === 0) {
    l.warn(
      `ATTESTED CONFIRM heard=0 candidates=${String(candidates.length,)}: nobody answered, the extraction quorum stands`,
    );
    return {
      details: extracted,
      confirmFindings: [
        ...gather.findings,
        'reference attestation confirmation heard nobody; the extraction quorum stands',
      ],
    };
  }
  // A LONE VOICE DECIDES NOTHING, either way (ledger B29): it can neither
  // confirm a candidate nor discard what the extraction quorum kept, so a
  // confirmation heard by fewer voices than a decision needs is treated as
  // one nobody answered.
  if (heard < MIN_STAGE_VOICES) {
    l.warn(
      `ATTESTED CONFIRM heard=${String(heard,)} candidates=${String(candidates.length,)}: fewer voices than the ${
        String(MIN_STAGE_VOICES,)
      } a decision needs, the extraction quorum stands`,
    );
    return {
      details: extracted,
      confirmFindings: [
        ...gather.findings,
        `reference attestation confirmation heard ${String(heard,)} voice, fewer than the ${
          String(MIN_STAGE_VOICES,)
        } a decision needs; the extraction quorum stands`,
      ],
    };
  }
  /**
   Distinct confirming voices a candidate needs.
   */
  const needed = attestationVotesNeeded({ heard, },);
  /**
   Candidates confirmed by enough voices.
   */
  const details = confirmedDetails({
    candidates,
    ballots: gather.voices
      .map(function toBallot(voice,) {
        return {
          modelId: voice.modelId,
          confirmed: voice.value
            .confirmed,
        };
      },),
    needed,
  },);
  l.info(
    `ATTESTED CONFIRM heard=${String(heard,)} candidates=${String(candidates.length,)} needed=${
      String(needed,)
    } confirmed=${String(details.length,)}`,
  );
  return {
    details,
    confirmFindings: [
      ...gather.findings,
      `reference attestation confirmed ${String(details.length,)} of ${String(candidates.length,)} ${
        wordForCount({
          count: candidates.length,
          one: 'candidate',
          many: 'candidates',
        },)
      } by ${
        String(heard,)
      } ${
        wordForCount({
          count: heard,
          one: 'voice',
          many: 'voices',
        },)
      }`,
    ],
  };
}

/**
 Asks the bench which archive details a cited reference states and keeps the
 ones enough voices attested with quotes that verify.

 @param client - provider client

 @param modelIds - bench asked

 @param sourceText - original document

 @param archiveText - archive rendering as inherited

 @param referenceContext - reference lines, one per page

 @param signal - entry's abort

 @param exchangeTimeoutMs - per-call timeout

 @param l - entry logger

 @param fanOut - seats each round asks: the window of quorum plus one by
 default, or the whole bench a fixture scripting every seat asks for, since
 which seats the window picks follows a hash of the prompt (ledger X2)

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Attested details, their sheet lines and findings

 @example
 ```ts
 const attestation = await attestCitedReferences({ client, modelIds, sourceText, archiveText, referenceContext, signal, exchangeTimeoutMs, l, stageSettings, },);
 ```
 */
export async function attestCitedReferences(
  {
    client,
    modelIds,
    sourceText,
    archiveText,
    referenceContext,
    signal,
    exchangeTimeoutMs,
    l,
    fanOut,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly modelIds: readonly RosterModelId[];
    readonly sourceText: string;
    readonly archiveText: string;
    readonly referenceContext: string;
    readonly signal: AbortSignal;
    readonly exchangeTimeoutMs: number;
    readonly l: Logger;
    readonly fanOut?: FanOutMode;
    readonly stageSettings: StageSettings;
  }>,
): Promise<ReferenceAttestation> {
  /**
   Logger pre-tagged with this function's name.
   */
  const al = tagged({
    tag: attestCitedReferences.name,
    l,
  },);
  /**
   Quorum-bounded replies.
   */
  const gather = await gatherStageVoices<ReferenceAttestWire>({
    client,
    modelIds,
    messages: buildReferenceAttestMessages({
      sourceText,
      archiveText,
      referenceContext,
    },),
    signal,
    exchangeTimeoutMs,
    responseFormat: REFERENCE_ATTEST_RESPONSE_FORMAT,
    validate: isReferenceAttestWire,
    stage: 'reference-attest',
    l: al,
    ...((fanOut === undefined) ? {} : { fanOut, }),
    graceMs: stageSettings
      .grace
      .roundMs,
  },);
  /**
   Voices heard.
   */
  const heard = gather.voices
    .length;
  // THE QUOTES ARE CHECKED IN THE ARCHIVE AS THE SHEET SHOWED IT, without the notes between translators, cut
  // against the same original (`buildReferenceAttestMessages`): a quote copied across the place a note was cut
  // from is in the text the voice read and in no stored text. Positions placed in it are used only to group
  // overlapping quotes of this one text, so no stored offset is taken from it.
  /**
   The archive as the voices read it.
   */
  const archiveShown = withoutTranslatorsNotes({
    text: archiveText,
    original: sourceText,
  },);
  /**
   What the check made of every item, from every voice (ledger B25).
   */
  const verdicts: readonly AttestationVerdict[] = gather.voices
    .flatMap(function verdictsOf(voice,): readonly AttestationVerdict[] {
      return attestationVerdicts({
        modelId: voice.modelId,
        items: voice.value
          .attested,
        archiveText: archiveShown,
        referenceContext,
      },);
    },);
  for (const verdict of verdicts)
    al.info(attestationVerdictLine({ verdict, },),);
  /**
   Items whose quotes verify, each under the reference that states it.
   */
  const verified: readonly VerifiedAttestation[] = keptAttestations({ verdicts, },);
  /**
   Items answered in all, verified or not.
   */
  const answered = gather.voices
    .map(function countOf(voice,): number {
      return voice.value
        .attested
        .length;
    },)
    .reduce(
    function sum(
      total,
      count,
    ): number {
      return total + count;
    },
    0,
    );
  /**
   Distinct voices a detail needs.
   */
  const needed = attestationVotesNeeded({ heard, },);
  /**
   Details enough voices gave to the open question alone.
   */
  const extracted = mergedAttestations({
    verified,
    archiveText: archiveShown,
    heard,
    needed,
  },);
  /**
   Every verified detail, one voice or more, as a candidate for the
   confirmation round (class forty-three).
   */
  const candidates = mergedAttestations({
    verified,
    archiveText: archiveShown,
    heard,
    needed: 1,
  },);
  /**
   Details after the confirmation round, or the extraction's own when there
   is nothing to confirm or nobody answers.
   */
  const {
    details,
    confirmFindings,
  } = await confirmCandidates({
    client,
    modelIds,
    sourceText,
    archiveText,
    referenceContext,
    candidates,
    extracted,
    signal,
    exchangeTimeoutMs,
    l: al,
    ...((fanOut === undefined) ? {} : { fanOut, }),
    stageSettings,
  },);
  /**
   Lines the sheets carry.
   */
  const lines = attestedDetailLines({ details, },);
  al.info(
    `ATTESTED heard=${String(heard,)} answered=${String(answered,)} verified=${String(verified.length,)} needed=${
      String(needed,)
    } details=${String(details.length,)}`,
  );
  for (const line of lines)
    al.info(line,);
  return {
    details,
    lines,
    findings: [
      ...gather.findings,
      ...confirmFindings,
      ...((answered === verified.length)
        ? []
        : [`reference attestation discarded ${String(answered - verified.length,)} of ${
          String(answered,)
        } ${
          wordForCount({
            count: answered,
            one: 'item',
            many: 'items',
          },)
        } whose quotes were not found word for word`,]),
    ],
  };
}

//endregion Reference attestation stage
