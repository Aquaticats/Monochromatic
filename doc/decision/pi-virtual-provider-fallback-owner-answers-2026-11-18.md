# Pi virtual provider fallback owner answers 2026 11 18

## Status

Moved on 2026-11-23 from the old handover, verbatim;
not yet edited into a record of its own.

## Moved from the old handover

From lines 44831 to 44831 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

  provider-given reset ("according to weather"), and uses Neuralwatt's published `flex_floor` for the flex member.

From lines 44852 to 44856 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- The virtual provider on OpenAI's "possible risk" stop: "Fall back automatically (Recommended)": the writer pool sends
  the same transcript to the next member, and the session record names the member that answered. This reverses the
  first run's refusal and makes rule `RKD`'s hand resume automatic for runs on the provider. It is a separate task from
  the lint run now going (`pi-provider-that-falls-back-across-providers-2`), so it goes to a fresh writer in the same
  worktree after that run ends, then the package's DeepSeek review.

From lines 45608 to 45623 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:


### The virtual Pi provider is built now, beside the restarted queue (2026-11-18, about 21:30)

The owner, in plain text: "If you feel like doing the virual provider first is faster, tell me." Asked through the
question tool (Pi 1.5.2's `ModelRegistry` has `find(provider, modelId)` and `getApiKeyAndHeaders(model)`, so a
provider plugin can fall back per request mid-session; its build time is unmeasured), the owner answered "A. And a lot
of these providers limit rates according to weather, so we need to account for that as well." Option A was: restart
the queue now with fallback by hand (Qwen plan first), and build issue 6314 now with one GPT writer and a DeepSeek
review, switching the queue to it once it is in main. This session's reading of the second sentence, open to the
owner's veto: a provider's limit changes over time, so the virtual provider holds no fixed number as a member's limit,
and after a refusal moves on and later tries the preferred member again rather than demoting it for good.
Then, in plain text: "Some providers do have published rate limits or end points. These should be preferred and called
at key times." Read as: a member's published limit or limit endpoint is used over guessing from refusals, and called at
the moments that decide something (before returning to a member that refused, when a refusal arrives, when a provider is
first used), with the member order unchanged; open to the owner's veto. Sent to the running writer through its inbox
with the endpoints this repository records (Neuralwatt `/v1/quota` and `/v1/rate_limits`, Synthetic `/v2/quotas`).

From lines 45624 to 45631 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:


### A virtual Pi provider with models writer and reviewer, deferred (2026-11-18, evening)

In plain text: "Open an issue saying the GPT -> Qwen -> Synthetic -> opencode-go -> Neuralwatt flex fallback chain
should be a new model "writer" under a new virtural provider. And the new virtural provider should have "reviewer" too.
This specific issue+implementation can wait." Opened as issue 6314, labelled `needs-triage` and marked deferred; its
`reviewer` chain is read as the current DeepSeek reviewer pool (Qwen, Synthetic, `opencode-go`, Neuralwatt flex), to be
confirmed when it is taken up. Not started; task 383 (the driver's pool table) stays as it is until then.

From lines 45658 to 45662 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

### A run OpenAI stops as a "possible risk" resumes on DeepSeek (2026-11-18, evening)

In plain text, after the ZFS-clones research ended on OpenAI's filter:
"Whenever a "possible risk" is triggered from OAI, switch to deepseek resuming the session.
Make this a standing rule in AGENTS.md . Use "possible risk" instead of anything more specific."
