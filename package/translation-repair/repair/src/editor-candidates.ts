import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import {
  applyPatchOperations,
  type PatchOutcome,
  type PreservationMode,
} from '@monochromatic-dev/translation-repair-page-pass/ts';
import {
  type Candidate,
  mergeProducers,
} from '@monochromatic-dev/translation-repair-house-rule/ts';
import {
  type EditorReportWire,
  resolveEditorEdits,
} from './edit-wire.ts';
import type { EditorCandidate, } from './editor-selection-result.ts';
import {
  type EditableEnvelope,
  isUnreadableReason,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { HeardVoice, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Editor candidate assembly
// Turning heard editor voices into comparable candidates, and turning those
// candidates into the set the chunk-level judges see. Kept apart from the stage
// itself so the stage reads as the sequence of decisions rather than as the
// bookkeeping under them.
//
// Everything here is ordered by ROSTER position rather than by arrival. Voices
// come back in whatever order the provider answered, so ordering by arrival
// would make the anonymized candidate numbering, the duplicate-collapse winner,
// and the fallback choice all vary between runs over identical inputs.

/**
 Editor candidates with the wire irregularities found while building them.

 @example
 ```ts
 const { candidates, findings, } = buildEditorCandidates({ voices, ... },);
 ```
 */
export type EditorCandidateSet = {
  /**
   One candidate per heard editor, in roster order.
   */
  readonly candidates: readonly EditorCandidate[];

  /**
   Wire irregularities across every editor, in scorecard-stable wording.
   */
  readonly findings: readonly string[];
};

/**
 One editor's candidate paired with the findings its own reply raised.

 Named rather than inferred, because an inferred object literal carries
 writable properties and the two unwrapping maps that read this list then take
 mutable parameters they never mutate.
 */
type ResolvedVoice = Readonly<{
  /**
   Patched candidate this voice proposed.
   */
  candidate: EditorCandidate;

  /**
   Wire irregularities attributed to this voice.
   */
  findings: readonly string[];
}>;

/**
 Resolves every heard editor's reply into its own patch through the same
 deterministic gate, so the candidates are directly comparable.

 @param voices - heard editor replies in arrival order

 @param editorModelIds - roster, fixing candidate order

 @param promptEnvelopes - envelopes in prompt numbering order

 @param targetText - translation chunk text

 @param envelopes - envelopes of this chunk

 @returns Candidates in roster order plus findings

 @example
 ```ts
 const set = buildEditorCandidates({ voices, editorModelIds, ... },);
 ```
 */
export function buildEditorCandidates(
  {
    voices,
    editorModelIds,
    promptEnvelopes,
    targetText,
    envelopes,
    preservation,
  }: {
    readonly voices: readonly HeardVoice<EditorReportWire>[];
    readonly editorModelIds: readonly RosterModelId[];
    readonly promptEnvelopes: readonly EditableEnvelope[];
    readonly targetText: string;
    readonly envelopes: readonly EditableEnvelope[];
    readonly preservation: PreservationMode;
  },
): EditorCandidateSet {
  /**
   Voices sorted by roster position so downstream order never depends on
   which model answered first.
   */
  const ordered = [...voices,].toSorted(function byRoster(
    left,
    right,
  ) {
    return editorModelIds.indexOf(left.modelId,)
      - editorModelIds.indexOf(right.modelId,);
  },);

  /**
   One resolved candidate per voice, each carrying its own findings.
   */
  const resolved = ordered.map(function toCandidate(voice,): ResolvedVoice {
    /**
     Operations bound through the prompt plan.
     */
    const resolution = resolveEditorEdits({
      wire: voice.value,
      envelopes: promptEnvelopes,
    },);
    /**
     This voice's operations through the apply gate.
     */
    const patch = applyPatchOperations({
      targetText,
      envelopes,
      operations: resolution.operations,
      preservation,
    },);
    return {
      candidate: {
        modelId: voice.modelId,
        patch,
      },
      findings: [
        ...resolution.findings,
        // An edit whose text no grammar reads is refused as this voice's and
        // said so here, since a voice left with no applied edit carries its
        // refusals nowhere else.
        ...patch.rejected
          .filter(function unreadable(rejection,): boolean {
            return isUnreadableReason({ reason: rejection.reason, },);
          },)
          .map(function toReason(rejection,): string {
            return rejection.reason;
          },),
      ]
        .map(function attribute(finding,) {
          return `${voice.modelId}: ${finding}`;
        },),
    };
  },);

  return {
    candidates: resolved.map(function toCandidateOnly(entry,) {
      return entry.candidate;
    },),
    findings: resolved.flatMap(function toFindings(entry,) {
      return [...entry.findings,];
    },),
  };
}

/**
 Chunk-level candidate set after duplicates were collapsed.

 @example
 ```ts
 const { candidates, collapsed, } = buildChunkCandidates({ candidates, composite, },);
 ```
 */
export type ChunkCandidateSet = {
  /**
   Distinct proposals judges will compare, in roster order with the
   composite last.
   */
  readonly candidates: readonly Candidate<PatchOutcome>[];

  /**
   Proposals collapsed into an earlier identical one; several models writing
   the same text is real agreement worth recording, but showing judges the
   same text twice would only split the ballot into a spurious tie.
   */
  readonly collapsed: number;
};

/**
 Presents one editor's patch as a judgeable candidate.

 SHARED BY THE SLATE AND THE FALLBACK, deliberately. The whole-chunk judges
 see candidates built here, and the patch that ships when they decline is
 built here too, so the producer recorded against shipped text is the same
 shape either way. Building the fallback's producer separately is how the two
 drift, and the discount on a checker judging its own work reads that
 producer.

 @param candidate - one editor's proposal, already through the apply gate

 @returns Candidate naming that editor as its sole producer

 @example
 ```ts
 const offered = chunkCandidateOf(candidate,);
 ```
 */
export function chunkCandidateOf(candidate: EditorCandidate,): Candidate<PatchOutcome> {
  return {
    producer: {
      kind: 'model',
      modelId: candidate.modelId,
    },
    value: candidate.patch,
    rendered: candidate.patch
      .patchedText,
  };
}

/**
 Assembles the whole-chunk candidate set, dropping the composite when it
 repairs nothing and collapsing candidates whose text is identical.

 @param candidates - editor candidates in roster order

 @param composite - patch assembled from per-envelope winners

 @param contributors - models whose operations the composite carries

 @returns Distinct candidates plus how many collapsed

 @example
 ```ts
 const set = buildChunkCandidates({ candidates, composite, contributors, },);
 ```
 */
export function buildChunkCandidates(
  {
    candidates,
    composite,
    contributors,
  }: {
    readonly candidates: readonly EditorCandidate[];
    readonly composite: PatchOutcome;
    readonly contributors: readonly RosterModelId[];
  },
): ChunkCandidateSet {
  /**
   Every proposal worth judging: each editor's own patch, then the composite
   when per-envelope selection actually assembled a repair. An empty
   composite is the untouched translation, which competes later against the
   repaired candidate and must not displace it here.
   */
  const offered: readonly Candidate<PatchOutcome>[] = [
    ...candidates.map(function toChunkCandidate(candidate,): Candidate<PatchOutcome> {
      return chunkCandidateOf(candidate,);
    },),
    ...(composite.applied
      .length
      === 0
      ? []
      : [
        {
          producer: {
            kind: 'composite',
            contributors,
          },
          value: composite,
          rendered: composite.patchedText,
        } satisfies Candidate<PatchOutcome>,
      ]),
  ];

  /**
   Kept candidates by their rendered text, merging the stakes of every
   duplicate into the survivor.
   */
  const byText = new Map<string, Candidate<PatchOutcome>>();
  for (const candidate of offered) {
    /**
     Earlier candidate with identical text, when one exists.
     */
    const kept = byText.get(candidate.rendered,);
    if (kept === undefined) {
      byText.set(
        candidate.rendered,
        candidate,
      );
      continue;
    }
    byText.set(
      candidate.rendered,
      {
        ...kept,
        producer: mergeProducers({
          left: kept.producer,
          right: candidate.producer,
        },),
      },
    );
  }

  return {
    candidates: [...byText.values(),],
    collapsed: offered.length - byText.size,
  };
}

/**
 Picks the editor whose patch ships when chunk-level judges decline.

 The fallback must repair something. Falling back to the untouched
 translation would discard fixes the panel already ruled real, turning a
 disagreement about wording into a lost repair, so this picks the editor that
 landed the most operations and breaks ties by roster order.

 @param candidates - editor candidates in roster order, none empty

 @returns Candidate with the most applied operations, named so shipped text has an author

 @throws {@link Error} when handed an empty candidate list

 @example
 ```ts
 const fallback = pickFallbackCandidate({ candidates, },);
 ```
 */
export function pickFallbackCandidate(
  {
    candidates,
  }: {
    readonly candidates: readonly EditorCandidate[];
  },
): EditorCandidate {
  /**
   Roster-first candidate and the rest, so the fold starts from a real
   incumbent instead of an absent one.
   */
  const [
    first,
    ...rest
  ] = candidates;

  /**
   Editor that landed the most operations, earliest in roster order winning
   ties because `>` leaves an equal count with the incumbent.
   */
  const best = rest.reduce(
    function moreApplied(
      leader: EditorCandidate,
      candidate,
    ): EditorCandidate {
      return candidate.patch
        .applied
        .length
        > leader.patch
          .applied
          .length
        ? candidate
        : leader;
    },
    nonNullishOrThrow(first,),
  );
  return best;
}

//endregion Editor candidate assembly
