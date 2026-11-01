/**
 Retirement rule deciding which catalog entries a newer sibling supersedes.

 The rule abstains whenever id text cannot order two entries, because the settled
 decisions in `doc/planning/pi-model-retirement.md` removed every runtime override: a
 false positive is recoverable only by rebuilding the package, while a false negative
 costs one extra row in the picker.

 @module
 */

import {
  parseModelId,
  type ModelIdParse,
} from './id-tokens.ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  compareRecency,
  UNORDERED,
} from './retirement-order.ts';

//region Constants

/**
 Separator joining the identity parts of a catalog entry.

 A control character, so no provider, api, or model id can produce a collision.
 */
const IDENTITY_SEPARATOR = '\u0000';

//endregion Constants

//region Types

/**
 One catalog entry the rule can order.

 `api` is part of the identity because three ids in pi's bundled catalog are served
 under two APIs by the same provider, and an entry must never retire its own sibling
 route.
 */
export type CatalogEntry = {
  /**
   Provider id owning the entry.
   */
  readonly provider: string;
  /**
   API type the entry is served under.
   */
  readonly api: string;
  /**
   Model id exactly as the catalog carries it.
   */
  readonly modelId: string;
};

/**
 One retirement the rule decided.
 */
export type Retirement = {
  /**
   Provider id owning both entries.
   */
  readonly provider: string;
  /**
   API type both entries are served under.
   */
  readonly api: string;
  /**
   Id the picker stops offering.
   */
  readonly retiredId: string;
  /**
   Id that supersedes it.
   */
  readonly keeperId: string;
};

/**
 Why the rule declined to retire entries it saw.
 */
export type AbstentionCounts = {
  /**
   Pairs where keeper selection inside a family could not order two candidates.
   */
  readonly keeperAmbiguity: number;
  /**
   Pairs whose version evidence the ordering rules leave unordered.
   */
  readonly unorderedPair: number;
  /**
   Pairs where the candidate loser orders newer than the chosen keeper.
   */
  readonly loserNewerThanKeeper: number;
  /**
   Entries carrying no version token, which are never retired.
   */
  readonly versionlessProtected: number;
  /**
   Entries repeating an identity already present in the same family.
   */
  readonly duplicateIdentity: number;
};

/**
 Outcome of one rule pass over a catalog.
 */
export type RetirementDecision = {
  /**
   Retirements in catalog order.
   */
  readonly retirements: readonly Retirement[];
  /**
   Declined retirements grouped by reason.
   */
  readonly abstentions: AbstentionCounts;
};

/**
 One catalog entry paired with the age evidence parsed from its id.
 */
type FamilyMember = {
  /**
   Entry the catalog reported.
   */
  readonly entry: CatalogEntry;
  /**
   Age evidence parsed from that entry's id.
   */
  readonly parse: ModelIdParse;
};

/**
 Mutable abstention tally accumulated during one pass.
 */
type AbstentionTally = {
  keeperAmbiguity: number;
  unorderedPair: number;
  loserNewerThanKeeper: number;
  versionlessProtected: number;
  duplicateIdentity: number;
};

//endregion Types

//region Grouping

/**
 Build the family key that groups entries the rule may compare.

 @param entry - catalog entry under consideration

 @param parse - age evidence extracted from that entry's id

 @returns key unique to one provider, one API, and one name shape

 @example
 ```typescript
 familyKey({ entry, parse: parseModelId('glm-5.2') });
 ```
 */
function familyKey(
  {
    entry,
    parse,
  }: {
    readonly entry: CatalogEntry;
    readonly parse: ModelIdParse;
  },
): string {
  return [
    entry.provider,
    entry.api,
    parse.nameShape,
  ].join(IDENTITY_SEPARATOR,);
}

/**
 Build the identity key that stops one catalog row retiring its own duplicate.

 @param entry - catalog entry under consideration

 @returns key unique to one API and model id inside a provider family

 @example
 ```typescript
 identityKey({ provider: 'hyper', api: 'openai-completions', modelId: 'glm-5.3' });
 ```
 */
function identityKey(entry: CatalogEntry,): string {
  return [
    entry.api,
    entry.modelId,
  ].join(IDENTITY_SEPARATOR,);
}

/**
 Group a catalog into families.

 @param entries - catalog entries in registry order

 @returns families keyed by provider, API, and name shape

 @example
 ```typescript
 groupFamilies({ entries });
 ```
 */
function groupFamilies(
  {
    entries,
  }: {
    readonly entries: readonly CatalogEntry[];
  },
): Map<string, FamilyMember[]> {
  /**
   Families collected so far, in first-seen order.
   */
  const families = new Map<string, FamilyMember[]>();
  for (const entry of entries) {
    /**
     Age evidence for the current entry.
     */
    const parse = parseModelId(entry.modelId,);
    /**
     Family this entry belongs to.
     */
    const key = familyKey({
      entry,
      parse,
    },);
    /**
     Members collected for that family so far.
     */
    const members = families.get(key,) ?? [];
    members.push({
      entry,
      parse,
    },);
    families.set(
      key,
      members,
    );
  }
  return families;
}

