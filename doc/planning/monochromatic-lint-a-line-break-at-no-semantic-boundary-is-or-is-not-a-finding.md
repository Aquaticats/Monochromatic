# A line break at no semantic boundary: a finding, or not

Status:
written 2026-11-27 by the round that fixed issue 6642
(`fix/monochromatic-lint-issue-6642-the-line-break-fix-joins-what-the-old-wrap-cut-and-leaves-no-lone-word`).
**Unapproved and non-final:**
nothing here is built,
the rule reports exactly what it reported before,
and the owner declares completion.
The coordinating session's pick for this round was to propose this question and build nothing of it,
because a yes adds findings to text that passes today,
which tightens a rule.

## Why this question is open

`markdown/semantic-line-breaks` answers one question:
does a break belong after a prose break-point character that is followed by a space on the same line
(`package/linter/monochromatic-lint/src/markdown_break_points.rs`,
`break_offsets`)?
It never looks at a break the author already wrote.
So a paragraph an author hard-wrapped at a spot with no break-point character in front of the break
passes the rule,
while the repository's rule in `AGENTS.md` rule MD1 asks for the opposite:
"Break lines at semantic boundaries,
under 120 chars,
so text reads without editor wrapping."

This round repaired the half of that the fix itself caused:
a fragment the fix's own break cut off now joins the clause that continues it
(`package/linter/monochromatic-lint/src/markdown_fragment_joins.rs`).
What remains is a break the author wrote,
and whether the rule says anything about it.

## What "at no boundary" means in this measurement

One break is counted when all of these hold.
The break sits inside a paragraph,
so a blank line,
a fence,
a heading,
a table and a list marker end the walk.
The line the break ends holds at least one visible character.
The last visible character of that line,
after any closing delimiter (`"`,
`'`,
`)`,
`]`,
`}`,
`”`,
`’`,
`»`,
`›`),
is not one of `,`
`.`
`;`
`:`
`?`
`!`,
which is the set `markdown_break_points.rs` breaks after.
`9,860` of the `221,518` breaks at no boundary end a parenthetical or a quote,
which reads as a boundary of its own,
so they are counted apart and the `199,796` below leaves them out.

## What this repository holds today

Measured once over every tracked `.md`,
`.mdx` and `.rs` file of this worktree (`10,453` paths),
with Rust doc comments read through the linter's own processors,
so a doc comment is the Markdown the rule sees:

```text
breaks inside paragraphs:                                        1,539,644
breaks inside paragraphs that already carry a finding:              25,587
breaks at no boundary:                                             221,518
breaks at no boundary in paragraphs the rule reports nothing for:   199,796
breaks at no boundary that leave the next line one or two words:     71,261
breaks at no boundary a removal fix could join upward:              158,514
breaks at no boundary a removal fix could not join:                  41,282
files holding at least one of the 199,796:                            8,623
files holding one of the 158,514:                                     8,091
```

Ten of them,
each the line the break ends:

- `.agents/skills/android-emulator/SKILL.md:22`:
  ` Briefs for agents that use an emulator repeat the headful`
- `.agents/skills/choosing-technology/SKILL.md:151`:
  `because the user may delegate the enumeration to a cheaper`
- `.agents/skills/css/SKILL.md:52`:
  ` check the Firefox ESR 140 caniuse percentage`
- `.agents/skills/dum-dum-non-ts/SKILL.md:8`:
  `Fires when writing or editing source files in non-TypeScript general-purpose`
- `.agents/skills/pi-as-subagent/SKILL.md:226`:
  a 123-character command line of `check-in-on-pi-runs --runs-folder=<folder> --unit=<that unit>`
- `.agents/skills/project-code-review/PHILOSOPHY.md:11`:
  `Background context fades as conversation length grows -- t`
- `.agents/skills/project-code-review/SKILL.md:344`:
  `   any deviation from the preferred pattern must have a co`
- `.agents/skills/runbook/copilot-review-draft-pr.md:14`:
  ` registers within about one second of the`
- `.agents/skills/source-audited-report/SKILL.md:22`:
  ` The Verge longform) --`
- `.agents/skills/testing-practices/SKILL.md:10`:
  ` or reviewing`

