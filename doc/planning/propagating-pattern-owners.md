# Owners for eight propagating patterns

Eight proposals for patterns that keep being copied because nothing owns them.
Each was found by classifying the 147 clusters proposed for dismissal in #656,
#657,
#658 and #659 by whether the pattern propagates,
which is recorded in `doc/planning/slopo-dismissed-cluster-actionability.md`.

Every design here is drafted to be reviewed rather than chosen from scratch:
the code shape,
the migration,
the risk,
and what would falsify it.
Site counts were measured on 2026-10-09 at HEAD `f4095e0f3` or later.

## 1. A base error class that derives its own name

Pattern:
 `public constructor(message: string,) { super(message,); this.name = 'XError'; }`.

Measured:
 241 `extends Error` declarations across 181 files;
210 `this.name = ` assignments across 146 files;
154 of them the pure two-statement form;
21 forward an `options` argument to `super`;
58 add fields or build a logger inside the constructor.
All 209 literals checked match their enclosing class name,
by a comparison proven able to fail against a fixture carrying one planted mismatch.
Three sites already assign `X.name` instead of a literal,
as in `this.name = SkillMirrorManifestError.name` in root `file-enforcer.config.ts`,
so the pattern that makes the literal redundant already exists here.

Design:

```typescript
// package/module/error-named/src/named-error.ts
/**
 Error subclass whose `name` follows the concrete class, so a subclass declares no
 constructor when the message is all it carries.

 @remarks
 `new.target.name` is the leaf class even through further subclassing. Published
 artifacts keep it because every rolldown config sets `mangle: false`, documented
 in `package/config/rolldown/src/index.ts` as "Mangle breaks func.name and makes
 output difficult for users to audit".
 */
export abstract class NamedError extends Error {
  public constructor(message: string, options?: ErrorOptions) {
    super(message, options);
    this.name = new.target.name;
  }
}
```

The 154 pure sites become one line with no constructor:

```typescript
export class BranchWorktreeViolationError extends NamedError {}
```

The 58 that add fields keep their constructor and drop only the name assignment.

Migration:
 mechanical per package,
each step verified by re-running the name-against-class comparison,
which must stay at zero mismatches.
`PP4` is unaffected:
one class per failure mode remains,
and only the constructor body disappears.

Risks and what would falsify the design:

- A class that needs a `name` different from its identifier.
   Measured today:
   none.
   The falsifier would appear as a mismatch in the comparison,
   and the remedy is an optional `name` override parameter.
- `new.target` under a bundler that mangles.
   Checked:
   `mangle: false` in `index.ts`,
   `index.node.ts` and `index.client.ts`.
   If a future config enables mangling,
   derived names break silently,
   so the base class needs a test that asserts the name after a built-artifact load.
- Published `module/*` packages gain a dependency edge.
   Two ways out:
   put the base in the existing `throws/core` package,
   which is already published and currently exports one function,
   or accept a new package under `AP1` and `SGD`.
   Putting it in `throws/core` stretches that package's stated purpose,
   which its README describes as an expression-position throwing helper.
- `PPX` prefers `#private` members.
   The base class has none,
   so nothing conflicts.

## 2. A Rust macro family for message-forwarding error types

Pattern:
 `impl Display` bodies that write the message field,
and one-line constructors that build the struct from a message.

Measured:
 `write_str(self.message.as_str())` at 11 sites across 11 files,
byte-identical,
out of 50 `Display` impls across 32 files;
`String::from(message` at 17 sites across 16 files.
The 11 identical impls are cluster C13,
which #656 proposed to dismiss as trivial boilerplate.

Design:

```rust
// package/rust-module/error-support/src/lib.rs
/// Implements `Display` for an error type that stores its text in a `message` field.
#[macro_export]
macro_rules! display_message {
    ($ty:ty) => {
        impl ::core::fmt::Display for $ty {
            fn fmt(&self, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                formatter.write_str(self.message.as_str())
            }
        }
    };
}

/// Builds a one-field error struct from a message.
#[macro_export]
macro_rules! message_constructor {
    ($ty:ident) => {
        impl $ty {
            pub fn new(message: impl Into<String>) -> Self {
                Self { message: message.into() }
            }
        }
    };
}
```

Requires each type to store `message: String`,
which all 11 `Display` sites already do.

Alternative:
 `thiserror`,
