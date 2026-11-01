/**
 Exact semantic classification for default-library collection members.
 
 @module
 */

import type { Node, } from 'typescript/ast';
import {
  isIdentifier,
  isInterfaceDeclaration,
  isMethodSignatureDeclaration,
} from 'typescript/ast/is';
import type { Project, } from 'typescript/sync';

import {
  UNPAIRED_VIEW_UNKNOWN,
  unpairedViewMembers,
} from './effect-unpaired-view-authority.ts';
import {
  defaultLibraryViewMembers,
  READONLY_VIEW_INTERFACE_PREFIX,
} from './effect-default-library-view-members.ts';
import {
  collectionMemberUserCodeChannel,
  MEMBER_CHANNEL_INTERNAL_SLOT_AND_ARGUMENTS,
  MEMBER_CHANNEL_RECEIVER_INDEX_AND_COERCION,
  MEMBER_CHANNEL_UNPROVEN,
  memberInvokesObserver,
} from './effect-member-channel-authority.ts';

/**
 Member leaves the receiver's own structure intact.
 */
export const COLLECTION_STRUCTURE_PRESERVED: unique symbol = Symbol(
  'default-library member preserves receiver structure',
);

/**
 Member restructures the receiver.
 */
export const COLLECTION_STRUCTURE_MUTATED: unique symbol = Symbol(
  'default-library member restructures receiver',
);

/**
 Member belongs to no collection whose structural effect is derivable.
 */
export const COLLECTION_UNRECOGNIZED: unique symbol = Symbol(
  'declaration is not a derivable default-library collection member',
);

/**
 How a default-library collection member treats its receiver's structure.
 */
export type CollectionStructureClaim =
  | typeof COLLECTION_STRUCTURE_PRESERVED
    | typeof COLLECTION_STRUCTURE_MUTATED
    | typeof COLLECTION_UNRECOGNIZED;

/**
 Classify a resolved declaration's effect on its receiver's structure.
 
 Two cases are derivable, both from upstream's own declarations. A member of a
 `Readonly*` view preserves structure by definition, since the view exists to
 name the operations available to a holder that may not mutate. A member of a
 mutable collection preserves structure when the paired view declares the same
 name, and restructures the receiver when it does not: TypeScript builds each
 view by removing exactly the mutators, so the difference between the two
 interfaces is upstream's own mutator list.
 
 Measured against TypeScript 7.0.2, that difference is `add`, `clear` and
 `delete` for `Set`; `clear`, `delete`, `getOrInsert`, `getOrInsertComputed`
 and `set` for `Map`; and `copyWithin`, `fill`, `pop`, `push`, `reverse`,
 `shift`, `sort`, `splice` and `unshift` for `Array`. No view declares a member
 its mutable interface lacks, so the partition is exact in both directions.
 
 A collection with no paired view, `WeakMap`, `WeakSet`, a typed array, or any
 host interface, is unrecognized and keeps failing closed.
 
 This answers only what happens to the receiver's structure. What user code the
 member can run remains a separate obligation for the caller.
 
 @param project - TypeScript project proving default-library ownership.
 
 @param declaration - Selected callable declaration.
 
 @returns structural claim derivable for declaration.
 
 @example
 ```typescript
 collectionStructureClaim({ project, declaration }) === COLLECTION_STRUCTURE_MUTATED;
 ```
 */
export function collectionStructureClaim({
  project,
  declaration,
}: {
  readonly project: Project;
  readonly declaration: Node;
}): CollectionStructureClaim {
  if ((!isMethodSignatureDeclaration(declaration,))
    || (!isIdentifier(declaration.name,))
    || (!project
      .program
      .isSourceFileDefaultLibrary(declaration.getSourceFile(),)))
    return COLLECTION_UNRECOGNIZED;
  /**
   Default-library interface selected as method owner.
   */
  const owner = declaration.parent;
  if ((!isInterfaceDeclaration(owner,)) || (!isIdentifier(owner.name,)))
    return COLLECTION_UNRECOGNIZED;
  /**
   Owner interface name deciding which claim is derivable.
   */
  const ownerName = owner
    .name
    .text;
  // This accepts any default-library interface whose name starts with `Readonly`,
  // which is broader than "one of the paired collection views" and is an assumption
  // rather than a derivation. Measured at TypeScript 7.0.2, the default library
  // declares exactly four: `ReadonlyArray`, `ReadonlyMap`, `ReadonlySet` and
  // `ReadonlySetLike`. The first three are the paired views; `ReadonlySetLike`
  // declares only `has`, `keys` and `size`, so it preserves structure too and the
  // breadth costs nothing today.
  //
  // What would break it is a future library adding a `Readonly`-prefixed interface
  // with a member that restructures its receiver. Nothing here would notice. The
  // narrower test is to require the name minus the prefix to name a mutable
  // default-library interface, which `ReadonlySetLike` would fail, so tightening
  // means deciding what happens to a view with no mutable counterpart.
  if (ownerName.startsWith(READONLY_VIEW_INTERFACE_PREFIX,))
    return COLLECTION_STRUCTURE_PRESERVED;
  /**
   Member names on the read-only view paired with this owner, when one exists.
   */
  const pairedViewMembers = defaultLibraryViewMembers({ project, },)
    .get(`${READONLY_VIEW_INTERFACE_PREFIX}${ownerName}`,);
  /**
   Member name deciding whether the read-only view retains this operation.
   */
  const memberName = declaration.name
    .text;
  if (pairedViewMembers !== undefined)
    return pairedViewMembers.has(memberName,)
      ? COLLECTION_STRUCTURE_PRESERVED
      : COLLECTION_STRUCTURE_MUTATED;
  /* The library pairs most of what this rule cares about and leaves some of it unpaired,
   * `DataView` being the measured case, so the derivation above has nothing to diff and
   * answered "unrecognized" for every buffer write. The authority declares the membership
   * the library omits, which lets the identical diff run rather than introducing a second
   * rule that could disagree with it. */
  /**
   Declared read-only membership for an interface the library never paired.
   */
  const declaredViewMembers = unpairedViewMembers({ ownerName, },);
  if (declaredViewMembers === UNPAIRED_VIEW_UNKNOWN)
    return COLLECTION_UNRECOGNIZED;
  return declaredViewMembers.has(memberName,)
    ? COLLECTION_STRUCTURE_PRESERVED
    : COLLECTION_STRUCTURE_MUTATED;
}

