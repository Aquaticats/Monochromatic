# Declaring read-only view membership the default library omits

Accepted 2026-08-06 by the repository owner,
 over leaving the limit documented and over treating every
member of an unpaired interface as restructuring its receiver.

## The derivation needs a pair, and the library does not always provide one

`collectionStructureClaim` in `effect-default-library-readonly-view.ts` decides whether a member
restructures its receiver by diffing an interface against its `Readonly` counterpart.
 `Array` declares
`push` and `ReadonlyArray` does not,
 so `push` mutates.
 Everything the rule knows about mutation comes from
that one diff,
 which is why the rule needs no catalogue of mutators.

`lib.es5.d.ts` declares `DataView` and no `ReadonlyDataView`,
 checked rather than assumed.
 So the diff has
nothing to compare,
 the claim answers `COLLECTION_UNRECOGNIZED`,
 and `recordCollectionMemberEffect` returns
before reaching any other question.
 Every buffer write lands on the opaque boundary.

Measured before this landed:
 21 findings across the workspace name a `DataView` or typed-array member and
14 name nothing else.
 `writeEndOfCentralDirectory` in `package/module/zip-writer/src/serialize.ts` carries
a `ForeignBorrowed` marker and a hand-written `@mutates view` for what is a specification-defined store,
which is the escape hatch being reached for in place of an answer.

## What was decided

Declare the membership the library omits,
 in `effect-unpaired-view-authority.ts`,
 and let the existing diff
run against it unchanged.

The alternative shape,
 listing the mutating members instead,
 was rejected for a specific reason rather than
on taste:
 it would introduce a second derivation that can disagree with the first,
 and the two would then
have to be kept consistent by hand.
 Declaring the view states the same fact in the same shape the library
states it for every paired interface,
 so there is one rule and one place it can be wrong.

Treating every member of an unpaired interface as mutating was rejected because it is false.
 `getUint16`
would be recorded as mutating its receiver,
 which makes a stale `@mutates` contract look accurate and puts a
wrong fact into summaries other analysis reads.
 A wrong inference is worse than an absent one.

## What it is held to

`effect-unpaired-view.unit.test.ts` drives a real `DataView` over a real `ArrayBuffer` and,
 for every member
the engine exposes,
 compares the buffer's bytes either side of the call.
 A declared member that changes them
fails,
 and an undeclared member that changes nothing fails.
 That is the same standard the other authorities
are held to:
 an entry added without a passing probe is a defect.

The count is pinned in two files,
 so adding a member means changing a number in
`catalog-free-architecture.unit.test.ts` as well,
 which is the point at which the probe becomes unavoidable.

## What it does, measured

Both halves landed.
 The channel probe gained a `DataView` receiver:
 a real view over a real
`ArrayBuffer`,
 driven with plain numeric arguments on purpose,
 since a recording argument would report
its own coercion and the question here is what the member reaches on the *receiver*.
 The species
recorder the probe already installs proves no constructor hook is consulted.

Workspace either side:
 3020 errors to 3015,
 1682 rule findings to 1677,
 semantic failures and read-only
offers unchanged at 4 and 33.
 Twelve findings naming a `view` parameter disappeared across
`package/mvm/cli`,
 `package/figma/kiwi` and `package/module/zip-writer`,
 and seven reappeared naming only
`buffer`,
 which is the sound remainder:
 a `Uint8Array` is a different interface and this authority says
nothing about it.
 No parameter became opaque that was not opaque before.

`writeEndOfCentralDirectory` now reads `referentMutated=[0]` with `opaque=[]`,
 so its `@mutates view`
contract is satisfied by inference and its `ForeignBorrowed` marker is no longer doing any work.

## Date, the second unpaired interface

Accepted 2026-08-07,
 on the same ground and by the same shape.
 `lib.es5.d.ts` declares `Date` and no `ReadonlyDate`,
 checked rather than assumed,
