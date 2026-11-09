# Git usage audit

Audit of how this repo's git history is actually produced,
 measured against the working tree on
2026-06-05.
 Scope is mechanics of committing,
 branching,
 pushing,
 and message hygiene,
 not code quality.

## Verdict

The commit messages are clean and the cadence is disciplined,
 so the day-to-day craft is fine.
 The
problems are structural:
 a wall of unpushed work that exists on one disk,
 a branch graveyard nobody
prunes,
 message bodies mangled into single lines,
 and a permanently bloated `.git` from binaries that
were committed and later deleted.
 None of it is fatal.
 All of it is the kind of thing that bites once.

## What was measured

- 2620 commits,
   2025-02-02 through 2026-06-05 (about 16 months).
- 2615 commits authored as `Aquaticat`,
   but 1485 (57 percent) carry a `Co-authored-by` bot trailer.
  This history is overwhelmingly agent-produced;
   "your git usage" is really "the usage you own and
  ship under your name.
  "
- Commands used:
   `git rev-list --left-right`,
   per-branch ahead/behind,
   files-per-commit and
  inter-commit-gap distributions,
   blob-size ranking over `--all`,
   `git check-ignore`,
   subject-length
  histogram.

## Findings, worst first

### 1. 71 commits live on exactly one disk

```txt
git rev-list --left-right --count origin/main...main  ->  0   71
```

Local `main` is 71 commits ahead of `origin/main`.
 `origin/main` last moved 2026-06-04;
 local has a
full extra day on top.
 That is a day-plus of work whose only copy is this machine.
 Disk failure,
 a bad
`reset --hard`,
 or a stolen laptop erases all 71.
 Severity here is not style,
 it is durability:
 the
single most valuable thing git gives you (an off-machine backup) is switched off most of the time.

Fix:
 push `main` at least at the end of every working session.
 If 71-deep backlogs are normal,
 that is
the smoking gun that pushing is not part of the loop.

### 2. The branch graveyard

24 refs,
 almost all dead and never pruned:

- `origin/refactor(vitest)`:
   2595 behind,
   last touched 2025-05-12,
   over a year stale,
   and the name
  contains parentheses.
   A `type(scope)` string is a commit-message shape,
   not a branch name;
   it makes
  the ref awkward to reference and reads like a mistake.
- `_reserve/91hs22`:
   2019 behind,
   cryptic name,
   no recoverable intent.
- `origin/claude/*`,
   `origin/copilot/*`,
   `origin/add-claude-github-actions-*`,
  `origin/before-trying-to-migrate-to-mise`,
   `origin/stub-copilot-review`:
   all 1900 to 2186 behind,
  all from late 2025,
   all abandoned where they were left.
- Local `feat/module-pipe` (ahead 3,
   behind 554) and `feat/module-pipe-opus` (ahead 4,
   behind 554):
  two parallel attempts at the same thing,
   both abandoned 554 commits behind.
   The work that survived
  was redone directly on `main`;
   these are just litter now.

Several branches sit at `ahead=1` with a single dangling commit that was never merged or deleted.
 The
pattern is:
 spin up a branch,
 do a little,
 abandon it on the server,
 never garbage-collect.
 The cost is
a `git branch -a` you cannot read and real history that is invisible because it never landed on `main`.

Fix:
 delete merged and abandoned branches (`git branch -d` / `git push origin --delete`).
 For anything
worth keeping,
 tag it and delete the branch.
 Rename or drop `refactor(vitest)`.

Caveat discovered while cleaning up:
 the local merged branches delete fine,
 but every *remote* delete is
rejected with `GH006 Cannot delete this branch`.
 The cause is a classic branch-protection rule on the
wildcard pattern `*` with `allowsDeletions: false` (a second rule pins `main` the same way).
 So the
remote graveyard cannot be pruned from the CLI until that rule is changed.
 The same `*` rule also sets
`requiresStatusChecks: true` and `requiresConversationResolution: true` on every branch,
 which imposes a
PR-style gate on throwaway branches and is a plausible contributor to finding 6 (work goes straight to
`main` to avoid the friction).
 Recommended:
 scope protection to `main` only,
 or at minimum flip
`allowsDeletions` to true on the `*` rule,
 then prune.

### 3. Commit message bodies get mangled into one line

696 commits have a subject longer than 72 characters.
 The worst is `40d9c51`,
 whose entire multi
paragraph body is jammed onto the subject line as literal backslash-n escapes:

```txt
docs(*): migrate build system references from Moon to Mise\n\nUpdate all documentation...
```

That is a 1024-character "subject" with `\n\n` printed as text instead of being real newlines.
 Whatever
passed the message (an agent or a `-m` string with escaped newlines) never got interpreted,
 so
`git log --oneline`,
 GitHub,
 and every tool that shows subjects gets a paragraph where one line belongs.
This violates the repo's own GCG rule as it stood when this audit was taken (two lines per group,
 blank line between subject and body;
 as of 2026-11-09 GCG states the subject shape `<type>(<scope>): <subject>` and how to name the scope,
 and the body shape this paragraph asks for is written down nowhere) and
breaks Conventional Commits' 50/72 convention wholesale on those 696.

Fix:
 write bodies with real newlines (a HEREDOC,
 or `-m subject -m body`,
 never a single `-m` carrying
`\n`).
 The agent recipe that produced `40d9c51` needs the same fix at the source.

