import { requireExactKeys, } from '../artifact-exact-guard.ts';
import {
  ArtifactParseError,
  requireArray,
  requireRecord,
  requireString,
  requireStringList,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import type {
  ArtifactConsolidateSlice,
  ArtifactConsolidationTerminal,
} from './artifact-two-lane-consolidate.ts';
import {
  type GateBallot,
  isGateChoice,
} from '@monochromatic-dev/translation-repair-consolidate/ts';

//region Artifact version 2 consolidation parts
// The leaf shapes one consolidated slice is built from, split off the subject
// at the file-length limit on the seam between WHAT A FIELD IS and WHAT A SLICE
// MEANS. `artifact-two-lane-read-consolidate.ts` holds the second.
//
// THE SHIPPED FIELD IS WHY THIS IS STRICT. Every other field here is evidence
// about a decision; `shipped` is the decision`s OUTPUT, and a consumer writes
// its text into the document. So the terminal and the shipped kind are checked
// against each other rather than read independently: exactly the `consolidated`
// terminal carries text, and a record disagreeing with itself about that would
// either ship a passage nobody settled on or silently drop one that was.

/**
 @internal

 Reads what one slice contributes to the document.

 @param value - shipped field as the slice carries it

 @param terminal - how that slice left the stage

 @param path - dotted path for error messages

 @returns Wording to write, or a stated absence

 @throws {@link ArtifactParseError} when the shipped kind and the terminal
 disagree about whether this slice replaces anything

 @example
 ```ts
 const shipped = parseShipped({ value: record.shipped, terminal, path, },);
 ```
 */
export function parseShipped(
  {
    value,
    terminal,
    path,
  }: {
    readonly value: unknown;
    readonly terminal: ArtifactConsolidationTerminal;
    readonly path: string;
  },
): ArtifactConsolidateSlice['shipped'] {
  /**
   Shipped field as a record.
   */
  const record = requireRecord({
    value,
    path,
  },);

  /**
   Whether the terminal says this slice replaces what stood.
   */
  const replaces = terminal === 'consolidated';
  if (record.kind === 'unchanged') {
    requireExactKeys({
      record,
      allowed: ['kind',],
      path,
    },);
    if (replaces) {
      throw new ArtifactParseError({
        path,
        reason: 'the text a slice whose terminal is consolidated ships',
      },);
    }
    return { kind: 'unchanged', };
  }
  // THE ARCHIVE KEPT CARRIES NO TEXT (owner, 2026-09-27, "Keep archive,
  // ship"): its wording is the comparison row's, so text here would be a
  // second copy free to disagree with it.
  if (record.kind === 'archive') {
    requireExactKeys({
      record,
      allowed: ['kind',],
      path,
    },);
    if (replaces) {
      throw new ArtifactParseError({
        path,
        reason: 'the consolidation a slice whose terminal is consolidated ships, rather than the archive',
      },);
    }
    return { kind: 'archive', };
  }
  if (record.kind === 'incumbent') {
    requireExactKeys({
      record,
      allowed: [
        'kind',
        'text',
      ],
      path,
    },);
    if (replaces) {
      throw new ArtifactParseError({
        path,
        reason: 'the consolidation a slice whose terminal is consolidated ships, rather than the incumbent',
      },);
    }
    return {
      kind: 'incumbent',
      text: requireString({
        value: record.text,
        path: `${path}.text`,
      },),
    };
  }
  if (record.kind !== 'consolidated') {
    throw new ArtifactParseError({
      path: `${path}.kind`,
      reason: 'one of archive, consolidated, incumbent, unchanged',
    },);
  }
  requireExactKeys({
    record,
    allowed: [
      'kind',
      'text',
    ],
    path,
  },);
  if (!replaces) {
    throw new ArtifactParseError({
      path,
      reason: `no text to ship, since a slice whose terminal is ${terminal} settled on no change`,
    },);
  }
  return {
    kind: 'consolidated',
    text: requireString({
      value: record.text,
      path: `${path}.text`,
    },),
  };
}

/**
 @internal

 Reads one voice`s structural verdict on its proposal.

 @param value - verdict as the slice carries it

 @param path - dotted path for error messages

 @returns Verdict this version names

 @throws {@link ArtifactParseError} when the verdict is the wrong shape

 @example
 ```ts
 const verdict = parseVerdict({ value: entry, path, },);
 ```
 */
export function parseVerdict(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): ArtifactConsolidateSlice['verdicts'][number] {
  /**
   Verdict as a record.
   */
  const record = requireRecord({
    value,
    path,
  },);
  requireExactKeys({
    record,
    allowed: [
      'modelId',
      'kind',
      'findings',
    ],
    path,
  },);
  if ((record.kind !== 'valid') && (record.kind !== 'invalid')) {
    throw new ArtifactParseError({
      path: `${path}.kind`,
      reason: 'one of valid, invalid',
    },);
  }
  return {
    modelId: requireString({
      value: record.modelId,
      path: `${path}.modelId`,
    },),
    kind: record.kind,
    findings: requireStringList({
      value: record.findings,
      path: `${path}.findings`,
    },),
  };
}


/**
 @internal

 Reads one judge`s gate ballot.

 THE EVIDENCE FIELDS ARE READ AS CHOICES, not as prose. The gate was once found
 shipping a rendering its own ballots named faultier because nothing counted
 them; a name outside the three would be counted as nothing and would weaken
 that evidence silently.

 @param value - ballot as the gate recorded it

 @param path - dotted path for error messages

 @returns Ballot this version names

 @throws {@link ArtifactParseError} when a field is missing or names a
 rendering that does not exist

 @example
 ```ts
 const ballot = parseGateBallot({ value: entry, path, },);
 ```
 */
export function parseGateBallot(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): GateBallot {
  /**
   Ballot as a record.
   */
  const record = requireRecord({
    value,
    path,
  },);
  requireExactKeys({
    record,
    allowed: [
      'choice',
      'unsupported',
      'unsupportedRaw',
      'dropped',
      'droppedRaw',
      'reason',
    ],
    path,
  },);
  return {
    choice: requireGateChoice({
      value: record.choice,
      path: `${path}.choice`,
    },),
    unsupported: requireGateChoices({
      value: record.unsupported,
      path: `${path}.unsupported`,
    },),
    unsupportedRaw: requireStringList({
      value: record.unsupportedRaw,
      path: `${path}.unsupportedRaw`,
    },),
    dropped: requireGateChoices({
      value: record.dropped,
      path: `${path}.dropped`,
    },),
    droppedRaw: requireStringList({
      value: record.droppedRaw,
      path: `${path}.droppedRaw`,
    },),
    reason: requireString({
      value: record.reason,
      path: `${path}.reason`,
    },),
  };
}

/**
 Reads a value that must name one of the renderings or the refusal.

 @param value - name as recorded

 @param path - dotted path for error messages

 @returns Name this version accepts

 @throws {@link ArtifactParseError} when it names nothing this gate offers

 @example
 ```ts
 const choice = requireGateChoice({ value: record.choice, path, },);
 ```
 */
function requireGateChoice(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): GateBallot['choice'] {
  if (!isGateChoice(value,)) {
    throw new ArtifactParseError({
      path,
      reason: 'one of consolidated, standing, neither',
    },);
  }
  return value;
}

/**
 Reads a list of rendering names.

 @param value - list as recorded

 @param path - dotted path for error messages

 @returns Names this version accepts, in the order recorded

 @throws {@link ArtifactParseError} when an entry names nothing this gate
 offers

 @example
 ```ts
 const named = requireGateChoices({ value: record.dropped, path, },);
 ```
 */
function requireGateChoices(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): readonly GateBallot['choice'][] {
  return requireArray({
    value,
    path,
  },)
    .map(function readOne(
      entry,
      at,
    ): GateBallot['choice'] {
      return requireGateChoice({
        value: entry,
        path: `${path}[${String(at,)}]`,
      },);
    },);
}

//endregion Artifact version 2 consolidation parts
