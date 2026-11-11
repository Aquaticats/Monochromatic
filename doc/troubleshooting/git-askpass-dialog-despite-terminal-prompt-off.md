# Git 2.56.0 and git-lfs 3.8.1 open the desktop password dialog even with `GIT_TERMINAL_PROMPT=0`

## Symptom

A KDE dialog titled for SSH credentials appears on the desktop with a prompt such as:

```text
Username for "http://127.0.0.1:18765"
```

It was seen at 2026-11-07T06:21:00-05:00,
open for 115 seconds by then,
while an agent ran an LFS end-to-end test against a fake LFS server on `127.0.0.1:18765`;
it was closed at 2026-11-07T06:23:10-05:00.
The dialog was `/usr/bin/ksshaskpass` (PID 3682988).
Its parent had already exited,
so `systemd --user` had adopted it,
and it sat open with nobody left to read the answer.
Its control group was `app.slice/claude-code-bash` and its working folder was the test's throwaway repository,
so an agent's shell command started it,
not a desktop program.

The quoting tells which program asked:

- git-lfs's own prompt quotes the URL with double quotes:
  `Username for "http://…"`.
- Git's prompt quotes it with single quotes and a colon:
  `Username for 'http://…': `.

The command had `GIT_TERMINAL_PROMPT=0` set,
which many scripts treat as "never ask".
It is not.

One `git lfs push` against a server that keeps answering 401 asked for a username and password six times each
(see "Verification"),
so one test run can open several dialogs in a row.

## Root cause

This desktop session exports `SSH_ASKPASS=/usr/bin/ksshaskpass`,
and the systemd user manager carries it too
(`systemctl --user show-environment` lists `SSH_ASKPASS=/usr/bin/ksshaskpass`,
`DISPLAY=:0` and `WAYLAND_DISPLAY=wayland-1`).
Every shell an agent starts,
and every user service,
inherits it.

### Git tries three prompt programs before it looks at `GIT_TERMINAL_PROMPT`

[`prompt.c:46-77` at v2.56.0][git-prompt]:

```c
// prompt.c (git v2.56.0, a018953688f1b10bddf91bff8747068f5f4746a4)
char *git_prompt(const char *prompt, int flags)
{
	char *r = NULL;

	if (flags & PROMPT_ASKPASS) {
		const char *askpass;

		askpass = getenv("GIT_ASKPASS");
		if (!askpass)
			askpass = repo_config_values(the_repository)->askpass_program;
		if (!askpass)
			askpass = getenv("SSH_ASKPASS");
		if (askpass && *askpass)
			r = do_askpass(askpass, prompt);
	}

	if (!r) {
		const char *err;

		if (git_env_bool("GIT_TERMINAL_PROMPT", 1)) {
			r = git_terminal_prompt(prompt, flags & PROMPT_ECHO);
			err = strerror(errno);
		} else {
			err = "terminal prompts disabled";
		}
		if (!r) {
			/* prompts already contain ": " at the end */
			die("could not read %s%s", prompt, err);
		}
	}
	return r;
}
```

The order is `GIT_ASKPASS`,
then `core.askPass`,
then `SSH_ASKPASS`.
`GIT_TERMINAL_PROMPT` is read only when none of them produced an answer,
and it only decides whether to fall back to the terminal.
Two details matter for the fix:

- `if (!askpass)` tests for an unset variable,
  so a `GIT_ASKPASS` that is set to the empty string stops the search:
  `core.askPass` and `SSH_ASKPASS` are never read.
- `if (askpass && *askpass)` then skips the empty value,
  so no program runs and Git reaches the
  "terminal prompts disabled" error.

### git-lfs reads the same three settings itself

git-lfs does not only go through `git credential fill`;
it has its own prompt helper.
[`creds/creds.go:104-121` at v3.8.1][lfs-creds-order]:

