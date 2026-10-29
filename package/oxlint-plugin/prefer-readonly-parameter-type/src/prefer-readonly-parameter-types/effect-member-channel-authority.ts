/**
 The one permitted authority: which user-code channel each collection member opens.
 
 @module
 */

/**
 Member reaching no user code at all, reading and writing internal slots only.
 
 `Map.prototype.get` consults `[[MapData]]`, never a property of its receiver, so
 no accessor and no coercion hook can observe the call.
 
 What the probe establishes is narrower than that sentence, and the gap is worth
 stating. It installs an own `size` accessor and finds it untouched, which shows
 these members do not read `size`; it does not enumerate every property, because the
 receiver is a real `Map` or `Set` rather than a general trap, and a `Proxy` cannot
 stand in for one since these members reject a receiver without the internal slot.
 So the broad claim rests on the specification, and the probe guards against drift
 in the one channel it watches.
 */
export const MEMBER_CHANNEL_INTERNAL_SLOT: unique symbol = Symbol(
  'collection member reads internal slots and reaches no user code',
);

/**
 Member whose only user-code channel is own-index access on its own receiver.
 
 `Array.prototype.includes` performs `Get(receiver, "0")`, which runs an indexed
 accessor if the caller installed one. That is the same channel `values[0]` opens,
 and this rule already treats a plain indexed read as a pure read, so admitting
 these members widens nothing: the assumption is pre-existing, not introduced here.
 
 "Own-index access" is wider than `Get` and `Set`, and an earlier version of this
 comment named only those two, which made the channel look narrower than the members
 in it. Measured against a fully trapped receiver: `indexOf`, `lastIndexOf`,
 `unshift`, `copyWithin`, `reverse` and `shift` reach `has`, and `pop` and `shift`
 reach `deleteProperty`. Every one of those is still inside the baseline, because the
 baseline is what this rule accepts on a parameter, not what an implementation was
 guessed to do: `0 in values` keeps a read-only offer, and `delete values[0]` reports
 a plain mutation. `effect-member-channel-traps.unit.test.ts` derives that baseline
 executably and fails any member reaching outside it.
 
 The assumption is worth naming, because it is load-bearing and unsound in the
 exotic case. Measured: an accessor installed at index 0 that pushes during its
 getter turns `includes` into a call that restructures its own receiver, taking a
 one-element array to two. The identical getter fires for `values[0]`. So this
 rule's model, everywhere and not only here, assumes caller-owned collections hold
 ordinary data properties. This channel is exactly that assumption, made explicit.
 */
export const MEMBER_CHANNEL_RECEIVER_INDEX: unique symbol = Symbol(
  'collection member reaches user code only through own-index access',
);

/**
 Member reaching own-index access and the species construction channel.
 
 `Array.prototype.slice` builds its result through `ArraySpeciesCreate`, which reads
 `constructor[Symbol.species]` and calls what it returns. That is caller-selected code,
 and the stated trust baseline in
 `doc/decision/prefer-readonly-member-channel-authority.md` admits it: an own
 `constructor` and an own `Symbol.iterator` are both ordinary data properties, so
 refusing species while `for...of` and spread are accepted drew a line no principle
 supports.
 
 Kept distinct from the own-index channel rather than merged into it, because what each
 probe must prove differs. A member here is permitted to reach the species hook and
 nothing wider, so the probe still fails it for element coercion or a property read.
 
 This says nothing about what the result carries. `effect-result-provenance-authority.ts`
 answers that separately, and a container result stays tracked through its own relation.
 */
export const MEMBER_CHANNEL_RECEIVER_INDEX_AND_SPECIES: unique symbol = Symbol(
  'collection member reaches own-index access and the species construction channel',
);

/**
 Member whose channel no probe has established, so it stays failing closed.
 
 Absence from the table is never a claim that a member dispatches, only that
 nothing here has shown it does not.
 */
/**
 Member reads its receiver by index and then coerces each element it read.
 
 Narrow conditionally rather than absolutely, which is why it is its own symbol.
 `memberChannelIsVerifiedNarrow` discharges it only where every element type is strictly
 primitive, and withholds otherwise.
 */
export const MEMBER_CHANNEL_RECEIVER_INDEX_AND_COERCION: unique symbol = Symbol(
  'collection member reads its receiver by index and coerces each element it read',
);