so the diff had nothing to compare and every date member landed on the opaque boundary.

### The spike came before the table

Ordering that matters,
 because the `DataView` half of this document records that a membership table alone
changed nothing until channel entries joined it.
 So the smallest possible version went in first:
 `Date` declaring one member,
 `toISOString`,
 with one channel entry beside it,
 and the real
`package/module/toml-edit/src/values.ts` probed either side.

Without it,
 `encodeValue` and the three callables around it all read `opaque=[0]`.
 With it,
 all four read
`opaque=[]`.
 That is the whole payoff established before any of the table was written,
 which is what the
`Object.getPrototypeOf` reader entry immediately before it had failed to establish:
 that entry removed a
cause from 46 messages and cleared one finding,
 because a second cause,
 this one,
 stood behind it.

The blocker was never narrowing,
 which is worth recording because it looked like it was.
 `encodeValue` takes
`input: unknown` and reaches `toISOString` through `input instanceof Date`,
 and a first probe of that shape
stayed opaque.
 The confound was in the probe:
 its fallback branch called `String(input)`,
 which coerces
through `toString` or `valueOf` and is a genuine user-code channel.
 Isolated,
 a parameter narrowed from
`unknown` by `instanceof` and used for nothing else discharges cleanly.

### Membership is complete and the channel is not, deliberately

The two tables carry opposite defaults,
 so completeness is required of one and forbidden of the other.

Omitting a member from the membership claims it restructures its receiver,
 so that table has to name every
preserving member or it states something false.
 It names 29:
 the eighteen field readers,
 the ten
formatting and conversion members,
 and `valueOf`.
 A `Date` has one mutable specification slot,
 `[[DateValue]]`,
 and the fifteen declared setters write it.

Omitting a member from the channel table claims only that nothing has proven it narrow,
 so that table names
one member.
 `toISOString` reads the slot and formats it,
 reaching no property of its receiver.
 Its siblings
are not all so simple:
 ECMA-262 has `toJSON` perform `ToPrimitive` and then `Invoke(O, "toISOString")`,
 both
lookups a caller can answer,
 and the locale members process caller-supplied `locales` and `options` that a
member name alone cannot describe.
 `toJSON` stays off until a probe drives it,
 and the membership table still
names it,
 because preserving the receiver and dispatching to user code are separate questions.
 The locale
members joined the channel table later,
 on a condition rather than outright,
 which is described below.

Three names the engine provides are absent from both,
 and the absence claims nothing:
 `getYear`,
 `setYear`
and `toGMTString` are Annex B members TypeScript never declares,
 so no lookup can reach the table holding
one.
 `getVarDate` is declared,
 but only by `lib.scripthost.d.ts`,
 which `"lib": ["ESNext"]` in
`package/config/typescript/tsconfig.options.json` never includes and no engine here provides;
 an entry for it
could not be probed,
 and an unprobed entry is the one thing this design refuses.
 `[Symbol.toPrimitive]` is
declared and preserving and still absent,
 because `collectionStructureClaim` rejects a computed member name
before consulting the table:
 it stays unrecognized rather than being claimed to mutate.

### The probe had a hole and the extension closed it

The `DataView` probe called each member once with a fixed argument.
 Run against `Date`,
 that design read
`setUTCDate` as read-only,
 because `setUTCDate(1)` on an epoch date writes the day it already holds.
 A
vacuous pass,
 of exactly the kind this document warned about when it argued the typed arrays were not
worth taking,
 and it was found by extending the probe rather than by reasoning about it.

Five corrections landed together,
 and they apply to both interfaces rather than only the new one.
 Each
member now runs from a receiver of its own,
 against three distinct arguments,
 so a write of a value the
receiver already holds cannot read as inert.
 The date's slot is read through the intrinsic obtained from its
own property descriptor,
 so a member that shadowed `getTime` could not be measured by itself.
 The snapshot
