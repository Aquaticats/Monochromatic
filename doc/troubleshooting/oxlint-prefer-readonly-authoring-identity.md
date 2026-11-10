# Semantic readonly checks for identity-preserving policy authoring APIs

Note 2026-11-09T16:55:17-05:00:
these citations in this record name paths that no longer exist.
`package/git-policy/api/src/policy-types.ts` left the tree on 2026-10-13 in `a2b56c548c`.
`package/git-policy/api/` is not in the tree;
its files were deleted on 2026-10-13 in `a2b56c548c`.
`package/git-policy/api/src/authoring.ts` left the tree on 2026-10-13 in `a2b56c548c`.
`package/git-policy/cli/src/api/authoring.ts` left the tree on 2026-10-13 in `642b6d72cb`.
`package/git-policy/cli/src/api/` is not in the tree;
its files were deleted on 2026-10-13 in `642b6d72cb`.

## Symptom

Cli-git's authoring helpers preserve caller-provided declarations by identity:

```typescript
const options = definePolicyOptions(schema);
const policy = definePolicy(definition);
```

A shallow `Readonly<T>` did not satisfy the former native
`typescript/prefer-readonly-parameter-types` rule.
The values contain Valibot schemas,
callbacks,
and generic input/output positions,
so their complete structures are not deeply readonly.

The same issue appears in internal policy registries and adapters that retain executable callbacks for sequencing and
failure-path tests.

## Root cause

These APIs make an ownership promise,
not a structural immutability promise.
The helper receives a caller-owned runtime capability and must return or store the same identity.
Cloning,
freezing,
or projecting the value would change the API contract.

Valibot `1.4.2` defines `GenericSchema` as an alias of `BaseSchema` in its published `dist/index.d.mts`.
That contract includes callable behavior and generic positions whose effects are not represented by `Readonly<T>`.
Cli-git's `PolicyDefinition` in `package/git-policy/api/src/policy-types.ts` combines a schema with an executable
policy callback.
That path is gone:
commit `a2b56c548c` (2026-10-13) deleted the four git-policy authoring packages,
`package/git-policy/api/` among them.
No TypeScript `PolicyDefinition` remains;
the policy surface is now the Rust tree under `package/git-policy/cli/src/native/policy/`.

A type-name allowlist cannot prove that a particular function observes the capability without invoking or mutating it.
Conversely,
treating every method as readonly would hide genuine mutable map,
set,
and class state elsewhere.

## Verified resolution

The public authoring boundary states foreign ownership exactly once:

```typescript
export function definePolicyOptions<const TInput, const TOutput>(
  schema: ForeignBorrowed<GenericSchema<TInput, TOutput>>,
): GenericSchema<TInput, TOutput> {
  return schema;
}

export function definePolicy<
  const TOptions,
  const TName extends string,
>(
  definition: ForeignBorrowed<PolicyDefinition<Readonly<TOptions>, TName>>,
): PolicyDefinition<Readonly<TOptions>, TName> {
  return definition;
}
```

The corresponding sources are:

- `package/git-policy/api/src/authoring.ts`;
- `package/git-policy/cli/src/api/authoring.ts`.

`package/git-policy/api/src/authoring.ts` was removed on 2026-10-13 in `a2b56c548c` (refactor(*):
delete the dependent-version and configuration differentials and the four git-policy authoring
packages);
no file in the working tree carries it now.
`package/git-policy/cli/src/api/authoring.ts` was removed on 2026-10-13 in `642b6d72cb`
(refactor(*):
delete the TypeScript Git policy wrapper,
its tasks,
bins and trust workflow);
no
file in the working tree carries it now.

Both are gone:
`a2b56c548c` (2026-10-13) deleted `package/git-policy/api/`,
and `642b6d72cb` of the same day deleted `package/git-policy/cli/src/api/`
with the TypeScript wrapper.

