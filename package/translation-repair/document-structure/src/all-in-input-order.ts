//region All in input order
// Waits for members that already run side by side and reports what happened
// to them as a property of the INPUT, never of which member finished first.
//
// WHY THIS EXISTS. `Promise.all` rejects with whichever member rejects first
// BY COMPLETION. Where several members fail in one call (two page reads of a
// commit the clone lacks, two logs that do not exist, two providers that
// refuse), each failure names its own member, so the message an operator, a
// log line or a case reads said whichever git child, file read or request
// happened to end first. Under load that is not the member the input lists
// first, and one input printed two different refusals on two runs.
//
// WHAT THIS KEEPS. The work still runs side by side: the members are started
// by the caller before this is handed them. Where every member succeeds the
// result is in input order and nothing waited longer than `Promise.all` would
// have. Where members fail, the failure thrown is the one of the first member
// in input order, the very value that member rejected with, so its class, its
// message and its cause are exactly what that member reports on its own.
//
// WHAT THIS COSTS. A failure at one position cannot be reported until every
// earlier position has settled, since an earlier member may still fail and
// then it, not the later one, is what the input says. A later member that
// never settles after an earlier one failed costs nothing: the earlier
// failure is thrown without waiting for it. An earlier member that never
// settles holds the report for as long as it holds the whole call under
// `Promise.all`.
//
// WHAT THIS DOES NOT DO. It cancels nothing and gives no abort precedence: a
// site whose failure must stop its siblings, or where a caller's abort has to
// win over any other failure, keeps its own abort handling around this call,
// and an abort a member rejected with is reported like any other failure,
// as the value that member rejected with.

/**
 What one member came to, kept apart from the others so a rejection is held
 as data until the reader reaches its position.
 */
type MemberOutcome =
  | { readonly kind: 'fulfilled'; }
    | {
      readonly kind: 'failure';
      readonly error: unknown;
    };

/**
 Awaits members that already run side by side, handing back every value in
 input order, or throwing the failure of the first failing member in input
 order.

 @param members - promises already started, listed in the order a failure
 must be reported in

 @returns Every member's value, at the member's own position, typed
 position by position as `Promise.all` types it

 @throws Whatever the first failing member, in input order, rejected with,
 unchanged, once every member listed before it has fulfilled

 @example
 ```ts
 const [sourceText, targetText,] = await allInInputOrder({
   members: [
     readFile({ pin, relPath: 'people/whiskers/page.md', },),
     readFile({ pin, relPath: 'people/whiskers/page.en.md', },),
   ],
 },);
 ```
 */
export async function allInInputOrder<const Members extends readonly Promise<unknown>[],>(
  { members, }: { readonly members: Members; },
): Promise<{ -readonly [Position in keyof Members]: Awaited<Members[Position]>; }> {
  /**
   One outcome per member, every member awaited at once here so that a
   member rejecting while an earlier one is still pending has its rejection
   handled, which an unhandled rejection would otherwise turn into the end
   of the process.
   */
  const outcomes = members.map(async function outcomeOf(member,): Promise<MemberOutcome> {
    try {
      await member;
      return { kind: 'fulfilled', };
    }
    catch (error) {
      return {
        kind: 'failure',
        error,
      };
    }
  },);

  for (const outcome of outcomes) {
    /* oxlint-disable no-await-in-loop -- reading the members in input order is the point: a later failure may be reported only once every earlier member has fulfilled */
    /**
     How the member at this position ended.
     */
    const settled = await outcome;
    /* oxlint-enable no-await-in-loop */
    if (settled.kind === 'failure')
      throw settled.error;
  }

  // EVERY MEMBER HAS FULFILLED HERE, so this resolves at once and cannot
  // reject; it is what gives the result the tuple type `Promise.all` gives
  // `members`, which a hand-built array could only be given by assertion.
  return await Promise.all(members,);
}

//endregion All in input order
