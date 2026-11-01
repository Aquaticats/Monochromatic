# Isolate auto-mode failures to proposed tool operations

## Status and purpose

Proposed on 2026-10-15 after reproduced provider-switch and tree-navigation failures.
The user identified the recurring breakages as an architectural problem.
This proposal revises the integration scope recommended in the
[whole-system review](pi-auto-mode-whole-system-review.md).
The user accepted alternative A, "Replace the integration", on 2026-10-15.
The [accepted decision](../decision/pi-auto-mode-host-isolation.md) authorizes implementation.
The replacement consumer matrix passes, and release `native-release-Yhc7sX` is installed on 2026-10-15.
The [current handover](../handover/pi-auto-mode-current-frontier.md)
records final evidence and remaining warning cleanup.

## What the new evidence changes

The [incident reproduction](../troubleshooting/pi-auto-mode-session-interference.md)
uses ordinary prompts without tools.
Provider selection fails because auto-mode validates against the startup endpoint.
Tree navigation succeeds in Pi but makes auto-mode's next root capture fail.

These failures are not policy-decision mistakes.
The installed guard has acquired responsibilities for ordinary provider transport and session availability.
Replacing the decision core left those responsibilities intact.
The previous recommendation to retain native ownership needed a narrower definition:
exact operation ownership is useful;
a parallel state machine governing the whole Pi session is not established as necessary by that requirement.

## Proposed interface and responsibility split

Pi owns provider selection, streaming, retries, navigation, reload, session replacement, and disposal.
Auto-mode owns the decision for one proposed tool group and the validity of that decision until execution ends.

The integration seam begins at Pi's original proposed-group dispatch,
before argument preparation and its callbacks.
Preparation therefore remains inside the accepted 15-second shared budget.
The current `executeToolBatch` integration already supplies the original assistant response,
tool definitions, context, signal, preparation callbacks, and native execution callbacks.
Its existence is evidence of a useful seam,
not proof that removing the other owners preserves every contract.

At that seam:

1.  Pi supplies the original proposed group and a current host-owned context/source snapshot.
2.  Auto-mode prepares exact arguments, captures dependencies, selects current policy and visible context,
    and obtains semantic assessments.
3.  Deterministic policy resolution and scoped confirmation decide whether that exact group may proceed.
4.  The original native executor runs only the prepared group while its evidence remains current.
5.  Completion, cancellation, navigation, or changed dependencies invalidate that operation's evidence.
    A later operation obtains its own snapshot and decision.

Any request/source observation still required for authenticity must be passive for the primary request.
It may record an observation failure that sends the dependent tool decision to manual approval;
it must not replace a successful chat response with a guard error.
Do not invent provenance when observation fails.
Do not turn that failure into automatic approval or execution.
The user's 2026-10-15 correction requires manual fallback when observation or assessment evidence is unavailable.

This design does not assume a permanent root lease must be repaired for every UI command.
Currentness is a comparison against the operation's captured native state,
not a global permission to keep chatting.

## Retain, replace, and remove

Retain the decision core, full current policy, legacy visible-message selection,
bounded assessment transport, deterministic resolver, scoped confirmation,
prepared-argument ownership, nested execution identity, dependency checks, and native cancellation.
Requalify each retained mechanism at the proposed-group seam.

Replace the main-request observer's authority to reject chat transport with evidence capture scoped to tool decisions.
Replace persistent session-root admission with operation currentness.
Remove now-unneeded stream claim, runtime lifecycle, forwarding,
and generated-state paths after consumer coverage exists.
Do not add another wrapper that leaves both ownership systems live.

The exact deletion set depends on tracing the remaining source-authenticity consumers.
It is not safe to remove all capture because a no-tool probe passes.
Likewise, preserving every existing internal test is not a goal when its contract is being replaced.

## Alternatives and ranking

### A: Replace the host coupling with operation-scoped admission

Benefit: prevents a tool-policy mechanism from owning unrelated chat and navigation availability.
It keeps the accepted policy semantics and useful prepared-execution checks.
Cost: requires integration migration and renewed proof of source identity, freshness, and cancellation.
The accepted implementation subsequently passed the consumer evidence recorded in the current handover.

### B: Repair the current endpoint and navigation ownership

Benefit: addresses the two reproductions within the current implementation.
Cost: keeps ordinary prompts dependent on auto-mode's provider profile and session state.
Navigation repair also needs consistent leases in host disposal and runtime replacement,
not merely a catch that reopens one observer's root.

### C: Rebuild policy semantics and host integration together

Benefit: permits replacing every current module.
Cost: discards already-verified policy resolution and exact execution behavior
without evidence that both need replacement.
It expands the set of contracts to re-establish.

Ranking: A > B > C.
A precedes B because it corrects failure scope instead of synchronizing more long-lived guard state with Pi.
B precedes C because it retains useful tested behavior rather than replacing unrelated policy semantics.

## Acceptance checks

- Initial and follow-up chat, provider switching, automatic retry, in-flight cancellation,
  tree navigation, message editing, reload, fork, resume, and shutdown remain usable with no tools.
- Inject an auto-mode observation or assessment failure: ordinary chat continues;
  the affected tool group opens manual approval explaining the failure and the complete prepared operation.
  Allow executes that exact operation; Stop and cancellation execute nothing.
- A successful tool group after provider switching and navigation proves the guard still operates,
  rather than being silently bypassed to make chat pass.
- Cancelled navigation preserves a still-current operation where Pi permits it;
  actual branch change invalidates old decisions without invalidating later prompts.
- Changed prepared arguments, source policy, tools, or required dependencies invalidate prior approval.
- Permission, prohibition, independent authorization requirements,
  uncertainty, and explicit bypass retain accepted behavior.
- Exercise the original installed CLI with all configured packages and copied instructions in a disposable environment.
- Verify both authored-response controls and authorized live model consumers before activation.
- Keep the currently installed artifact intact until the replacement passes these checks.

## Completion

Accepted alternative A is implemented, verified, and activated.
No change to the accepted policy cutoff, budget, approval semantics, or inspected program profiles is proposed.
