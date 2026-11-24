# Pi driver and provider owner answers 2026 11 09 to 2026 11 11

## Status

Moved on 2026-11-23 from the old handover,
verbatim;
not yet edited into a record of its own.

## Moved from the old handover

From lines 30381 to 30389 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

### Pi tooling lives in the workspace, and alphas run no extra release work (2026-11-11T17:55:17-05:00)

The owner,
at about 17:50,
after the coordinator had again edited scratch scripts:
"I've said to fold pi-runs set of scripts into a proper workspace package before.
I believe there is also a launcher already as a proper workspace package.
I don't wanna see you edit pi-runs outside the workspace again.
Please pick that up."
So no script under `~/temp/agent/pi-runs` is created or edited any more;
only data lives there (briefs,
batch lists,
inbox notes,
run folders).

From lines 33039 to 33056 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

The owner came back at 2026-11-11T05:30-05:00 and asked "What is going on?
Why did you downgrade everything to 6 and 7?",
then decided two things:
"I've re-enabled auto-renew.
Treat the prev Neuralwatt 429s void."
(the launcher's pools are back to standard DeepSeek 20,
DeepSeek flex 22,
Kimi flex 22;
the DeepSeek flex 7 had been an unmeasured assumption),
and,
asked whether to hold `0.6.1-alpha.10` for the new data-loss findings,
picked "Release now,
fixes in alpha.11".
That choice is also an older standing rule the coordinator misapplied overnight:
"didn't I also say reviews and mutation testings don't block publishing?"
(recorded under "Working rules for this effort"),
so review findings become follow-up fixes and later releases,
never a hold.

From lines 33778 to 33862 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- The provider ended a burst of runs with `terminated` between 2026-11-09T11:24:25-05:00 and 2026-11-09T11:29:23-05:00:
  `discarding-commands-findings-fixed` runs 5 to 7,
  `conflict-resolution-findings-fixed` runs 6 and 7,
  `wrapper-tests-that-fail-only-in-a-suite` run 4,
  and `push-route-findings-fixed` run 7.
  Runs 7 of the first two were fresh sessions,
  and the feed of the first shows an empty answer after 18 s of one model response,
  so resuming is not the cause.
  Context size does not explain it either:
  the ended runs held 448 to 550 thousand tokens,
  and `merge-route-findings-fixed` run 10 finished normally at 887 thousand.
  An unauthenticated `GET` of the provider's model list answered 200 in the same minutes,
  and other agents kept producing events,
  so the cause is unknown.
  The owner wrote at 2026-11-09T11:32-05:00:
  "I'm in touch with the provider right now.
  Meanwhile use opencode-go/deepseek-v4.1-flash ."
  So new runs pass `--provider=opencode-go --model=deepseek-v4.1-flash` to `start-pi-run-if-a-slot-is-free.ts`
  until the owner says the provider is back,
  and a run the provider ends restarts on that model.
  Audits and reviews stay on `openai/gpt-6.1-sol`.
  The same error then ended 3 of 110 responses on `opencode-go`,
  at 2026-11-09T16:40:52Z,
  16:40:56Z and 16:43:09Z,
  so it is not specific to the first provider.
  The owner then wrote:
  "Try synthetic's hf:deepseek-ai/DeepSeek-V4.1-Flash-1017 too.
  If it also shows the same failure pattern then this has become a mystery."
  `discarding-commands-findings-fixed` run 10 is that trial.
  Evidence for the provider,
  redacted and verified,
  is `~/temp/agent/neuralwatt-investigation-2026-11-09.zip`,
  which the owner asked for and sends.
  At 2026-11-09T17:08:33Z to 17:08:36Z four responses ended `terminated` on `neuralwatt` and `opencode-go`,
  all started between 17:06:48Z and 17:07:22Z.
  A WireGuard interface `fr-par-fr1` had become the route for provider traffic at 17:06:46Z,
  and the journal does not name what started it.
  Seven other responses in flight at that moment finished normally:
  two started before the interface appeared and five started at or after 17:08:23Z.
  The journal shows no network device event during the earlier clusters,
  so this burst may be a separate incident.
  A resumed session larger than the new model's context window compacts first,
  and the driver's 30 s wait for a prompt's answer ended that run as `prompt-rejected`;
  the owner wrote "\"That's a limit in our driver \" - so fix it.",
  and `driver-prompt-waits-through-compaction` is that fix.
  The owner wrote
  "And we still need to find out why the earlier neuralwatt and opencode-go responses were cutoff.
  I'm suspecting my wifi driver."
  The radio is `wlp9s0` on `ath12k_wifi7_pci` with power save off.
  It has stayed associated since the driver's last kernel line at 2026-11-06T09:37:09Z,
  and the kernel,
  `wpa_supplicant` and NetworkManager logged nothing about it during any burst.
  Every burst before 2026-11-09T17:08:33Z cut responses of one provider only,
  while responses of other providers in flight across it finished normally.
  This does not exclude a short silent stall:
  `iw` reads zero retries and failures,
  which this driver may simply not report,
  and a cut-off's record says only `terminated`.
  `connection-cut-off-watch.service` and `network-change-watch.service` record Pi's connections,
  gateway pings and address changes in `~/temp/agent/connection-cut-off-watch-2026-11-09/`.
  The owner wrote "Make our wg-quicker avoid proxying opencode-go,
  qwen,
  synthetic,
  neuralwatt,
  deepseek,
  pi traffic."
  and "Apply every exemption mechanic.";
  `wg-quicker-model-providers-off-the-tunnel` is that work,
  and applying it to the running tunnel is still to do.
  Waiting for a free agent slot:
  health route run 3 with `resume-after-wall-clock-prompt.md`,
  literal pathspec run 5 with `resume-after-provider-error-prompt.md`,
  worktree add round 8,
  commit ceiling run 5 with `resume-after-wall-clock-prompt.md`,
  worktree copy round 6,
  the locks round 6 if its start did not take,
  and conversion round 5.
  Briefs are in `~/temp/agent/pi-runs/`.
  Cloudflare's status page opened "Network Performance Issues in Eastern North America" at 2026-11-09T16:34:35Z,
  identified a fix at 16:56:39Z and moved to monitoring at 17:21:50Z;
  this host reaches Cloudflare through Toronto,
  and neuralwatt and opencode-go are served through Cloudflare while Synthetic and DeepSeek's own API are not.
  The bursts at 15:05Z and 15:08Z came before that first post.
  The locks review `locks-and-transactions-round-4` and the conversion review `subvolume-conversion-round-3` need fix rounds before those branches land.
  A fresh session on a brief that stands alone costs about 30 s when it fails again.
