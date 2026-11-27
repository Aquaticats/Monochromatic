# What the commit service should do with a push the server refuses with GH009

**Unapproved and non-final.**
The owner declares completion,
and nothing here is a decision.

Written 2026-11-27T03:39:42-05:00 by the Pi round working in
`.claude/worktrees/pi-issue-6559-proposal-for-pushes-the-server-refuses-with-gh009`,
on branch `docs/planning-issue-6559-proposal-for-pushes-the-server-refuses-with-gh009`,
from `main` at `e0521efb8c`,
for the owner's answer to
[issue 6559](https://github.com/Aquaticats/Monochromatic/issues/6559).

What is not known yet,
and what this proposal therefore cannot settle:

- what the owner decides;
- whether the two shapes of refusal the service recorded are one condition of GitHub's
  or two;
- whether a retry of the refused commit itself ever lands it,
  since the service's journal never shows a retried push landing the commit it was planned for;
- where the rounds quoted in the issue read the `GH009` line from,
  since the service's own record of those three pushes holds no `GH009` line at all.

## What the issue asks

The issue reports three pushes refused on 2026-11-26 with
`remote: error: GH009: Git LFS object integrity could not be checked`,
and says each cleared on a plain `git push` moments later,
as one earlier refusal did on 2026-10-15.
It reads the package's `SPEC.md` as making such a refusal final for the commit service,
and asks the owner
"whether a `GH009` refusal gets a bounded retry as `GH006` does",
and for the three sightings to be added to
[the LFS note](../troubleshooting/github-lfs-integrity-rejection.md),
which records only the one of 2026-10-15.
The three sightings are now a section of that note,
written from the measurement below.

## What happens today

### The words the refusal arrives in

GitHub's own message reaches a push as `remote:` lines,
and Git prints a line of its own for every refused reference,
`! [remote rejected] <old> -> <ref> (<reason>)`.
The service reads both,
in `../../package/git-policy/cli/src/native/push/push_retry_failure.rs:192`:

```text
// package/git-policy/cli/src/native/push/push_retry_failure.rs:192
pub fn final_failure(output: &str) -> Option<FinalFailure> {
    if output.contains(CONFLICT_PHRASE) {
        return Some(FinalFailure::Conflict);
    }
    let protected: bool = protected_branch_refusal(output).is_some();
    for line in rejection_lines(output) {
        if !protected && line.contains(HOOK_DECLINED) {
            return Some(FinalFailure::Declined);
        }
    }
    for phrase in LOGIN_PHRASES {
```

`HOOK_DECLINED` is the pair of words the reason carries,
declared at line 72:

```text
// package/git-policy/cli/src/native/push/push_retry_failure.rs:72
pub const HOOK_DECLINED: &str = "hook declined";
```

When GitHub sends the reason `pre-receive hook declined`,
as it did for the thirteen refusals measured here,
the push is read as a decline,
unless the push's output also holds GitHub's protected-branch words,
which it does not
(`../../package/git-policy/cli/src/native/push/push_retry_failure.rs:96`):

```text
// package/git-policy/cli/src/native/push/push_retry_failure.rs:96
pub const PROTECTED_BRANCH_REFUSAL: &str = "GH006: Protected branch update failed";
```

Only that one code is exempt from the decline rule,
and it is given a counted retry instead
(`../../package/git-policy/cli/src/native/push/push_retry_failure.rs:127`,
five tries whose default pauses add up to two and a half minutes).
`SPEC.md:9732` states the rule the classifier implements:
"A hook decline with no such code anywhere in the push's output stays final."
`README.md:5889` says the same to a reader of the package:

```markdown
<!-- package/git-policy/cli/README.md:5889 -->
  and a hook on the server that declined the push (a `[remote rejected]` line whose reason ends in "hook declined",
  with no protected-branch code anywhere in the push's output).
```

The verdict becomes a failure the service stops on,
in `../../package/git-policy/cli/src/native/landing/service_work_pushes.rs:165`:

```text
// package/git-policy/cli/src/native/landing/service_work_pushes.rs:165
            Ok(result) => match final_failure(result.shown.as_str()) {
                Some(cause) => Verdict::Failed(match plain_words(cause) {
                    Some(words) => format!("{words}\n{}", result.shown),
                    None => result.shown,
                }),
```

No plan is written for such a push,
so nothing tries it again until the branch moves:
the person's next commit is a new push of the branch,
and that push carries the refused commit with it.

### What the person reads

The failure record starts with the decline's plain words
(`../../package/git-policy/cli/src/native/push/push_retry_failure.rs:409`),
above GitHub's own lines:

```text
monochromatic-git-policy-cli: the server refused this push: a hook on the server (a check the server runs before it accepts a push) declined it, with the message shown below. The commit service stops pushing this branch until your next commit. A push by hand may succeed if the cause has cleared: fix what the message names, or wait, then push by hand or commit again.
remote: error: GH009: Git LFS object integrity could not be checked. Please contact GitHub support.
remote: pre-receive hook declined
 ! [remote rejected] 0bd15253d5db8d2dfc3a522868582cec24c5abb4 -> main (pre-receive hook declined)
error: failed to push some refs to 'https://github.com/Aquaticats/Monochromatic'
```

That record is the one the journal holds for the refusal at
`2026-11-27T01:26:26-05:00`,
with the record's own first line quoted from the source above it.
`SPEC.md:9760` lists what needs the person:
"for a hook on the server that declined the push without a protected-branch code",
and `SPEC.md:9771` says such a decline
"may succeed if the cause has cleared,
since a decline says what the server answered then and nothing about a later try".

### What the same condition does when the words are missing

The same pre-receive hook decline arrives in a second shape,
with no `remote:` line and the reason `failed`,
`! [remote rejected] <old> -> <ref> (failed)`.
`final_failure` reads no `hook declined` in that shape,
so the push is retried on the plan's own schedule,
which has no count of tries
(`README.md:5898`):
"A rejection by the server whose reason is none of these is tried again,
and the journal names its line."
The service's journal shows both shapes on the same machine,
minutes apart,
which is measured below.

## What was measured

### The count

The service writes its journal to journald,
which this machine's journal keeps for one boot,
from `2026-11-26T09:53:47-05:00` to `2026-11-27T03:21:07-05:00`.
A bounded read of the user journal since `2026-11-26 00:00`
(the whole boot) gave 211,113 lines,
and 1,311 push records among them,
`pushed branch="..." oid=... items=N outcome=...`.

By outcome:

- 1,203 landed (`outcome=done`);
- 61 retried (`outcome=retrying`);
- 34 skipped,
  nothing was sent;
- 13 failed (`outcome=failed`),
  and all 13 are the readable `GH009` shape.

The 61 retried records are,
by why:

- 9 with `GH006: Protected branch update failed`;
- 49 a server's pre-receive refusal whose reason is `failed` and whose output holds no code,
  the shape the issue's three pushes arrived in;
- 3 a push that stalled or was offline.

So every failure the service stopped on in this boot is a `GH009` refusal,
and 13 of them happened in about seven hours.
The 13 fall between `2026-11-26T20:12:53-05:00` and `2026-11-27T03:19:13-05:00`,
and the 49 unreadable refusals fall between `2026-11-26T16:46:32-05:00` and
`2026-11-27T03:10:02-05:00`,
interleaved with the readable ones and often within the same minute
(`20:12:53` readable,
`20:13:04` unreadable;
`21:26:20` readable,
`21:26:21` unreadable;
`01:25:57` readable,
`01:27:29` unreadable).
The interleaving is consistent with one condition of GitHub's
and is not proof of it.

### What the refused pushes carried

The repository tracks eight image kinds through Git LFS
(`.gitattributes`:
`*.png`,
`*.jpg`,
`*.jpeg`,
`*.gif`,
`*.webp`,
`*.heif`,
`*.avif`,
`*.jxl`),
and its LFS store holds 2,180 objects,
395 MB.
For each of the 13 refused pushes,
the commits the push carried were read from the journal's own record of the branch's
previous landed push up to the refused commit,
and **none of the 13 carried a single path under an LFS pattern**.
The 13 pushed between one and fifteen commits each.

That is the measurement GitHub's documented cause does not fit:
its LFS integrity check blocks a push whose referenced LFS files were not uploaded,
and these pushes referenced none.

### How each refusal ended

All 13 refused commits are on GitHub now
(`gh api repos/Aquaticats/Monochromatic/commits/<oid>` answered each).

Of the 13,
11 were carried by a later landed push of the same branch,
`0.1` to `39.7` minutes after the refusal,
median `3.8` minutes.
The other two are on GitHub by other paths:
one is an ancestor of `main` today,
and the other's branch head on origin is a later commit.

For comparison,
the 9 `GH006` refusals were each carried by a later landed push of the branch
`0.1` to `218.7` minutes later,
median `1.1` minutes,
and 46 of the 49 `(failed)` refusals were followed by a landed push of the branch,
`0.1` to `66.8` minutes later,
median `6.7` minutes.
Those spans measure when the branch next published the commit,
which is usually the next commit of the same session,
never the clearance of the refusal itself.

The journal cannot show a retry landing its own commit:
all 70 planned retries in the boot were followed by a push of a tip that had moved,
so no retried record landed the same oid.
Whether a retry of the same commit would have landed is therefore unmeasured.

### What a plain push does with the same words

A fixture repository in this round's scratch folder,
a bare repository whose `pre-receive` hook printed the two lines captured from the journal
and exited 1,
answered `git push origin HEAD` with exit code 1 in 0.4 seconds:

```text
remote: error: GH009: Git LFS object integrity could not be checked. Please contact GitHub support.
To <the fixture's bare repository>
 ! [remote rejected] HEAD -> main (pre-receive hook declined)
error: failed to push some refs to '<the fixture's bare repository>'
```

Plain Git retried nothing and said nothing the wrapper does not already print.

### What GH009 is, from GitHub and from the LFS client's maintainers

GitHub documents the condition,
not the code,
on
[Resolving Git Large File Storage upload failures](https://docs.github.com/en/repositories/working-with-files/managing-large-files/resolving-git-large-file-storage-upload-failures),
fetched 2026-11-27:

> The Git LFS integrity check ensures that all referenced Git LFS files in a push have been
> uploaded properly.
> If the check detects referenced files that have not been uploaded,
> you will receive an error message and your push will be blocked.

Its remedy is client-side:
"you must reinstall your local Git LFS client",
then `git lfs install` and `git lfs push --all origin`.
The page names no error code and says nothing about whether such a refusal clears by itself.

The `git-lfs` maintainer reads the same text differently,
in
[the git-lfs maintainers' comment on issue 4787](https://github.com/git-lfs/git-lfs/issues/4787#issuecomment-995811913):

> This message comes from GitHub and it means that something went wrong on their end.
> You'll want to contact GitHub support as directed to see if they can help you figure out
> what's wrong.

GitHub's page for a push blocked by secret scanning
([Working with push protection from the command line](https://docs.github.com/en/code-security/how-tos/secure-your-secrets/work-with-leak-prevention/push-protection-on-the-command-line),
fetched 2026-11-27)
prints no code either,
and the package's own comment records that GitHub answers refusals of more than one kind with codes of
one family
(`../../package/git-policy/cli/src/native/push/push_retry_failure.rs:88`),
so the code alone never identifies the condition.
The words after the code do,
and they say the server could not check the integrity of the push's LFS objects.

**A refusal that will never succeed cannot be told apart from one that will**:
GitHub documents no marker,
the client's output carries none,
and the two shapes measured here differ in nothing but the words the server happened to send.
The only way to learn whether a given refusal would have cleared is to try it again.

### Where the refusal is recorded

The service's own state folder
(`~/.local/state/monochromatic-git-policy-cli`)
holds the instances' landing journals,
the retry plans (`push-retries`,
43 instances,
one plan file at the time of reading),
and the claim watches,
and **no file in it holds a refusal's words**:
a search of the folder's 24 MB for `GH009` and for `GH006` found nothing.
The refusal lives in journald and in the failure record the next command shows,
which removes it as it shows it.
An earlier refusal than this boot's journal is therefore not recoverable from the service's own records,
and the one of 2026-10-15 survives only in
[the LFS note](../troubleshooting/github-lfs-integrity-rejection.md).

### What could not be measured

- Whether the retried shape clears faster than the final shape:
  both groups end at the branch's next push,
  which is usually a new commit.
- What any of the 13 refusals would have done on a second try seconds later,
  which would need a live server answering the same way twice.
- The cost of the client-side check option D proposes,
  since `git lfs push --all origin` against the real repository would upload to it.
- How much of the machine's whole history this boot's journal covers:
  one boot,
  from 2026-11-26T09:53:47-05:00,
  so the three sightings of the issue are inside it and the one of 2026-10-15 is not.

## Options

Each option decides for both shapes of the refusal,
since the measurement shows the shapes differ by the words GitHub sent and not by what the person must do.

### A. Keep the refusal final, and rewrite the words

The service keeps stopping at a `GH009` refusal,
and the record says what GitHub documents and what the machine measured:
that the server could not check the push's LFS objects,
that GitHub's own page suggests `git lfs install` and `git lfs push --all origin`,
and that the same refusal has cleared on a plain `git push` within minutes.

#### What a person at the terminal sees

```text
monochromatic-git-policy-cli: the server refused this push: it could not check the integrity of the Git LFS objects this push references, with the message shown below. The commit service stops pushing this branch until your next commit. GitHub's own page for this refusal asks for `git lfs install` and `git lfs push --all origin`; the same refusal has also cleared on a plain `git push` within minutes, so trying again by hand is worth it.
remote: error: GH009: Git LFS object integrity could not be checked. Please contact GitHub support.
```

#### Pros

- Nothing new runs,
  nothing new can fail,
  and the record claims nothing the evidence does not support.
- It stops claiming a person can "fix what the message names",
  which for this refusal names GitHub support.

#### Cons

- A branch nobody commits to again stays unpublished,
  and the person who was not watching learns nothing until their next command.
- It keeps the service's behaviour dependent on which words GitHub sent:
  the same condition still gets retried when the reason reads `failed`.
- On this machine it would leave 13 of 13 refusals to a later push,
  which arrived because sessions keep committing,
  not because the service acted.

### B. Give it the bound GH006 gets

A `GH009` refusal joins `PROTECTED_BRANCH_REFUSAL` as a counted cause:
`LFS_INTEGRITY_TRIES` tries in all,
counted on the push's own try number,
with the pauses the plan already makes,
and the record after the last try names the tries,
the span,
both ways out,
and GitHub's documented remedy,
the way `out_of_tries_words` does for `GH006` today.
The first pause is 10 seconds by default,
so five tries end two and a half minutes after the refusal,
which is the span the `GH006` bound chose from three recorded clearances.
No recorded clearance of a `GH009` refusal exists to size the number from,
so five is borrowed,
not measured.

#### What a person at the terminal sees

```text
monochromatic-git-policy-cli: the commit service tries this push again by itself in 10 seconds (try 2 of 5). The server could not check the integrity of the Git LFS objects this push references; if it refuses try 5 as well, the service stops trying this branch and tells you what the server said.
remote: error: GH009: Git LFS object integrity could not be checked. Please contact GitHub support.
```

#### Pros

- A quiet branch gets up to five more tries with nobody watching,
  which is the failure the issue describes.
- The count and the span are stated to the reader,
  as the `GH006` bound's are.
- It can be measured against this machine's record:
  the journal's `failed` records are exactly the population the change would act on.

#### Cons

- The bound cannot come from the measured spans,
  whose median is `3.8` minutes and whose longest is `39.7`,
  so five tries over two and a half minutes would end before most of them.
  The spans are mostly new commits,
  though,
  and a new commit restarts the count anyway.
- It adds up to four pushes per refusal,
  and a push on this machine took `2.0` to `601.7` seconds,
  median `4.2`.
- It leaves the unreadable shape's unbounded retry in place unless the owner asks for one rule.

### C. Retry it on the schedule every unclassified refusal gets

A `GH009` refusal stops being a decline:
`final_failure` returns nothing for a pre-receive decline,
and the plan retries it with doubling pauses up to 600 seconds
until it lands or the person's next commit replaces the plan.
Nothing new is built:
the path the 49 unreadable refusals already take is the path this option gives the readable ones.

#### What a person at the terminal sees

```text
monochromatic-git-policy-cli: the commit service tries this push again by itself in 10 seconds (try 2), and keeps trying with longer pauses until the push lands or a conflict needs you.
remote: error: GH009: Git LFS object integrity could not be checked. Please contact GitHub support.
```

#### Pros

- It is the same answer for both shapes,
  so the service stops depending on which words GitHub happened to send.
- It covers the longest measured span,
  `39.7` minutes,
  with nobody watching.

#### Cons

- Against the cause GitHub documents,
  a push that references an object the server lacks,
  no retry can ever succeed,
  so the plan would retry for ever every ten minutes and the record would promise
  "keeps trying" about something that cannot pass.
- The person is never told that the LFS upload may need attention,
  which is the one action GitHub's page asks for.
- On a loaded host it is unbounded work:
  one push every ten minutes for as long as the branch is refused.

### D. Re-upload the branch's LFS objects, then a bounded retry

On a `GH009` refusal the service runs `git lfs push --all origin` in the repository
with its own bound,
reads whether the upload reported every object present,
then runs the bounded retry of option B.
A failed upload stops the push and shows the upload's own words.

#### What a person at the terminal sees

```text
monochromatic-git-policy-cli: the server could not check the integrity of the Git LFS objects this push references, so the commit service offered this repository's LFS objects again (2180 objects, nothing the server lacked) and tries the push again in 10 seconds (try 2 of 5).
remote: error: GH009: Git LFS object integrity could not be checked. Please contact GitHub support.
```

#### Pros

- It acts on the cause GitHub documents,
  and its record says whether the repository's own objects were the problem.
- An upload that reports a missing object is a real,
person-fixable finding,
  which the other options never reach.

#### Cons

- On this machine no refused push referenced an LFS object at all,
  so the upload would have had nothing to fix in all 13 cases,
  and the check's statement would be a new reason to think the cause was found when it is not.
- A successful upload does not predict the server's check:
  the check is GitHub's,
  the maintainer says the failure is GitHub's,
  and the client cannot read it.
- The cost is unmeasured:
  offering 2,180 objects to the LFS server is one batch request whose size and duration were not probed,
  because the only way to probe it is to upload to the real repository.
- It is the most machinery of the five:
  a new child,
  a new bound,
  new words,
  and a new failure mode to test on every refusal.

### E. One more try after the first pause, then final

The refusal gets a second try at the first pause,
ten seconds by default,
and is final if the server refuses that as well.
The record says so in one sentence.

#### What a person at the terminal sees

```text
monochromatic-git-policy-cli: the server could not check the integrity of the Git LFS objects this push references; the commit service tries this push once more by itself in 10 seconds, and if the server refuses that try too it stops pushing this branch and this record stays for you.
remote: error: GH009: Git LFS object integrity could not be checked. Please contact GitHub support.
```

#### Pros

- It covers the shape the issue's three sightings described,
  a refusal that cleared within moments,
  for one extra push per refusal.
- Almost nothing to build:
  the plan already counts tries and already has the first pause.

#### Cons

- The measured median span is `3.8` minutes,
  so a ten-second bound ends before most refusals have cleared.
- It leaves the same asymmetry between the two shapes unless it is also given to the unreadable one,
  and giving it there would cut a retry that today runs without a count.

## Ranking

B > E > C > A > D.

- B beats E:
  both bound the retry,
  and B's five tries over two and a half minutes reach four times further into the
  measured spans than E's single ten-second try,
  for at most three more pushes of about four seconds each on this host.
- E beats C:
  GitHub documents a cause that no retry can mend and the client's maintainer attributes the refusal to
  the service,
  so a promise to keep trying for ever cannot be kept in the one case the reader most needs told;
  E ends with words the person can act on,
  C never ends.
- C beats A:
  both leave a person who is not watching to learn the outcome from a record,
  and C at least attempts the publication the service exists to make,
  where A waits for a commit that a finished round will never make.
- A beats D:
  D's repair has nothing to repair in this machine's record,
  thirteen refused pushes that referenced no LFS path at all,
  while its cost and its central claim both stay unmeasured,
  so A's words deliver the same publication and a truer account for a fraction of the machinery.

What the ranking weighs,
as the owner asked:
what a person at the terminal sees,
and what happens on a loaded host where pushes take four seconds and a branch may be quiet for hours.
Effort to build is stated per option and weighs nothing in the order.

## What each option costs to build

- A:
  one string and the tests that hold it,
  in `../../package/git-policy/cli/src/native/push/push_retry_failure.rs`,
  plus the `SPEC.md` and `README.md` passages that quote it.
- B:
  the counted-cause path that `PROTECTED_BRANCH_TRIES` already has,
  generalised to a second code,
  its words,
  and the controls of `../../package/git-policy/cli/src/native/landing/service_push_retry_tests.rs`.
- C:
  removing the decline reading for a pre-receive reason and one passage in `SPEC.md`;
  the smallest change of the four that alter behaviour,
  and the one whose promise cannot always be kept.
- D:
  the most:
  a child process with its own bound in the service's push path,
  its words,
  a control against a fixture LFS server,
  and the `SPEC.md` passage for the new step.
- E:
  the plan's existing count,
  set to two for this cause,
  and its words.

## Questions for the owner

1.  Does a `GH009` refusal get a bound,
    and is the bound two tries as E has it,
    five as B has it,
    or no count as C has it?
2.  Should the answer be one rule for both shapes of the refusal,
    the readable `GH009` and the unreadable `(failed)` pre-receive decline,
    so the service's retry no longer depends on which words GitHub sent?
3.  If a `GH009` refusal outlasts its tries,
    should the record name GitHub's documented remedy,
    `git lfs install` and `git lfs push --all origin`,
    above the server's words,
    as A and B propose?
4.  Is the client-side upload check of option D wanted at all,
    given that no refused push in this boot referenced an LFS object?
5.  Does the owner accept the three sightings as written in
    [the LFS note](../troubleshooting/github-lfs-integrity-rejection.md),
    which records them from the commit service's journal rather than from the rounds' reports?