/**
 Member reads its own internal slot and whatever the caller handed it, and nothing else.
 
 Narrow conditionally, like the coercion channel and for the same kind of reason: what the
 member can reach is decided at the call site rather than by the member name.
 `memberChannelIsVerifiedNarrow` discharges it only where the call passes no arguments at
 all, and withholds otherwise.
 
 The date locale members are the case. ECMA-402 has them process caller-supplied `locales`
 and `options`, reading properties off the options object and so running any accessor on
 it, which is why the name alone cannot answer. Called with an empty argument list there is
 no such object: the member reads `[[DateValue]]`, consults ambient locale data, and
 returns a string. Measured on `pubDateDate.toLocaleString()` in
 `package/rss/app/src/html-item.ts`, which passes nothing.
 
 Ambient locale and time zone change what the string says and not what the receiver holds,
 so they are not a channel into caller state.
 */
export const MEMBER_CHANNEL_INTERNAL_SLOT_AND_ARGUMENTS: unique symbol = Symbol(
  'collection member reads its internal slot and whatever the caller passed it',
);

/**
 Member opens a channel this authority has not verified.
 */
export const MEMBER_CHANNEL_UNPROVEN: unique symbol = Symbol(
  'collection member has no verified user-code channel',
);

/**
 Which user-code channel one default-library collection member opens.
 */
export type MemberUserCodeChannel =
  | typeof MEMBER_CHANNEL_INTERNAL_SLOT
    | typeof MEMBER_CHANNEL_INTERNAL_SLOT_AND_ARGUMENTS
    | typeof MEMBER_CHANNEL_RECEIVER_INDEX
    | typeof MEMBER_CHANNEL_RECEIVER_INDEX_AND_SPECIES
    | typeof MEMBER_CHANNEL_RECEIVER_INDEX_AND_COERCION;

/**
 Default-library collection members whose user-code channel is verified, by owner.
 
 This is an authority, not a derivation, and it exists by the amendment in
 `doc/decision/prefer-readonly-member-channel-authority.md`. Which channel a member
 opens is a fact about ECMA-262 that no declaration exposes: `Map.get` touches no
 property while `Array.slice` consults species, and even their return types do not
 separate them, since `toReversed` and `with` build new arrays without species
 while `flat` uses it.
 
 An earlier revision of this table claimed every listed member ran no user code.
 That was false for the whole `Array` half and was caught by probing an
 accessor-bearing receiver rather than by reading it back: `at`, `includes`,
 `indexOf`, `lastIndexOf`, `toReversed`, `toSpliced`, `pop` and `copyWithin` all
 invoke an indexed getter. Hence two channels rather than one flat set.
 
 Every entry is enforced by `effect-member-channel-authority.unit.test.ts`, which
 probes a real engine per member and fails when a member reaches a channel wider
 than the one claimed. That enforcement is the whole difference between this and
 the unverified catalogs the audit removed, so an entry added without a passing
 probe is a defect.
 
 Members absent from this table are not asserted to be narrow; they are simply
 unproven and keep failing closed.
 */
