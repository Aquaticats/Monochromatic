# Cli-git Windows nested registry ACLs

## Symptom

After Windows Git resolution was fixed,
hosted run `29169096258` reached the MJS trust lifecycle but failed normal trust writes with:

```text
TrustStorageError: Unsafe Windows trust ACL: ...\registry\records
```

Linux and macOS passed the same strict and relaxed trust suites.

## Diagnosis

`ensureRegistryRoot` protected only the registry root.
Record preparation then called recursive `mkdir` for a deep identity path and immediately verified every component.

On Windows,
new child directories inherit allowed access rules from the protected root,
but inherited children do not report `AreAccessRulesProtected` themselves.
Cli-git's verifier intentionally requires every registry-owned component
to disable inheritance and contain only the current account and built-in administrators.

Recursive creation therefore produced intermediate directories such as `records`,
filesystem identity,
and `path` without applying cli-git's explicit ACL script to each component.
Verification correctly rejected those intermediates.

## Root cause

The creation path assumed protecting an ancestor made every descendant satisfy the stronger per-component invariant.
That assumption is false for Windows ACL protection metadata:
inherited rules can be safe in content while `AreAccessRulesProtected` remains false on each child.

## Fix

`ensurePrivateRegistryDirectory` now walks missing descendants in order.
For each newly created component it:

- creates only that component;
- applies the private Windows ACL immediately;
- continues to the next component only after protection succeeds.

Existing components are not repaired automatically.
They proceed to the existing strict metadata and ACL verification,
so a pre-created unsafe path still fails closed.

Both MJS-specific and generic TypeScript record preparation
use this component-wise primitive before creating writer locks or candidate snapshots.

## Verification

The existing trust-service suite passes on Linux after the storage change.
Final cross-platform workflow run `29171565815` passed required Windows user-boundary verification for:

- strict and relaxed MJS records;
- strict and relaxed TypeScript records;
- trust paths and concurrency;
- explicit ACL broadening rejection.

## Rejected hypotheses

### The verifier was too strict

Rejected because per-component protected ACLs are the documented trust-storage invariant.
Weakening verification would permit inherited access to change when an ancestor ACL changes.

### GitHub runner temporary directories were inherently unusable

Rejected because the registry root's ACL was applied and verified successfully in the same temporary tree.
Only recursively created descendants lacked explicit protection.

### POSIX mode bits should secure Windows descendants

Rejected because Windows trust enforcement uses ACLs;
`0o700` creation modes do not establish the required protected discretionary ACL.

## Upstream status

No upstream report is appropriate.
Windows exposed the expected distinction between inherited access rules and protected child ACLs.
The defect was cli-git's recursive directory creation strategy.
