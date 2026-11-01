//region Artifact version 2 vocabulary
// The small unions a version 2 reader DISPATCHES on, frozen under names this
// version owns.
//
// Every one of these has a live twin in the pipeline, and the twins are the
// right thing for the pipeline to keep changing: this session alone added a
// fifth lane outcome and split one delivery word into two axes. Importing the
// live union into the persisted shape would make each of those changes expand
// what version 2 MEANS, silently, for artifacts already on disk. A reader
// written against version 2 would then meet a member its dispatch has never
// heard of while the version number still says 2.
//
// So these are copies, and the duplication is the point rather than a cost to
// be removed later. The rule they encode: the live unions may grow whenever the
// pipeline needs them to, and the day one of them grows in a way this file does
// not describe, the artifact becomes version 3.
//
// THE COMPILER ENFORCES THAT, which is why the copies are worth having rather
// than merely well intentioned. It takes two mechanisms, and knowing which does
// what matters, because assuming one covers both is how a schema drifts:
//
//  -   MEMBER GROWTH is caught by `artifact-two-lane-project.ts`, which ends each
//      projection on a `never` binding. A live union that gains a member leaves
//      that member unhandled, the binding stops being `never`, and the file
//      stops compiling. Plain assignment catches this too; the projection makes
//      it deliberate rather than incidental.
//  -   FIELD GROWTH is NOT caught, by either mechanism, and saying otherwise is
//      how a schema drifts. Excess property checking applies to object
//      LITERALS, so a live record that gains a field assigns into these types
//      cleanly; the projection does not fail on it either, it simply leaves the
//      new field out. What the projection buys is that the new field cannot
//      reach the written bytes, so version 2 artifacts keep meaning what this
//      file says and the version 2 parser keeps accepting them.
//
// So a live union that grows meets the version question as a build error, and a
// live record that grows keeps writing valid version 2 until somebody decides
// the new field belongs on disk. An exact-shape type test would turn the second
// into a build error too; it is not here, and that is the gap to close if a
// silently omitted field ever turns out to have mattered.
//
// DELIBERATELY NOT the whole artifact. The two raw lane results are recorded as
// EVIDENCE and typed by their live shapes, because they are large, they grow
// additively, and nothing dispatches on their shape: a reader takes what it
// recognizes and ignores the rest. What is frozen here is exactly the part a
// reader has to understand completely to read a row at all.

/**
 What a lane did about one slice, as version 2 records it.

 @example
 ```ts
 const outcome: ArtifactSliceOutcome = { kind: 'decided', acceptedText: 'The cat naps.', };
 ```
 */
export type ArtifactSliceOutcome = {
  /**
   Lane produced a wording, whether or not the document carries it.
   */
  readonly kind: 'decided';

  /**
   Wording it decided on.
   */
  readonly acceptedText: string;
} | {
  /**
   Lane never reached this slice.
   */
  readonly kind: 'not-evaluated';
} | {
  /**
   Lane reached it, produced nothing, and the archive had nothing either.
   */
  readonly kind: 'unfilled';

  /**
   Present when the lane never bought a candidate because the bench that decides the slice read below its floor, absent
   when a panel was asked and declined. Part of artifact generation twenty-one, so a reader of an older record refuses
   the key.
   */
  readonly benchBelowFloor?: true;
} | {
  /**
   Lane reached it, produced nothing, and the archive's wording therefore
   stands by default rather than by anyone's choice.
   */
  readonly kind: 'incumbent-fallback';
} | {
  /**
   Lane reached it and the work it does has no input there at all.
   */
  readonly kind: 'not-applicable';
};

/**
 What one lane's document carries at one slice, as version 2 records it.

 @example
 ```ts
 const delivery: ArtifactSliceDelivery = { kind: 'replacement-shipped', };
 ```
 */
export type ArtifactSliceDelivery = {
  /**
   Document carries the lane's decision, which differs from the archive.
   */
  readonly kind: 'replacement-shipped';
} | {
  /**
   Lane decided a replacement and the document does not carry it.
   */
  readonly kind: 'replacement-withdrawn';

  /**
   Which mechanism took it back: the per-slice assembly guard, or the
   whole-document refusal that never assembled at all.
   */
  readonly reason: 'assembly-integrity' | 'blocked-non-translation';
} | {
  /**
   Document carries the archive's own wording.
   */
  readonly kind: 'incumbent-retained';
} | {
  /**
   Passage is missing, and the archive never had it either.
   */
  readonly kind: 'gap-remains';
};

