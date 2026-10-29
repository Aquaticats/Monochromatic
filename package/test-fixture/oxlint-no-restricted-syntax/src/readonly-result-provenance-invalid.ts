/**
 * Fixture probing where a collection member's result carries receiver state.
 *
 * Every function here mutates caller-owned state reached through a call result
 * rather than through the receiver directly. Nothing currently records that a
 * result aliases the receiver, so each lands on the opaque boundary instead of
 * being attributed to the parameter it actually changes.
 *
 * @module
 */

/**
 * Element carrying mutable state, so an interior write is observable.
 */
type Labelled = {
  label: string;
};

/**
 * Hands back a container holding the receiver's own elements.
 *
 * Half of the pair proving a returned container is a fact callers can propagate. The array
 * returned is fresh and the elements in it are the caller's, so `expressionOrigins` finds
 * nothing here and the element origins find the parameter. Before both were asked, this
 * recorded no returned origin at all and every caller of it was left with nothing to
 * substitute.
 *
 * @param rows - Rows whose elements the result carries.
 *
 * @returns fresh container of the caller's own rows.
 *
 * @example
 * ```ts
 * returnsReceiverElements([],);
 * ```
 */
export function returnsReceiverElements(rows: readonly Labelled[],): readonly Labelled[] {
  return rows.slice(0,);
}

/**
 * Hands back the receiver's elements through two composed container members.
 *
 * The shape the corpus actually writes, and the one a single-hop element walk answered
 * nothing for. `orderedRoots` in `package/file-manager/variant-electron/src/strip.ts`
 * returns `panes.filter(rootLike,).toSorted(bySpawnOrder,)`; this is that reduced to its
 * parts. The outer member's receiver is the inner call, whose own value origins are empty
 * because the array it returns is fresh, so a chain of relations each of which holds
 * reported no origin between them.
 *
 * @param rows - Rows whose elements the composed result carries.
 *
 * @returns fresh container of the caller's own rows, reversed.
 *
 * @example
 * ```ts
 * returnsComposedReceiverElements([],);
 * ```
 */
export function returnsComposedReceiverElements(
  rows: readonly Labelled[],
): readonly Labelled[] {
  return rows
    .slice(0,)
    .toReversed();
}

/**
 * Writes the caller's row through a composed container another callable returned.
 *
 * The half that makes the composition matter, and the control against the single-hop pair
 * beside it: if the walk stops at the first relation, this attributes nothing while its
 * one-member sibling attributes the write, and the two disagree about identical state
 * reached through one extra member.
 *
 * @param rows - Rows whose element is rewritten through a composed container.
 *
 * @example
 * ```ts
 * writesThroughComposedContainer([],);
 * ```
 */
export function writesThroughComposedContainer(rows: readonly Labelled[],): void {
  /**
   * Container returned by the composing callable, holding the caller's rows.
   */
  const carried = returnsComposedReceiverElements(rows,);
  /**
   * Row reached through the composed container.
   */
  const first = carried[0];
  if (first === undefined)
    throw new Error('Expected a composed row to rewrite.',);
  first.label = 'rewritten';
}

/**
 * Writes the caller's row through a container another callable returned.
 *
 * The half that makes the first half matter. The write lands on a row `rows` holds, reached
 * through a container `returnsReceiverElements` built, so it is attributed to `rows` only
 * when that callable records its returned origin. Its element sibling, writing through a
 * returned element rather than a returned container, worked before this and is the control.
 *
 * @param rows - Rows whose element is rewritten through a returned container.
 *
 * @example
 * ```ts
 * writesThroughReturnedContainer([],);
 * ```
 */
export function writesThroughReturnedContainer(rows: readonly Labelled[],): void {
  /**
   * Container returned by the callable above, holding the caller's rows.
   */
  const carried = returnsReceiverElements(rows,);
  /**
   * Row reached through the returned container.
   */
  const first = carried[0];
  if (first === undefined)
    throw new Error('Expected a carried row to rewrite.',);
  first.label = 'rewritten';
}

/**
 * Hands back a callable that returns the receiver's elements, rather than the elements.
 *
 * The control on which callable the discharge reasons about. `rows.slice(0,)` is returned
 * outright here exactly as in `returnsReceiverElements`, but the `return` belongs to `inner`
 * while the callers being enumerated are this function's. Those callers substitute for this
 * function, whose result is a callable and not a container, so nothing they record accounts
 * for a write made through what `inner` later returns.
 *
 * Its two siblings differ from it only in where the `return` is written, which is the whole
 * claim: the position test accepts a `ReturnStatement` wherever it appears, so the callable
 * it belongs to has to be checked separately.
 *
 * Measured 2026-08-07, and the measurement corrects what this comment first said. Removing the
 * containment check leaves this program's diagnostics byte-identical, so it pins the shape and
 * does not isolate that check. `rows` is charged here either way, by a path that does not
 * depend on the discharge. No program was found that isolates it, which makes the check
 * defence in depth rather than a fix with a failing case behind it.
 *
 * Retested the same day after the position condition widened to attribute returns, on the
 * theory that crediting a nested `return` would need the check. An unexported twin with an
 * in-file caller, compared on charges rather than offers, was byte-identical again: the escape
 * test treats any reference inside a nested callable as escaping, so the parameter is charged
 * before the discharge is consulted. The twin was removed for isolating nothing.
 *
 * @param rows - Rows the returned callable hands back.
 *
 * @returns callable handing back a fresh container of the caller's own rows.
 *
 * @example
 * ```ts
 * returnsFromNestedCallable([],);
 * ```
 */
export function returnsFromNestedCallable(
  rows: readonly Labelled[],
): () => readonly Labelled[] {
  /**
   * Hands back the caller's rows in a fresh container, from its own `return`.
   */
  function inner(): readonly Labelled[] {
    return rows.slice(0,);
  }
  return inner;
}

/**
 * Hands back the receiver's elements without letting any other file reach this callable.
 *
 * The positive control the guard programs never had, and the reason they proved nothing. A
 * program only reaches the returned-result discharge when two things hold that none of them
 * held: it must not be exported, since `callersAreEnumerable` refuses any callable another
 * file can import, and it must have a caller here, since an empty enumeration is refused.
 *
 * This is `returnsReceiverElements` with both satisfied. It is offered read-only, and the
 * assertion that it is offered is what proves the harness can observe this feature at all.
 * Without such a control, a probe reporting no difference is indistinguishable from a probe
 * that cannot see the thing it is probing, which is exactly what happened to the six programs
 * written before it.
 *
 * It also fixes the reach of requiring closed-world callers. Every other program exercising
 * the discharge here is exported and so refused, which made the feature look dead; this one
 * shows it is scoped rather than gone.
 *
 * @param rows - Rows whose elements the result carries.
 *
 * @returns fresh container of the caller's own rows.
 *
 * @example
 * ```ts
 * readsLocalContainerLength([],);
 * ```
 */
function localReceiverElements(rows: readonly Labelled[],): readonly Labelled[] {
  return rows.slice(0,);
}

/**
 * Reads how many rows the unexported container holds, so its callee has a caller.
 *
 * @param rows - Rows counted through an unexported container.
 *
 * @returns how many rows it holds.
 *
 * @example
 * ```ts
 * readsLocalContainerLength([],);
 * ```
 */
export function readsLocalContainerLength(rows: readonly Labelled[],): number {
  return localReceiverElements(rows,)
    .length;
}

