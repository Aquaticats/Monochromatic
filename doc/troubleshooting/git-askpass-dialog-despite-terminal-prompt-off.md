# Git 2.56.0 and git-lfs 3.8.1 open the desktop password dialog even with `GIT_TERMINAL_PROMPT=0`

## Symptom

A KDE dialog titled for SSH credentials appears on the desktop with a prompt such as:

```text
Username for "http://127.0.0.1:18765"
```

It was seen on 2026-11-07 while an agent ran an LFS end-to-end test against a fake LFS server on `127.0.0.1:18765`.
The dialog was `/usr/bin/ksshaskpass` (PID 3682988).
Its parent had already exited, so `systemd --user` had adopted it,
and it sat open with nobody left to read the answer.
Its control group was `app.slice/claude-code-bash` and its working folder was the test's throwaway repository,
so an agent's shell command started it, not a desktop program.

The quoting tells which program asked:

- git-lfs's own prompt quotes the URL with double quotes: `Username for "http://…"`.
- Git's prompt quotes it with single quotes and a colon: `Username for 'http://…': `.

The command had `GIT_TERMINAL_PROMPT=0` set, which many scripts treat as "never ask".
It is not.

One `git lfs push` against a server that keeps answering 401 asked for a username and password six times each
(see "Verification"),
so one test run can open several dialogs in a row.

## Root cause

This desktop session exports `SSH_ASKPASS=/usr/bin/ksshaskpass`,
and the systemd user manager carries it too
(`systemctl --user show-environment` lists `SSH_ASKPASS=/usr/bin/ksshaskpass`, `DISPLAY=:0` and `WAYLAND_DISPLAY=wayland-1`).
Every shell an agent starts, and every user service, inherits it.

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

The order is `GIT_ASKPASS`, then `core.askPass`, then `SSH_ASKPASS`.
`GIT_TERMINAL_PROMPT` is read only when none of them produced an answer,
and it only decides whether to fall back to the terminal.
Two details matter for the fix:

- `if (!askpass)` tests for an unset variable, so a `GIT_ASKPASS` that is set to the empty string stops the search:
  `core.askPass` and `SSH_ASKPASS` are never read.
- `if (askpass && *askpass)` then skips the empty value, so no program runs and Git reaches the
  "terminal prompts disabled" error.

### git-lfs reads the same three settings itself

git-lfs does not only go through `git credential fill`; it has its own prompt helper.
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

`ok` is true for a variable that is set but empty, so an empty `GIT_ASKPASS` again stops the search,
and `len(askpass) > 0` then leaves the prompt helper out.

The double-quoted prompt in the dialog comes from [`creds/creds.go:280-282`][lfs-creds-prompt]:

```go
// creds/creds.go (git-lfs v3.8.1)
	// 'cmd' will run the GIT_ASKPASS (or core.askpass) command prompting
	// for the desired valueType (`Username` or `Password`)
	cmd, errVal := subprocess.ExecCommand(a.Program, a.args(fmt.Sprintf("%s for %q", valueString, u))...)
```

`%q` is Go's double-quoted string, which is why the dialog read `Username for "http://127.0.0.1:18765"`.

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
The probe counted prompts, not requests per endpoint, so how the six username prompts split across requests was not measured.

### Why the dialog outlived its command

`ksshaskpass` is a separate process.
When the agent's command was stopped, the Git and git-lfs processes went away but the dialog did not:
the observed dialog's parent was `systemd --user` (PID 5697).
So a command time limit does not close the dialog it opened.

## Verification

### Versions

- Git 2.56.0 at `/usr/bin/git` (`git --version`), source tag `v2.56.0`, commit `a018953688f1b10bddf91bff8747068f5f4746a4`.
- git-lfs 3.8.1 from the repository toolchain (`git-lfs version` prints `git-lfs/3.8.1 (GitHub; linux amd64; go 1.27.3; git a662a6db)`),
  source tag `v3.8.1`, commit `a662a6dbfedf706cfa9fc7a5b7bcc7ba2e2c8a65`.

### Harness

The probe starts an HTTP server on a free local port that answers every request with `401` and
`WWW-Authenticate: Basic realm="probe"`.
A recorder program stands in for `ksshaskpass`:
it appends its prompt to a log and answers `probe-user`, so no dialog opens.
Each run uses an empty global configuration (`GIT_CONFIG_GLOBAL=/dev/null`, `GIT_CONFIG_NOSYSTEM=1`),
a throwaway `HOME`, `PATH` limited to `/usr/bin:/bin` plus a link to `git-lfs`,
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

It ran on 2026-11-07 in `systemd-run --user --scope` with `CPUQuota=200%`, `MemoryMax=1G`, `TasksMax=200` and `IOWeight=10`.

### Runs that open a prompt program

`git ls-remote`, control arm: exit 128, the recorder was called twice.

```text
Username for 'http://127.0.0.1:44799': 
Password for 'http://probe-user@127.0.0.1:44799':
fatal: Authentication failed for 'http://127.0.0.1:44799/probe.git/'
```

`git lfs push origin HEAD`, control arm: exit 2, the recorder was called twelve times,
six username prompts each followed by a password prompt,
all in git-lfs's double-quoted form, ending with:

```text
batch response: too many authentication attempts
```

### Runs that open no prompt program

`git ls-remote`, fix arm (`GIT_ASKPASS` set but empty) and alternative arm (both unset): exit 128, the recorder was never called.

```text
fatal: could not read Username for 'http://127.0.0.1:44799': terminal prompts disabled
```

`git lfs push origin HEAD`, fix arm and alternative arm: exit 2, the recorder was never called.

