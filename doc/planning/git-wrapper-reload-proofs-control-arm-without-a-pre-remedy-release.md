# The reload proof's control arm cannot reproduce the harm with any release this host holds

Status:
proposal written 2026-11-27T10:05:00-05:00 by the round on branch
`fix/git-policy-cli-issue-6587-service-manager-proofs-run-to-their-end`.
**Not approved and not final**:
the owner picks one option or rejects all of them,
and no code,
task or test is changed for this document.

## What the control arm is for

`mise run //package/git-policy/cli:test:service-manager`
(`package/git-policy/cli/bin/service-manager-proof.ts`)
runs one scenario on two machines,
one arm per machine.

The arm named `this worktree` runs the release built from this branch
on a machine that the release from `main` provisioned first,
which is the migration the remedy has to survive.
The arm named `main` runs the release the account installed from `main`
on a second,
fresh machine,
and the proof refuses to call the run a pass unless that second arm did two things:
it asked the manager to read every unit it holds at least once,
and it ended the unrelated `Type=dbus` service and the target bound to it.

Those two refusals are the sentences
`the release from main asked for no reload, so this fixture would prove nothing about it`
and
`the release from main did not end the unrelated desktop, so this fixture does not reproduce the harm`,
in `main_` of `service-manager-proof.ts`.
The proof's own docblock states the same contract:

> The arm from `main` has to end the unrelated service and its target,
> and the arm from this worktree has to keep both and ask for no manager-wide operation at all.

The task's `description` in `package/git-policy/cli/mise.toml`
and the README's section "The two service-manager proofs"
repeat it.

So the arm carries two jobs,
and both have to be named to judge it:

1. **It is the fixture's positive control.**
   The manager is a real `systemd --user`,
   loaded with about 1,500 units so that a reload of it costs what a reload of this host costs,
   and the desktop stand-in is a real unit pair with a start deadline
   (`bin/service-manager-machine.ts`,
   `bin/service-manager-desktop.ts`).
   Unless a manager-wide reload is seen to end that stand-in,
   the other arm's `0 reloads` and `the stand-in never ended`
   are the reading of a fixture that may be blind.
   `main_` says the same about a weaker control:
   "a control that only counted `Reload` calls in a stand-in manager would prove nothing about a real manager".
2. **It reproduces the harm on the code that caused it.**
   On 2026-11-12 a manager-wide reload during a commit held the manager for 139,770 ms,
   a reply to the window manager's unit expired while it did,
   and the session ended
   (`doc/troubleshooting/plasma-session-ended-after-service-manager-timeouts.md`).
   That the reload then ended the stand-in is measured in
   `doc/planning/git-wrapper-reload-remedy.md`,
   section "What the merged code measured":
   one reload of 304 ms and one of 308 ms,
   with the stand-in ended at the first step after it.

## What the arm printed on 2026-11-27