which derives both.
It appears in no `Cargo.toml` in this repository and only in third-party audits
(`doc/audit/turso-cargo-toml-0.6.1.md` and two vetting docs),
so adopting it is a `choosing-technology` decision with a source audit,
not a dependency addition.
The macro route has no new dependency and covers the measured shapes.

Risks:
 macro hygiene and path resolution are handled by `::core::` prefixes;
the impl must live in the crate that defines the type,
which is where the current impls already are;
and a macro hides the impl from readers who grep for `impl Display`,
which is a real cost in a repository that greps its own source during review.
That cost is why the proposal is scoped to the message-forwarding shape only,
not to all 50 `Display` impls.

## 3. One helper that chooses an `io::ErrorKind`

Pattern:
 `io::Error::new(io::ErrorKind::InvalidInput, "...")` and its siblings.

Measured:
 15 sites across 8 files,
11 of them in `package/wg/cli-quicker-exempt`.
C998 is the cluster,
dismissed as boilerplate-trivial.

Design:
 start inside the package that holds 11 of the 15 sites.

```rust
// package/wg/cli-quicker-exempt/src/error.rs
/// One place deciding which `ErrorKind` a failure carries, so the mapping is a
/// reviewable table instead of 11 independent choices.
pub(crate) fn invalidInput(message: impl Into<String>) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidInput, message.into())
}
```

Promote to a shared crate only when a second package needs the same mapping,
which is the reverse of the usual instinct and is deliberate:
at 15 sites across 8 files the shared part is one constructor call,
and the policy worth centralising is which kind maps to which failure,
which is currently only contested inside `wg-quicker-exempt`.

Risk:
 low.
 Falsifier:
 if the 4 sites outside that package disagree about kinds for the same
failure,
 the mapping belongs in a shared crate immediately rather than later.

## 4. A base for shadow-dom elements

Pattern:
 `super(); this.#shadow = this.attachShadow({ mode: 'open', },);`.

Measured:
 22 `attachShadow(` sites across 22 files,
of 25 `extends HTMLElement` declarations.
11 sites are in `done/app` and 11 in `done-postcss`,
which the `done-*` pattern in `slopo.conf.yaml` excludes from indexing and #69 tracks as a
deliberate fork.
C305 is the cluster covering 9 of the `done` sites.

Design:
 a factory rather than a base class,
because the shadow root is stored in a `#private` field per `PPX` and `#private` state is not
inherited usefully.

```typescript
// package/hyperscript/core/src/dom/define-shadow-element.ts
/**
 Wraps a custom-element class so its shadow root exists before the first
 `connectedCallback`, with the mode stated once instead of per component.

 @returns the same class, with `shadow` available as a readonly instance property
 */
export function defineShadowElement<
  const TBase extends abstract new (...args: readonly never[]) => HTMLElement,
>(
  { base, mode = 'open' }: { readonly base: TBase; readonly mode?: ShadowRootMode },
): TBase { ... }
```

The home is a design question this proposal does not settle:
`hyperscript/core` owns the repository's UI primitives and already exports `escapeHtml`,
but it does not currently own DOM element lifecycle,
and adding element bases changes what that package is.
The alternative is a new `package/module/shadow-element`.

Risks:
 web-component upgrade timing,
where a shadow root created in a constructor behaves differently from one created on connect;
the 11 sites in the excluded tree must be migrated in the same pass or the fork diverges further;
and #69's consolidation may make the question moot by removing one tree.
Sequencing:
 decide #69 first,
 then this.

## 5. A null-prototype record helper

Pattern:
 `Object.create(null)` as a map whose keys come from input.

Measured:
 21 sites across 14 files.
C355 is the cluster.

Design:
 the value here is not saving a call,
it is making the security reason explicit and testable.

```typescript
// package/module/null-record/src/null-record.ts
/**
 Builds a record with no prototype, so keys such as `__proto__` and `constructor`
 from untrusted input cannot reach `Object.prototype`.

 @returns an empty record whose only properties are the ones assigned to it
 */
export function nullRecord<K extends string, V>(): { [key in K]?: V } {
  return Object.create(null) as { [key in K]?: V };
}
```

Worth pairing with a lint rule that requires it where the key source is untrusted,
since 21 hand-written sites are 21 chances to use a plain object literal instead.

Risk:
 the typed return hides that prototype methods are absent,
