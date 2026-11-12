# GitHub's protected-branch refusal that clears by itself, and what the wrapper does with it

## Symptom

At `2026-11-09T23:30:11-05:00` a push of commit `29bae9bb87` to `main` reached GitHub and was refused:

```text
remote: error: GH006: Protected branch update failed for refs/heads/main.
```

The reason after the code was empty.
The wrapper's operator printed the record for a server hook's decline,
which then said:

```text
monochromatic-git-policy-cli: the server refused this push: a hook on the server declined it, with the message shown below. Trying again would be refused the same way, so the commit service stops pushing this branch until your next commit. Fix what the message names, then push by hand or commit again.
```

A plain `git push origin main` of the same commit,
with nothing changed,
then succeeded.

Releases before 2026-11-11 still stop on such a refusal,
and a person reading their record is told the branch waits for their next commit.
A release from 2026-11-11 on retries it,
by the decision quoted under "What does not work".

## Root cause

Two causes,
of different kinds.

The wrapper's sentence was our defect:
`plain_words` in `package/git-policy/cli/src/native/push/push_retry_failure.rs`
said that trying again would be refused the same way,
which the push by hand disproved inside the same hour.
A server's refusal is its answer at that moment;
the wrapper cannot know what the next try will meet,
and nothing in the push's output tells it.

The wrapper's practice was our defect too,
and the owner answered it on 2026-11-11 with
"Make the wrapper auto retry on GH006":
the classifier read every `[remote rejected]` line whose reason ends in `hook declined` as a decline no retry can mend,
and GitHub writes that same reason,
`protected branch hook declined`,
for the refusal it answers with `GH006`.
So the one refusal the repository recorded as clearing by itself three times
was the one the wrapper never tried again.
`PROTECTED_BRANCH_REFUSAL` and `protected_branch_refusal` in that file now read GitHub's words for a refused
update,
`PROTECTED_BRANCH_TRIES` bounds the retry at five tries,
and `out_of_tries` stops it;
a hook decline with no such code anywhere in the push's output stays final,
as the owner's decision of 2026-11-03 keeps it.

GitHub's own cause is not known and is not established here:
the service is closed source,
so no call chain can be read,
and one observation cannot name it.
What the record around it holds:

- The commit was signed,
  and `main` requires signatures,
  so a forgotten signature is one candidate,
  ruled out for this push only by the message naming `GH006` and not a signature error.
- GitHub's status API listed Git Operations as operational.
- In the same minute another push was answered with `Internal Server Error`
  and a third with `GH009`,
  which points at the service rather than at the commit.

None of these is a cause,
and this page does not claim one.

## Verification

The observation is the whole evidence:
one refusal at a second-precise time,
and one plain push of the same commit that landed minutes later.
There is no harness that makes GitHub itself refuse a protected-branch update on demand,
so the catalog of GitHub's own causes cannot be built,
and the reading of those refusals rests on the recorded events rather than on a reproduction.
What the wrapper does with one is reproduced below with a local remote whose hook answers with GitHub's code.

It happened again at `2026-11-10T02:24:59-05:00`,
on the push of commit `1535eab9bf`,
with the same code and the same empty reason.
This time the signature candidate was checked directly:
`git log origin/main..main --format='%G?'` printed `G` for all 14 unpushed commits.
A plain `git push origin main`,
with nothing changed,
then moved `origin/main` from `a5851271a0` to `1535eab9bf`.
So a good signature on every commit does not prevent this refusal,
and the cause is still not known.

It happened a third time at `2026-11-11T06:42-05:00`,
on the commit service's auto-push of `main`,
one minute after another session had pushed `main`:
GitHub refused it with the same code and the same empty reason,
and a plain `git push origin main` of the same commit about two minutes later was taken as a fast-forward.
That third refusal had a measured clearance for the first time,
and it is the figure the retry's bound cites:
five tries whose default pauses add up to 150 seconds put the last try past it.

The wrapper's side is verifiable and is verified by tests.
`the_words_of_a_final_failure_say_what_is_known_and_promise_no_outcome`
and `a_github_protected_branch_refusal_is_tried_again`
in `package/git-policy/cli/src/native/push/push_retry_failure_tests.rs`
hold a plain decline's words to the three facts the wrapper knows
(the server's hook declined this push,
the service stops pushing this branch until the next commit,
a push by hand may succeed),
hold them away from any claim about what a retry would meet,
and hold GitHub's own recorded refusal out of that decline.
`a_protected_branch_refusal_is_given_a_fixed_number_of_tries`
and `the_words_of_a_refusal_that_outlasted_its_tries_name_the_tries_and_the_time`
hold the bound and the words of the record that ends it.

The service's own behaviour is verified against a local bare remote whose `pre-receive` hook answers as GitHub did:
`a_push_a_server_refuses_with_gh006_is_tried_again_and_lands_without_a_new_commit`
refuses the first push and takes the next,
and the service lands the commit with no new commit and no push by hand;
`a_protected_branch_refusal_that_keeps_coming_back_ends_and_tells_the_person`
refuses every push and holds the service to its five tries and to the words of the record that ends them.
Both are in `package/git-policy/cli/src/native/landing/service_push_retry_tests.rs`.
The hook prints the `GH006` line on its standard error,
which a client reads as a `remote:` line,
so the code reaches the push's output the way GitHub's does;
the reason Git prints beside it is its own `pre-receive hook declined`,
which is why the classifier reads the code and not the reason.

The behaviour of a plain decline did not change,
because it is the owner's decision:

> Offline is retried,
> but auth and server hook declines stop until your next commit.

That decision is quoted in `package/git-policy/cli/SPEC.md` and `package/git-policy/cli/README.md`
with the same wording,
and GitHub's refusal is carved out of it by the owner's answer of 2026-11-11:

> Make the wrapper auto retry on GH006

## Verified workaround

Trust a push by hand over the record's claim about the next try:

```sh
git push origin <branch>
```

Its tradeoff is that the person cannot know either:
the same command that succeeded once can be refused again,
so the workaround is to try,
not a promise that it lands.

The wrapper's own record says what is known
(`package/git-policy/cli/src/native/push/push_retry_failure.rs`,
`plain_words` for a plain decline,
`out_of_tries_words` for a refusal that outlasted its tries):
the server's hook declined this push,
the commit service stops pushing this branch until the next commit,
and a push by hand may succeed if the cause has cleared.
A release from 2026-11-11 on usually makes the push by hand unnecessary:
it tries the refusal five times itself first.

## What does not work

- Reading a `GH006` refusal as a property of the branch or of the commit.
  The same commit landed with nothing changed,
  so the refusal described one moment.
- Making the wrapper retry a decline without a bound.
  The owner asked on 2026-11-11 for the protected-branch code to be retried,
  and a hosting service can refuse every push of a branch for reasons only a person can mend
  (an unsigned commit on a branch that requires signed commits is one),
  so the retry stops after `PROTECTED_BRANCH_TRIES` tries and the person is told what the server said.
  A hook decline with no such code is still not retried at all.
- Naming GitHub's cause from the wrapper's side.
  The refusal carries a code and an empty reason,
  and a hosting service can print any text in that position,
  so no rule about the text can tell the two apart.

## Upstream filing decision

Nothing to file.
Constraint 1 of the filing check fails:
the service is closed source,
so there is no call chain to trace and no version to test against.
Constraint 2 fails with it:
a single refusal at an unknown cause cannot be handed to a vendor as a diagnosis,
and the part that is ours,
the sentence,
was fixed here.
