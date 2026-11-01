# CIDR library for `wg-allowedips`

## Status

Accepted on 2026-07-28.
The user authorized adoption and implementation after reviewing the completed technology evaluation.

## Decision

Adopt `cidr-tools` with a pnpm catalog floor of `>=12.1.3` for
`@monochromatic-dev/wg-cli-allowedips`.
The exact validated release is `cidr-tools` 12.1.3 with `ip-bigint` 9.0.7.

The governing evaluation is
[`doc/audit/tech-wg-allowedips-cidr-library-vet-2026-07-28.md`](../audit/tech-wg-allowedips-cidr-library-vet-2026-07-28.md).
It used `choosing-technology` at commit `a05818ad70a40e5769a36de669697ba109891b31`,
SHA-256 `393eb68c5b2b2f7b16c8f7f90c100fb8be43eefa4501511360cd0572e4ae8087`.

## Constraints and weights

The dependency must provide exact dual-stack union-minus-union,
a minimized sorted CIDR cover,
inspectable JavaScript or TypeScript,
compatible licensing,
Node 22 support,
and no runtime native,
Wasm,
prebuilt,
downloaded,
or install-lifecycle boundary.

The frozen score weights were:

- interface fit and production-code footprint:
  5;
- validation clarity and error ergonomics:
  5;
- runtime dependency surface and human auditability:
  4;
- upstream test quality:
  3;
- maintenance and release hygiene:
  1.

## Ranking

The validated finalist ranking was:

1. `cidr-tools` 12.1.3:
   `63 / 72`,
   or 87.5%;
2. `@h3mantd/ip-kit` 1.1.0:
   provisional `35 / 72`,
   or 48.6%.

`cidr-tools` won because `excludeCidr(allowed, disallowed)` directly matches the required operation,
its audited runtime surface is smaller,
its validation ownership is clearer,
and its exact release has stronger relevant tests and cross-platform CI.
The order survived every required one-at-a-time sensitivity test.

Other candidates did not enter the numeric ranking:

- `fast-cidr-tools` 0.3.4 returns an incorrect set for one exclusion spanning multiple disjoint bases;
- `ip-num` 1.6.1 runs an install lifecycle and fails a hard constraint;
- `ip-address` 10.3.1 has no collection union,
  subtraction,
  or minimal CIDR-cover engine.

## Integration boundary

Production code must call `excludeCidr(allowedNetworks, disallowedNetworks)` directly.
It must not add a merge,
interval,
or per-family subtraction layer.

`cidr-tools` deliberately has rudimentary validation.
Before subtraction,
the CLI must:

- validate the original address text with `isIP` from `node:net`;
- parse CIDRs with `parseCidr`;
- reject IPv4 prefixes above 32;
- reject IPv6 prefixes above 128.

The vet report exercised this composed validation boundary against valid and malformed dual-stack inputs.

## Migration

Implementation adds `cidr-tools: '>=12.1.3'` to the pnpm catalog and uses `catalog:` in the new package.
There is no existing package or persisted state to migrate.
The implementation follows [`doc/planning/wg-allowedips.md`](../planning/wg-allowedips.md).

## Exit and rollback

Before release,
rollback removes the new package and catalog entry.
After release,
rollback restores the preceding package version and lock resolution,
then reruns the full package build,
test suite,
and built-CLI consumer verification.
No output format or persistent state requires data migration.

Replace `cidr-tools` only after a new evaluation validates another candidate at the same consumer boundary.
Do not substitute a parser-only library plus project-owned interval arithmetic.

## Revisit triggers

Reopen the evaluation when:

- a resolved `cidr-tools` version no longer passes the fixed dual-stack subtraction vectors;
- Node 22 or required operating-system support regresses;
- the runtime graph gains native code,
  Wasm,
  downloaded artifacts,
  or install lifecycle;
- license compatibility changes;
- the validation or direct subtraction API changes;
- maintenance stops producing fixes or releases for a material correctness defect;
- a direct-operation alternative materially reduces consumer code without weakening validation clarity.