`ForeignBorrowed` records that the object and its reachable capabilities remain caller-owned.
It does not claim structural immutability.
The semantic rule still verifies direct mutations,
callback invocation,
unknown calls,
and transitive effects.
Proven caller-observable effects permit mutable types without requiring `@mutates`.
A present contract must remain accurate,
and unresolved possible effects require complete contracts.

Descendant properties,
aliases,
callback parameters,
and internal helpers do not repeat the marker.
Guaranteed foreign provenance flows through property and element access,
destructuring,
owned calls,
audited callbacks,
and synchronous iteration.
A helper inherits that provenance only when every owned inbound call supplies wholly foreign mutable state.

## Verification

The original native-rule investigation established that:

- `Readonly<GenericSchema<TInput, TOutput>>` still reported;
- `Readonly<PolicyDefinition<Readonly<TOptions>, TName>>` still reported;
- adding `GenericSchema` and resolved `BaseSchema` names to a package allowlist did not fix nested behavior.

The current source check is:

```sh
rg --line-number "ForeignBorrowed<GenericSchema|ForeignBorrowed<PolicyDefinition" \
  package/git-policy/api/src/authoring.ts \
  package/git-policy/cli/src/api/authoring.ts
```

That check no longer resolves:
both files were deleted on 2026-10-13 (`a2b56c548c` for `package/git-policy/api/`,
`642b6d72cb` for `package/git-policy/cli/src/api/`).
The recorded result below stands as the observation it was.
Both authoring mirrors placed the marker on the actual ingress parameter while they existed.
The full cli-git semantic migration remains an active package in
`doc/planning/replace-prefer-readonly-parameter-types.md`;
this document does not claim that every cli-git finding is resolved.

The semantic rule's provenance fixtures separately cover:

- property and element descendants;
- nested destructuring and aliases;
- owned helper calls;
- audited array callbacks;
- synchronous `for...of` elements;
- mixed foreign and owned inbound paths.

## What does not work

### Wrap the capability in `Readonly`

`Readonly` changes only surface property modifiers.
It does not rewrite nested schemas,
callbacks,
or behavioral capabilities,
and it can falsely suggest semantic immutability.

### Add schema aliases to a global allowlist

A type name does not establish ownership or effects at one call boundary.
It would also exempt unrelated uses of the same capability type.

### Clone or deeply freeze the declaration

The authoring API promises exact identity.
Transforming the value would violate that contract and could change third-party schema behavior.

### Treat every method as readonly

That broad setting can hide real mutation through maps,
sets,
and stateful capabilities.

### Mark every descendant `ForeignBorrowed`

Repeating the marker obscures where foreign ownership entered and can hide a helper that also receives ordinary owned
mutable input.
The semantic provenance index must carry the boundary fact instead.

### Use `@mutates` when no caller-observable effect exists

A contract must describe real effects,
not an analyzer limitation.
Identity helpers that only return their input need no invented mutation statement.

## Upstream filing artifact

### Upstream filing decision

1. **Is it really upstream's fault?**
    No. Structural readonly rules correctly cannot infer this repository's ownership
   boundary from a third-party capability type.
2. **Can upstream fix it?**
    Not without a project-specific ownership and effect model.
3. **Are they supporting this use case?**
    The native rule supports structural type checks,
   not exact identity-preserving capability contracts.
4. **Would the repo welcome our contribution?**
    No generally applicable upstream change was identified.
5. **Will they likely fix it?**
    Not applicable because no upstream defect is claimed.
6. **Have we prototyped a minimal fix compatible with their architecture?**
    The project-owned marker,
   TypeScript 7 semantic bridge,
   provenance propagation,
   and verified effect contracts are the implemented fix.

Nothing should be filed upstream.

## Source audit boundary

The investigation followed the published Valibot declaration to cli-git's public authoring and policy-definition
sources.
A repository search found the related branded-nesting and ESTree documents;
they cover different type-resolution mechanisms.
A root `.out-of-scope` search for readonly,
Oxlint,
TSDoc,
and foreign-ownership topics found no applicable entry.