```text
fatal: could not read Username for 'http://127.0.0.1:44799': terminal prompts disabled
batch response: Git credentials for http://127.0.0.1:44799 not found.
```

### Not exercised

- A `core.askPass` entry in the configuration: no arm set one.
- A credential helper that does answer (for example `credential.helper store`):
  no arm configured one, so the claim that helpers still work with an empty `GIT_ASKPASS`
  rests on reading `prompt.c` (the prompt runs only when no helper answered), not on a run.
- SSH remotes: OpenSSH reads `SSH_ASKPASS` itself for key passphrases and host prompts,
  and `GIT_ASKPASS` does not change that; no SSH arm ran.

## Verified workarounds

### Set `GIT_ASKPASS` to the empty string and `GIT_TERMINAL_PROMPT=0`

```sh
# Any agent-run Git or git-lfs command that may reach a server needing a login.
GIT_ASKPASS= GIT_TERMINAL_PROMPT=0 git lfs push origin HEAD
```

In Rust tests: `.env("GIT_ASKPASS", "").env("GIT_TERMINAL_PROMPT", "0")`.

Tradeoffs:

- It also skips a `core.askPass` from the configuration (Git and git-lfs both stop at a set `GIT_ASKPASS`),
  so a deliberately configured prompt program is ignored too.
  The probe did not set `core.askPass`; this follows from the source quoted in "Root cause".
- A command that needs a login and has no credential helper answer now fails at once instead of waiting for a person.
  That is the intent for unattended runs, and wrong for a person who wants to type a password.
- OpenSSH prompts are unaffected, as noted in "Not exercised".

### Remove both `SSH_ASKPASS` and `GIT_ASKPASS` from the environment

```sh
env --unset=SSH_ASKPASS --unset=GIT_ASKPASS GIT_TERMINAL_PROMPT=0 git lfs push origin HEAD
```

Tradeoffs: a `core.askPass` from the configuration still opens its program,
because Git and git-lfs read the configuration when `GIT_ASKPASS` is unset.
It also stops OpenSSH from using the desktop dialog for key passphrases.
The empty `GIT_ASKPASS` workaround is the stronger one for Git and git-lfs.

### Existing repo practice: `GIT_ASKPASS=/bin/true`

`package/git-policy/cli/src/native/landing/service_push_retry_tests.rs` sets
`("GIT_TERMINAL_PROMPT", "0"), ("GIT_ASKPASS", "/bin/true")` so its tests never reach a person.
That also prevents the dialog, but the prompt program answers with an empty name,
so Git sends a request with empty credentials before failing with an authentication error
rather than "terminal prompts disabled",
and git-lfs may retry with those empty answers.
The probe did not run this form.

## What does not work

- `GIT_TERMINAL_PROMPT=0` alone: the control arms ran with it and still called the prompt program
  twice for Git and twelve times for git-lfs.
- A time limit on the command: it stops Git and git-lfs but leaves the dialog open,
  as the orphaned `ksshaskpass` with parent `systemd --user` showed.
- Running without a terminal: Git does not need a terminal to run a prompt program,
  and the dialog needs only the inherited `DISPLAY` or `WAYLAND_DISPLAY`.

## Open question for this repository

The wrapper's production code sets neither `GIT_ASKPASS` nor `GIT_TERMINAL_PROMPT`
(`rg 'GIT_ASKPASS|GIT_TERMINAL_PROMPT' package/git-policy/cli/src` finds them only in tests and in the list of
names a plain read tolerates, `repo_read_layout_environment_rules.rs`).
Its background service runs under the systemd user manager, which carries `SSH_ASKPASS` and a display,
so a background push whose credential helper gives no answer could open this dialog.
That is a reading of the code, not a run; whether background Git may ever open a desktop dialog is the owner's decision.

## Upstream filing decision

Checked on 2026-11-07.
`.out-of-scope/` has no entry for Git or git-lfs.
Nothing is drafted, because the first constraint fails.

1. Is it upstream's fault?
   No.
   The order is Git's documented behavior: the `gitcredentials` manual lists `GIT_ASKPASS`, `core.askPass` and `SSH_ASKPASS`
   before the terminal ([gitcredentials][git-doc-credentials]),
   and `GIT_TERMINAL_PROMPT` is documented as controlling terminal prompts only ([git][git-doc-terminal-prompt]).
   git-lfs follows the same order.
2. Can upstream fix it?
   They could change the order, but that would break every user who relies on a desktop prompt with terminal prompts off.
3. Are they supporting this use case?
   Yes: a desktop prompt program is a supported way to log in.
4. Would the repo welcome a contribution?
   Not checked, since the first constraint fails.
5. Will they likely fix it?
   Not checked, for the same reason.
6. Prototype?
   None: there is no defect to fix upstream; the fix belongs in how we start Git.

[git-prompt]: https://github.com/git/git/blob/a018953688f1b10bddf91bff8747068f5f4746a4/prompt.c#L46-L77
[lfs-creds-order]: https://github.com/git-lfs/git-lfs/blob/a662a6dbfedf706cfa9fc7a5b7bcc7ba2e2c8a65/creds/creds.go#L104-L121
[lfs-creds-prompt]: https://github.com/git-lfs/git-lfs/blob/a662a6dbfedf706cfa9fc7a5b7bcc7ba2e2c8a65/creds/creds.go#L280-L282
[lfs-auth]: https://github.com/git-lfs/git-lfs/blob/a662a6dbfedf706cfa9fc7a5b7bcc7ba2e2c8a65/lfsapi/auth.go#L18-L32
[git-doc-credentials]: https://git-scm.com/docs/gitcredentials
[git-doc-terminal-prompt]: https://git-scm.com/docs/git