/**
 * Hands back elements of whichever rows a reassignable name last held.
 *
 * `localReceiverElements` with one statement added, which is what makes it a test rather than
 * another program. The declaration hop follows a name to the value it was declared with, and
 * that hop is shared with a walk that uses it to add origins, where ignoring later assignment
 * over-attributes harmlessly. Proving a receiver is *not* foreign-owned reads the same property
 * backwards, so a name that could have moved is refused instead of followed.
 *
 * @param rows - Rows the name is declared with.
 *
 * @param other - Rows the name is pointed at instead.
 *
 * @returns fresh container of whichever rows the name last held.
 *
 * @example
 * ```ts
 * readsLocalReassignedLength([], [],);
 * ```
 */
function localReassignedElements({
  rows,
  other,
}: {
  rows: readonly Labelled[];
  other: readonly Labelled[];
},): readonly Labelled[] {
  /* oxlint-disable no-restricted-syntax/no-function-root-let -- A reassignable binding whose
   * value survives to the return is the whole program under test, and every remedy the rule
   * names removes it: `const` deletes the reassignment, and both the helper shape and the
   * named-function IIFE require the body to end in `return <identifier>` where the discharge
   * requires the member call itself to be returned. Rule source read at
   * `package/oxlint-plugin/no-restricted-syntax/src/rule/no-function-root-let.ts`, whose own
   * message prescribes this disable for the unavoidable case. */
  /**
   * Rows held first from one parameter and then from the other.
   */
  let held = rows;
  /* oxlint-enable no-restricted-syntax/no-function-root-let */
  held = other;
  return held.slice(0,);
}

/**
 * Reads how many rows the reassigned-name container holds, so its callee has a caller.
 *
 * @param rows - Rows the name is declared with.
 *
 * @param other - Rows the name is pointed at instead.
 *
 * @returns how many rows it holds.
 *
 * @example
 * ```ts
 * readsLocalReassignedLength([], [],);
 * ```
 */
export function readsLocalReassignedLength({
  rows,
  other,
}: {
  rows: readonly Labelled[];
  other: readonly Labelled[];
},): number {
  return localReassignedElements({ rows, other, },)
    .length;
}

/**
 * Hands back elements of a parameter pointed somewhere else first.
 *
 * The endpoint no declaration can answer for, and the sibling of the reassignable-name case. A
 * `let` carries its own answer; a parameter is declared once and any statement may point it
 * elsewhere. The ownership marker does not stop that, since `ForeignBorrowed<Value>` intersects
 * an *optional* property and so assigns to a plain `Value` with no error.
 *
 * @param rows - Rows pointed at the other parameter before use.
 *
 * @param other - Rows the parameter is pointed at.
 *
 * @returns fresh container of whichever rows the parameter last held.
 *
 * @example
 * ```ts
 * readsLocalRepointedLength([], [],);
 * ```
 */
function localRepointedElements({
  rows,
  other,
}: {
  rows: readonly Labelled[];
  other: readonly Labelled[];
},): readonly Labelled[] {
  rows = other;
  return rows.slice(0,);
}

/**
 * Reads how many rows the repointed-parameter container holds, so its callee has a caller.
 *
 * @param rows - Rows pointed at the other parameter before use.
 *
 * @param other - Rows the parameter is pointed at.
 *
 * @returns how many rows it holds.
 *
 * @example
 * ```ts
 * readsLocalRepointedLength([], [],);
 * ```
 */
export function readsLocalRepointedLength({
  rows,
  other,
}: {
  rows: readonly Labelled[];
  other: readonly Labelled[];
},): number {
  return localRepointedElements({ rows, other, },)
    .length;
}

/**
 * Hands back elements of a repointed parameter hidden behind a type assertion.
 *
 * The wrapper unwrap's program, and it works by making another guard unreachable rather than
 * by being refused itself. `bindingAssignedWithin` can only answer about an `Identifier`, so an
 * assertion around the base hides the name from it and the written-endpoint check silently
 * passes on a parameter that was pointed elsewhere.
 *
 * That is the shape of the whole family: every structural test in the descent asks what kind of
 * expression the base is, and a wrapper answers for itself. `as`, parentheses, `!` and
 * `satisfies` all do it, and this is the ordinary spelling.
 *
 * @param rows - Rows pointed at the other parameter before use.
 *
 * @param other - Rows the parameter is pointed at.
 *
 * @returns fresh container of whichever rows the parameter last held.
 *
 * @example
 * ```ts
 * readsLocalAssertedRepointedLength([], [],);
 * ```
 */
function localAssertedRepointedElements({
  rows,
  other,
}: {
  rows: readonly Labelled[];
  other: readonly Labelled[];
},): readonly Labelled[] {
  rows = other;
  return (rows as readonly Labelled[]).slice(0,);
}

/**
 * Reads how many rows the asserted repointed container holds, so its callee has a caller.
 *
 * @param rows - Rows pointed at the other parameter before use.
 *
 * @param other - Rows the parameter is pointed at.
 *
 * @returns how many rows it holds.
 *
 * @example
 * ```ts
 * readsLocalAssertedRepointedLength([], [],);
 * ```
 */
export function readsLocalAssertedRepointedLength({
  rows,
  other,
}: {
  rows: readonly Labelled[];
  other: readonly Labelled[];
},): number {
  return localAssertedRepointedElements({ rows, other, },)
    .length;
}

/**
 * Holder keeping a container beyond the call, written without invoking anything.
 *
 * A property assignment rather than a `push`, deliberately. A call into a collection member
 * charges the parameter on its own, which would hide whichever condition the store is meant to
 * test; a bare store reaches the escape test and nothing else.
 */
const carriedHolder: { current: readonly Labelled[]; } = { current: [], };

/**
 * Hands back the receiver's elements through a local bound first.
 *
 * Route the discharge does not yet accept. `callIsReturnedOutright` requires the call to be
 * the returned expression itself, and here a `const` stands between, though the value reaching
 * the caller is the same one.
 *
 * @param rows - Rows whose elements the result carries.
 *
 * @returns fresh container of the caller's own rows.
 *
 * @example
 * ```ts
 * readsLocalBoundLength([],);
 * ```
 */
function localBoundElements(rows: readonly Labelled[],): readonly Labelled[] {
  /**
   * Fresh container holding the caller's rows.
   */
  const copy = rows.slice(0,);
  return copy;
}

/**
 * Reads how many rows the bound container holds, so its callee has a caller.
 *
 * @param rows - Rows counted through a bound container.
 *
 * @returns how many rows it holds.
 *
 * @example
 * ```ts
 * readsLocalBoundLength([],);
 * ```
 */
export function readsLocalBoundLength(rows: readonly Labelled[],): number {
  return localBoundElements(rows,)
    .length;
}

/**
 * Hands back the receiver's elements through a transparent wrapper.
 *
 * The other route, and the smaller one: an assertion erases at runtime, so the returned value
 * is the call's own.
 *
 * @param rows - Rows whose elements the result carries.
 *
 * @returns fresh container of the caller's own rows.
 *
 * @example
 * ```ts
 * readsLocalWrappedLength([],);
 * ```
 */
function localWrappedElements(rows: readonly Labelled[],): readonly Labelled[] {
  return (rows.slice(0,) as readonly Labelled[]);
}

/**
 * Reads how many rows the wrapped container holds, so its callee has a caller.
 *
 * @param rows - Rows counted through a wrapped container.
 *
 * @returns how many rows it holds.
 *
 * @example
 * ```ts
 * readsLocalWrappedLength([],);
 * ```
 */
export function readsLocalWrappedLength(rows: readonly Labelled[],): number {
  return localWrappedElements(rows,)
    .length;
}