/**
 Tests whether a member's user-code channel is no wider than a property read.
 
 Consults the one permitted authority, whose entries are enforced against a real
 engine by `effect-member-channel-authority.unit.test.ts`. A verified channel
 discharges the reachable-user-code claim about the receiver, and nothing else:
 the structural claim keeps deriving from the paired read-only view, so a verified
 mutator such as `Set.add` still reports its mutation.
 
 Both verified channels answer the same way here. An internal-slot member reaches
 no user code at all, and an own-index member reaches only what `values[0]`
 already reaches, which this rule treats as a pure read everywhere else. Keeping
 them distinct in the authority is about what each probe must prove, not about
 what either discharges.
 
 A member that invokes a caller-supplied observer is refused here whatever its
 ambient channel is, and that refusal is the invariant rather than an accident of
 which members the table currently lists. The ambient half and the observer half
 are separate obligations: `filter` reaches own-index access and default species,
 both trusted under the stated baseline, and it also runs whatever predicate the
 caller passed. Only `recordReadonlyViewApplications` can answer the second half,
 by resolving that predicate to owned source. Discharging on the first half alone
 would accept `rows.filter(foreignMutatingPredicate)`, whose predicate received
 every element of the receiver.
 
 @param project - TypeScript project proving default-library ownership.
 
 @param declaration - Selected callable declaration.
 
 @returns whether the member opens a verified narrow channel.
 
 @example
 ```typescript
 memberChannelIsVerifiedNarrow({ project, declaration });
 ```
 */
export function memberChannelIsVerifiedNarrow({
  project,
  declaration,
  elementsArePrimitive,
  argumentsAreAbsent,
}: {
  readonly project: Project;
  readonly declaration: Node;
  readonly elementsArePrimitive: boolean;
  readonly argumentsAreAbsent: boolean;
}): boolean {
  if ((!isMethodSignatureDeclaration(declaration,))
    || (!isIdentifier(declaration.name,))
    || (!project
      .program
      .isSourceFileDefaultLibrary(declaration.getSourceFile(),)))
    return false;
  /**
   Default-library interface selected as method owner.
   */
  const owner = declaration.parent;
  if ((!isInterfaceDeclaration(owner,)) || (!isIdentifier(owner.name,)))
    return false;
  /**
   Member name deciding both the ambient channel and the observer obligation.
   */
  const memberName = declaration.name
    .text;
  if (memberInvokesObserver({ memberName, },))
    return false;
  /**
   Channel this member opens on its receiver.
   */
  const channel = collectionMemberUserCodeChannel({
    ownerName: owner.name
      .text,
    memberName,
  },);
  /* One channel is narrow conditionally: a member coercing what it read reaches user code
   * exactly when an element is not primitive. `parts.join(' ')` over strings runs nothing, and
   * the same call over `{ readonly label: string }` reaches that value's own `toString`, which
   * no shape in the type system constrains. */
  if (channel === MEMBER_CHANNEL_RECEIVER_INDEX_AND_COERCION)
    return elementsArePrimitive;
  /* The other conditional channel, decided at the call site for the same kind of reason.
   * A date locale member reads properties off whatever options object it is handed, so it
   * runs any accessor on one; handed nothing, there is no object and no accessor to run.
   * `pubDateDate.toLocaleString()` passes nothing, which is the shape that made this worth
   * separating from an outright refusal. */
  if (channel === MEMBER_CHANNEL_INTERNAL_SLOT_AND_ARGUMENTS)
    return argumentsAreAbsent;
  return channel !== MEMBER_CHANNEL_UNPROVEN;
}
