import type { ReadonlyEntry, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Roster configuration
// Whether a lane was configured with anybody to ask, checked before it spends
// an hour discovering it was not.
//
// THE DEFECT THIS EXISTS FOR is silence that looks like success. A stage whose
// voices all fail settles rather than throwing, deliberately, because throwing
// on an outage would write an answer the pipeline never received into the slice
// cache. A stage configured with an EMPTY roster produces the same observation:
// no exchanges, no findings, a settled unchanged document. Downstream nothing
// can tell them apart, so a corpus pass under a misconfiguration spends hours
// writing a directory of vacuous artifacts that later analysis reads as clean.
//
// CONFIGURATION ONLY, never `heardCritics === 0` or any other run-time count.
// The first is deterministic and the operator's to fix; the second is the
// transient silence the cache rule exists to tolerate, and refusing it would
// turn a provider outage into a failed pass.
//
// EVERY EMPTY ROLE IN ONE ERROR, because an operator fixing one at a time pays
// a whole preflight per role.

/**
 Raised when a lane is asked to run with a role that has nobody in it.

 A configuration error rather than a provider or stage error, so a runner
 catching it stops the pass rather than retrying the entry or writing a
 per-entry failure artifact: the next entry would fail identically.

 @example
 ```ts
 throw new RosterConfigurationError({ message: 'repair lane has no editorModelIds', },);
 ```
 */
export class RosterConfigurationError extends Error {
  /**
   Builds the error with a message naming the lane and every empty role.

   @param message - what is unconfigured

   @example
   ```ts
   throw new RosterConfigurationError({ message: 'repair lane has no editorModelIds', },);
   ```
   */
  constructor({ message, }: { readonly message: string; },) {
    super(message,);
    this.name = 'RosterConfigurationError';
  }
}

/**
 Refuses a lane whose required roles have nobody in them.

 Called at every depth a caller can enter at, because each of the others is
 bypassable: the document drivers are called directly by the combined driver,
 the combined driver is bypassable by a library caller, and a corpus preflight
 is bypassed by both. The check is cheap and idempotent, so running it three
 times costs nothing worth measuring.

 @param lane - which lane is being configured, named in the message

 @param roles - required rosters by the name a caller configures them under;
 an optional role belongs nowhere in this map, since its empty list is how it
 is turned off

 @throws RosterConfigurationError naming every empty role at once

 @example
 ```ts
 assertRostersConfigured({ lane: 'translate', roles: { translatorModelIds, judgeModelIds, }, },);
 ```
 */
export function assertRostersConfigured(
  {
    lane,
    roles,
  }: {
    readonly lane: string;
    readonly roles: Readonly<Record<string, readonly string[]>>;
  },
): void {
  /**
   Roles configured with nobody in them, in the order they were given.
   */
  const empty = Object.entries(roles,)
    .filter(function hasNobody([, models,]: ReadonlyEntry<string, readonly string[]>,): boolean {
      return models.length === 0;
    },)
    .map(function toName([name,]: ReadonlyEntry<string, readonly string[]>,): string {
      return name;
    },);
  if (empty.length === 0)
    return;
  throw new RosterConfigurationError({
    message: `${lane} lane is configured with nobody in ${
      empty.join(', ',)
    }: a stage that can never speak settles exactly like one whose voices all `
      + 'failed, so the run would look clean rather than misconfigured',
  },);
}

//endregion Roster configuration