//endregion Grouping

//region Keeper selection

/**
 Result of choosing one family's newest candidate.
 */
type KeeperSelection = {
  /**
   Member every compared candidate lost to, or tied with.
   */
  readonly keeper: FamilyMember;
  /**
   Candidate pairs the id text could not order during selection.
   */
  readonly ambiguousPairs: number;
};

/**
 Choose the newest candidate inside one family.

 Selection walks the candidates once in catalog order, because the prototype this
 rule was validated against counted an ambiguity per undecided step rather than per
 unordered pair, and the recorded measurements use that count.

 @param candidates - family members carrying version evidence, at least two of them

 @returns the keeper and how many steps selection could not order

 @example
 ```typescript
 selectKeeper({ candidates });
 ```
 */
function selectKeeper(
  {
    candidates,
  }: {
    readonly candidates: readonly FamilyMember[];
  },
): KeeperSelection {
  /**
   First candidate, which selection starts from, and every candidate after it.
   */
  const [firstCandidate, ...remainingCandidates] = candidates;
  /**
   Mutable selection state, held in an object so no function-root binding mutates.
   */
  const selection = {
    keeper: nonNullishOrThrow(firstCandidate,),
    ambiguousPairs: 0,
  };
  for (const candidate of remainingCandidates) {
    /**
     Recency comparison between the current keeper and this candidate.
     */
    const outcome = compareRecency({
      left: selection.keeper
        .parse,
      right: candidate.parse,
    },);
    if (outcome === UNORDERED) {
      selection.ambiguousPairs += 1;
      continue;
    }
    if (outcome === 'right')
      selection.keeper = candidate;
  }
  return {
    keeper: selection.keeper,
    ambiguousPairs: selection.ambiguousPairs,
  };
}

//endregion Keeper selection

//region Decision

/**
 Decide every retirement in one catalog.

 @param entries - catalog entries in the order the registry reports them

 @returns retirements plus the abstention tally explaining what the rule declined

 @example
 ```typescript
 decideRetirements({ entries: [{ provider: 'hyper', api: 'openai-completions', modelId: 'glm-5.2' }] });
 ```
 */
export function decideRetirements(
  {
    entries,
  }: {
    readonly entries: readonly CatalogEntry[];
  },
): RetirementDecision {
  /**
   Families grouped by provider, API, and name shape.
   */
  const families = groupFamilies({ entries, },);
  /**
   Abstention tally for this pass.
   */
  const tally: AbstentionTally = {
    keeperAmbiguity: 0,
    unorderedPair: 0,
    loserNewerThanKeeper: 0,
    versionlessProtected: 0,
    duplicateIdentity: 0,
  };
  /**
   Retirements in catalog order.
   */
  const retirements: Retirement[] = [];
  for (const members of families.values()) {
    /**
     Members carrying version evidence, the only ones eligible to keep or lose.
     */
    const candidates = members.filter(function hasVersionEvidence(member,) {
      return member.parse
        .versionParts
        .length
        > 0;
    },);
    if (candidates.length < 2)
      continue;
    /**
     Newest member of this family, and how many steps selection could not order.
     */
    const selection = selectKeeper({ candidates, },);
    tally.keeperAmbiguity += selection.ambiguousPairs;
    /**
     Keeper every other member of this family is compared against.
     */
    const {keeper} = selection;
    /**
     Identities already accounted for in this family.
     */
    const seen = new Set<string>([identityKey(keeper.entry,),],);
    for (const member of members) {
      /**
       Identity of the member under consideration.
       */
      const identity = identityKey(member.entry,);
      if (seen.has(identity,)) {
        if (member !== keeper)
          tally.duplicateIdentity += 1;
        continue;
      }
      seen.add(identity,);
      if (member.parse
        .versionParts
        .length
        === 0) {
        tally.versionlessProtected += 1;
        continue;
      }
      /**
       Recency comparison between the member and the family keeper.
       */
      const outcome = compareRecency({
        left: member.parse,
        right: keeper.parse,
      },);
      if (outcome === 'left') {
        tally.loserNewerThanKeeper += 1;
        continue;
      }
      if (outcome === UNORDERED) {
        tally.unorderedPair += 1;
        continue;
      }
      retirements.push({
        provider: member.entry
          .provider,
        api: member.entry
          .api,
        retiredId: member.entry
          .modelId,
        keeperId: keeper.entry
          .modelId,
      },);
    }
  }
  return {
    retirements,
    abstentions: { ...tally, },
  };
}

//endregion Decision
