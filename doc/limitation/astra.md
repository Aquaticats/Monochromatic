# Astra limitations

Observed failures from repository work,
with concrete evidence and corrective behavior.
These records describe incidents,
not measured failure rates across all Astra sessions.

## Automatic-mode migration scope drift on 2026-10-14

### Intended work

Migrate Pi automatic mode while keeping its purpose:
check model-initiated tool calls against applicable instructions and authorization.
The proposed tool operation,
its arguments and effects,
and the execution decision are the product boundary.

### Expanded an implementation mechanism into a requirement

The agent introduced complete provenance and field accounting for the API request sent to the primary model.
It then made a whole-request interpretation and derivation proof an independent prerequisite for tool approval.
The model request was useful context for interpreting instructions;
that did not make every provider protocol field a separate approval subject.

This expansion produced an observable mismatch.
An isolated configured Hyper host emitted `thinking: { type: "enabled" }`.
The built request-envelope checker rejected the captured request with
`Request field thinking has no supported native instruction-use derivation`.
The same captured request without `thinking` passed that checker.
The agent initially treated this as another provider-coverage task,
instead of questioning why that metadata could affect tool approval.

The user corrected the scope explicitly:

> Our auto-mode is only meant to check model-initiated tool calls.

The implicated implementation was
`package/pi-plugin/auto-mode/native/contract/integration/native-batch/request-derivation-envelope.ts`
and the independent obligations in
`package/pi-plugin/auto-mode/src/request-material-obligations.ts`.
The [migration handover](../handover/pi-auto-mode-current-frontier.md#tool-call-scope-correction)
records the original capture and correction.

### Verified the detour more thoroughly than the intended feature

The agent accumulated source-ownership checks,
serializer contrasts,
omission controls,
model-format experiments,
and supporting documentation while installed end-to-end tool approval remained unfinished.
Passing those checks demonstrated the implemented mechanisms;
it did not establish that the mechanisms were necessary or that the user's feature worked.

The agent also reported that Luna admitted 29 of 84 retained answers at the selected `0.60` cutoff
as a delivery obstacle before reassessing whether the expanded question set matched the intended scope.
That count was a measurement of the agent's assessment design,
not a measure of product completion or decision accuracy.

### Reported progress against a moving target

When asked how much work remained,
the agent listed provider-request coverage as a delivery gap.
After the user rejected that scope,
the agent acknowledged that the core was built and installed but working end-to-end approval was still unfinished.
It could not provide a defensible duration estimate.

The failure was not an incorrect numerical deadline.
It was repeatedly presenting implementation progress without a stable,
user-grounded completion boundary.
This obscured the remaining delivery risk and made the user correct the objective again.

### Corrective behavior

- State the operation being authorized before adding a prerequisite.
  Every new blocking condition must address a concrete way that operation could violate applicable instructions.
- Keep contextual evidence separate from the authorization subject.
  Provider metadata is not itself a model-initiated tool call.
- Exercise the installed host's tool-call behavior before expanding supporting proof machinery.
  Passing internal tests cannot substitute for approval,
  denial,
  human approval,
  and cancellation at the user boundary.
- When a correction invalidates an abstraction,
  remove its production dependencies and reassess related tests and progress claims.
  Do not merely add another supported case to the rejected design.
- Report completion against the intended feature.
  Label unresolved runtime behavior and unknown effort plainly;
  source inventories and test counts are not completion percentages.

### Recovery status

Commit `3ec320b8fa` removes whole-request admission selection and its independent obligation
from the native tool-policy compiler and evaluator.
Commit `09bf9f9080` removes provider-field derivation from the native observer path.
The configured Hyper host passes isolated permission,
prohibition,
and unspecified-policy controls with no whole-request question.
Fresh-confirmation integration subsequently passes scripted Allow,
Stop,
denial,
and cancellation through the original RPC host.
Those checks use authored semantic answers;
they do not establish live interpretation quality or genuine human approval.
The scope correction is recorded;
this entry does not claim the migration is complete.
