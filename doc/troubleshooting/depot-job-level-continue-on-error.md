# Depot CI job-level continue-on-error permits dependencies but retains failed aggregate verdicts

## Symptom

Native `kjf78stmkb` and GitHub `39122132487`
execute the same ordinary allowed-failure fixture at
`f446852c42d4a382ba7931e6730a17cb3d8308e0`.
Both consumers observe `needs.allowed_failure.result` as `success`.
The native run aggregate and its GitHub App suite conclude failure;
the GitHub workflow concludes success.
There is no timeout or cancellation in this isolated workflow.

The deliberate failed operation emits:

```text
IntentionalJobPolicyFailure [Error]: Intentional ordinary failure in a job with job-level continue-on-error true
```

The [public receipt](../audit/depot-job-policy-history-2026-10-18.evidence.json)
retains the paired job observations,
native App checks/suite,
original ZIPs and exact source.

## Source contract and unknown backend cause

The deployed [compatibility guide](https://depot.dev/docs/ci/compatibility.md)
marks `jobs.<job_id>.continue-on-error` supported,
with purpose `Allow job failure`.
The installed CLI `2.102.23`,
source `1f96d6d488f938de97ea640b115e9567ebfd6fbf`,
agrees in `pkg/ci/compat/rules.go:229`:

```text
# depot/cli, pkg/ci/compat/rules.go
Continue-on-error is supported.
```

That is a compatibility-analyzer declaration,
not the private runtime's aggregate implementation.
The normalized dependency result is proven;
the aggregate behavior differs from the paired GitHub workflow.
No inspected public source identifies why the backend or check publisher retains the failed aggregate.
This is an observed compatibility boundary,
not a claim about an established backend mechanism.

## Verification

The first comparison has one intentionally failed job with job-level continuation,
one passing downstream observer,
and an independent passing read-only history job on native CI.
Its original archive shows the intended failure before the consumer's successful observation.
The paired GitHub workflow has the same failed operation and passing observer.
Both upload original JSON receipts with the same harness identity.

An additional native control removes the observer's explicit `always()` condition.
Its workflow is `.depot/workflows/ordinary-job-default-consumer.yml`,
at `714a3cad36c20da75969454fe33f28dab7770e9b`.
Run `pfzfjrqdpq` still executes the default downstream observer successfully.
Its raw aggregate remains failed.
This establishes actual default dependency eligibility,
rather than inferring it from the normalized result alone.

## Verified workaround

For an advisory job containing one failure-prone operation,
the isolated `.depot/workflows/advisory-step-result.yml`
moves continuation to that step and leaves job-level continuation unset.
Run `lrhfzwrwl2` finishes its sole job and workflow successfully.
The observer records `steps.advisory.outcome` as `failure`
and `steps.advisory.conclusion` as `success`.

```yaml
# .depot/workflows/advisory-step-result.yml, isolated research workflow excerpt
steps:
  - id: advisory
    continue-on-error: true
    run: mise run //.depot/research/native-semantics:policy-fail
    working-directory: .depot/research/native-semantics
  - run: mise run //.depot/research/native-semantics:policy-step-consumer
    working-directory: .depot/research/native-semantics
```

The failure helper's diagnostic retains its original job-level wording in this control.
The committed workflow determines the actual step-level setting.
The tradeoff is changed continuation granularity:
later ordinary steps can execute after that operation fails.
This is not a generic substitute for a job with multiple dependent failure-prone operations.
The successful advisory workflow does not erase the separate deliberately failed workflow at the same source.

## What does not work

The earlier broad semantics workflow also contains a cancelled timeout job.
Its failed aggregate alone could not isolate ordinary job continuation.
That confounder is excluded in this paired comparison.
An `always()` observer alone also cannot prove default downstream eligibility;
the separate default-condition run provides that evidence.

A passing downstream receipt does not establish a passing native aggregate.
Required-check design must use the actual named check conclusions and failure controls.
No production gate or routing was changed in this investigation.

## Upstream filing decision

No upstream message was sent.
The retained evidence is suitable for a repository-local compatibility decision.

- Responsibility:
  the difference lies at the native aggregate/check boundary;
  the private implementation cause is unknown.
- Fixability:
  a provider change may be possible,
  but public source does not permit a verified backend patch.
- Supported use:
  the deployed compatibility guide explicitly marks job-level continuation supported.
- Contribution policy:
  no external report is authorized by this research task.
- Maintainer intent:
  the guide promises the feature;
  no deciding source establishes intent for this aggregate difference.
- Minimal patch:
  the owned step-level consumer workaround is cloud-verified;
  no speculative backend patch is claimed.
