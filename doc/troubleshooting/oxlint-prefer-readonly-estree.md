# Oxlint 1.65 native readonly allowlists missed bundled ESTree symbol names

## Symptom

Oxlint's former native `typescript/prefer-readonly-parameter-types` rule reported parameters typed as
`ESTree.Node`,
`ESTree.Function`,
and related `@oxlint/plugins` declarations even when their surface names appeared in the configured package allowlist.

The installed declaration bundle exposed renamed internal symbols such as `Node$1` while re-exporting them through the
`ESTree` namespace.
The native matcher compared the resolved symbol name rather than the authored surface spelling,
so an allow entry for `Node` did not match `Node$1`.

## Root cause

The deciding tsgolint code first matched a type's resolved alias or symbol name,
then applied package,
file,
or lib provenance.
A declaration-bundler rename therefore failed before the package gate could help.

The `@oxlint/plugins` declaration bundle contained both top-level oxc-estree declarations and namespace-exported ESTree
declarations.
The bundler disambiguated collisions with suffixes such as `$1`.
Those suffixes were implementation artifacts,
not stable public type names.

This was a native-rule allowlist problem.
It is not the current semantic rule's resolution path.

## Verified resolution

The repository retired:

- `package/config/oxlint/src/rule/prefer-readonly-parameter-types.ts`;
- `package/config/oxlint/src/rule/prefer-readonly-parameter-types.allow-pkg.ts`.

`package/config/oxlint/src/rule/prefer-readonly-parameter-types.ts` is not in the working tree and
no commit in this repository carries it.
`package/config/oxlint/src/rule/prefer-readonly-parameter-types.allow-pkg.ts` is not in the working
tree and no commit in this repository carries it.

Both files are absent in the current tree.
The replacement rule is
`prefer-readonly-parameter-type/prefer-readonly-parameter-types` in
`package/oxlint-plugin/prefer-readonly-parameter-type`.

Oxlint visitor nodes are foreign-owned values supplied by the plugin host.
Actual visitor ingress can use
`ForeignBorrowed<ESTree.Node>` from `@monochromatic-dev/ownership-marker-foreign-borrowed/ts` without claiming that the
AST is immutable.
Internal aliases,
properties,
elements,
and helper parameters must inherit that origin through semantic provenance rather than repeat the marker.

The current provenance implementation follows property and element access,
destructuring,
aliases,
audited callbacks,
synchronous iteration,
and owned calls.
A helper is foreign only when every owned inbound call supplies wholly foreign mutable state.
The completed migration keeps markers only at parser ingress,
retained AST storage,
and exported AST-emission boundaries.

## Verification

A direct path check confirmed that both retired configuration files are absent.

Current semantic-plugin acceptance uses:

```sh
mise run //package/oxlint-plugin/prefer-readonly-parameter-type:lint:types
mise run //package/oxlint-plugin/prefer-readonly-parameter-type:lint:oxlint
mise run //package/oxlint-plugin/prefer-readonly-parameter-type:test:unit
```

All three tasks pass after the invoked-capability and foreign-provenance changes.
The unit suite covers exact marker identity,
foreign and owned mixed inbound calls,
readonly classification,
intrinsic declaration provenance,
and external-consumer publication.

Workspace package imports use `/ts` source subpaths.
Commit `300ccac29` corrects 58 standalone marker imports;
current plugin changes also use the `/ts` spelling.

## What does not work

### Add bundler suffixes to a type-name allowlist

Names such as `Node$1` and `Function$1` are declaration-bundle artifacts that can change when dependencies rebuild.
A type name also says nothing about ownership or effects at one function boundary.

### Enumerate every ESTree union member

The approach couples configuration to the complete external AST union and still cannot prove what a function does with a
node.

### Project the AST through `ReadonlyDeep`

ESTree includes identity-sensitive,
callback-bearing,
and visitor-compatible shapes.
A deep projection can break required assignability while still failing to prove semantic observation.

### Mark every AST descendant `ForeignBorrowed`

That hides where the host-owned value entered and can conceal a helper that also receives an ordinary owned mutable
value.
Only host callback ingress or deliberate retained storage is a boundary.

### Match the authored namespace spelling only

`ESTree.Node` is source syntax,
not exact declaration identity.
Semantic checks must resolve declaration provenance rather than trust surface spelling.

## Upstream filing artifact

### Upstream filing decision

1.  **Is it really upstream's fault?
    ** The old mismatch resulted from a native matcher contract combined with ordinary declaration-bundler renaming.
2.  **Can upstream fix it?
    ** Upstream could broaden alias matching,
    but that would change the retired rule rather than this repository's active semantic rule.
3.  **Are they supporting this use case?
    ** The native rule supported type-name allow specifiers,
    not project-specific ownership provenance.
4.  **Would the repo welcome our contribution?
    ** No current repository dependency requires that native-rule change.
5.  **Will they likely fix it?
    ** No prediction is needed because the repository no longer consumes the affected path.
6.  **Have we prototyped a minimal fix compatible with their architecture?
    ** The project-owned TypeScript 7 bridge,
    exact declaration provenance,
    ownership marker,
    and fixed-point propagation are implemented and tested.

Nothing should be filed upstream.

## Source audit boundary

The original investigation read the installed `@oxlint/plugins` declarations and tsgolint matcher source,
then reproduced package,
file,
and bare-name allow forms.
The current update checked the retired configuration paths,
semantic marker implementation,
provenance fixtures,
and current plugin verification.
A repository search found the branded-nesting and authoring-identity documents;
they cover different mechanisms.
A root `.out-of-scope` search for readonly,
Oxlint,
TSDoc,
and foreign-ownership topics found no applicable entry.
