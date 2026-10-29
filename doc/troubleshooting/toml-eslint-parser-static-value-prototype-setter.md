# `toml-eslint-parser` 1.0.3 `getStaticTOMLValue` loses `__proto__` keys and can create Date impostors

## Symptom

A long TOML fuzz campaign failed in the semantic-equality oracle after 1,289 generated cases.
Fast-check minimized the input to:

```toml
# Minimal failing shape
value = { "__proto__" = 1979-05-27T07:32:00Z }
```

The thrown error was:

```text
TypeError: this is not a Date object.
```

The complete discovered counterexample used the same inline-table key:

```toml
00zzz = { "__proto__" = 1979-05-27T07:32:00Z }
[_aB-9_B]
[-B0zaBz]
[a9z09-a]
[0-a09B9-9]
[-z9]
```

This is not a TOML editor materialization defect.
It occurs when the fuzz oracle calls upstream `getStaticTOMLValue`.
The package-owned `tomlGetValue` path preserves `__proto__` as an own property on an ordinary object.

## Root cause

The installed dependency is `toml-eslint-parser` 1.0.3.
Its lockfile entry is `pnpm-lock.yaml:6569-6570`.
The corresponding upstream release is tag `v1.0.3`,
commit `6008e35499342f1116262162d16dbcab0c84760c`.

Upstream creates root and inline-table projections as ordinary object literals.
`toml-eslint-parser` `src/utils.ts:92-125` contains:

```typescript
Program(node: TOMLProgram, baseTable: TOMLTableValue<V> = {}) {
  return resolveValue(node.body[0], baseTable);
},

TOMLInlineTable(node: TOMLInlineTable) {
  const table: TOMLTableValue<V> = {};
  for (const body of node.body) {
    resolveValue(body, table);
  }
  return table;
},
```

A key-value node resolves its authored key and delegates to `set`.
`toml-eslint-parser` `src/utils.ts:104-107` contains:

```typescript
TOMLKeyValue(node: TOMLKeyValue, baseTable: TOMLTableValue<V> = {}) {
  const value = resolveValue(node.value);
  set(baseTable, resolveValue(node.key), value);
  return baseTable;
},
```

The final write uses ordinary bracket assignment.
`toml-eslint-parser` `src/utils.ts:231-251` ends with:

```typescript
function set<V>(baseTable: TOMLTableValue<V>, keys: string[], value: any) {
  let target: TOMLTableValue<V> = baseTable;
  // Intermediate-key handling omitted.
  target[last(keys)!] = value;
}
```

For an ordinary object,
assignment to `target['__proto__']` invokes the inherited legacy prototype setter instead of creating an own data
property.
A primitive value is silently discarded by that setter.
An object value becomes the target's prototype.
When the value is a parsed TOML datetime,
the table inherits from a `Date` instance and therefore satisfies `instanceof Date`,
but the table itself has no Date internal slot.
Calling `Date.prototype.getTime` with that table as its receiver throws the observed error.

The earlier fuzz-oracle claim that upstream projection lossiness was always symmetric was wrong.
Both projections can lose the key,
but a prototype change can also alter runtime classification or silently make two lossy objects compare equal.

The package-owned materializer does not use ordinary assignment.
`package/toml-edit/core/src/value-materialize.ts:78-92` creates a fresh object with a computed property:

```typescript
return {
  ...source,
  [key]: value,
};
```

Computed object properties create own data properties,
including for `__proto__`.

## Verification

The verified dependency inputs are:

- installed `toml-eslint-parser` 1.0.3;
- npm integrity `sha512-A5F0cM6+mDleacLIEUkmfpkBbnHJFV1d2rprHU2MXNk7mlxHq2zGojA+SRvQD1RoMo9gqjZPWEaKG4v1BQ48lw==`;
- upstream tag `v1.0.3` at commit `6008e35499342f1116262162d16dbcab0c84760c`;
- Node 26.5.0.