covers own properties and their descriptors as well as the specification slot,
 since a member could add or
redefine a property without touching the slot.
 A call that throws disqualifies its unchanged reading as
evidence,
 rather than passing as proof the member changes nothing.
 And the surface comes from
`Reflect.ownKeys`,
 so the symbol member is excluded by a filter that names it instead of by an enumeration
that never mentioned it.

The channel probe gained a `Date` receiver recording every conversion lookup a date member can dispatch to:
`toString`,
 `valueOf` and `[Symbol.toPrimitive]`.
 `toISOString` is deliberately not shadowed,
 since the probe
resolves members off the receiver and a recorder in that position would measure the recorder.
 `toJSON` is
the control:
 it is excluded from the channel table,
 it reaches those recorders,
 and without it "the listed
member reached nothing" would be indistinguishable from a receiver carrying no instrumentation at all.

### What Date did, measured

Workspace either side:
 2952 errors to 2924,
 1586 rule findings from 1614,
 warnings unchanged at 3902.
 The cause
`input.toISOString` went from 48 mentions to none,
 and 28 findings cleared outright.

The read-only offer count moved,
 33 to 34,
 for the first time in this line of work,
 and the new offer is
correct rather than a regression.
 It lands on `encodeInlineTable` in the same file:
 the callable reads
`Object.entries(input,)`,
 maps each entry to a string and joins them,
 writing nothing,
 while its declared
`Record<string, unknown>` carries a writable index signature.
 The offer says to close it.
 No offer landed
on a parameter typed `Date`,
 which was the specific hazard to check,
 since `Readonly<Date>` would not stop
`setTime` and an offer there would be correct and useless at once.

Stale `@mutates` findings went from 11 to 14.
 All three are contracts this change made provably wrong,
including `@mutates input` on `encodeInlineTable`,
 which is the outcome the rule exists to produce rather
than a cost:
 a contract describing an effect that cannot happen is the failure `AGENTS.md` names in `JCH`.

### The locale members joined the channel on a condition, 2026-08-07

The `Date` channel table listed one member and named the locale members as the reason it could not list
more:
 ECMA-402 has them read properties off a caller-supplied options object,
 so they run any accessor on
one,
 and a member name alone cannot say whether such an object was passed.

That is a fact about the call rather than about the member,
 which is what makes it answerable.
`pubDateDate.toLocaleString()` in `package/webapp-productivity/rss/src/html-item.ts` passes nothing at all,
so there is no options object and no accessor to reach.
 The member reads `[[DateValue]]`,
 consults ambient
locale data,
 and returns a string.
 Ambient locale and time zone change what that string says and not what
the receiver holds,
 so they are not a channel into caller state.

Modelled on the conditional channel that already existed rather than as a new kind of thing.
 The coercion
channel is discharged only where every element is strictly primitive;
 this one only where the call site
passes no arguments.
 Measured:
 no arguments discharges,
 a locale string withholds,
 an options object
withholds,
 and `toISOString` beside them is unaffected.

Deliberately coarse in one direction.
 A bare `'en-US'` carries no accessor and still withholds,
 because the
condition is an empty argument list rather than primitive arguments.
 That is the conservative side of the
line and it can be narrowed later on the same evidence the coercion channel narrowed on.

### The probe found a bug in itself, which is the point of driving both directions

The tripwire had to fire for an options object as well as stay silent for none,
 or the entry would claim
something no probe had seen.
 Driving it that way failed immediately,
 and not for the reason expected:
 every
locale call threw `probeArguments is not a function or its return value is not iterable`.

The probe looks its arguments up in an object literal keyed by member name.
 `toLocaleString` is a member of
`Object.prototype`,
 so the lookup found the inherited function rather than `undefined`,
 the nullish
fallback never fired,
 and the caller spread a function.
 The channel assertion reported that as the member
reaching a hook,
 which is the right shape of failure for the wrong reason.