const CHANNELS_BY_OWNER: Readonly<
  Record<string, Readonly<Record<string, MemberUserCodeChannel>>>
  > = {
    Array: {
      join: MEMBER_CHANNEL_RECEIVER_INDEX_AND_COERCION,
      at: MEMBER_CHANNEL_RECEIVER_INDEX,
      slice: MEMBER_CHANNEL_RECEIVER_INDEX_AND_SPECIES,
      includes: MEMBER_CHANNEL_RECEIVER_INDEX,
      indexOf: MEMBER_CHANNEL_RECEIVER_INDEX,
      lastIndexOf: MEMBER_CHANNEL_RECEIVER_INDEX,
      with: MEMBER_CHANNEL_RECEIVER_INDEX,
      toReversed: MEMBER_CHANNEL_RECEIVER_INDEX,
      toSpliced: MEMBER_CHANNEL_RECEIVER_INDEX,
      push: MEMBER_CHANNEL_RECEIVER_INDEX,
      pop: MEMBER_CHANNEL_RECEIVER_INDEX,
      shift: MEMBER_CHANNEL_RECEIVER_INDEX,
      unshift: MEMBER_CHANNEL_RECEIVER_INDEX,
      fill: MEMBER_CHANNEL_RECEIVER_INDEX,
      copyWithin: MEMBER_CHANNEL_RECEIVER_INDEX,
      reverse: MEMBER_CHANNEL_RECEIVER_INDEX,
      keys: MEMBER_CHANNEL_RECEIVER_INDEX,
      values: MEMBER_CHANNEL_RECEIVER_INDEX,
      entries: MEMBER_CHANNEL_RECEIVER_INDEX,
    },
    ReadonlyArray: {
      join: MEMBER_CHANNEL_RECEIVER_INDEX_AND_COERCION,
      at: MEMBER_CHANNEL_RECEIVER_INDEX,
      slice: MEMBER_CHANNEL_RECEIVER_INDEX_AND_SPECIES,
      includes: MEMBER_CHANNEL_RECEIVER_INDEX,
      indexOf: MEMBER_CHANNEL_RECEIVER_INDEX,
      lastIndexOf: MEMBER_CHANNEL_RECEIVER_INDEX,
      with: MEMBER_CHANNEL_RECEIVER_INDEX,
      toReversed: MEMBER_CHANNEL_RECEIVER_INDEX,
      toSpliced: MEMBER_CHANNEL_RECEIVER_INDEX,
      keys: MEMBER_CHANNEL_RECEIVER_INDEX,
      values: MEMBER_CHANNEL_RECEIVER_INDEX,
      entries: MEMBER_CHANNEL_RECEIVER_INDEX,
    },
    DataView: {
      getBigInt64: MEMBER_CHANNEL_INTERNAL_SLOT,
      getBigUint64: MEMBER_CHANNEL_INTERNAL_SLOT,
      getFloat16: MEMBER_CHANNEL_INTERNAL_SLOT,
      getFloat32: MEMBER_CHANNEL_INTERNAL_SLOT,
      getFloat64: MEMBER_CHANNEL_INTERNAL_SLOT,
      getInt8: MEMBER_CHANNEL_INTERNAL_SLOT,
      getInt16: MEMBER_CHANNEL_INTERNAL_SLOT,
      getInt32: MEMBER_CHANNEL_INTERNAL_SLOT,
      getUint8: MEMBER_CHANNEL_INTERNAL_SLOT,
      getUint16: MEMBER_CHANNEL_INTERNAL_SLOT,
      getUint32: MEMBER_CHANNEL_INTERNAL_SLOT,
      setBigInt64: MEMBER_CHANNEL_INTERNAL_SLOT,
      setBigUint64: MEMBER_CHANNEL_INTERNAL_SLOT,
      setFloat16: MEMBER_CHANNEL_INTERNAL_SLOT,
      setFloat32: MEMBER_CHANNEL_INTERNAL_SLOT,
      setFloat64: MEMBER_CHANNEL_INTERNAL_SLOT,
      setInt8: MEMBER_CHANNEL_INTERNAL_SLOT,
      setInt16: MEMBER_CHANNEL_INTERNAL_SLOT,
      setInt32: MEMBER_CHANNEL_INTERNAL_SLOT,
      setUint8: MEMBER_CHANNEL_INTERNAL_SLOT,
      setUint16: MEMBER_CHANNEL_INTERNAL_SLOT,
      setUint32: MEMBER_CHANNEL_INTERNAL_SLOT,
    },
  /* One member rather than the twenty-nine the read-only view membership declares, and
   * the asymmetry is the design rather than an unfinished table. Membership must be
   * complete, because omitting a member there claims it restructures its receiver. A
   * channel entry claims the opposite direction, so an absent member simply stays
   * unproven, and listing only what a probe has driven is what keeps this table sound.
   *
   * `toISOString` reads `[[DateValue]]` and formats it, reaching no property of its
   * receiver. Its siblings are not all so simple: `toJSON` performs `ToPrimitive` and
   * then `Invoke(O, "toISOString")`, both receiver-selected lookups, and the locale
   * members process caller-supplied `locales` and `options` this table cannot see from a
   * member name alone. They stay off until a probe drives them. */
    Date: {
      toISOString: MEMBER_CHANNEL_INTERNAL_SLOT,
    /* Asserted for one reason and it is not this table's semantics: `toLocaleString` is a
     * member of `Object.prototype`, so an object literal checked against
     * `Record<string, MemberUserCodeChannel>` meets the apparent member at that key and
     * widens the `unique symbol` to `symbol`. Reduced to four lines and reproduced against
     * TypeScript 7.0.2, where the same literal accepts `toISOString` and
     * `toLocaleDateString` beside it and rejects only this key. Its two siblings need no
     * assertion, which is the tell.
     *
     * What holds this entry is the probe rather than the type. An assertion accepts any
     * member of the union, so the compiler would take a wrong channel here as readily as
     * the right one; `effect-member-channel-authority.unit.test.ts` drives all three locale
     * members in both directions and is what would catch that. */
      toLocaleString: MEMBER_CHANNEL_INTERNAL_SLOT_AND_ARGUMENTS as MemberUserCodeChannel,
      toLocaleDateString: MEMBER_CHANNEL_INTERNAL_SLOT_AND_ARGUMENTS,
      toLocaleTimeString: MEMBER_CHANNEL_INTERNAL_SLOT_AND_ARGUMENTS,
    },
    Map: {
      get: MEMBER_CHANNEL_INTERNAL_SLOT,
      has: MEMBER_CHANNEL_INTERNAL_SLOT,
      set: MEMBER_CHANNEL_INTERNAL_SLOT,
      delete: MEMBER_CHANNEL_INTERNAL_SLOT,
      clear: MEMBER_CHANNEL_INTERNAL_SLOT,
      keys: MEMBER_CHANNEL_INTERNAL_SLOT,
      values: MEMBER_CHANNEL_INTERNAL_SLOT,
      entries: MEMBER_CHANNEL_INTERNAL_SLOT,
    },
    ReadonlyMap: {
      get: MEMBER_CHANNEL_INTERNAL_SLOT,
      has: MEMBER_CHANNEL_INTERNAL_SLOT,
      keys: MEMBER_CHANNEL_INTERNAL_SLOT,
      values: MEMBER_CHANNEL_INTERNAL_SLOT,
      entries: MEMBER_CHANNEL_INTERNAL_SLOT,
    },
    Set: {
      has: MEMBER_CHANNEL_INTERNAL_SLOT,
      add: MEMBER_CHANNEL_INTERNAL_SLOT,
      delete: MEMBER_CHANNEL_INTERNAL_SLOT,
      clear: MEMBER_CHANNEL_INTERNAL_SLOT,
      keys: MEMBER_CHANNEL_INTERNAL_SLOT,
      values: MEMBER_CHANNEL_INTERNAL_SLOT,
      entries: MEMBER_CHANNEL_INTERNAL_SLOT,
    },
    ReadonlySet: {
      has: MEMBER_CHANNEL_INTERNAL_SLOT,
      keys: MEMBER_CHANNEL_INTERNAL_SLOT,
      values: MEMBER_CHANNEL_INTERNAL_SLOT,
      entries: MEMBER_CHANNEL_INTERNAL_SLOT,
    },
  };

