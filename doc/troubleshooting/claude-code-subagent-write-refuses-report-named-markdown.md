# Claude Code 2.1.331 refuses a subagent's Write of a Markdown file whose name starts with "report", "summary", "findings" or "analysis"

## Symptom

A subagent (an agent started with the Agent tool) calls Write on a file such as `keep/report-pi-round.md` or `Summary.md`,
and the tool answers:

```text
Subagents should return findings as text, not write report files. Include this content in your final response instead.
```

Nothing is written.
The same subagent's Write of `review.md`,
`my-report.md` or `report-check.txt` succeeds.
The main session's Write is not affected.

On 2026-11-23 the old handover's sorters met this when told to write `keep/report-*.md`,
and the handover then recorded the wider claim "Subagents can't write report files".
That claim was wrong:
on the same evening a release pass wrote `keep/review.md` from a subagent without trouble.

## Root cause

Claude Code is distributed as a single bundled program,
`~/.local/share/claude/versions/2.1.331` on this machine,
and its source is not published.
The text below was read from that bundle with `rg --text`;
the names in it are the minifier's.
The Write tool's `validateInput` refuses when the call comes from a subagent (`agentId` is set)
and the target's base name matches a fixed pattern:

```js
// ~/.local/share/claude/versions/2.1.331 (Claude Code 2.1.331), the Write tool's validateInput, minified
let C=lk(h);if(g.agentId&&/^(REPORT|SUMMARY|FINDINGS|ANALYSIS).*\.md$/i.test(ets(C)))return a("tengu_subagent_md_report_blocked",{contentBytes:Buffer.byteLength(y)}),{result:!1,message:"Subagents should return findings as text, not write report files. Include this content in your final response instead.",errorCode:5};
```

So the rule is:
the base name starts with `report`,
`summary`,
`findings` or `analysis`,
in any case,
and ends with `.md`.
A name that only contains one of those words (`my-report.md`),
or another extension (`report-check.txt`),
passes.
The check sits in the Write tool alone;
it does not stop a shell command from writing the same file.
It records a telemetry event named `tengu_subagent_md_report_blocked`,
so the refusal is a deliberate policy of the harness,
not a fault.

## Verification

Version under test:
Claude Code `2.1.331` (`claude --version`).

The harness:
a subagent told to call Write once each,
content `check`,
in `~/temp/agent/claude-code-subagent-report-file-name-check-2026-11-24/keep/`
(scratch,
deleted on 2026-11-27;
what this record uses from it is stated here),
on 2026-11-24.

Refused,
with the message above:

- `report-check.md`
- `Summary.md`

Written:

- `review-check.md`
- `report-check.txt`
- `my-report.md`

The three written files are in that folder.

## Verified workarounds

- Name a subagent's report file for what it is without those four leading words,
  such as `review.md`,
  `state-so-far.md` or `ledger.jsonl`.
  Tradeoff:
  a brief that names its own report path has to follow the rule,
  and a subagent free to choose a name may still pick a refused one.
- Have the subagent give its report as its final message,
  and let the coordinator save it.
  The coordinator can write the final message to a file without reading it back with
  `jq --slurp --raw-output '[.[] | select(.type == "assistant") | .message.content[]? | select(.type == "text") | .text] | last' <the agent's output file> > <report file>`.
  Tradeoff:
  the report exists only once the coordinator acts,
  and a subagent cut off before its last message leaves nothing.

## What does not work

- Telling a subagent to write `keep/report-<name>.md`:
  refused,
  as the symptom shows.
- Reading the refusal as "subagents cannot write files":
  only the four name prefixes with `.md` are refused.

## Upstream filing artifact

### Upstream filing decision

`.out-of-scope/` was not consulted,
because the first constraint already settles it.

1.  Is it upstream's fault?
    No:
    the check is a deliberate rule with its own telemetry event,
    and its message says what to do instead.
2.  Can upstream fix it?
    Not applicable;
    nothing is broken.
3.  Do they support the use case?
    The message says they want findings returned as text.
4.  Would the repository welcome a contribution?
    Not checked;
    there is nothing to contribute.
5.  Would they fix it?
    Not applicable.
6.  A prototype?
    None;
    nothing to fix.

Nothing is filed and no draft is kept.