A removal fix,
the operation every option below would need,
replaces the break and the next line's indentation with one space,
refuses a line that already ends in a hard break,
refuses a following line that carries a quote or list marker,
and refuses a joined line that would pass 120 characters.
No such fix exists in the tree;
`158,514` and `41,282` are that model's answer over this repository,
not a measurement of shipped code.

The harness was a temporary test of this round,
run under the package's `cargo nextest` and deleted before the round ended,
and it reports no fix,
so it changed no byte of the repository.

## Option A: report every break at no boundary

Reports `199,796` findings in `8,623` files.
`158,514` of them clear with one `--fix` pass;
`41,282` do not,
because the joined line would pass 120 characters or the following line carries a marker.

Pros:

- It answers the whole of `AGENTS.md` rule MD1,
  not only its "break after a break-point character" half.
- One definition,
  no class of breaks left unexplained,
  and the `41,282` that need a human are exactly the lines that are too long to join,
  which a reader must rewrap anyway.

Cons:

- `--fix` cannot return the repository to green:
  `41,282` findings stay after a run,
  and the commit path runs `--fix` and reports what is left.
- The diff reaches `158,514` line endings in `8,091` files,
  and every reflowed paragraph has to be read again.

## Option B: report only a break the fix can remove

Reports `158,514` findings in `8,091` files,
and every one of them clears with one `--fix` pass.

Pros:

- The rule and its fix cannot disagree:
  a finding exists exactly where the fix proves it can act,
  so the commit path returns to green after one run.
- The guards are the ones the shipped forward join already applies,
  so this option adds no second definition of a safe join.
- It still names the whole of the fixable defect:
  `158,514` of the `199,796`.

Cons:

- It ties a rule's report to what a fix can do,
  which is backwards from how every other rule here is written,
  and a later fix would have to widen the rule in the same commit.
- It hides `41,282` breaks,
  and a hidden break is one the author never learns about.
- The removal fix is a second join direction beside the forward join this round shipped,
  and the two must be proved not to oscillate over one paragraph.

## Option C: report nothing, as today

Reports no new finding.

Pros:

- No churn:
  the fix repairs a fragment whenever another break in the same paragraph is inserted,
  which is how the wrapper's `0.6.1-alpha.51` doc comments are put right in their own round.
- No new operation,
  no new finding class,
  and nothing for the owner to veto.

Cons:

- The `199,796` breaks stay,
  including the `71,261` that leave a line of one or two words,
  and nothing tells an author about one.
- Text that passes the rule still reads worse than text the fix wrote,
  which is the inconsistency this question exists for.

Ranking:
B beats A because every B finding is one `--fix` clears,
so the commit path returns to green in one run,
where A leaves `41,282` findings no command can close.
A beats C because A names a real defect that C leaves invisible,
and `AGENTS.md` rule MD1 asks the rule for it.

## What was considered and is not an option

Reporting only a break whose next line holds one or two words,
`71,261` findings,
was dropped as a class of its own:
a word count is the language heuristic this rule refuses by design,
most of those lines end at a break-point character where the rule wants a line to end,
and the removal fix proves only a part of them clearable.
It is useful as a measurement,
never as a rule.

## What the owner has to decide

1.
Whether the rule reports a break at no semantic boundary at all,
which adds findings to text that passes today,
and which of the classes above it reports.
2.
If a class is reported,
whether its removal fix ships in the same release,
because a finding without a fix blocks the repository's `format:monochromatic-lint` task.
3.
Whether the bound a join uses is the rule's own or the repository's:
the rule holds no length limit,
and the fix this round shipped uses 120 characters from `AGENTS.md` rule MD1.

## What this round built, and what it did not

Built:
the forward join,
where a fragment this fix's own break cut off joins the clause that continues it,
and the indent a paragraph's first continuation line carries,
used for every line the fix writes.
Measured:
`package/git-policy/cli/src` as it stood before the merge that raised the issue,
where the released fix left `834` doc lines of one or two words among the `4,613` it wrote,
and this fix leaves `732` among the `4,577` it writes.
Not built:
nothing of the three options above,
and no rule reports a new finding.