```go
// creds/creds.go (git-lfs v3.8.1, a662a6dbfedf706cfa9fc7a5b7bcc7ba2e2c8a65)
	askpass, ok := osEnv.Get("GIT_ASKPASS")
	if !ok {
		askpass, ok = gitEnv.Get("core.askpass")
	}
	if !ok {
		askpass, _ = osEnv.Get("SSH_ASKPASS")
	}
	if len(askpass) > 0 {
		askpassfile, err := tools.TranslateCygwinPath(askpass)
		if err != nil {
			tracerx.Printf("Error reading askpass helper %q: %v", askpassfile, err)
		}
		if len(askpassfile) > 0 {
			c.askpassCredHelper = &AskPassCredentialHelper{
				Program: askpassfile,
			}
		}
	}
```

`ok` is true for a variable that is set but empty,
so an empty `GIT_ASKPASS` again stops the search,
and `len(askpass) > 0` then leaves the prompt helper out.

The double-quoted prompt in the dialog comes from [`creds/creds.go:280-282`][lfs-creds-prompt]:

```go
// creds/creds.go (git-lfs v3.8.1)
	// 'cmd' will run the GIT_ASKPASS (or core.askpass) command prompting
	// for the desired valueType (`Username` or `Password`)
	cmd, errVal := subprocess.ExecCommand(a.Program, a.args(fmt.Sprintf("%s for %q", valueString, u))...)
```

`%q` is Go's double-quoted string,
which is why the dialog read `Username for "http://127.0.0.1:18765"`.

### git-lfs retries each request

[`lfsapi/auth.go:18-32`][lfs-auth]:

```go
// lfsapi/auth.go (git-lfs v3.8.1)
var (
	defaultEndpointFinder  = NewEndpointFinder(nil)
	defaultMaxAuthAttempts = 3
)

// DoWithAuth sends an HTTP request to get an HTTP response. It attempts to add
// authentication from netrc or git's credential helpers if necessary,
// supporting basic authentication.
func (c *Client) DoWithAuth(remote string, access creds.Access, req *http.Request) (*http.Response, error) {
	maxAuthAttempts := defaultMaxAuthAttempts
	if access.Mode() == creds.NoneAccess {
		maxAuthAttempts++
	}

	for i := range maxAuthAttempts {
```

Each LFS request is tried up to three times (four when it starts with no credentials),
and one push makes more than one request,
which explains the repeated prompts in the probe.
The probe counted prompts,
not requests per endpoint,
so how the six username prompts split across requests was not measured.

### Why the dialog outlived its command

`ksshaskpass` is a separate process.
When the agent's command was stopped,
the Git and git-lfs processes went away but the dialog did not:
the observed dialog's parent was `systemd --user` (PID 5697).
So a command time limit does not close the dialog it opened.

## Verification

### Versions

- Git 2.56.0 at `/usr/bin/git` (`git --version`),
  source tag `v2.56.0`,
  commit `a018953688f1b10bddf91bff8747068f5f4746a4`.
- git-lfs 3.8.1 from the repository toolchain (`git-lfs version` prints `git-lfs/3.8.1 (GitHub; linux amd64; go 1.27.3; git a662a6db)`),
  source tag `v3.8.1`,
  commit `a662a6dbfedf706cfa9fc7a5b7bcc7ba2e2c8a65`.

### Harness

The probe starts an HTTP server on a free local port that answers every request with `401` and
`WWW-Authenticate: Basic realm="probe"`.
A recorder program stands in for `ksshaskpass`:
it appends its prompt to a log and answers `probe-user`,
so no dialog opens.
Each run uses an empty global configuration (`GIT_CONFIG_GLOBAL=/dev/null`,
`GIT_CONFIG_NOSYSTEM=1`),
a throwaway `HOME`,
`PATH` limited to `/usr/bin:/bin` plus a link to `git-lfs`,
and `GIT_TERMINAL_PROMPT=0` in every arm.