/**
 * Hands back the receiver's elements after also storing them outside this callable.
 *
 * The negative control, and the condition widening the route has to carry. Returning is the one
 * escape whose destination this analysis follows; a store into module state is not, and nothing
 * a caller substitutes accounts for a write made through the stored copy. So this must keep its
 * report however the return is spelled.
 *
 * @param rows - Rows whose elements are both stored and returned.
 *
 * @returns fresh container of the caller's own rows.
 *
 * @example
 * ```ts
 * readsLocalStoredLength([],);
 * ```
 */
function localBoundAndStoredElements(rows: readonly Labelled[],): readonly Labelled[] {
  /**
   * Fresh container holding the caller's rows, kept beyond this call.
   */
  const copy = rows.slice(0,);
  carriedHolder.current = copy;
  return copy;
}

/**
 * Reads how many rows the stored container holds, so its callee has a caller.
 *
 * @param rows - Rows counted through a stored container.
 *
 * @returns how many rows it holds.
 *
 * @example
 * ```ts
 * readsLocalStoredLength([],);
 * ```
 */
export function readsLocalStoredLength(rows: readonly Labelled[],): number {
  return localBoundAndStoredElements(rows,)
    .length;
}

/**
 * Reads only how many rows a returned container holds.
 *
 * The control keeping the returned origin from becoming a blanket attribution. Nothing here
 * reaches an element, so nothing may be recorded against `rows`: a returned origin says a
 * caller *can* reach the parameter through the result, not that this caller did.
 *
 * @param rows - Rows counted through a returned container.
 *
 * @returns how many rows the returned container holds.
 *
 * @example
 * ```ts
 * readsReturnedContainerLength([],);
 * ```
 */
export function readsReturnedContainerLength(rows: readonly Labelled[],): number {
  return returnsReceiverElements(rows,)
    .length;
}

/**
 * Mutates a parameter held inside an object this callable built.
 *
 * The first of the two programs `doc/planning/prefer-readonly-container-value-provenance.md`
 * names as the ones a careless container fix would wrongly offer read-only. The write goes
 * *through* the fresh object to the caller's own value, so it has to stay attributed.
 *
 * The container record that suppresses `stack.pop()` must not touch this: it is consulted
 * only where a member restructures its receiver, and this is an ordinary property write
 * whose origins run through the value path. Emptying those origins instead, which is the
 * redesign that document costs and rejects, empties this.
 *
 * @param box - Value written through a locally built object.
 *
 * @example
 * ```ts
 * heldObjectMutationEffect({ label: 'original', },);
 * ```
 */
export function heldObjectMutationEffect(box: Labelled,): void {
  /**
   * Object built here, holding the caller's value.
   */
  const held = {
    inner: box,
  };
  held.inner.label = 'rewritten';
}

/**
 * Mutates a parameter held inside an array this callable built.
 *
 * The second of the two programs, through the element path rather than the property path.
 * `held` is a container this callable constructed, so the record covers it and the
 * structural charge on it is suppressed; the write below is still the caller's value being
 * rewritten and is attributed by the element step.
 *
 * @param box - Value written through a locally built array.
 *
 * @example
 * ```ts
 * heldArrayMutationEffect({ label: 'original', },);
 * ```
 */
export function heldArrayMutationEffect(box: Labelled,): void {
  /**
   * Array built here, holding the caller's value.
   */
  const held = [box,];
  /**
   * Element read back out of the local array.
   */
  const first = held[0];
  if (first === undefined)
    throw new Error('Expected the held element to rewrite.',);
  first.label = 'rewritten';
}

/**
 * Restructures a container this callable built while holding a parameter.
 *
 * The case the container record exists for, and the direction opposite to the two above.
 * Nothing writes `box`: `pop` restructures the fresh array, and the parameter is what the
 * array holds rather than what holds the array. Reported as a mutation before the record
 * existed, on the work-stack idiom `AGENTS.md` requires over recursion.
 *
 * @param box - Value the local array holds and nothing writes.
 *
 * @returns whether the array still holds anything.
 *
 * @example
 * ```ts
 * heldContainerRestructureEffect({ label: 'original', },);
 * ```
 */
export function heldContainerRestructureEffect(box: Labelled,): boolean {
  /**
   * Array built here, holding the caller's value.
   */
  const stack = [box,];
  stack.pop();
  return stack.length > 0;
}

/**
 * Restructures a container the caller owns, reached through one local hop.
 *
 * The control that keeps the record from covering every local. `inner` names the caller's
 * own array rather than one built here, so its binding arrives through a property step,
 * the record is not set, and the structural charge stands. A record keyed on locality
 * rather than on how the value was built would discharge this and lose a real write.
 *
 * @param carrier - Carrier whose own array is restructured.
 *
 * @example
 * ```ts
 * borrowedContainerRestructureEffect({ rows: [], },);
 * ```
 */
export function borrowedContainerRestructureEffect(
  carrier: { rows: Labelled[]; },
): void {
  /**
   * Caller's own array, reached through one local hop.
   */
  const inner = carrier.rows;
  inner.push({ label: 'appended', },);
}

/**
 * Mutates the element an observer member handed back on its own.
 *
 * The pair to `boundLookupMutationEffect`, for the arm of the result gate that answers
 * a bare value rather than a container. `find` carries the verified relation saying its
 * result is one of the receiver's own elements, and it takes an owned observer, so the
 * receiver is discharged and the write below is recorded against `rows` directly.
 *
 * Before that arm existed the call reported opacity instead, which read as caution and
 * was a hole: `at` carries the identical relation and answered from the channel table,
 * so the same write through the same kind of element was attributed for one member and
 * merely reported for the other.
 *
 * Both directions fail here. Remove the value arm from `viewResultUnaccounted` and this
 * reports opacity rather than a mutation; remove the element attribution beneath it and
 * the mutation disappears while the discharge stays, which is the wrong offer this
 * fixture exists to prevent.
 *
 * @param rows - Rows whose matching element is rewritten through a result.
 *
 * @example
 * ```ts
 * observerValueResultMutationEffect([{ label: 'target' },]);
 * ```
 */
export function observerValueResultMutationEffect(rows: Labelled[],): void {
  /**
   * Matching element obtained through an owned predicate.
   */
  const found = rows.find(function isTarget(candidate: { readonly label: string; },): boolean {
    return candidate.label === 'target';
  },);
  if (found === undefined)
    throw new Error('Expected a match to rewrite.',);
  found.label = 'rewritten';
}

/**
 * Mutates a looked-up value through an intermediate binding.
 *
 * The shape the rule's own `addUncertaintyProvenance` uses, and the reason
 * `readonlyEffectSelfHostingOverride` cannot narrow: `Map.get` reaches no user code
 * and returns a value that carries state, and the body mutates exactly that value.
 *
 * @param facts - Map whose stored set is extended.
 *
 * @param key - Lookup key.
 *
 * @example
 * ```ts
 * boundLookupMutationEffect(new Map(), 'k');
 * ```
 */
export function boundLookupMutationEffect(
  facts: Map<string, Set<string>>,
  key: string,
): void {
  /**
   * Stored set obtained from the receiver, or a fresh accumulator.
   */
  const stored = facts.get(key,) ?? new Set<string>();
  stored.add('recorded',);
  facts.set(
    key,
    stored,
  );
}

