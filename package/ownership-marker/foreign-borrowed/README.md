# `@monochromatic-dev/ownership-marker-foreign-borrowed`

Zero-runtime TypeScript ownership marker used by semantic readonly analysis.

```typescript
import type {
  ForeignBorrowed,
  ForeignHostCapability,
} from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

function inspectNode(node: ESTree.Node,): void {
  void node.type;
}

export function acceptForeignAst(
  root: ForeignBorrowed<ESTree.Node>,
): void {
  inspectNode(root,);
}
```

`ForeignBorrowed<T>` records that a mutable value's ownership and type are dictated by a foreign interface.
It does not claim immutability and does not waive mutation analysis.
Direct or transitive caller-observable mutation still requires an accurate `@mutates` contract.

Place the marker only where foreign ownership enters or is deliberately retained.
Do not repeat it on aliases,
properties,
elements,
destructured values,
callback parameters,
iteration bindings,
or every internal helper.
The semantic rule propagates guaranteed provenance through those paths.
A helper parameter inherits foreign provenance only when every owned inbound call supplies wholly foreign mutable state.

`ForeignHostCapability<T>` is narrower authority
for runtime-owned capabilities
whose exact implementation remains unavailable
after source and source-map inference.
It also carries foreign ownership.
Use it only on host boundary inputs such as an extension runtime API or native cancellation capability.
Every callable whose unresolved behavior is admitted by this marker
must document possible caller-observable effects with
`@mutates`.
An ordinary `ForeignBorrowed<T>` never admits unresolved behavior.

Workspace consumers import the package's `/ts` subpath so cross-package resolution targets TypeScript source.
Do not import the package root from another workspace package.

The package contains type declarations,
no runtime code,
and no runtime dependencies.

See
the [foreign-provenance guide](../../../doc/troubleshooting/oxlint-prefer-readonly-foreign-provenance.md)
for the analyzer model,
negative mixed-origin case,
and migration verification.
