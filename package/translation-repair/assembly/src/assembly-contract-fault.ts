import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import { UnhandledMemberInvariantError, } from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Assembly contract faults
// What an assembly's change sets or returned document contradict, as a union
// the class words itself. Its own file because `assembly-invariant.ts` holds
// the three checks and would cross the file-length limit with this beside them.

/**
 Which change set is meant.

 @example
 ```ts
 const set: ChangeSetName = 'shipped';
 ```
 */
export type ChangeSetName = 'shipped' | 'withdrawn';

/**
 What an assembly's sets or document contradict.

 @example
 ```ts
 const fault: AssemblyContractFault = { kind: 'index-beyond-count', index: 9, sliceCount: 4, };
 ```
 */
export type AssemblyContractFault = {
  /**
   A replacement names a slice the preparation never produced.
   */
  readonly kind: 'replacement-unproduced';

  /**
   Slice named.
   */
  readonly sliceIndex: number;
} | {
  /**
   A replacement claims a change and carries the archive wording.
   */
  readonly kind: 'replacement-unchanged';

  /**
   Slice named.
   */
  readonly sliceIndex: number;
} | {
  /**
   A change set holds something other than an integer.
   */
  readonly kind: 'index-not-integer';

  /**
   Value held.
   */
  readonly index: number;
} | {
  /**
   A change set names a slice before the first.
   */
  readonly kind: 'index-negative';

  /**
   Index named.
   */
  readonly index: number;
} | {
  /**
   A change set names a slice twice.
   */
  readonly kind: 'set-repeats';

  /**
   Set that repeats.
   */
  readonly set: ChangeSetName;
} | {
  /**
   Both sets name the same slices.
   */
  readonly kind: 'both-shipped-and-withdrawn';

  /**
   Slices named by both.
   */
  readonly indices: readonly number[];
} | {
  /**
   A change set names a slice beyond the prepared count.
   */
  readonly kind: 'index-beyond-count';

  /**
   Index named.
   */
  readonly index: number;

  /**
   Slices prepared.
   */
  readonly sliceCount: number;
} | {
  /**
   Returned document is not what its surviving replacements assemble to.
   */
  readonly kind: 'reassembly-differs';

  /**
   Replacements that survived.
   */
  readonly survivors: number;
} | {
  /**
   Returned document equals the archive while slices are named as changed.
   */
  readonly kind: 'unchanged-with-claims';

  /**
   Slices named as changed.
   */
  readonly indices: readonly number[];
};

/**
 Words an assembly contract fault from set names, kinds and numbers.

 @param fault - what the sets or document contradict

 @returns Sentence written here

 @throws {@link UnhandledMemberInvariantError} when a member of `AssemblyContractFault` has no branch here, which the compiler rules out

 @example
 ```ts
 const sentence = assemblySentence({ fault: { kind: 'reassembly-differs', survivors: 0, }, },);
 ```
 */
export function assemblySentence({ fault, }: { readonly fault: AssemblyContractFault; },): string {
  if (fault.kind === 'replacement-unproduced')
    return `replacement names slice ${String(fault.sliceIndex,)}, which this preparation never produced`;
  if (fault.kind === 'replacement-unchanged')
    return `slice ${String(fault.sliceIndex,)} claims a change and carries the archive wording`;
  if (fault.kind === 'index-not-integer')
    return `change set holds ${String(fault.index,)}, which is not a slice index`;
  if (fault.kind === 'index-negative')
    return `change set names slice ${String(fault.index,)}, and no slice is before the first`;
  if (fault.kind === 'set-repeats')
    return `${fault.set} slices repeat`;
  if (fault.kind === 'both-shipped-and-withdrawn') {
    /**
     Slices named by both sets.
     */
    const { indices, } = fault;
    return `slices ${indices.join(', ',)} are named as both shipped and withdrawn`;
  }
  if (fault.kind === 'index-beyond-count')
    return `change set names slice ${String(fault.index,)} of ${String(fault.sliceCount,)} prepared`;
  if (fault.kind === 'reassembly-differs') {
    return `returned document is not what its ${String(fault.survivors,)} surviving ${
      wordForCount({
        count: fault.survivors,
        one: 'replacement',
        many: 'replacements',
      },)
    } ${
      wordForCount({
        count: fault.survivors,
        one: 'assembles',
        many: 'assemble',
      },)
    } to`;
  }

  if (fault.kind === 'unchanged-with-claims') {
    /**
     Slices named as changed.
     */
    const { indices, } = fault;
    return `returned document equals the archive while slices ${
      indices.join(', ',)
    } are named as changed: a set that reassembles to the archive is a net-zero assembly, which `
      + '`guardFootnoteAssembly` canonicalizes to no survivors, so pass what the guard let stand rather than '
      + 'what a lane proposed';
  }
  throw new UnhandledMemberInvariantError({
    what: 'assembly contract fault',
    member: fault,
  },);
}

//endregion Assembly contract faults
