# Second review of the Kitesurf screening record

## Open work

`doc/research/kitesurf-as-agent-browser-replacement.md` had one independent review,
by qwen4-max-preview through Pi,
of the record as of commit `b68463d293`.
Everything added after that commit has had no independent review:
the sections "What the source shows" and "What a run on the playground shows",
the page-retention paragraph in "Hosted forms cannot reach the pages being checked",
and the follow-up decision.

At 2026-11-13T17:15:29-05:00 the review could not start:
`systemctl --user list-units --no-legend --plain --state=active 'pi-*.service'` counted 17 Pi runs,
over the cap of 16 in `doc/handover/agents-at-once-on-this-machine.md`.
A starter script then checked the count once a minute until 2026-11-13T18:02:05-05:00,
when its deadline passed with 25 Pi runs active,
so the review has still not started.
Its worktree was removed again,
since step 2 adds it at start.

That attempt left a ready brief and starter in scratch,
which a reboot may wipe;
the steps under "How to start it" are enough without them:

- brief:
  `${HOME}/temp/agent/pi-runs/kitesurf-screening-record-second-review-on-qwen4-2026-11-13-prompt.md`,
  written for a worktree at commit `ffb1d0d9e4`;
- diff:
  `${HOME}/temp/agent/qwen-trials/kitesurf-screening-record-second-review-2026-11-13/change.diff`,
  from `b68463d293` to `ffb1d0d9e4`;
- starter that waits for a place under the cap,
  then runs the unit:
  `${HOME}/temp/agent/pi-runs/start-kitesurf-second-review-when-a-place-frees-2026-11-13.ts`.

## How to start it

1.  Count agents as `doc/handover/agents-at-once-on-this-machine.md` says,
    and start only under the cap.
2.  Add a detached worktree at the record's latest commit under `.claude/worktrees/`.
3.  Write the diff from `b68463d293` to that commit for
    `doc/research/kitesurf-as-agent-browser-replacement.md` into the review's scratch folder.
4.  Brief it like the review of `b68463d293`:
    read the diff and the whole record,
    check each claim against the repository files and web pages it cites,
    and report only concrete defects.
    The new claims about Kitesurf's source cite paths at commit
    `4a2903813a22b08d4f3cc64a4b35339d0293a5d1` of `github.com/cloudflare/kitesurf`,
    so the brief must allow a shallow `gh repo clone` of that repository into the review's scratch folder.
5.  Run it with `--provider=qwen-token-plan-individual --model=qwen4-max-preview`,
    as `AGENTS.md` asks for reviews of a change,
    through `package/pi-shared/subagent-driver`.

## When it is done

Fold each finding that holds into the record,
then delete this file.