/**
 Verified channels indexed for lookup, keyed by declaring interface then member.
 */
export const MEMBER_CHANNELS_BY_INTERFACE: ReadonlyMap<
  string,
  ReadonlyMap<string, MemberUserCodeChannel>
  > = new Map(Object.entries(CHANNELS_BY_OWNER,)
    .map(function ownerChannels(
      [ownerName, members,],
    ): readonly [
      string,
      ReadonlyMap<string, MemberUserCodeChannel>,
    ] {
      return [
        ownerName,
        new Map(Object.entries(members,),),
      ];
    },),);

/**
 Total verified entries, pinned so silent table growth fails the guard.
 
 The `catalog-free effect architecture` guard asserts this equals the summed entry
 count. Adding a member therefore cannot pass unnoticed: the author must change
 this number, which is the point at which the decision document and the probe
 requirement are unavoidable.
 */
export const VERIFIED_MEMBER_CHANNEL_COUNT = 81;

/**
 Members returning an iterator, whose entries claim creation and drainage together.
 
 These were excluded while the authority described one invocation, because creating
 an iterator reaches nothing and advancing it reads the receiver, so an
 inert-creation claim would have been read as an inert-consumption claim.
 
 The entries now claim the union of both operations, which is what the one consumer
 can use. `receiverClaimAnswerable` asks this authority while inspecting the
 creating call, and there is no second call to ask about: for-of and spread advance
 an iterator through no `CallExpression` at all, so a creation-only fact would have
 had nothing to consume it. Both operations land inside a channel already admitted
 here, which is why the union is expressible: internal-slot for `Map` and `Set`,
 own-index for an array, whose drainage reads `length` and then each index.
 
 The union is also the conservative representation, not merely the convenient one.
 A member whose creation is narrow and whose drainage is not would take the wider
 channel, and a channel wider than both admitted here is absent, which fails closed.
 
 What the union gives up is worth naming. It answers "this call and any later
 built-in advancement of its result", not "this call", so it cannot distinguish
 partial consumption, and it proves the built-in `next` path rather than every
 future use of the returned object: source that replaces the iterator's own `next`
 runs something this claim never measured. Both are behind the same assumption the
 own-index channel already rests on, that caller-owned collections hold ordinary
 data properties.
 
 Kept as a named list because the probe in
 `effect-member-channel-authority.unit.test.ts` measures creation and drainage as
 separate steps and needs to know which members to drain. `Symbol.iterator` is
 absent and unreachable either way: lookup here is by `declaration.name.text`, and a
 computed name is not an identifier.
 */
