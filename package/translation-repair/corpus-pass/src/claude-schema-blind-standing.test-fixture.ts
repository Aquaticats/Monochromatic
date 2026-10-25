import {
  identifierName,
  isTreeNode,
  unwrapped,
} from '@monochromatic-dev/translation-repair-source-scan/ts/source-scan.test-fixture.ts';

import type { BlindContext, } from './claude-schema-blind-positions.test-fixture.ts';
import {
  resolveExport,
  resolvePath,
} from './claude-schema-resolve.test-fixture.ts';
import { targetOf, } from './claude-schema-target.test-fixture.ts';
import {
  memberNameOf,
  moduleHeadOf,
  settledSpecifierOf,
} from './claude-schema-tree.test-fixture.ts';
import { keyOf, } from './claude-schema-usage.test-fixture.ts';

//region What a chain of names stands for
// The first half of the second total of the check behind the schema search: a chain of names is read from its first
// name, a name the file binds, a name the file does not bind, or a load read in place, then one member at a time, and
// each prefix of it stands for a builder, for a module or object that carries one, or for neither. Where the chain
// sits is read by `claude-schema-blind-chains.test-fixture.ts`.

/**
 The bindings and exports already followed when a walk starts.
 */
const NOTHING_FOLLOWED: ReadonlySet<string> = new Set<string>();

/**
 The member a chain holds for a member that is computed from anything but a string, which no member written as a name
 can be.
 */
export const COMPUTED_MEMBER = '\u0000computed';

/**
 A chain of names as a file writes it: the first name, and the members after it, each `COMPUTED_MEMBER` when it is
 computed from anything but a string.
 */
export type Chain = {
  /**
   The name the chain starts with: a name, or the name a load read in place is read through.
   */
  readonly head: string;

  /**
   The members in order.
   */
  readonly members: readonly string[];
};

/**
 What a chain of names stands for when it is no name at all: no head, no members.
 */
export const NO_CHAIN: Chain = {
  head: '',
  members: [],
};

/**
 What a prefix of a chain stands for: a builder or wrapper, a forwarder, a module or object that carries a builder, or
 none of them.
 */
type Standing = 'builder' | 'carrier' | 'forwarder' | 'other';

/**
 The chain of names an expression is written with, the members that cannot be read kept as `COMPUTED_MEMBER`.

 @param node - expression read, of any kind or none

 @returns Chain, `NO_CHAIN` for any expression that is no name, no load read in place and no member of either

 @example
 ```ts
 const chain = chainOf({ node: callee, },);
 ```
 */
export function chainOf({ node, }: { readonly node: unknown; },): Chain {
  /**
   The expression without the type marks and parentheses around it.
   */
  const { inner, } = unwrapped({ node, },);
  if (!isTreeNode(inner,))
    return NO_CHAIN;
  if (inner.type === 'Identifier') {
    return {
      head: identifierName({ node: inner, },),
      members: [],
    };
  }
  /**
   The literal path of the module the expression loads, empty when it loads none.
   */
  const specifier = settledSpecifierOf({ node: inner, },);
  if (specifier !== '') {
    return {
      head: moduleHeadOf({ specifier, },),
      members: [],
    };
  }
  if (inner.type !== 'MemberExpression')
    return NO_CHAIN;
  /**
   The chain of the object the member is read off.
   */
  const objectChain = chainOf({ node: inner.object, },);
  /**
   The member's own name, empty when it is computed from anything but a string.
   */
  const member = memberNameOf({ node: inner, },);
  return (objectChain === NO_CHAIN)
    ? NO_CHAIN
    : {
      head: objectChain.head,
      members: [
        ...objectChain.members,
        (member === '') ? COMPUTED_MEMBER : member,
      ],
    };
}

/**
 What a chain, cut after some of its members, stands for.

 @param context - what the file's names are read against

 @param chain - chain read

 @param count - how many members to keep

 @returns Whether it is a builder, a module or object that carries one, or neither, whether the name is free, and the
 forwarder when it stands for one: the key of the function (the module that declares it and its name), or the bare name
 when the name is free, which no key can equal

 @example
 ```ts
 const standing = standingAfter({ context, chain, count: 1, },);
 ```
 */