```ts
// ${HOME}/temp/agent/askpass-probe-script/probe.ts (scratch; the run's root was ~/temp/agent/askpass-probe-lQKTMI)
// Arms, each run against `git ls-remote http://127.0.0.1:<port>/probe.git`
// and against `git lfs push origin HEAD` in a repository whose lfs.url is the probe server:
const arms: [string, Record<string, string>][] = [
  ['control: SSH_ASKPASS set, GIT_ASKPASS unset', { SSH_ASKPASS: recorder }],
  ['fix: SSH_ASKPASS set, GIT_ASKPASS set but empty', { SSH_ASKPASS: recorder, GIT_ASKPASS: '' }],
  ['alternative: SSH_ASKPASS and GIT_ASKPASS both unset', {}],
];
```

It finished at 2026-11-07T06:25:36-05:00 (the time of its `result.json`) in `systemd-run --user --scope` with `CPUQuota=200%`,
`MemoryMax=1G`,
`TasksMax=200` and `IOWeight=10`.

### Runs that open a prompt program

`git ls-remote`,
control arm:
exit 128,
the recorder was called twice.

```text
Username for 'http://127.0.0.1:44799': 
Password for 'http://probe-user@127.0.0.1:44799':
fatal: Authentication failed for 'http://127.0.0.1:44799/probe.git/'
```

`git lfs push origin HEAD`,
control arm:
exit 2,
the recorder was called twelve times,
six username prompts each followed by a password prompt,
all in git-lfs's double-quoted form,
ending with:

```text
batch response: too many authentication attempts
```

### Runs that open no prompt program

`git ls-remote`,
fix arm (`GIT_ASKPASS` set but empty) and alternative arm (both unset):
exit 128,
the recorder was never called.

```text
fatal: could not read Username for 'http://127.0.0.1:44799': terminal prompts disabled
```

`git lfs push origin HEAD`,
fix arm and alternative arm:
exit 2,
the recorder was never called.

```text
fatal: could not read Username for 'http://127.0.0.1:44799': terminal prompts disabled
batch response: Git credentials for http://127.0.0.1:44799 not found.
```

### Not exercised

- A `core.askPass` entry in the configuration:
  no arm set one.
- A credential helper that does answer (for example `credential.helper store`):
  no arm configured one,
  so the claim that helpers still work with an empty `GIT_ASKPASS`
  rests on reading `prompt.c` (the prompt runs only when no helper answered),
  not on a run.
- SSH remotes:
  OpenSSH reads `SSH_ASKPASS` itself for key passphrases and host prompts,
  and `GIT_ASKPASS` does not change that;
  no SSH arm ran.

## Verified workarounds

### Set `GIT_ASKPASS` to the empty string and `GIT_TERMINAL_PROMPT=0`

```sh
# Any agent-run Git or git-lfs command that may reach a server needing a login.
GIT_ASKPASS= GIT_TERMINAL_PROMPT=0 git lfs push origin HEAD
```

In Rust tests:
`.env("GIT_ASKPASS", "").env("GIT_TERMINAL_PROMPT", "0")`.

Tradeoffs:

- It also skips a `core.askPass` from the configuration (Git and git-lfs both stop at a set `GIT_ASKPASS`),
  so a deliberately configured prompt program is ignored too.
  The probe did not set `core.askPass`;
  this follows from the source quoted in "Root cause".
- A command that needs a login and has no credential helper answer now fails at once instead of waiting for a person.
  That is the intent for unattended runs,
  and wrong for a person who wants to type a password.
- OpenSSH prompts are unaffected,
  as noted in "Not exercised".

### Remove both `SSH_ASKPASS` and `GIT_ASKPASS` from the environment

```sh
env --unset=SSH_ASKPASS --unset=GIT_ASKPASS GIT_TERMINAL_PROMPT=0 git lfs push origin HEAD
```

Tradeoffs:
a `core.askPass` from the configuration still opens its program,
because Git and git-lfs read the configuration when `GIT_ASKPASS` is unset.
It also stops OpenSSH from using the desktop dialog for key passphrases.
The empty `GIT_ASKPASS` workaround is the stronger one for Git and git-lfs.

### Existing repo practice: `GIT_ASKPASS=/bin/true`

`package/git-policy/cli/src/native/landing/service_push_retry_tests.rs` sets
`("GIT_TERMINAL_PROMPT", "0"), ("GIT_ASKPASS", "/bin/true")` so its tests never reach a person.
That also prevents the dialog,
but the prompt program answers with an empty name,
so Git sends a request with empty credentials before failing with an authentication error
rather than "terminal prompts disabled",
and git-lfs may retry with those empty answers.
The probe did not run this form.

## What does not work

- `GIT_TERMINAL_PROMPT=0` alone:
  the control arms ran with it and still called the prompt program
  twice for Git and twelve times for git-lfs.
- A time limit on the command:
  it stops Git and git-lfs but leaves the dialog open,
  as the orphaned `ksshaskpass` with parent `systemd --user` showed.
- Running without a terminal:
  Git does not need a terminal to run a prompt program,
  and the dialog needs only the inherited `DISPLAY` or `WAYLAND_DISPLAY`.

## What this repository does about it

The wrapper's production code set neither `GIT_ASKPASS` nor `GIT_TERMINAL_PROMPT` at main `73173e0f41` (2026-11-07T06:30:33-05:00)
(`rg 'GIT_ASKPASS|GIT_TERMINAL_PROMPT' package/git-policy/cli/src` found them only in tests and in the list of
names a plain read tolerates,
`repo_read_layout_environment_rules.rs`).
Its background service runs under the systemd user manager,
which carries `SSH_ASKPASS` and a display,
so a background push whose credential helper gives no answer could open this dialog.
That was a reading of the code,
not a run.

The owner's answers,
recorded in `ad1802df3c` (2026-11-07T06:37:42-05:00):

- Asked what the wrapper's background Git should do,
  the owner picked "Never prompt (Recommended)":
  background Git and git-lfs run with `GIT_ASKPASS=` and `GIT_TERMINAL_PROMPT=0`,
  so a missing credential fails at once and is reported,
  built with a test that first reproduces the dialog.
  Foreground commands the user types and the wrapper forwards keep Git's normal prompting.
  The work is on branch `fix/git-policy-cli-background-git-never-prompts`.
- Asked how agents should be kept from opening the dialog,
  the owner answered "Neither,
  rule pkg-local.":
  no rule in the root `AGENTS.md` and no Claude Code settings change,
  but a tagged rule in each affected package's own docs.
  The `package/git-policy/cli` rule comes with the branch named in the previous answer.
  `package/config/lfs-r2-worker`,
  the package the LFS work that ran the dialog-opening test is about,
  gets its rule when the owner lifts the pause on that work ("pause the lfs thread until I say unpause lfs").
- The wrapper now does what the first answer says,
  in `2cdf47c5bc` (2026-11-07T07:00:17-05:00),
  after the red controls of `06337a9b05` (2026-11-07T06:53:59-05:00).
  The commit service's Git and git-lfs children (landing and upkeep through the service's `GitContext`,
  and the push,
  the reconcile fetch and the LFS upload through each push's environment) start from `service_child_overlay` in
  `package/git-policy/cli/src/native/invocation/child_environment.rs`,
  which adds `GIT_ASKPASS` set to the empty string and `GIT_TERMINAL_PROMPT=0`.
  `SSH_ASKPASS` stays in the environment for OpenSSH.
  Commands the user types and the wrapper forwards,
  and the probes and the LFS upload of a typed `git push`,
  are unchanged.
  Before the fix the push control's recorder,
  standing in for `ksshaskpass` through `SSH_ASKPASS`,
  was asked
  `Username for 'http://127.0.0.1:<port>'` and `Password for 'http://probe-user@127.0.0.1:<port>'`,
  and the LFS control's
  stand-in `git-lfs` asked it `Username for "http://127.0.0.1"`;
  after the fix neither is asked and the push ends as the
  login failure the person is told about ("terminal prompts disabled",
  final,
  no retry).
  The LFS control uses a stand-in that follows the `creds/creds.go` order quoted above,
  because the gate's image has no
  `git-lfs`;
  the real git-lfs is covered by the probe in "Verification",
  not by the wrapper's tests.
  The `package/git-policy/cli` rule is `NDG` in that package's `README.md`,
  section "Git commands that may need a login".

### A control of the invocation overlay asks the terminal itself

`90991fa86e` (2026-11-10T07:47:34-05:00) made the overlay of a typed command `invocation_overlay`,
which adds the never-prompt pair when the process has no controlling terminal.
The controls that compared a plan against a hardcoded overlay
(`entry_tests.rs`'s `forward`,
read by `entry_announced_tests.rs` and `entry_stop_tests.rs`)
expected the pair in no case at all,
so ten of the twenty-nine tests of `invocation::entry` failed in a session without a terminal
while the whole module passed under one.
The ten are `a_global_prefix_keeps_the_query_announced`,
`the_four_spellings_plan_an_announced_child_run`,
`the_other_queries_stay_an_exact_replacement`,
`forward_target_naming_this_executable_stops`,
`inspection_commands_are_forwarded_without_wrapper_controls`,
`invocations_git_answers_itself_are_forwarded_unchanged`,
`inspection_commands_do_not_load_configuration`,
`policy_commands_outside_a_worktree_use_defaults`,
`policy_commands_validate_configuration_then_run_policies` and
`the_opt_out_and_short_circuit_forms_are_not_settled`.
The expectation is `expected_overlay` now,
which asks `control_terminal()` the way the plan does,
so a session with a terminal and one without each check the overlay its own answer produces.
Measured on branch `fix/git-policy-cli-fsmonitor-off-for-the-wrappers-own-git`
after it merged the jam-fix integration,
with `mise run //package/git-policy/cli:native:test:host -- invocation::entry::`:
`29 tests run: 19 passed, 10 failed` without a controlling terminal,
`29 tests run: 29 passed` with one (through `script -qec`),
and `29 tests run: 29 passed` without one after the fix.

