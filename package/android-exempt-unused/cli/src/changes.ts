/**
 Pure diff between current device state and the user's selection.
 
 This is the heart of the no-toggle UX: the multiselect's checked set is the
 desired exempt state, so the work to do is the difference from what is
 already exempted. Both directions fall out of one comparison.
 
 @module
 */

/**
 Application ids to change, split by direction. `toExempt` get
 {@link ./constants.ts MODE_IGNORE}; `toRevert` get
 {@link ./constants.ts MODE_DEFAULT}.
 */
export type Changes = {
  readonly toExempt: readonly string[];
  readonly toRevert: readonly string[];
};

/**
 Compute the changes needed to make the device match `selected`.
 
 Everything is intersected with `all` so stale or out-of-scope ids never
 produce work. An app newly checked (in `selected`, not yet exempted) is an
 exempt; an app unchecked (was exempted, no longer selected) is a revert.
 
 @param all - Every third-party application id offered in the list, in display
              order; output order follows it.
 
 @param currentlyExempted - Application ids already exempted on the device.
 
 @param selected - Application ids the user left checked.
 
 @returns Changes with `toExempt` and `toRevert` partitions.
 
 @example
 ```ts
 computeChanges({
   all: ['a', 'b', 'c',],
   currentlyExempted: ['b',],
   selected: ['a', 'b',],
 },);
 // { toExempt: ['a'], toRevert: [] }
 ```
 */
export function computeChanges({
  all,
  currentlyExempted,
  selected,
}: {
  readonly all: readonly string[];
  readonly currentlyExempted: readonly string[];
  readonly selected: readonly string[];
},): Changes {
  /**
   Set of in-scope ids; selection and exemption are intersected with it.
   */
  const inScope: ReadonlySet<string> = new Set(all,);
  /**
   In-scope ids currently exempted on the device.
   */
  const exemptedSet: ReadonlySet<string> = new Set(
    currentlyExempted.filter(function inScopeName(name,): boolean {
      return inScope.has(name,);
    },),
  );
  /**
   In-scope ids the user left checked.
   */
  const selectedSet: ReadonlySet<string> = new Set(
    selected.filter(function inScopeName(name,): boolean {
      return inScope.has(name,);
    },),
  );
  return {
    toExempt: all.filter(function needsExempt(name,): boolean {
      return selectedSet.has(name,) && (!exemptedSet.has(name,));
    },),
    toRevert: all.filter(function needsRevert(name,): boolean {
      return (!selectedSet.has(name,)) && exemptedSet.has(name,);
    },),
  };
}