/**
 * Mutates a looked-up value with no intermediate binding at all.
 *
 * The chained form. Provenance that only registers variable declarations cannot
 * see this, because there is no binding to register: the receiver of `add` is
 * itself a call expression.
 *
 * @param facts - Map whose stored set is extended in place.
 *
 * @param key - Lookup key.
 *
 * @example
 * ```ts
 * chainedLookupMutationEffect(new Map(), 'k');
 * ```
 */
export function chainedLookupMutationEffect(
  facts: Map<string, Set<string>>,
  key: string,
): void {
  facts.get(key,)
    ?.add('recorded',);
}

/**
 * Writes through a result reached by property access, with no binding.
 *
 * `values.at(0)` hands back an element, and the write targets a property of that
 * element. The write target's root is a call expression rather than an identifier,
 * so root resolution stops before reaching the parameter.
 *
 * @param values - Array whose first element is rewritten.
 *
 * @example
 * ```ts
 * chainedElementWriteEffect([]);
 * ```
 */
export function chainedElementWriteEffect(values: Labelled[],): void {
  /**
   * Element obtained without binding, whose property is overwritten.
   */
  const element = values.at(0,);
  if (element !== undefined)
    element.label = 'rewritten';
}

/**
 * Destructures a looked-up value and mutates the extracted part.
 *
 * Provenance must survive destructuring, or the extracted binding carries no
 * origin and the write is attributed to nothing.
 *
 * @param rows - Map whose stored row is rewritten through a destructured element.
 *
 * @param key - Lookup key.
 *
 * @example
 * ```ts
 * destructuredLookupMutationEffect(new Map(), 'k');
 * ```
 */
export function destructuredLookupMutationEffect(
  rows: Map<string, Labelled[]>,
  key: string,
): void {
  /**
   * Stored row, destructured to its first element.
   */
  const [first,] = rows.get(key,) ?? [];
  if (first !== undefined)
    first.label = 'rewritten';
}

/**
 * Passes a looked-up value to a callee this rule cannot inspect.
 *
 * The escape case. Attributing the mutation is not enough here: once the result
 * leaves through an unresolved call, the parameter it came from must be reported as
 * reaching that call, exactly as a direct argument would be.
 *
 * @param facts - Map whose stored set escapes into an unresolved call.
 *
 * @param key - Lookup key.
 *
 * @example
 * ```ts
 * escapingLookupEffect(new Map(), 'k');
 * ```
 */
export function escapingLookupEffect(
  facts: Map<string, Set<string>>,
  key: string,
): void {
  /**
   * Stored set handed to a host serializer.
   */
  const stored = facts.get(key,);
  if (stored !== undefined)
    JSON.stringify([...stored,],);
}

/**
 * Returns a looked-up value, letting caller-owned state leave the callable.
 *
 * Provenance says this result is reachable from `facts`. Whether a returned alias
 * is itself an effect is a separate question from attribution, and this function
 * exists to pin whichever answer the model gives rather than to assert one.
 *
 * @param facts - Map whose stored set is handed back.
 *
 * @param key - Lookup key.
 *
 * @returns stored set, aliasing receiver interior.
 *
 * @example
 * ```ts
 * returnedLookupEffect(new Map(), 'k');
 * ```
 */
export function returnedLookupEffect(
  facts: Map<string, Set<string>>,
  key: string,
): Set<string> | undefined {
  return facts.get(key,);
}

/**
 * Reads a looked-up value without mutating it.
 *
 * The control. Provenance must not turn every lookup into an effect: this receiver
 * is only read, so it must still be offered as read-only. Absent this, every
 * assertion here would hold against a fixture nothing linted.
 *
 * @param facts - Map read through a lookup.
 *
 * @param key - Lookup key.
 *
 * @returns stored count.
 *
 * @example
 * ```ts
 * readOnlyLookupEffect(new Map(), 'k');
 * ```
 */
export function readOnlyLookupEffect(
  facts: Map<string, Set<string>>,
  key: string,
): number {
  return (facts.get(key,) ?? new Set<string>()).size;
}

/**
 * Mutates a looked-up value reached through computed member access.
 *
 * The syntax-shape probe. `facts['get']` is an element access rather than a property
 * access, and both the result relation and the opaque boundary test for a property
 * access before doing anything, so this call may be invisible to each. Any diagnostic
 * at all is the minimum; silence means a mutation of caller state went unreported.
 *
 * @param facts - Map whose stored set is extended through a computed lookup.
 *
 * @param key - Lookup key.
 *
 * @example
 * ```ts
 * computedLookupMutationEffect(new Map(), 'k');
 * ```
 */
export function computedLookupMutationEffect(
  facts: Map<string, Set<string>>,
  key: string,
): void {
  const stored = facts['get'](key,) ?? new Set<string>();
  stored.add('recorded',);
  facts['set'](key, stored,);
}

/**
 * Mutates a looked-up value narrowed by an assertion.
 *
 * The transparent-form probe: `as` erases at runtime, so the value is the lookup's own.
 *
 * @param facts - Map whose stored set is extended after an assertion.
 *
 * @param key - Lookup key.
 *
 * @example
 * ```ts
 * assertedLookupMutationEffect(new Map(), 'k');
 * ```
 */
export function assertedLookupMutationEffect(
  facts: Map<string, Set<string>>,
  key: string,
): void {
  const stored = facts.get(key,) as Set<string>;
  stored.add('recorded',);
}

/**
 * Restructures a receiver through computed member access.
 *
 * The sharpened syntax-shape probe. Unlike the map cases, `string[]` has an sound
 * deeply read-only projection, so nothing suppresses a suggestion here on grounds of
 * the value type. If computed access is invisible to the collection handling, this
 * parameter is offered as read-only while the body pushes to it, and applying that
 * suggestion does not compile.
 *
 * @param values - Array restructured through a computed member call.
 *
 * @example
 * ```ts
 * computedStructureEffect([]);
 * ```
 */
export function computedStructureEffect(values: string[],): void {
  values['push']('appended',);
}

/**
 * Second element shape, so a map value type can be a union of object types.
 */
type Tagged = {
  tag: string;
};

/**
 * Mutates a looked-up value whose declared type is a union of object types.
 *
 * The normalization probe. `Map<string, Labelled | Tagged>.get` returns
 * `Labelled | Tagged | undefined`, whose constituents are the two object types plus
 * absence, while the receiver's held position is the union `Labelled | Tagged` as one
 * type object. Asking whether any result constituent is identical to that union finds
 * nothing, because the union object never appears among its own flattened
 * constituents, so this mutation went unattributed.
 *
 * @param records - Map whose stored union-typed value is rewritten.
 *
 * @param key - Lookup key.
 *
 * @example
 * ```ts
 * unionValueLookupEffect(new Map(), 'k');
 * ```
 */
export function unionValueLookupEffect(
  records: Map<string, Labelled | Tagged>,
  key: string,
): void {
  const stored = records.get(key,);
  if (stored === undefined)
    return;
  if ('label' in stored)
    stored.label = 'rewritten';
  else
    stored.tag = 'rewritten';
}

/**
 * Mutates a destructured property its contract does not name.
 *
 * The callee half of the restricted-walk probe. Its parameter is destructured and it
 * carries a contract, which is the shape that makes the argument analysis walk a
 * caller's object literal with only the contract-named properties.
 *
 * @param named - Property the contract declares as mutated.
 *
 * @param unnamed - Property the contract omits while the body still mutates it.
 *
 * @mutates named - Adds a recorded entry.
 *
 * @example
 * ```ts
 * mutateBeyondContract({ named: new Set(), unnamed: new Set() });
 * ```
 */
