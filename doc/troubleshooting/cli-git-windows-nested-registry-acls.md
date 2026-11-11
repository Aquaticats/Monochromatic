# Cli-git Windows nested registry ACLs

Note 2026-11-10T14:38:27-05:00:
the functions this record names left the tree with the TypeScript wrapper.
`ensureRegistryRoot`,
`ensurePrivateRegistryDirectory` and the trust storage they created were deleted in
`642b6d72cb` (refactor(*):
delete the TypeScript Git policy wrapper,
its tasks,
bins and trust workflow,
2026-10-13T17:15:06-04:00),
which removed all 112 files of `package/git-policy/cli/src/trust/`
(`registry-storage.ts`,
`registry-record-preparation.ts` and `windows-acl.host-evidence.ts` among them)
and the hosted trust workflow `.github/workflows/cli-git-trust.yml`.
The trust decision survives as a retired command:
`package/git-policy/cli/src/native/invocation/management.rs:71`
maps `RetiredCommand::Trust` to the command word `trust`,
and `retired_explanation` (`package/git-policy/cli/src/native/invocation/management.rs:93`)
prints that the wrapper now reads its settings file as data and runs no repository-supplied code,
so there is no code execution to approve,
revoke or report.
No Windows registry ACL invariant has a carrier in the tree now:
no tracked file under `package/` contains `ensureRegistryRoot`,
`ensurePrivateRegistryDirectory`,
`TrustStorageError` or `AreAccessRulesProtected`,
and the ACL work this record describes applies to the deleted TypeScript wrapper only.
The trust records older releases wrote are left in place and are no longer read.

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

## Verified workarounds

None.
No consumer-side workaround was measured on an affected release,
and the Fix section above is not one:
it protects each component as it is created,
and a component that already exists is left to the strict verification that refused it.
That is the tradeoff to state beside any workaround a later reader finds:
existing components are never repaired automatically,
so rerunning the same command cannot clear a path an older release created.

## What does not work

### The verifier was too strict

Rejected because per-component protected ACLs are the documented trust-storage invariant.
Weakening verification would permit inherited access to change when an ancestor ACL changes.

### GitHub runner temporary directories were inherently unusable

Rejected because the registry root's ACL was applied and verified successfully in the same temporary tree.
Only recursively created descendants lacked explicit protection.

### POSIX mode bits should secure Windows descendants

Rejected because Windows trust enforcement uses ACLs;
`0o700` creation modes do not establish the required protected discretionary ACL.

## Upstream filing artifact

No upstream report is appropriate:
nothing to file,
nothing to add to an existing thread,
and no draft is kept.

The defect was cli-git's recursive directory creation strategy,
and Windows behaved as its ACL model documents.

### Upstream filing decision

1. **Is it really upstream's fault?**
   No.
   The failure is our directory creation strategy meeting a documented Windows distinction:
   inherited rules can be safe in content while `AreAccessRulesProtected` remains false on each child.
   The distinction is documented Windows behavior,
   not wording and not an architectural restriction.
2. **Can upstream fix it?**
   Not applicable.
   There is no upstream defect to fix;
   the correct caller creates each registry-owned component and protects it before descending.
3. **Are they supporting this use case?**
   Yes,
   the case is ordinary ACL use.
   Per-component protected ACLs are what the Windows registry and directory ACL APIs exist for,
   and the failing run's own temporary tree carried a protected ACL on its registry root.
4. **Would the repo welcome our contribution?**
   Not applicable,
   since there is nothing to contribute.
   No policy question arises because no filing is made.
5. **Will they likely fix it?**
   Not applicable.
   Nothing is broken upstream to fix,
   and this record asserts no leaning either way.
6. **Have we prototyped a minimal fix compatible with their architecture?**
   Not applicable upstream.
   The minimal fix was made in our own code instead,
   as the component-wise primitive the Fix section describes,
   and hosted workflow run `29171565815` verified it on Windows.

The default policy is not to file,
and this record files nothing.