The same collision appears in the type system,
 and was reduced to four lines against TypeScript 7.0.2:
 an
object literal checked against `Record<string, MemberUserCodeChannel>` widens the `unique symbol` at a
`toLocaleString` key to `symbol`,
 while accepting `toISOString` and `toLocaleDateString` beside it.
 Both
sites now say so where the workaround sits,
 because a reader who does not know the collision will read
either as arbitrary.

### What the locale channel measured

Workspace either side:
 2907 errors to 2904,
 1569 rule findings to 1566,
 warnings unchanged at 3902,
stale contracts at 14,
 and the read-only offer set byte-identical at 34.
 The cause `toLocaleString` went
from three mentions to none and three findings cleared,
 with nothing added.

One of the three is worth naming.
 `getIndexHtmlBody` in `package/webapp-productivity/rss/src/html.ts` was
not reporting at all two increments earlier;
 composing container relations connected its `options` to the
locale call through the elements,
 which added the finding,
 and this entry then discharged it.
 The finding
was true while it stood:
 the parameter really does reach that call.
 Surfacing it and answering it are
different steps and both were needed.

### The typed arrays are the obvious next extension and should not be taken

Measured rather than assumed,
 because the `buffer` remainder makes them look like an easy repeat of this
work.

The stake is small:
 about ten findings,
 with `subarray` at eight mentions and `set` at six,
 and
`Uint8Array` is the only typed array any of them uses,
 checked against the parameter declarations in
`package/module/zip-writer/src/headers.ts`,
 `package/figma/kiwi/src/zip.ts` and `package/mvm/cli`.

The cost is not small,
 and the reason is this authority's own semantics.
 Membership means "preserves
structure",
 so every member omitted from a declared interface is thereby claimed to mutate.
 A partial
`Uint8Array` entry covering only the members that appear in findings would therefore record `map` and
`filter` as restructuring their receiver,
 which is false and is exactly the wrong-inference failure this
document rejects in its own alternatives.
 The entry has to be complete:
 33 preserving members against 7
mutating ones,
 counting `setFromBase64` and `setFromHex` as writes,
 which they are.

Completeness is what makes the probe hard rather than long.
 `DataView` was probeable because every member
takes numbers,
 so one generic call exercised all of them.
 A third of the `Uint8Array` surface takes a
callback or a replacement array,
 and a member that throws for want of an argument leaves the buffer
unchanged,
 which this probe would read as proof that it preserves structure.
 A vacuous pass is worse than
no entry,
 and per-member arguments for 40 members is a different piece of work from what landed here.

So the ratio is ten findings against a table that is unsound if partial and a probe that is vacuous if
careless.
 Left undone deliberately,
 on the same ground as `Set` and `ReadonlySet` in the result-relation
work:
 an increment whose cost is dominated by making its own evidence trustworthy is not improved by
doing it quickly.

## What it did not do on its own, measured

Nothing,
 on its own.
 `receiverClaimAnswerable` asks for a verified user-code channel before it asks the
structure claim anything,
 and `MEMBER_CHANNELS_BY_INTERFACE` has no `DataView` entry.
 Probed with the
authority in place and no channel entry:
 `writeEndOfCentralDirectory` still reports `view.setUint16`.
 With
channel entries added by hand it reads `referentMutated=[0]` and `opaque=[]`,
 which is the intended answer
and is what the second half would buy.

The channel entries are not landed with this,
 and the reason is that they cannot yet be probed.
`instrumentedReceiver` in `effect-member-channel-authority.unit.test.ts` builds a `Map`,
 a `Set` or an array
and instruments elements,
 species and index hooks.
 A `DataView` has none of those,
 so listing its members
there produces a probe that passes without measuring anything,
 which is the failure mode that authority's
own design is meant to prevent.

That was the state at the first commit,
 which landed the authority alone as a prerequisite.
 The channel
probe extension was accepted separately and is described above.
