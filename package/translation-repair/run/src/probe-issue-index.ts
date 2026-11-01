import type { OwnedProbeReading, } from '@monochromatic-dev/translation-repair-artifact/ts';
import type { TelemetryProbeReading, } from '@monochromatic-dev/translation-repair-lane-contest/ts';

//region Probe issue index
// The join a graded sheet position takes to reach a probe verdict: position to
// issue id through the manifest, then issue id to reading here.
//
// This lives in its own module rather than beside the task that uses it. A task
// script is an ENTRY POINT, and exporting from one drags its top-level work
// into the package bundle: `score-probe.ts` called `main()` unguarded, so
// exporting this function from there made importing the LIBRARY scan a corpus
// directory and print to stdout. That call is guarded now, but a module the
// package exports should not be one that also runs something.

/**
 Raised when two shipped records claim one issue id.

 @example
 ```ts
 throw new ProbeIssueIndexError({ message: 'two shipped records claim issue 4f2a', },);
 ```
 */
export class ProbeIssueIndexError extends Error {
  /**
   Builds refusal carrying what could not hold.

   @param message - which id more than one record claims

   @example
   ```ts
   throw new ProbeIssueIndexError({ message: 'two shipped records claim issue 4f2a', },);
   ```
   */
  public constructor({ message, }: { readonly message: string; },) {
    super(message,);
    this.name = 'ProbeIssueIndexError';
  }
}

/**
 Indexes readings by the issue that OWNS each one.

 Ownership comes from the record the reading was written on, never from the
 issue lists inside its regions. A region names every issue it serves, and one
 replacement can serve several accepted issues, so reading ownership off those
 lists attaches whichever record happened to be indexed last. That is not a
 rare collision: it is the ordinary case whenever an envelope served more than
 one issue, and the joint counts would look perfectly normal while describing
 the wrong record.

 @param owned - readings paired with their owning issue, across every artifact

 @returns Issue-keyed readings

 @throws {@link Error} when two records claim one issue id, which would mean
 the identity this join rests on is not unique

 @example
 ```ts
 const byIssueId = indexReadingsByIssue({ owned, },);
 ```
 */
export function indexReadingsByIssue(
  { owned, }: { readonly owned: readonly OwnedProbeReading[]; },
): ReadonlyMap<string, TelemetryProbeReading> {
  /**
   Issue-keyed readings, filled with a conflict check per insertion.
   */
  const byIssueId = new Map<string, TelemetryProbeReading>();
  for (const entry of owned) {
    /**
     Reading already recorded for this issue, absent on first sighting.
     */
    const existing = byIssueId.get(entry.issueId,);
    if ((existing !== undefined) && (existing !== entry.reading))
      throw new ProbeIssueIndexError({
        message: `two shipped records claim issue ${entry.issueId}. The graded sheet `
          + 'joins to probe verdicts through this id, so a duplicate would '
          + 'attach one record\'s verdict to another record\'s position '
          + 'without the counts showing it.',
      },);
    byIssueId.set(
      entry.issueId,
      entry.reading,
    );
  }
  return byIssueId;
}

//endregion Probe issue index