/**
 One row of one lane's delivery ledger, as version 2 records it.

 @example
 ```ts
 const row: ArtifactDeliveryRow = { sliceIndex: 0, sourceText: '猫', ... };
 ```
 */
export type ArtifactDeliveryRow = {
  /**
   Global slice index, which every join uses.
   */
  readonly sliceIndex: number;

  /**
   Original this slice was translated from.
   */
  readonly sourceText: string;

  /**
   Whether the archive holds any wording at this slice at all.
   */
  readonly incumbentKind: 'present' | 'absent';

  /**
   Archive's own English for it.
   */
  readonly incumbentText: string;

  /**
   What the lane did.
   */
  readonly outcome: ArtifactSliceOutcome;

  /**
   Wording the lane's document carries here.
   */
  readonly shippedText: string;

  /**
   How it came to carry that.
   */
  readonly delivery: ArtifactSliceDelivery;
};

/**
 How the two lanes' own decisions relate at one slice, as version 2 records
 it.

 @example
 ```ts
 const decisions: ArtifactDecisionComparison = { kind: 'comparable', verdict: 'same', };
 ```
 */
export type ArtifactDecisionComparison = {
  /**
   Both lanes decided a wording.
   */
  readonly kind: 'comparable';

  /**
   Whether those wordings match, character for character.
   */
  readonly verdict: 'same' | 'different';
} | {
  /**
   At least one lane decided nothing here.
   */
  readonly kind: 'not-comparable';

  /**
   Which lanes those were, in lane order.
   */
  readonly undecidedLanes: readonly ('repair' | 'translate')[];
};

/**
 How the two documents relate at one slice, as version 2 records it.

 @example
 ```ts
 const laneRelation: ArtifactLaneRelation = 'both-differ';
 ```
 */
export type ArtifactLaneRelation =
  | 'archive-stands'
    | 'repair-only'
    | 'translate-only'
    | 'both-agree'
    | 'both-differ'
    | 'gap-remains';

/**
 One slice as both lanes left it, as version 2 records it.

 @example
 ```ts
 const row: ArtifactComparisonRow = { sliceIndex: 0, laneRelation: 'both-differ', ... };
 ```
 */
export type ArtifactComparisonRow = {
  /**
   Slice both lanes name it by.
   */
  readonly sliceIndex: number;

  /**
   Whether the archive holds any wording here.
   */
  readonly incumbentKind: 'present' | 'absent';

  /**
   Archive's own English for it.
   */
  readonly incumbentText: string;

  /**
   Wording the repair document carries.
   */
  readonly repairText: string;

  /**
   Wording the translate document carries.
   */
  readonly translateText: string;

  /**
   Which lanes changed this slice, and whether their changes agree.

   NOT SPELLED `verdict`, which it was until 2026-08-22. That put one key
   name over two meanings at sibling paths of one artifact:
   `laneSelection.slices[].verdict` says who WON, which is the question
   about shipping, while this says which lanes CHANGED anything, naming no
   winner. Reading the wrong one does not throw and does not surface as
   `undefined` anywhere a reader looks; a probe that took the string form
   for the object form reported 142 of 142 slices as audit subjects, a
   false total that read as an answer.
   */
  readonly laneRelation: ArtifactLaneRelation;

  /**
   What the repair lane did.
   */
  readonly repairOutcome: ArtifactSliceOutcome;

  /**
   What the translate lane did.
   */
  readonly translateOutcome: ArtifactSliceOutcome;

  /**
   Whether their decisions were comparable, and how they came out.
   */
  readonly decisionComparison: ArtifactDecisionComparison;

  /**
   How the repair document came to carry what it carries.
   */
  readonly repairDelivery: ArtifactSliceDelivery;

  /**
   How the translate document did.
   */
  readonly translateDelivery: ArtifactSliceDelivery;
};

//endregion Artifact version 2 vocabulary
