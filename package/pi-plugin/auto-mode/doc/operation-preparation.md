# Operation preparation

## Purpose and interface

`native/contract/integration/native-batch/handler.mjs` connects the original complete assistant tool group
to prepared dispatch and review.
`native/contract/integration/native-batch/tools.mjs` wraps the actual session tool registry.
The group is prepared before review and executes through its original member identities.

This module exists to prevent reviewing one command or tool definition and executing another.
A model-provided tool name or an argument record is data;
the installed registry supplies the actual executable definition.

## Retained ownership

`native/contract/integration/native-batch/prepared-group.mjs` owns the group's review disposition
and scoped manual confirmation.
`prepared-group-context.ts` publishes native membership once for both automatic and manual review.
Source collection borrows those exact records and the original prepared serialization.
Prepared data alone does not permit execution.

`native/contract/collector/program-rule-batch-sdk-copy-v2/input/prepared-dispatch.mjs`
prepares the complete group,
requests its decision,
and runs those same native preparations.
Root calls and supported nested program calls use this dispatcher.
The generated Pi loop supplies its original preparation,
execution,
and finalization functions through a fixed phase factory.
It replaces the former child turn queues and suspended before-hook gates.

Factory-owned child slots reserve the existing native invocation before preparation starts.
Their terminal continuation retains results even after parent return;
starting another invocation or consuming a script value still requires current ownership.
Withheld members retain native error outcomes without invoking an after hook.
`native/contract/lifecycle/native-program-groups` retains supported nested program lifetimes.
`native/contract/lifecycle/execution-context-carrier` binds tool context and its dependencies.
Each responsibility is needed where callbacks can replace tools,
modify arguments,
or outlive their original session.
The versioned directory names are historical implementation names,
not separate product features.

Every prepared member remains unentered until the group decision permits execution.
Original ordering,
signals,
callbacks,
and completion states remain with the dispatcher.
Freshness failures cannot reuse an established answer for changed inputs.

## Review lifetime

The native handler reads the full review before its operation lifetime closes.
The runtime keeps that actual latest review for diagnostics.
Completed group contexts retain their terminal outcome and reject further execution checks;
they release the full derived review and automatic-review references.
Original source receipts and native member results remain with their existing owners.
This prevents retained execution callbacks from preserving every past collection of derived source views.

`test:native:review-retirement` uses original native membership
and execution owners with visible fixture review storage.
It keeps completed contexts and the actual latest full review alive.
The old output retains 96,805,456 bytes across twelve groups and fails its 32 MiB budget.
A private retirement probe retains 8,792,696 bytes,
preserves terminal outcomes and the latest diagnostic,
rejects execution after closure,
and still rejects a changed latest source.
Complete consumer and lifecycle catalogs remain required after the maintained source rebuild.

## Model projection

`src/review-operation.ts` replaces `bash.command` and `process.start.command`
with locally parsed Bash structure for review.
It retains action,
working directory,
notification configuration,
and other original options.
Other process actions keep their original data shape.
The installed process extension launches its start command through a configured shell
or its existing Bash fallback;
the projection describes Bash syntax and does not establish runtime effects or evaluated expansion values.

Execution retains the original input object and source command.
A parser failure withholds automatic review
while leaving the exact original operation available for scoped confirmation.
The shared shell analyzer owns parsing;
the package no longer carries a pass-through command-analysis wrapper.

## Alternatives and verification

Reviewing calls individually would lose complete-group effects and ordering.
Executing a rewritten model command would change the operation after review.
Treating the process tool name as automatic permission would bypass its command effects.
The retained implementation prepares the actual group and changes only its review representation.

`test:native:judgment-preparation`,
`test:native:phases`,
`test:native:prepared-schedule`,
and `test:native:shell-projection` exercise these contracts.
The shell-projection regression checks provider-facing parent and prepared inputs,
preserved execution input,
malformed start commands,
and non-start process actions.

Task names use `mise run //package/pi-plugin/auto-mode:<task>`.