function mutateBeyondContract({
  named,
  unnamed,
}: {
  named: Set<string>;
  unnamed: Set<string>;
},): void {
  named.add('declared',);
  unnamed.add('undeclared',);
}

/**
 * Passes a looked-up value in a literal property the callee contract omits.
 *
 * If the restricted walk skips `unnamed`, the lookup result carries no origin into the
 * call, so mutating it inside the callee is attributed to nothing, while the escape
 * check calls literal membership attributed and licenses discharging the lookup. That
 * combination would be a silent miss, so this probe exists to find out whether the
 * restricted path is reachable from a literal the escape check accepts.
 *
 * @param facts - Map whose stored set is handed to a partially contracted callee.
 *
 * @param key - Lookup key.
 *
 * @example
 * ```ts
 * contractRestrictedLiteralEffect(new Map(), 'k');
 * ```
 */
export function contractRestrictedLiteralEffect(
  facts: Map<string, Set<string>>,
  key: string,
): void {
  const stored = facts.get(key,) ?? new Set<string>();
  mutateBeyondContract({
    named: new Set<string>(),
    unnamed: stored,
  },);
}

/**
 * Mutates both properties of a destructured parameter its contract fully names.
 *
 * Control isolating the contract-name restriction from callee routing in general. Its
 * parameter is destructured exactly like `mutateBeyondContract`, and the only
 * difference is that every mutated property appears in the contract.
 *
 * @param named - First mutated property.
 *
 * @param alsoNamed - Second mutated property.
 *
 * @mutates named - Adds a recorded entry.
 *
 * @mutates alsoNamed - Adds a recorded entry.
 *
 * @example
 * ```ts
 * mutateWithinContract({ named: new Set(), alsoNamed: new Set() });
 * ```
 */
function mutateWithinContract({
  named,
  alsoNamed,
}: {
  named: Set<string>;
  alsoNamed: Set<string>;
},): void {
  named.add('declared',);
  alsoNamed.add('also-declared',);
}

/**
 * Mutates properties reached through an identifier parameter.
 *
 * Control isolating the destructuring shape. Its parameter is a plain identifier, which
 * is the shape that makes the argument analysis walk a caller's literal with every
 * property rather than the contract-named subset.
 *
 * @param bag - Container whose stored sets are mutated.
 *
 * @mutates bag - Adds recorded entries to stored sets.
 *
 * @example
 * ```ts
 * mutateThroughIdentifier({ named: new Set(), unnamed: new Set() });
 * ```
 */
function mutateThroughIdentifier(bag: {
  named: Set<string>;
  unnamed: Set<string>;
},): void {
  bag.named
    .add('declared',);
  bag.unnamed
    .add('undeclared',);
}

/**
 * Passes a looked-up value in a literal property the callee contract names.
 *
 * Pairs with `contractRestrictedLiteralEffect`: same caller shape, same lookup, same
 * literal, and the callee differs only in naming every mutated property.
 *
 * @param facts - Map whose stored set is handed to a fully contracted callee.
 *
 * @param key - Lookup key.
 *
 * @example
 * ```ts
 * fullContractLiteralEffect(new Map(), 'k');
 * ```
 */
export function fullContractLiteralEffect(
  facts: Map<string, Set<string>>,
  key: string,
): void {
  const stored = facts.get(key,) ?? new Set<string>();
  mutateWithinContract({
    named: new Set<string>(),
    alsoNamed: stored,
  },);
}

/**
 * Passes a looked-up value in a literal handed to an identifier parameter.
 *
 * Pairs with `contractRestrictedLiteralEffect`: same caller shape, same lookup, same
 * literal, and the callee differs only in taking an identifier instead of a
 * destructuring pattern.
 *
 * @param facts - Map whose stored set is handed to an identifier-parameter callee.
 *
 * @param key - Lookup key.
 *
 * @example
 * ```ts
 * identifierParameterLiteralEffect(new Map(), 'k');
 * ```
 */
export function identifierParameterLiteralEffect(
  facts: Map<string, Set<string>>,
  key: string,
): void {
  const stored = facts.get(key,) ?? new Set<string>();
  mutateThroughIdentifier({
    named: new Set<string>(),
    unnamed: stored,
  },);
}

/**
 * Row shape whose single property the rule can express as readonly.
 */
type LabelledRow = {
  label: string;
};

/**
 * Mutates a row reached through a destructured property its contract omits.
 *
 * Row-typed counterpart of `mutateBeyondContract`, written so the caller's parameter has
 * a readonly form the rule is able to offer. That is what turns an unrecorded mutation
 * from a lost warning into a suggestion that does not compile.
 *
 * @param named - Row the contract declares as mutated.
 *
 * @param unnamed - Row the contract omits while the body still writes it.
 *
 * @mutates named - Overwrites recorded label.
 *
 * @example
 * ```ts
 * mutateBeyondContractRow({ named: { label: '' }, unnamed: { label: '' } });
 * ```
 */
function mutateBeyondContractRow({
  named,
  unnamed,
}: {
  named: LabelledRow;
  unnamed: LabelledRow;
},): void {
  named.label = 'declared';
  unnamed.label = 'undeclared';
}

/**
 * Passes an element in a literal property the callee contract omits.
 *
 * The parameter is an array of rows, whose readonly form the rule offers, so an
 * unrecorded write through the omitted property surfaces as an offer that fails to
 * compile once applied rather than as a merely missing warning.
 *
 * @param rows - Array whose element is handed to a partially contracted callee.
 *
 * @example
 * ```ts
 * contractRestrictedRowEffect([{ label: '' }]);
 * ```
 */
export function contractRestrictedRowEffect(rows: LabelledRow[],): void {
  const first = rows.at(0,) ?? { label: '', };
  mutateBeyondContractRow({
    named: { label: '', },
    unnamed: first,
  },);
}

/**
 * Passes a parameter itself in a literal property the callee contract omits.
 *
 * No collection lookup and no receiver opacity are involved, so this probe isolates the
 * contract-name narrowing from result provenance entirely. If the narrowing drops the
 * parameter's origin here, the resulting offer is wrong for reasons that predate any
 * provenance work.
 *
 * @param row - Row handed to a partially contracted callee through an omitted property.
 *
 * @example
 * ```ts
 * directRestrictedRowEffect({ label: '' });
 * ```
 */
export function directRestrictedRowEffect(row: LabelledRow,): void {
  mutateBeyondContractRow({
    named: { label: '', },
    unnamed: row,
  },);
}

/**
 * Passes a parameter directly to a callee whose contract omits the mutated property.
 *
 * Companion to `directRestrictedRowEffect` using a direct argument rather than a literal
 * property, so the two together separate the literal-walk path from argument handling in
 * general.
 *
 * @param bag - Container handed straight to a partially contracted callee.
 *
 * @example
 * ```ts
 * directArgumentRestrictedEffect({ named: { label: '' }, unnamed: { label: '' } });
 * ```
 */
export function directArgumentRestrictedEffect(bag: {
  named: LabelledRow;
  unnamed: LabelledRow;
},): void {
  mutateBeyondContractRow(bag,);
}

/**
 * Mutates one destructured property and only reads the other.
 *
 * Companion to `mutateBeyondContractRow` whose contract is accurate, used to measure what
 * dropping the contract-name narrowing costs in precision rather than in soundness.
 *
 * @param named - Row the body writes.
 *
 * @param unnamed - Row the body only reads.
 *
 * @mutates named - Overwrites recorded label.
 *
 * @example
 * ```ts
 * mutateOnlyNamedRow({ named: { label: '' }, unnamed: { label: '' } });
 * ```
 */