export const ITERATOR_MEMBER_NAMES: ReadonlySet<string> = new Set([
  'keys',
  'values',
  'entries',
],);

/**
 Members that invoke a caller-supplied observer, whatever their ambient channel is.
 
 The channel above and this set answer different halves of one question, and collapsing
 them was the first draft of the trust-baseline work. `filter` reaches own-index access
 and default species, both trusted, and it also calls whatever predicate the caller
 passed. Admitting it to the table on the strength of the first half alone would
 discharge `rows.filter(foreignMutatingPredicate)` on a receiver every element of which
 that predicate received.
 
 So the ambient half may be recorded in the table and the observer half may not be
 discharged there at all: it belongs to `recordReadonlyViewApplications`, which resolves
 the observer to owned source and derives what its effects do to the receiver, or leaves
 the call undischarged when it cannot.
 
 Keyed by member name alone, deliberately over-approximating. A name listed here can
 only withhold a discharge, never grant one, so a name that turns out to take no
 observer costs precision and nothing else. `sort` and `toSorted` are listed although
 their comparator is optional, because an absent comparator runs the default one, which
 coerces elements and is not owned source either way.
 
 Enforced by `effect-member-channel-authority.unit.test.ts`, which fails when any entry
 in the table names a member listed here.
 */
export const OBSERVER_BEARING_MEMBER_NAMES: ReadonlySet<string> = new Set([
  'every',
  'filter',
  'find',
  'findIndex',
  'findLast',
  'findLastIndex',
  'flatMap',
  'forEach',
  'map',
  'reduce',
  'reduceRight',
  'some',
  'sort',
  'toSorted',
],);

/**
 Every default-library collection member name this rule recognises, for any purpose.
 
 Derived from the tables rather than written out, so it cannot drift from them. It exists
 for one consumer, the diagnostic, which needs to know whether a finding is entirely about
 collection calls in order to say something true about them: the remediations that fit an
 unresolved package call fit none of these, which is what issue #414 reports.
 
 Recognition only. Membership here proves nothing about a member's channel or its result,
 and no discharge may consult it.
 */
export const COLLECTION_MEMBER_NAMES: ReadonlySet<string> = new Set([
  ...Object.values(CHANNELS_BY_OWNER,)
    .flatMap(function ownerMembers(members,): readonly string[] {
      return Object.keys(members,);
    },),
  ...OBSERVER_BEARING_MEMBER_NAMES,
],);

/**
 Tests whether a member invokes a caller-supplied observer.
 
 @param memberName - Member being called.
 
 @returns whether the member hands receiver state to a function the caller passed.
 
 @example
 ```ts
 memberInvokesObserver({ memberName: 'filter' });
 ```
 */
export function memberInvokesObserver({ memberName, }: { readonly memberName: string; },): boolean {
  return OBSERVER_BEARING_MEMBER_NAMES.has(memberName,);
}

/**
 Resolves which user-code channel a collection member is verified to open.
 
 @param ownerName - Declaring default-library interface name.
 
 @param memberName - Member being called.
 
 @returns verified channel, or the unproven sentinel.
 
 @example
 ```ts
 collectionMemberUserCodeChannel({ ownerName: 'ReadonlyMap', memberName: 'get' });
 ```
 */
export function collectionMemberUserCodeChannel({
  ownerName,
  memberName,
}: {
  readonly ownerName: string;
  readonly memberName: string;
},): MemberUserCodeChannel | typeof MEMBER_CHANNEL_UNPROVEN {
  return MEMBER_CHANNELS_BY_INTERFACE.get(ownerName,)
    ?.get(memberName,)
    ?? MEMBER_CHANNEL_UNPROVEN;
}
