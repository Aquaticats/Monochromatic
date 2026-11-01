import {
  requireArray,
  requireRecord,
  requireString,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import { requireKeyOf, } from '../artifact-exact-guard.ts';
import type { CandidateProducer, } from '@monochromatic-dev/translation-repair-house-rule/ts';
import {
  ROSTER_MODEL_IDS,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { UnhandledMemberInvariantError, } from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Artifact producer read
// WHO WROTE A RECORDED CANDIDATE, read back out of an artifact and checked
// against the roster as it stands today.
//
// IT REFUSES A MODEL THAT HAS LEFT THE ROSTER, and that refusal is the point
// rather than a limitation to work around. `RosterModelId` is a closed union of
// the models seated now; `hf:zai-org/GLM-4.7-Flash` was in it until 2026-08-24
// and is not any more. Reading a departed id as though it were current would
// let a standing mix rosters silently, and widening the production union so a
// reporting tool can hold one would loosen the type every seating decision is
// checked against. So a record from an older roster is NAMED, not read.

/**
 Provenance kinds a recorded candidate can name.

 KEYED BY THE KIND OF `CandidateProducer` rather than listed, so the compiler
 refuses a kind the live union gains that this reader lacks: every artifact
 carrying it would otherwise be refused whole. The order is the order a
 refusal names them in.
 */
const PRODUCER_KINDS: Readonly<Record<CandidateProducer['kind'], true>> = {
  model: true,
  composite: true,
  incumbent: true,
  lane: true,
};

/**
 Lanes a lane candidate can name (class forty, 2026-09-17), keyed by the lane
 of `CandidateProducer` for the reason `PRODUCER_KINDS` gives.
 */
const LANE_NAMES: Readonly<Record<Extract<CandidateProducer, { readonly kind: 'lane'; }>['lane'], true>> = {
  repair: true,
  translate: true,
};

/**
 Signals a recorded model that no longer holds a place in the roster.

 ITS OWN CLASS, so a caller can tell "this artifact predates the current
 roster" from "this artifact is malformed". The first is expected of anything
 settled before a seating change and says nothing bad about the record; the
 second is a defect.

 @example
 ```ts
 throw new OffRosterModelError({ modelId: 'hf:zai-org/GLM-4.7-Flash', path, },);
 ```
 */
export class OffRosterModelError extends Error {
  /**
   Declares this message safe to forward: it names a model and the artifact path that recorded it.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds failure naming the id and where it was read.

   @param modelId - id the record carried

   @param path - dotted path it was read at

   @example
   ```ts
   new OffRosterModelError({ modelId, path: 'chunks[0].rounds[1]', },);
   ```
   */
  public constructor(
    {
      modelId,
      path,
    }: {
      readonly modelId: string;
      readonly path: string;
    },
  ) {
    super(
      `${path} names ${modelId}, which no longer holds a place in the roster, so this record `
        + 'was written under an earlier seating and cannot be read as current',
    );
    this.name = 'OffRosterModelError';
  }
}

/**
 Reads one model id, refusing one the roster no longer seats.

 @param value - id as recorded

 @param path - dotted path for error messages

 @returns Id, narrowed to the roster

 @throws {@link OffRosterModelError} when the roster no longer seats it

 @throws {@link ArtifactParseError} when it is not a string at all

 @example
 ```ts
 const modelId = requireRosterModelId({ value, path, },);
 ```

 @internal
 */
export function requireRosterModelId(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): RosterModelId {
  /**
   Id as written, before it is checked against the roster.
   */
  const written = requireString({
    value,
    path,
  },);

  for (const seated of ROSTER_MODEL_IDS) {
    if (seated === written)
      return seated;
  }

  throw new OffRosterModelError({
    modelId: written,
    path,
  },);
}

/**
 Reads every model id in one list.

 @param value - list as recorded

 @param path - dotted path for error messages

 @returns Ids, each narrowed to the roster

 @example
 ```ts
 const contributors = requireRosterModelIds({ value, path, },);
 ```
 */
function requireRosterModelIds(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): readonly RosterModelId[] {
  return requireArray({
    value,
    path,
  },)
    .map(function one(
      entry,
      index,
    ): RosterModelId {
      return requireRosterModelId({
        value: entry,
        path: `${path}[${String(index,)}]`,
      },);
    },);
}

/**
 Reads one candidate's provenance.

 @param value - producer as recorded

 @param path - dotted path for error messages

 @returns Provenance in its three-way shape

 @throws {@link ArtifactParseError} when the kind or lane is not one `CandidateProducer` names, through `requireKeyOf`, or a field is malformed

 @throws {@link UnhandledMemberInvariantError} when a member of `CandidateProducer` has no branch here, which the compiler rules out

 @example
 ```ts
 const producer = requireProducer({ value, path, },);
 ```

 @internal
 */
export function requireProducer(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): CandidateProducer {
  /**
   Producer as a record.
   */
  const record = requireRecord({
    value,
    path,
  },);

  /**
   Which of the three shapes it claims.
   */
  const kind = requireKeyOf({
    value: record.kind,
    path: `${path}.kind`,
    record: PRODUCER_KINDS,
  },);

  if (kind === 'model')
    return {
      kind,
      modelId: requireRosterModelId({
        value: record.modelId,
        path: `${path}.modelId`,
      },),
    };

  if (kind === 'composite')
    return {
      kind,
      contributors: requireRosterModelIds({
        value: record.contributors,
        path: `${path}.contributors`,
      },),
    };

  if (kind === 'lane')
    return {
      kind,
      lane: requireKeyOf({
        value: record.lane,
        path: `${path}.lane`,
        record: LANE_NAMES,
      },),
      matched: requireRosterModelIds({
        value: record.matched,
        path: `${path}.matched`,
      },),
    };

  if (kind === 'incumbent') {
    return {
      kind,
      matched: requireRosterModelIds({
        value: record.matched,
        path: `${path}.matched`,
      },),
    };
  }
  throw new UnhandledMemberInvariantError({
    what: 'recorded candidate producer',
    member: kind,
  },);
}

//endregion Artifact producer read