function mutateOnlyNamedRow({
  named,
  unnamed,
}: {
  named: LabelledRow;
  unnamed: LabelledRow;
},): void {
  if (unnamed.label === '')
    named.label = 'declared';
}

/**
 * Passes one parameter to a mutated property and another to a read-only one.
 *
 * Records the precision the sound propagation gives up. The callee writes only `named`,
 * so only `first` is really mutated, and propagating every packaged origin credits
 * `second` as well. The cost is a withheld offer, never a wrong one, and recovering it
 * needs the callee's own measured per-property effects rather than its authored contract.
 *
 * @param first - Row the callee writes.
 *
 * @param second - Row the callee only reads.
 *
 * @example
 * ```ts
 * narrowingPrecisionCostEffect({ label: '' }, { label: '' });
 * ```
 */
export function narrowingPrecisionCostEffect(
  first: LabelledRow,
  second: LabelledRow,
): void {
  mutateOnlyNamedRow({
    named: first,
    unnamed: second,
  },);
}

/**
 * Invokes one destructured property and reads another.
 *
 * The callee half of the invocation-exclusion probe. Its contract names only the array it
 * appends to, so the property holding a caller-owned row is one an authored contract used
 * to filter out, and every destructured binding here maps to parameter index zero.
 *
 * @param run - Callback the body invokes.
 *
 * @param collected - Array the body appends to.
 *
 * @param spare - Row the body only reads.
 *
 * @mutates collected - Appends one recorded label.
 *
 * @example
 * ```ts
 * inspectWithCallback({ run: () => {}, collected: [], spare: { label: '' } });
 * ```
 */
function inspectWithCallback({
  run,
  collected,
  spare,
}: {
  run: () => void;
  collected: string[];
  spare: LabelledRow;
},): void {
  run();
  collected.push(spare.label,);
}

/**
 * Mutates a parameter directly and also passes it beside an invoked callback.
 *
 * Middle link of the invocation-exclusion probe. Propagating every packaged origin puts
 * this parameter into the invoked set, because the callee invokes a sibling property and
 * every destructured binding collapses to parameter zero. Mutation propagation subtracts
 * the invoked set, so the direct write here can stop reaching an outer caller.
 *
 * @param victim - Row written directly and passed onward beside a callback.
 *
 * @example
 * ```ts
 * middleInvokedExclusionEffect({ label: '' });
 * ```
 */
export function middleInvokedExclusionEffect(victim: LabelledRow,): void {
  victim.label = 'direct';
  inspectWithCallback({
    run(): void {},
    collected: [],
    spare: victim,
  },);
}

/**
 * Passes a parameter to a callable that both mutates and appears to invoke it.
 *
 * Outer link of the invocation-exclusion probe, and the position where suppression would
 * become a wrong offer rather than a lost fact.
 *
 * @param row - Row handed to the middle link.
 *
 * @example
 * ```ts
 * outerInvokedExclusionEffect({ label: '' });
 * ```
 */
export function outerInvokedExclusionEffect(row: LabelledRow,): void {
  middleInvokedExclusionEffect(row,);
}

/**
 * Invokes one destructured property and writes another.
 *
 * Both effects land on parameter index zero, because every destructured binding of one
 * object parameter shares that index. Mutation propagation subtracts the invoked set from
 * the mutated set by index, so one call that does both can cancel itself out.
 *
 * @param run - Callback the body invokes.
 *
 * @param target - Row the body writes.
 *
 * @mutates target - Overwrites recorded label.
 *
 * @example
 * ```ts
 * invokeAndMutate({ run: () => {}, target: { label: '' } });
 * ```
 */
function invokeAndMutate({
  run,
  target,
}: {
  run: () => void;
  target: LabelledRow;
},): void {
  run();
  target.label = 'mutated';
}

/**
 * Passes a parameter to a callee that both invokes a callback and writes the parameter.
 *
 * Contract-independent probe: the contract names `target`, so the property holding this
 * parameter is one the narrowing keeps. Anything wrong here comes from the index-level
 * subtraction of invoked from mutated, not from which properties were walked.
 *
 * @param row - Row the callee writes.
 *
 * @example
 * ```ts
 * invokedExclusionDirectEffect({ label: '' });
 * ```
 */
export function invokedExclusionDirectEffect(row: LabelledRow,): void {
  invokeAndMutate({
    run(): void {},
    target: row,
  },);
}

/**
 * Packages a parameter behind an object-literal getter.
 *
 * The argument walk reads literal properties and spreads. A getter's value comes from
 * running its body, so a parameter returned by one reaches the callee without ever
 * appearing as a property value the walk can see.
 *
 * @param row - Row the callee writes, reachable only through an accessor body.
 *
 * @example
 * ```ts
 * accessorPackagedEffect({ label: '' });
 * ```
 */
export function accessorPackagedEffect(row: LabelledRow,): void {
  mutateBeyondContractRow({
    named: { label: '', },
    get unnamed(): LabelledRow {
      return row;
    },
  },);
}

/**
 * Packages a parameter through a spread of a local object.
 *
 * Companion shape: the value reaches the callee through a spread rather than a named
 * property, which the walk is documented to follow.
 *
 * @param row - Row the callee writes, reached through a spread.
 *
 * @example
 * ```ts
 * spreadPackagedEffect({ label: '' });
 * ```
 */
export function spreadPackagedEffect(row: LabelledRow,): void {
  mutateBeyondContractRow({
    named: { label: '', },
    ...{ unnamed: row, },
  },);
}

/**
 * Packages a parameter behind accessors nested one literal deeper.
 *
 * Neighbour of `accessorPackagedEffect` checking that the accessor handling is reached
 * wherever a literal is walked rather than only at the argument's top level, and that a
 * setter writing straight through to a parameter counts as much as a getter reading from
 * one.
 *
 * @param row - Row both accessors reach.
 *
 * @example
 * ```ts
 * nestedAccessorPackagedEffect({ label: '' });
 * ```
 */
export function nestedAccessorPackagedEffect(row: LabelledRow,): void {
  mutateBeyondContractRow({
    named: { label: '', },
    unnamed: {
      get label(): string {
        return row.label;
      },
      set label(next: string,) {
        row.label = next;
      },
    },
  },);
}

/**
 * Calls a supplied method and writes through what it returns.
 *
 * Callee half of the method-return probe. Nothing in the caller writes, so the caller's
 * own direct-write scan cannot record anything: the origin has to travel through the
 * argument walk for this write to be attributed at all.
 *
 * @param get - Method the body calls for a row it then writes.
 *
 * @mutates get - Overwrites a label on whatever row is returned.
 *
 * @example
 * ```ts
 * callThroughMethodResult({ get: () => ({ label: '' }) });
 * ```
 */
function callThroughMethodResult({ get, }: { get: () => LabelledRow; },): void {
  get()
    .label = 'written-through-result';
}

/**
 * Packages a parameter behind an object-literal method the callee calls.
 *
 * The shape `passedContainerClosureSemanticEffect` does not cover. There the mutation is
 * written in the caller's own scope, so the caller's direct-write scan finds it whatever
 * the argument walk does. Here the write is in the callee, through the value the method
 * returns, which is the same position the accessor defect occupied.
 *
 * @param row - Row the callee writes after calling the supplied method.
 *
 * @example
 * ```ts
 * methodReturnPackagedEffect({ label: '' });
 * ```
 */
export function methodReturnPackagedEffect(row: LabelledRow,): void {
  callThroughMethodResult({
    get(): LabelledRow {
      return row;
    },
  },);
}