export function standingAfter(
  {
    context,
    chain,
    count,
  }: {
    readonly context: BlindContext;
    readonly chain: Chain;
    readonly count: number;
  },
): {
  readonly standing: Standing;
  readonly isFree: boolean;
  readonly held: string;
} {
  /**
   The members kept.
   */
  const kept = chain.members
    .slice(
    0,
    count,
    );
  /**
   What the kept part stands for.
   */
  const resolved = resolvePath({
    family: context.family,
    file: context.file,
    path: [
      chain.head,
      ...kept.filter(function isReadable(member,): boolean {
        return member !== COMPUTED_MEMBER;
      },),
    ],
    seen: NOTHING_FOLLOWED,
  },);
  if ((resolved.kind === 'function')
    && context.builders
      .keys
      .has(keyOf(resolved,),)) {
    return {
      standing: 'builder',
      isFree: false,
      held: '',
    };
  }
  if ((resolved.kind === 'free')
    && context.builders
      .names
      .has(resolved.name,)) {
    return {
      standing: 'builder',
      isFree: true,
      held: '',
    };
  }
  if ((resolved.kind === 'function')
    && context.forwarders
      .keys
      .has(keyOf(resolved,),)) {
    return {
      standing: 'forwarder',
      isFree: false,
      held: keyOf(resolved,),
    };
  }
  if ((resolved.kind === 'free')
    && context.forwarders
      .names
      .has(resolved.name,)) {
    return {
      standing: 'forwarder',
      isFree: true,
      held: resolved.name,
    };
  }
  /**
   Whether the thing is a module or an object that carries a builder.
   */
  const carries = ((resolved.kind === 'namespace')
    && context.bearing
      .has(resolved.file,))
    || ((resolved.kind === 'object')
      && context.carriers
        .has(keyOf(resolved,),));
  return {
    standing: carries ? 'carrier' : 'other',
    isFree: resolved.kind === 'free',
    held: '',
  };
}

/**
 Whether the first name of a chain is brought in as the default of a module that carries a builder and exports no
 default, so the search reads it as nothing and a member read off it as a member of nothing. A named import of a name
 the module exports as a constant is no such import, since the search records no export for a constant.

 @param context - what the file's names are read against

 @param head - first name of the chain

 @returns Whether the name is such an import

 @example
 ```ts
 const unreadable = isImportOfNothing({ context, head: 'contest', },);
 ```
 */
export function isImportOfNothing(
  {
    context,
    head,
  }: {
    readonly context: BlindContext;
    readonly head: string;
  },
): boolean {
  /**
   What the file binds the name to.
   */
  const binding = context.facts
    .bindings
    .get(head,);
  if ((binding?.kind !== 'import') || (binding.imported !== 'default'))
    return false;
  /**
   The module the import names, empty when it is outside the sources.
   */
  const target = targetOf({
    family: context.family,
    from: context.file,
    specifier: binding.specifier,
  },);
  return context.bearing
    .has(target,)
    && (resolveExport({
      family: context.family,
      file: target,
      name: binding.imported,
      seen: NOTHING_FOLLOWED,
    },)
      .kind
      === 'none');
}

/**
 Whether a name, read alone, could lead to a builder or to a module that carries one.

 @param context - what the file's names are read against

 @param name - first name of some chain

 @returns Whether it stands for either, is free and could begin the dotted name of one, or is an import of nothing

 @example
 ```ts
 const relevant = mayLeadToBuilder({ context, name: 'contest', },);
 ```
 */
export function mayLeadToBuilder(
  {
    context,
    name,
  }: {
    readonly context: BlindContext;
    readonly name: string;
  },
): boolean {
  /**
   What the name stands for alone.
   */
  const {
    standing,
    isFree,
  } = standingAfter({
    context,
    chain: {
      head: name,
      members: [],
    },
    count: 0,
  },);
  return (standing !== 'other')
    || isImportOfNothing({
      context,
      head: name,
    },)
    || (isFree && [...context.builders
      .names,].some(function startsWithName(known,): boolean {
        return known.startsWith(`${name}.`,);
      },));
}

//endregion What a chain of names stands for