The permanent reproduction is in
`package/toml-edit/core/src/fuzz/round-trip.property.unit.test.ts`.
It verifies all of these facts for the minimized source:

- `getStaticTOMLValue` does not create an own `__proto__` property;
- the projected inline table inherits from a `Date` instance;
- `tomlGetValue` does create the own property;
- the package-owned result retains `Object.prototype` as its prototype.

Run the bounded reproduction with:

```sh
mise run //package/toml-edit/core:test:unit
```

Run the campaign boundary with:

```sh
mise run //package/toml-edit/core:fuzz
```

### Inputs that project cleanly

- ordinary bare and quoted keys;
- datetimes stored under ordinary keys;
- nested arrays and tables without `__proto__` segments;
- the same `__proto__` document through package-owned `tomlGetValue` materialization.

### Inputs excluded from the upstream static oracle

- top-level `__proto__` keys;
- dotted keys containing a `__proto__` segment;
- inline-table `__proto__` keys;
- any value kind under those keys,
because primitives are lost and objects can replace the projection prototype.

## Verified workarounds

### Classify the AST before using the oracle

`staticSemanticOracleSupports` in `package/toml-edit/core/src/fuzz/equality.ts` parses the source,
walks authored key nodes iteratively,
and rejects a source containing any `__proto__` key segment before calling `getStaticTOMLValue`.

Round-trip and metamorphic properties use fast-check `pre` for that classification.
This keeps the generated input in the campaign while discarding only a case the external oracle cannot represent.

Tradeoff:
the upstream semantic oracle provides no assertion for those sources.
The deterministic package-owned materialization check covers the excluded key separately.

### Use package-owned materialization for editor behavior

Production reads use `tomlGetValue` and the computed-property materializer,
not `getStaticTOMLValue`.
This preserves prototype-shadowing TOML keys as own properties.

Tradeoff:
this does not change the separately re-exported upstream utility or make it suitable as an oracle for those keys.

## What does not work

### Treat both upstream projections as symmetrically lossy

This can hide a dropped key when both sides lose it.
A datetime also produces a Date-prototype impostor that can throw during equality checks.

### Make the equality helper tolerate the Date impostor

Avoiding `Date.getTime` would prevent the exception but would not restore the lost TOML key.
The property could pass after comparing two already-corrupted projections.

### Patch or fork the dependency inside this migration

The repository does not own `toml-eslint-parser`,
and the user identified this as a known upstream boundary that this work must document rather than fix.
The local package already has a prototype-safe product path.

## Upstream filing artifact

### Duplicate search

A GitHub issue search for `prototype pollution`,
`__proto__`,
and `getStaticTOMLValue` in `ota-meshi/toml-eslint-parser` returned no matching issue.
A broader issue listing confirmed the tracker query ran and returned unrelated open and closed issues.

### Upstream filing decision

1. **Is it really upstream's fault?
   ** Yes for `getStaticTOMLValue`.
   Ordinary object assignment cannot represent the valid TOML key faithfully.
2. **Can upstream fix it?
   ** Yes.
   Upstream could create own data properties explicitly or use a representation without the inherited setter.
   This repository cannot change the installed package implementation without carrying a fork or patch.
3. **Are they supporting this use case?
   ** Yes.
   The upstream README's direct-usage example presents `getStaticTOMLValue` as the conversion from parsed TOML to a
   nested JavaScript value.
4. **Would the repo welcome our contribution?
   ** Unknown but not prohibited.
   Release `v1.0.3` contains no `CONTRIBUTING.md`,
   issue template,
   pull-request template,
   or AI-assistance policy;
   its README links the issue tracker but states no contribution restriction.
5. **Will they likely fix it?
   ** Unknown.
   No matching issue or explicit won't-fix statement was found.
6. **Have we prototyped a minimal upstream fix compatible with their architecture?
   ** No.
   The authorized scope is documentation and a consumer-side oracle classification,
   not modification of the third-party implementation.

No upstream issue,
comment,
or patch is sent from this work.
The durable artifact is this diagnosis plus the minimized local reproduction.