so `record.hasOwnProperty` throws.
The TSDoc must say so,
and the repository's `SYB` rule about destination grammar is the same shape of hazard.

## 6. A comparator combinator

Pattern:
 hand-written multi-key comparators,
ascending and descending,
over strings and numbers.

Measured:
 40 `.localeCompare(` sites across 37 files,
plus the comparator clusters #659 dismissed as coincidental
(C315 ordering by file,
line and name;
C358's three two-key comparators,
one descending so fixes apply back to front).
#659's own caveat 1 proposed `compareBy([...keyFns])` while still dismissing the clusters.

Design,
 following `ST9` on destructured parameters:

```typescript
// package/module/compare-by/src/compare-by.ts
/**
 Builds a comparator over the given key selectors, applied in order, so the
 ordering rule exists once instead of per call site.

 @returns a comparator suitable for `toSorted`, named by `label` so stack traces
   and debugger frames still show what is being ordered
 */
export function compareBy<const TValue>(
  { keys, label = 'compareBy' }: {
    readonly keys: ReadonlyArray<(value: TValue) => string | number>;
    readonly label?: string;
  },
): (left: TValue, right: TValue) => number { ... }
```

Two constraints this repository imposes,
both learned from defects already on file:

- String keys must state whether comparison is locale-aware or by code unit.
   #627 records that `git-policy`'s byte-order comparators disagree between `latin1` and `utf8`,
   so a combinator that hides the choice would hide that class of defect.
   The design should take the string comparison as an explicit argument rather than defaulting it.
- Descending order is load-bearing,
   not stylistic:
   `cli/markdown-lint/src/fix.ts` applies fixes back to front.
   A `direction` per key,
   not a global reverse,
   is what C358's three comparators need.

Risk:
 replacing 40 call sites changes ordering wherever a site relied on `localeCompare`'s locale
sensitivity or on subtraction versus comparison semantics.
Migration must be per site with the ordering asserted before and after,
which is more work than the other seven proposals and is the reason this one is listed last
among the eight.

## 7. One module-id query stripper

Pattern:
 strip a query from a module id before resolving it.

Measured:
 2 sites.
`modulePath` in `git-policy/cli/src/trust/typescript-source-capture.ts` strips at the first `?`
unconditionally;
`stripAttrQuery` in `rolldown-plugin/import-attributes/src/patterns.ts` strips only at
`` `?${ATTR_QUERY_KEY}=` ``.
They share 31 of 38 tokens.
C187 is the cluster.

Design:

```typescript
/**
 Removes a query from a module id at the given marker, so the caller states which
 query it means instead of the helper guessing.

 @returns the id up to the marker, or the id unchanged when the marker is absent
 */
export function stripModuleQuery(
  { id, marker = '?' }: { readonly id: string; readonly marker?: string },
): string { ... }
```

The honest caveat:
at 2 sites in 2 packages,
a new shared package costs more than the duplication.
The propagation argument is that every future id-parsing plugin is a third site,
and each will choose its own marker semantics.
Two cheaper alternatives worth comparing during review:
put the helper in whichever package the next consumer already depends on,
or record the convention in `doc/` and let each package keep its own one-liner.
This proposal is the most likely of the eight to be rejected,
and rejecting it on cost is a defensible outcome.

## 8. A dispatch macro for the forbidden-regex batch entry points

Pattern:
 eight public wrappers that allocate an output vector,
match on the DFA,
and call one kernel or fall back.

Measured:
 8 units sharing 67 of 68 tokens in
`package/forbidden-strings/core/src/regex/batch.rs`,
differing only in the kernel name.
C604 is the cluster,
whose recorded rationale describes the kernels in `dfa/sheng.rs` and `dfa/sheng2.rs`
rather than these wrappers,
which contain no intrinsics and no `cfg`.

Design:

```rust
macro_rules! batch_entry {
    ($name:ident, $kernel:ident) => {
        pub fn $name(&self, lines: &[&[u8]]) -> Vec<bool> {
            let mut out = vec![false; lines.len()];
            match self.engine.table_dfa() {
                Some(dfa) => dfa.$kernel(lines, &mut out),
                None => self.engine.is_match_batch(lines, &mut out),
            }
            out
        }
    };
    ($name:ident, $kernel:ident, const $n:ident) => {
        pub fn $name<const $n: usize>(&self, lines: &[&[u8]]) -> Vec<bool> {
            let mut out = vec![false; lines.len()];
            match self.engine.table_dfa() {
                Some(dfa) => dfa.$kernel::<$n>(lines, &mut out),
                None => self.engine.is_match_batch(lines, &mut out),
            }
            out
        }
    };
}

batch_entry!(batch_sheng, is_match_batch_sheng);
batch_entry!(batch_sheng2, is_match_batch_sheng2);
batch_entry!(is_match_batch_scalar, is_match_batch_scalar);
batch_entry!(is_match_batch_interleaved, is_match_batch_interleaved);
batch_entry!(batch_inter_w, is_match_batch_interleaved_w, const N);
batch_entry!(batch_tight_w, is_match_batch_tight_w, const N);
```

`is_match_batch_bucketed` stays hand-written,
because it adds a length ordering before dispatch,
and `is_match_batch` stays as the fallback both arms call.

Risk:
 the const-generic arm needs verification against the crate's MSRV and against
`macro_rules` hygiene for generic parameters,
which is the one part of this design not yet proven by measurement.
A reviewer should expect a compile check before accepting the shape above.

## The two-sided habit: a policy rather than eight utilities

Forty clusters across 20 concepts in the four dismissal issues are instances of one habit:
write both sides by hand.
22 `*Async` function names today;
10 differential-fuzz adapter pairs across 13 sidecars;
3 complementary-predicate pairs;
`runPipe` against `runPipeAsync` sharing 519 of 575 tokens with 54 async markers as the only
variance;
and eight mirror families
(key handlers,
navigation,
stream pause and resume,
priority providers,
root markers,
column close,
edit conflicts,
single against batch).

No utility fixes this,
because each pair is closed and the propagation is at the level of the habit.
What is missing is a recorded choice.
The repository has already made one instance of it explicitly:
`slopo.ignore.txt` records that a polarity parameter would restate what two names carry.
That reasoning is sound and is worth generalising,
but it is currently written as a comment on a suppression rather than as a rule.

Landed as review rules,
 not `AGENTS.md` rules.
The maintainer's direction on 2026-10-09 was that both drafts were too narrow,
 applying only to
duplicate-code dismissals rather than to every development task,
and that they belong in the project code review skill.
They are now `GRW` and `TWS` in `.agents/skills/project-code-review/SKILL.md`,
under "Duplication and pattern ownership",
generalised so they fire on any diff that adds,
 keeps or approves a repeated shape:

- `GRW` requires a repeated shape to name its owner,
   a shared function,
   base class,
   macro,
   factory,
   lint rule or written convention,
   and rejects "it is only two places" and "it is too small to
   extract" as answers,
   because a pattern instantiated twice templates the third copy.
   Where nothing should own it,
   the reviewer states why the pattern is closed instead.
   Severity is WARNING at two places and BLOCKER when the diff adds a third copy or the shape
   already exists at ten or more sites.
- `TWS` requires two-sided code to record which arrangement applies:
   a shared core,
   one side
   generated from the other,
   or both hand-written on purpose.
   Severity is WARNING for a new pair with no stated choice and BLOCKER when the pair shares roughly
   50 or more tokens of identical body and still has none.

Both carry the differential-fuzz exception the draft called for,
 since identical adapter code on
both sides is what makes an observation difference mean a behaviour difference.
Both codes were checked against `AGENTS.md`,
 the skills directory,
 and
`forbidden-strings.append.txt` and `forbidden-strings.append.local.txt` on 2026-10-09.

The narrower dismissal-side form also landed,
 as rule 9 of the recording convention in
`doc/troubleshooting/slopo-threshold-tuning.md`,
 so an ignore-file entry must name the pattern's
owner,
 state that it is closed,
 or state the policy that accepts the copies.

## Sequencing

Each design is its own issue,
 filed 2026-10-09 and indexed at #665.

1. #667 and #673 are mechanical and independently verifiable,
    so they go first.
2. #668 and #670 are small and local.
3. #666 is the largest mechanical change and migrates package by package,
    with the
   name-against-class comparison run before and after each step.
4. #669 waits on #69's decision about the `done-postcss` fork.
5. #671 goes last and per site,
    because it is the only one that can silently change ordering.
6. #672 is a decision rather than an implementation,
    and rejecting it on cost is a defensible
   outcome.