`expected_overlay` was replaced on 2026-11-10 by `5f724cd494` (`inputs_at(root, Terminal)`):
the controlling terminal is a `ResolutionInputs` field gathered once beside the other process facts,
planning reads that answer instead of asking the kernel for it,
and `a_command_without_a_controlling_terminal_never_prompts` pins the overlay of both answers,
so no control's answer depends on the terminal the runner happens to have.

## Upstream filing decision

Checked for `ccdf827304` (2026-11-07T06:30:01-05:00).
`.out-of-scope/` has no entry for Git or git-lfs.
Nothing is drafted,
because the first constraint fails.

1.  Is it upstream's fault?
    No.
    The order is Git's documented behavior.
    The `gitcredentials` manual ([gitcredentials][git-doc-credentials]) says,
    read for `ccdf827304` (2026-11-07T06:30:01-05:00):
    "If the GIT_ASKPASS environment variable is set,
    the program specified by the variable is invoked.",
    then "Otherwise,
    if the core.askPass configuration variable is set",
    then "Otherwise,
    if the SSH_ASKPASS environment variable is set",
    and only then "Otherwise,
    the user is prompted on the terminal."
    The `git` manual ([git][git-doc-terminal-prompt]) says of `GIT_TERMINAL_PROMPT`:
    "If this Boolean environment variable is set to false,
    git will not prompt on the terminal
    (e.g.,
    when asking for HTTP authentication)."
    git-lfs follows the same order.
