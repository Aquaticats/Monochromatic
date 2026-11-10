# GitHub's protected-branch refusal that cleared by itself, and the wrapper sentence that called it permanent

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

## Root cause

Two causes,
of different kinds.

The wrapper's sentence was our defect:
`plain_words` in `package/git-policy/cli/src/native/push/push_retry.rs`
said that trying again would be refused the same way,
which the push by hand disproved inside the same hour.
A server's refusal is its answer at that moment;
the wrapper cannot know what the next try will meet,
and nothing in the push's output tells it.

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
There is no harness that makes GitHub refuse a protected-branch update on demand,
so neither catalog the reporting rules ask for can be built,
and the reading rests on the two recorded events rather than on a reproduction.

The wrapper's side is verifiable and is verified by a test:
`the_words_of_a_final_failure_say_what_is_known_and_promise_no_outcome`
in `package/git-policy/cli/src/native/push/push_retry_tests.rs`
holds the decline's words to the three facts the wrapper knows
(the server's hook declined this push,
the service stops pushing this branch until the next commit,
a push by hand may succeed),
and holds them away from any claim about what a retry would meet.

The behaviour the words describe did not change,
because it is the owner's decision:

> Offline is retried,
> but auth and server hook declines stop until your next commit.

That decision is quoted in `package/git-policy/cli/SPEC.md` and `package/git-policy/cli/README.md`
with the same wording.

## Verified workaround

Trust a push by hand over the record's claim about the next try:

```sh
git push origin <branch>
```

Its tradeoff is that the person cannot know either:
the same command that succeeded once can be refused again,
so the workaround is to try,
not a promise that it lands.

The wrapper's own record now says what is known
(`package/git-policy/cli/src/native/push/push_retry.rs`,
`plain_words`):
the server's hook declined this push,
the commit service stops pushing this branch until the next commit,
and a push by hand may succeed if the cause has cleared.

## What does not work

- Reading a `GH006` refusal as a property of the branch or of the commit.
  The same commit landed with nothing changed,
  so the refusal described one moment.
- Making the wrapper retry a decline.
  The owner's decision keeps declines out of the retry plan,
  and a hosting service can refuse every push of a branch for reasons only a person can mend.
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