/**
 * Packages a parameter behind an arrow function held in an ordinary property.
 *
 * Third form of the same shape. Here the property does have a value the walk reads, but
 * that value is a callable whose body is what reaches the parameter, so reading the
 * property value alone still finds no origin.
 *
 * @param row - Row the callee writes after calling the supplied function.
 *
 * @example
 * ```ts
 * arrowReturnPackagedEffect({ label: '' });
 * ```
 */
export function arrowReturnPackagedEffect(row: LabelledRow,): void {
  callThroughMethodResult({
    get: (): LabelledRow => {
      return row;
    },
  },);
}

/**
 * Holder returned by the shorthand and explicit packaging pair.
 */
export type PackagedRow = {
  /**
   * Row the holder was built around.
   */
  readonly held: LabelledRow;
};

/**
 * Packages a parameter into a returned literal written in shorthand form.
 *
 * The shorthand name resolves to the property rather than to the local it reads, so the
 * provenance walk asked the checker for the wrong symbol and recorded no returned origin.
 * Its sibling `packageRowExplicit` writes the identical literal in longhand and recorded
 * one, which is how the two are kept together: they must agree.
 *
 * @param held - Row the returned holder carries.
 *
 * @returns holder carrying the row it was given.
 *
 * @example
 * ```ts
 * packageRowShorthand({ label: '' });
 * ```
 */
export function packageRowShorthand(held: LabelledRow,): PackagedRow {
  return { held, };
}

/**
 * Packages a parameter into a returned literal written in longhand form.
 *
 * @param held - Row the returned holder carries.
 *
 * @returns holder carrying the row it was given.
 *
 * @example
 * ```ts
 * packageRowExplicit({ label: '' });
 * ```
 */
export function packageRowExplicit(held: LabelledRow,): PackagedRow {
  return { held: held, };
}

/**
 * Writes through a holder packaged in shorthand form.
 *
 * The write lands on the caller's own row, reached through the returned holder. Without the
 * shorthand value symbol the callee returned no origin, this write was attributed to
 * nothing, and the parameter it mutates kept its read-only offer while
 * `explicitPackagedWriteEffect` reported the identical write.
 *
 * @param row - Row this writes through the returned holder.
 *
 * @mutates row - Writes the label through a returned shorthand holder.
 *
 * @example
 * ```ts
 * shorthandPackagedWriteEffect({ label: '' });
 * ```
 */
export function shorthandPackagedWriteEffect(row: LabelledRow,): void {
  /**
   * Holder carrying the caller's row.
   */
  const packaged = packageRowShorthand(row,);
  packaged.held
    .label = 'written';
}

/**
 * Writes through a holder packaged in longhand form.
 *
 * The control for `shorthandPackagedWriteEffect`, identical in every respect except how the
 * callee wrote its literal.
 *
 * @param row - Row this writes through the returned holder.
 *
 * @mutates row - Writes the label through a returned longhand holder.
 *
 * @example
 * ```ts
 * explicitPackagedWriteEffect({ label: '' });
 * ```
 */
export function explicitPackagedWriteEffect(row: LabelledRow,): void {
  /**
   * Holder carrying the caller's row.
   */
  const packaged = packageRowExplicit(row,);
  packaged.held
    .label = 'written';
}

/**
 * Row whose every reachable property is readonly, so it draws no offer of its own.
 */
export type SealedRow = {
  /**
   * Label this fixture reads and never writes.
   */
  readonly label: string;
};

/**
 * Packages a primitive read into a freshly allocated object.
 *
 * The returned object holds one string and shares no identity with the parameter, so a
 * caller can reach nothing through it. The walk reached the parameter anyway, because
 * `expressionRoot` strips the property access back to the receiver, which answers what was
 * read rather than what can be reached. That made this indistinguishable from
 * `packageRowShorthand`, which does hand the row back, and the whole result-provenance
 * decision rests on telling those two apart.
 *
 * Its parameter is deeply readonly so the case pins the returned origin alone and adds no
 * diagnostic to the sibling count.
 *
 * @param row - Row this reads one primitive from.
 *
 * @returns freshly allocated holder carrying no caller identity.
 *
 * @example
 * ```ts
 * packageCountFresh({ label: '' });
 * ```
 */
export function packageCountFresh(row: SealedRow,): { readonly named: string; } {
  return { named: row.label, };
}

/**
 * Writes an element of a fresh container built from the parameter.
 *
 * `slice` hands back a new array holding the receiver's own rows, so this write lands on
 * the caller's row. Nothing attributes it today, and nothing has to: `slice` is
 * undischarged, so the parameter is opaque and no offer is made. The element facet is what
 * turns the opacity into an attribution, and until it exists this case records which of the
 * two is carrying the parameter.
 *
 * @param rows - Rows whose element this writes through a copy.
 *
 * @mutates rows - Writes a row reached through a fresh container.
 *
 * @example
 * ```ts
 * containerElementWriteEffect([{ label: '' }]);
 * ```
 */
export function containerElementWriteEffect(rows: LabelledRow[],): void {
  /**
   * Fresh array holding the caller's own rows.
   */
  const copy = rows.slice();
  /**
   * First row of the copy, which is the caller's row.
   */
  const first = copy[0];
  if (first !== undefined)
    first.label = 'written';
}

/**
 * Grows a fresh container built from the parameter.
 *
 * The opposite answer about the same value, and the reason the facet cannot be one set: the
 * push reaches the copy and nothing the caller shared, so `rows` must never be recorded as
 * mutated here however the element write above is attributed.
 *
 * @param rows - Rows this copies and does not write.
 *
 * @param fresh - Row appended to the copy.
 *
 * @example
 * ```ts
 * containerGrowthEffect([{ label: '' }], { label: '' });
 * ```
 */
export function containerGrowthEffect(
  rows: LabelledRow[],
  fresh: LabelledRow,
): void {
  /**
   * Fresh array holding the caller's own rows.
   */
  const copy = rows.slice();
  copy.push(fresh,);
}

/**
 * Writes an element of a fresh container bound by an array pattern.
 *
 * The second element-step spelling, and the first that writes no element access. A pattern
 * binds elements, so it asks the element question; an object pattern beside it keeps asking
 * the value question, because a container's properties are its own rather than its elements.
 *
 * @param rows - Rows whose element this writes through a destructured copy.
 *
 * @mutates rows - Writes a row bound out of a fresh container by pattern.
 *
 * @example
 * ```ts
 * destructuredContainerWriteEffect([{ label: '' }]);
 * ```
 */
export function destructuredContainerWriteEffect(rows: LabelledRow[],): void {
  /**
   * Fresh array holding the caller's own rows.
   */
  const copy = rows.slice();
  /**
   * First row, bound by a pattern rather than by an element access.
   */
  const [first,] = copy;
  if (first !== undefined)
    first.label = 'written';
}

/**
 * Writes every element of a fresh container reached by iteration.
 *
 * The third spelling. Iteration advances an iterator through no element access and no call
 * this walk can inspect, which is why the element question is asked of the iterated
 * expression rather than derived from its syntax.
 *
 * @param rows - Rows this writes through an iterated copy.
 *
 * @mutates rows - Writes rows reached by iterating a fresh container.
 *
 * @example
 * ```ts
 * iteratedContainerWriteEffect([{ label: '' }]);
 * ```
 */
export function iteratedContainerWriteEffect(rows: LabelledRow[],): void {
  /**
   * Fresh array holding the caller's own rows.
   */
  const copy = rows.slice();
  for (const row of copy) {
    row.label = 'written';
  }
}