### 4. `.git` is 82 MB because binaries were committed then deleted

The largest blobs ever committed (still in history,
 still shipped on every clone):

```txt
2.5 MB  package/site/astro-test/src/collections/blog/glass-house.jpg
2.4 MB  package/module/es/src/arrayLike.type.ints.ts   (generated giant type file)
1.9 MB  winter-tree.jpg
1.5 MB  .eslintcache                                      (a cache file, never belongs in git)
1.3 MB  glass-collection.jpg
1.2 MB  package/.../packages.generated.ts               (committed twice)
        package/theme/subtle/dist/temp/cache/compress/...(build cache)
        screenshots/*.png, resume-no-pii.pdf
```

None of these are tracked today (`git check-ignore` over `git ls-files` returns zero),
 so they were
committed,
 then removed.
 But removal does not reclaim history:
 every blob still rides in the 82 MB
`.git`.
 The `.eslintcache` and `dist/temp/cache/compress` entries are the tell that build output and
caches reached `index` at least once before `.gitignore` caught up.

Worth saying clearly,
 because it looks like the same smell but is not:
 the 21 tracked
`package/claude-code-plugin/*/dist/final/...` files are intentional.
 `.gitignore:322-328` explicitly
un-ignores them because marketplace distribution requires committed bundles.
 That is fine;
 leave it.

Fix going forward:
 keep caches and generated megabyte-files out of the index (they already are now).
 The
existing bloat only goes away with a history rewrite (`git filter-repo`),
 which is disruptive on a
pushed repo and only worth it if clone time actually hurts.
 At 82 MB it probably is not worth it yet;
note it and move on.

### 5. Machine-gun cadence: prescribed, but it has a bill

- 327 commits on 2026-05-14 alone.
   Multiple days over 90 to 134.
- Inter-commit gaps:
   17 percent of commits land within 60 seconds of the previous,
   46 percent within
  five minutes.
   Median gap is 6 minutes.
- 922 commits (35 percent) touch a single file;
   median is 2 files,
   mean 15 dragged up by a few
  thousand-file sweeps (max 1715 files in one commit).

This is not an accident,
 it is policy:
 `AGENTS.md` rules GCE and GCU told agents,
 as of 2026-06-05,
 to commit at the earliest
coherent checkpoint and never accumulate independent units
 (GCU left `AGENTS.md` on 2026-11-09,
 and GCE now asks for a commit before the next work step,
 never waiting for verification or completion).
 So this stays.
 But name the cost honestly:
`git bisect` and `git blame` get noisy,
 "what shipped this week" is unreadable without tooling,
 and the
occasional 1715-file commit is the opposite extreme,
 a mixed-concern sweep that violates the same
one-logical-unit rule the micro-commits serve.
 The discipline is real;
 the variance (1-file vs
1715-file) is where it slips.

### 6. Everything lands on `main`; there is effectively no merge or review gate

3 merge commits in 2620.
 2617 commits went straight onto a linear `main`.
 Feature branches exist but
are abandoned rather than merged (finding 2).
 That means no PR-shaped review boundary,
 no green-CI gate
before history is permanent,
 and 37 revert-flavored commits cleaning up after the fact on the trunk
itself.
 For a solo,
 agent-driven repo this is a defensible trade (speed over ceremony),
 but the revert
count plus the abandoned parallel branches show the backtracking that a pre-merge checkpoint would
absorb off `main`.

Correction after pushback:
 this is the weakest finding here and the first to retract.
 A "review gate"
assumes a second reviewer,
 and this repo is one person,
 so the human-review framing does not apply.
 The
only gate that means anything to a solo author is the machine kind:
 requiring CI to pass before code is
permanent on `main`.
 The repo does have CI workflows (`.github/workflows/`),
 but the `*` protection
rule's `requiredStatusChecks` lists zero contexts,
 so nothing is actually enforced,
 which is why a broken
commit can land and then need one of the 37 reverts.
 So the keep-able nugget is narrow:
 a local pre-push hook that
runs lint,
 types,
 and tests,
 so a red commit never reaches `main` in the first place,
 needing no PRs and
no second person.
 (Server-side required status checks are the equivalent only if you later adopt pull
requests,
 since they gate merges rather than direct pushes.
) Everything else about committing straight to
`main` is correct for a one-cat repo,
 and adding pull-request ceremony for an audience of one would be
pure overhead.

## What is actually fine, credit where due

- Conventional Commits adherence is high:
   only 226 of 2620 subjects (9 percent) miss the
  `type(scope):` shape,
   and almost none are `wip` / `oops` / `typo` junk.
   The verbs are accurate.
- Eager checkpoint committing is deliberate and serves recoverability;
   do not "fix" it into batching.
- The committed plugin `dist/` is an intentional,
   documented exception,
   not sloppiness.

## Do these four things

1. Push `main` at end of session.
    Stop carrying 71-deep unpushed backlogs.
2. Prune the branch graveyard;
    delete or tag-then-delete everything 500-plus behind,
    and kill
   `refactor(vitest)`.
3. Pass real newlines to commit bodies;
    find and fix the agent recipe that emitted literal `\n`.
4. Keep treating caches and generated blobs as un-committable (already true);
    only reach for
   `filter-repo` if 82 MB starts to hurt.