2.  Can upstream fix it?
    They could change the order,
    but that would break every user who relies on a desktop prompt with terminal prompts off.
3.  Are they supporting this use case?
    Yes:
    a desktop prompt program is a supported way to log in.
4.  Would the repo welcome a contribution?
    Not checked,
    since the first constraint fails.
5.  Will they likely fix it?
    Not checked,
    for the same reason.
6.  Prototype?
    None:
    there is no defect to fix upstream;
    the fix belongs in how we start Git.

[git-prompt]: https://github.com/git/git/blob/a018953688f1b10bddf91bff8747068f5f4746a4/prompt.c#L46-L77
[lfs-creds-order]: https://github.com/git-lfs/git-lfs/blob/a662a6dbfedf706cfa9fc7a5b7bcc7ba2e2c8a65/creds/creds.go#L104-L121
[lfs-creds-prompt]: https://github.com/git-lfs/git-lfs/blob/a662a6dbfedf706cfa9fc7a5b7bcc7ba2e2c8a65/creds/creds.go#L280-L282
[lfs-auth]: https://github.com/git-lfs/git-lfs/blob/a662a6dbfedf706cfa9fc7a5b7bcc7ba2e2c8a65/lfsapi/auth.go#L18-L32
[git-doc-credentials]: https://git-scm.com/docs/gitcredentials#:~:text=If%20the%20GIT_ASKPASS%20environment%20variable%20is%20set
[git-doc-terminal-prompt]: https://git-scm.com/docs/git#:~:text=git%20will%20not%20prompt%20on%20the%20terminal