/**
 * Writes an element of a container built by spreading another container.
 *
 * The fourth spelling, and the one that stacks two element steps: the spread carries the
 * receiver's rows into a new array, and the access takes one back out.
 *
 * @param rows - Rows this writes through a spread copy.
 *
 * @mutates rows - Writes a row reached through a spread of a fresh container.
 *
 * @example
 * ```ts
 * spreadContainerWriteEffect([{ label: '' }]);
 * ```
 */
export function spreadContainerWriteEffect(rows: LabelledRow[],): void {
  /**
   * Fresh array built by spreading another fresh array.
   */
  const copy = [...rows.slice(),];
  /**
   * First row of the spread copy, which is the caller's row.
   */
  const first = copy[0];
  if (first !== undefined)
    first.label = 'written';
}

/**
 * Writes an element of a filtered container built from the parameter.
 *
 * The same shape as `containerElementWriteEffect` through the other member carrying the
 * container relation, so neither member's behaviour rests on the other's case.
 *
 * @param rows - Rows whose element this writes through a filtered copy.
 *
 * @mutates rows - Writes a row reached through a filtered container.
 *
 * @example
 * ```ts
 * filteredElementWriteEffect([{ label: '' }]);
 * ```
 */
export function filteredElementWriteEffect(rows: LabelledRow[],): void {
  /**
   * Filtered array holding the caller's own rows.
   */
  const kept = rows.filter(function keepsEvery(): boolean {
    return true;
  },);
  /**
   * First row of the filtered copy, which is the caller's row.
   */
  const first = kept[0];
  if (first !== undefined)
    first.label = 'written';
}

/**
 * Hands back the receiver's elements from whichever branch supplies them.
 *
 * The container relation sits inside a selector rather than at the returned expression's
 * root. Value provenance already saw through `?:`, so the bare `return cond ? rows : [];`
 * recorded its origin, while this recorded nothing: the element walk asked the container
 * question only where the selector stood. Ten further spellings behaved the same way,
 * `??`, `||`, `&&`, parentheses, `as`, a non-null assertion, `satisfies` and the comma
 * operator among them, which is why the fix shares one definition of the family with the
 * value walk instead of naming the conditional.
 *
 * @param rows - Rows whose elements the selected result carries.
 *
 * @returns fresh container of the caller's own rows, or an empty one.
 *
 * @example
 * ```ts
 * returnsSelectedReceiverElements([],);
 * ```
 */
export function returnsSelectedReceiverElements(
  rows: readonly Labelled[],
): readonly Labelled[] {
  return rows.length > 0
    ? rows.slice(0,)
    : [];
}

/**
 * Writes the caller's row through a container reached past a selector.
 *
 * The half that makes the selection matter, and the reason it is not merely precision. With
 * the returned origin missing, this callable had nothing to substitute, so the write landed
 * on no parameter at all and `rows` became offerable while this rewrites a row it holds.
 * Its composed sibling is the control: both record `referentMutated=[0]` in
 * `effect-summaries.unit.test.ts`, and a walk that stops at the selector separates them.
 *
 * @param rows - Rows whose element is rewritten through a selected container.
 *
 * @example
 * ```ts
 * writesThroughSelectedContainer([],);
 * ```
 */
export function writesThroughSelectedContainer(rows: readonly Labelled[],): void {
  /**
   * Container returned past a selector, holding the caller's rows.
   */
  const carried = returnsSelectedReceiverElements(rows,);
  /**
   * Row reached through the selected container.
   */
  const first = carried[0];
  if (first === undefined)
    throw new Error('Expected a selected row to rewrite.',);
  first.label = 'rewritten';
}

/**
 * Writes an element of a container chosen by a conditional.
 *
 * The composition case, and the one that measured a wrong offer rather than a lost report.
 * Element origins are asked about `copy`, a name; the container resolver follows a name to
 * its declaration but answers only at a call, so a selector there stopped it; and the
 * selection family is empty for a name, so the walk never reached the initializer. Neither
 * half owned the composition, the write landed on no parameter, and `rows` was offered
 * read-only while this rewrites a row it holds.
 *
 * @param rows - Rows this writes through a conditionally chosen copy.
 *
 * @mutates rows - Writes a row reached through a container one branch supplies.
 *
 * @example
 * ```ts
 * selectedContainerWriteEffect([{ label: '' }]);
 * ```
 */
export function selectedContainerWriteEffect(rows: LabelledRow[],): void {
  /**
   * Fresh array holding the caller's own rows, supplied by whichever branch runs.
   */
  const copy = rows.length > 0
    ? rows.slice()
    : [];
  for (const row of copy) {
    row.label = 'written';
  }
}

/**
 * Writes an element of a container behind a wrapper that erases at runtime.
 *
 * The same composition reached by different syntax, kept because one holding is no evidence
 * for the other. `satisfies` rather than an assertion, because `rows.slice() as LabelledRow[]`
 * asserts the type the expression already has and invites removal as redundant.
 *
 * @param rows - Rows this writes through a wrapped copy.
 *
 * @mutates rows - Writes a row reached through a container behind an erasing wrapper.
 *
 * @example
 * ```ts
 * wrappedContainerWriteEffect([{ label: '' }]);
 * ```
 */
export function wrappedContainerWriteEffect(rows: LabelledRow[],): void {
  /**
   * Fresh array holding the caller's own rows, reached through a wrapper that erases.
   */
  const copy = rows.slice() satisfies LabelledRow[];
  for (const row of copy) {
    row.label = 'written';
  }
}

/**
 * Writes an element reached through two declarations and two selectors.
 *
 * The case proving the hop and the selection compose repeatedly rather than once. A single
 * pass of each answers the two cases beside this one and still answers nothing here, because
 * the second name's initializer is a selector whose operand is another name. Both steps
 * belong to one walk over one visited set, which is what this measures.
 *
 * @param rows - Rows this writes through two chained locals.
 *
 * @mutates rows - Writes a row reached through a container behind two declarations.
 *
 * @example
 * ```ts
 * nestedSelectorWriteEffect([{ label: '' }]);
 * ```
 */
export function nestedSelectorWriteEffect(rows: LabelledRow[],): void {
  /**
   * Container this callable built, or nothing when there was nothing to copy.
   */
  const maybe = rows.length > 0
    ? rows.slice()
    : undefined;
  /**
   * Container reached past a second selector over a name.
   */
  const copy = maybe ?? [];
  for (const row of copy) {
    row.label = 'written';
  }
}

/**
 * Hands an input already readonly at every level to a call this rule cannot resolve.
 *
 * The remaining shape of issue #414. The finding is true, since `readonly` is erased at compile
 * time and the callee receives the underlying array, but the general message closes by offering
 * to make the type sound and it already is. This one gets its own wording instead.
 *
 * @param handedNames - Strings handed to an unresolvable call.
 */
export function handsReadonlyNamesOnward(handedNames: readonly string[],): void {
  void JSON.parse(JSON.stringify(handedNames,),);
}

/**
 * Hands a mutable array into the readonly-typed parameter beside it.
 *
 * The control that keeps the new message from spreading. This parameter is not readonly, the
 * charge propagates here through `handsReadonlyNamesOnward`, and the type change the general
 * message advises does apply, so it must keep the general message.
 *
 * @param handedNames - Strings this callable owns and hands onward.
 */
export function handsMutableNamesOnward(handedNames: string[],): void {
  handsReadonlyNamesOnward(handedNames,);
}