The run was
`mise run //package/git-policy/cli:test:service-manager`
with `--image localhost/git-policy-native-test:gpc-gate-wrapper-next-alpha-on-1c8acd5bfb-20261127T075019-0500`
(this host's gate image)
and `--main-wrapper` naming the executable of `0.6.1-alpha.48`,
the newest release this host holds.
Both machines held 1,557 units,
and both arms ran every step of the scenario.
The closing lines of each arm,
as the proof printed them:

```text
arm main:
  the manager was asked to re-read its units 0 time(s) during this arm's steps, 1 in this machine's whole life
  commits that did not land: none
  the desktop stand-in ended: never

arm this worktree:
  the machine was seeded by the release at .../.cache/wrapper-release-0.6.1-alpha.48/.../monochromatic-git-policy-cli, as one that has its template installed: [main 75178d0] seed | status=0
  the template the seeding left: ExecStart=/run/user/0/monochromatic-git-policy-cli-bin monochromatic-git-policy-cli service run --journal %S/monochromatic-git-policy-cli/service/%i --git /usr/bin/git --instance %i
  the manager was asked to re-read its units 0 time(s) during this arm's steps, 1 in this machine's whole life
  commits that did not land: none
  the desktop stand-in ended: never

the arm from main asked for 0 manager-wide reload(s); the arm from this worktree asked for 0
```

Every one of both arms' eight step lines read
`service=active target=active owner=s ":1.1502"` or `:1.1503`,
and the run ended with

```text
NativeVerificationError: THE PROOF FAILED: the release from main asked for no reload, so this fixture would prove nothing about it; the release from main did not end the unrelated desktop, so this fixture does not reproduce the harm
```

The `1 in this machine's whole life` is the fixture's own reload:
`startMachine` asks the manager for one after it loads the units,
and each arm subtracts the reloads it finds before its first step.
No commit failed to land on either arm,
so this run also shows that the container's Git now makes every commit these proofs make.

## Why no release this host holds can make the arm pass

The remedy is `ef1bbc27b6`
(`fix(git-policy-cli): a commit never makes the user manager read every unit it holds`,
2026-11-12T09:44:57-05:00),
and the first release that contains it is `0.6.1-alpha.20`
(`git tag --contains ef1bbc27b6`).
Every wrapper this host holds was built after it:

- `0.6.1-alpha.44`,
  in `~/.local/share/monochromatic-dev/toolchain/cargo-monochromatic-git-policy-cli/`;
- `0.6.1-alpha.47` and `0.6.1-alpha.48`,
  in the checkout's `.cache/wrapper-release-0.6.1-alpha.*/`;
- a build of `main` at `ee0cf2268b` (2026-11-22),
  in `.cache/wrapper-built-by-cargo-from-main-ee0cf2268b-2026-11-22/`.

The arm's premise is that the release it runs asks the manager to read every unit
when a unit text changes:
that is a release from before the remedy,
which asked for a reload whenever the template it found was not the one it wrote.
A release from `0.6.1-alpha.20` onwards never asks for that,
so the arm cannot reach either of its two readings,
whatever the seed is.
`--harm-seed-wrapper` cannot help either:
a machine seeded with `0.6.1-alpha.48` itself,
which is the default
(`--seed-wrapper` defaults to `--main-wrapper`),
holds the same text the arm's own release writes,
which is why the migration arm's seeded template printed
`ExecStart=/run/user/0/monochromatic-git-policy-cli-bin ...`.

Measured for this document:
a machine seeded with `0.6.1-alpha.47` prints the same
`ExecStart=/run/user/0/monochromatic-git-policy-cli-bin ...` line,
because `PROGRAM_LINK` arrived with `ef1bbc27b6` itself.
So every release from `0.6.1-alpha.20` on writes one text,
and no pair of them is the text change the 2026-11-12 code reloaded for.

## A second reading of the same run: no arm meets an earlier release's template

The task's description says the arm from this worktree runs on
"a machine that has the earlier release's template installed",
and the README says `--harm-seed-wrapper` names an older release
"which is what makes the arm from `main` meet a template that release did not write".
Neither statement is exercised by the run above:
both arms' machines hold the text their own release writes.
The task therefore proves less today than its own description says,
in two places,
and the second one does not need the control arm to be fixed first.

What still writes a different text is a release from before `0.6.1-alpha.20`:

- `0.6.1-alpha.13` filled `@SERVICE_BINARY@` with the absolute path of the executable that wrote the text,
  which `doc/planning/git-wrapper-reload-remedy.md` records as
  `ExecStart=/opt/release-seed/git ...`;
- `0.6.1-alpha.14` filled it with the notes folder's `service-program` pointer,
  which the same section records as
  `ExecStart=/run/user/0/systemd/user/monochromatic-git-policy-cli-worktree-notes/service-program ...`.

So a machine that meets an earlier release's text needs a pre-remedy build,
or a text written by the fixture.

## Option A: keep a pre-remedy release for the control arm

What it is:
the `main` arm runs a build of a release from before `ef1bbc27b6`,
which is what the arm's docblock means by "the code the reload comes from".
Two pre-remedy builds are needed,
not one:
the arm's own release and the build that seeds its machine must write different texts,
and a single build at two paths writes the same text twice
(measured:
`0.6.1-alpha.47` at two paths printed one text).
The pair the remedy was measured with is `0.6.1-alpha.13` as the seed,
whose text names the executable's absolute path,
and a later pre-remedy release as the arm's wrapper,
whose text names the notes folder's pointer.

Where they would come from:
the published release assets.
`gh release view monochromatic-git-policy-cli-v0.6.1-alpha.19 --json assets`
answers one x86_64 asset,
`monochromatic-git-policy-cli-0.6.1-alpha.19-x86_64-unknown-linux-gnu.tar.gz`,
10,921,701 bytes with sha256
`c044c816b7bdf988fea6d6fbb74ad0d0d79a08f0ac1f9299c35c1bed62d2bfbb`,
so
`gh release download monochromatic-git-policy-cli-v0.6.1-alpha.19 --pattern '*x86_64-unknown-linux-gnu.tar.gz'`
fetches it,
and `0.6.1-alpha.13` publishes the same shape of asset.
They would live beside the two releases the checkout's `.cache` already holds.

Pros:

- the reload is asked by code nobody wrote for the test,
  so the fixture's sensitivity is demonstrated and not asserted;
- both jobs stand together:
  a real release asks for the reload,
  the manager records it,
  and the stand-in ends;
- it also closes the gap in "A second reading":
  a pre-remedy seed writes the text the branch build must migrate from,
  so the arm from this worktree would meet an earlier release's template for the first time since `0.6.1-alpha.20`.

Cons:

- every host that runs the task needs those two binaries:
  a network fetch from the published assets,
  or a copy carried by hand;
- it freezes 2026-11-13's code.
  The arm can never say anything about a current release,
  and if the assets are ever removed the task stops running;
- it keeps two machines of about 1,500 units each,
  which is the task's whole cost today;
- the pre-remedy builds have not been run in this host's gate image,
  so whether their commit path still works there (protocol,
  Git behaviour) is unmeasured;
- it makes `test:service-manager` depend on a binary this repository no longer builds or tests.

## Option B: a stand-in that asks the manager for the reload

What it is:
the `main` arm runs a small program of the proof's own instead of a release.
It makes the scenario's commits with real Git and,
at the point where the 2026-11-12 code asked,
calls `org.freedesktop.systemd1.Manager.Reload` on the machine's user manager
(the member `systemd_dbus.rs` named at `ef1bbc27b6^`,
called by `ask_manager` in `service_provision.rs`),
so the manager reads every unit and the desktop stand-in ends.

Pros:

- no frozen binary,
  no network,
  no release asset to depend on;
- the operation is the member the removed code called,
  and the manager is real;
- every run exercises the fixture's whole chain,
  so a manager upgrade that stopped ending the stand-in fails here.

Cons:

- the stand-in has to make the commits itself,
  so the test grows a second implementation of the thing it measures,
  and the control stops following the wrapper's own commit path;
- it proves the fixture and nothing about a release,
  so the arm's second job is gone and has to be replaced by the record;
- it keeps the second machine and its 1,500 units,
  and it makes `--harm-seed-wrapper` meaningless;
- it does not close the gap in "A second reading":
  the stand-in writes no earlier text either.

## Option C: retire the arm, and plant the reload in the arm that remains

What it is:
`test:service-manager` runs one machine instead of two,
and that machine is the migration:
the release built from this branch,
on a machine the installed release provisioned.
After the desktop stand-in is up,
the proof asks the manager for one manager-wide reload itself,
requires that the stand-in and its target end,
starts them again,
and then measures the arm's steps as it does now,
with the reload count taken after the planted one.
The judgement loses the two `onMain` problems and gains one:
a planted reload that did not end the stand-in fails the run.

What then proves the remedy:

- the release under test asks the manager for no manager-wide reload on a loaded machine
  (`0 time(s) during this arm's steps`),
  and the same run has just shown that this manager's reload does end the stand-in;
- the machine's manager is real and holds about 1,500 units,
  which is what makes the absence of a reload mean something;
- the 2026-11-12 reproduction stays where it is recorded,
  in `doc/planning/git-wrapper-reload-remedy.md` and in the incident's record,
  and the fixture's power is measured again on every run instead of once.

Pros:

- one machine and one arm:
  about half of the task's cost in time and memory;
- no frozen binary,
  no network,
  no second implementation;
- the fixture's sensitivity and the remedy are measured on one machine in one run,
  so a fixture that went blind fails at the planted step instead of passing quietly;
- the planted step is a line of the printed report,
  so nothing is claimed from silence.

Cons:

- a real release no longer reproduces the harm on each run,
  so if the container's manager changed in a way that made its reload harmless,
  the planted step would pass for the same reason a real release would,
  and neither would tell the two apart;
- it changes what a published task proves,
  so the proof,
  the task's description and the README's section all change;
- the report gains a step whose subject is the fixture,
  and a reader must not take it for something the wrapper did;
- it does not close the gap in "A second reading".

## A separable question: does any arm meet an earlier release's template?

Whichever option above is picked,
the arms still meet only the text their own release writes,
which is the second reading this round found.
That is a separate decision,
and the cheapest way to close it is option D.

### Option D: write an earlier release's unit text into the machine

What it is:
before the arm's first step,
the fixture writes the two unit files in the form a pre-remedy release wrote them,
which is a captured form and not a guess:
`ExecStart=/run/user/0/systemd/user/monochromatic-git-policy-cli-worktree-notes/service-program monochromatic-git-policy-cli service run ...`
(the `0.6.1-alpha.14` form recorded in `doc/planning/git-wrapper-reload-remedy.md`),
or the `0.6.1-alpha.13` form that names an absolute executable path.
The proof's `prepare` already writes the machine's units by making one commit through a release,
so this step replaces that seeding and needs no second binary.

Pros:

- the migration is exercised again:
  the branch build meets a text it did not write,
  and its own rule (drop and restart an instance held at an older text,
  ask for no reload) is measured;
- no binary,
  no network,
  and the machine is the one the task already starts;
- it works with any of options A,
  B and C.

Cons:

- the text is a stand-in for a release's output,
  so it must stay a copy of the recorded form and needs the comment saying what it is;
- it says nothing about the wrapper's own first commit after an *install*:
  the install proof covers that case with a real pair of releases.

## Ranking

For the control arm's own question,
**C > A > B**.

C over A:
both buy the fixture's sensitivity on every run,
and C buys it on the machine that measures the remedy,
with no frozen binary,
no network and half the machines.
A's extra purchase is a repetition of a fact that is already recorded and cannot recur,
because no release in use is pre-remedy,
and that purchase costs the proof a downloaded binary,
a dependency on a published asset
and a release that nothing here builds.

A over B:
A's reload is asked by the code that caused the incident,
while B asks it from a program written for the test,
so B's control drifts with the wrapper's commit path and says nothing about a release at all.
B's cost is the lower one,
but what it delivers is a fixture check,
which C delivers too,
on one machine.

For the earlier template,
**D > A**,
and the two are separable:
D costs one fixture step and no artifact,
and what it must reproduce is a fixed historical text,
so a recorded copy is as good as the release that wrote it.
A covers this question as well,
which is where it is strongest;
C and B leave it open on their own.

If the owner's priority is the fixture's evidence at the lowest cost,
the answer is C and then D.
If it is a real release reproducing the harm on each run,
the answer is A and then D.

## What a decision would need

- the owner's pick for the arm,
  or a rejection of all three;
- for A:
  whether the checkout's `.cache` may carry two pre-remedy releases,
  and whether the task may fetch them (the assets are public,
  and `gh` reaches them from this host);
- for C:
  whether a published task may stop running a second arm,
  and whether `the fixture's own reload, planted` is clear enough as a printed step name;
- for D:
  whether the fixture may write a unit text itself,
  and which pre-remedy form it copies;
- for all of them:
  whether `--harm-seed-wrapper` keeps its place (A and D keep it,
  B and C drop it).

## What this document does not do

It changes no code,
no task and no test.
It records what the control arm printed on 2026-11-27,
what it is for,
and the choices this round could find,
and it leaves the choice to the owner.
