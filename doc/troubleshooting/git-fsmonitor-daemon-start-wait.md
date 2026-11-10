# Git 2.56.0 on Linux: the first `core.fsmonitor = true` command after the daemon is gone blocks until the daemon has put an inotify watch on every directory

## Symptom

No error is printed.
A Git command that normally takes milliseconds stalls,
then succeeds.
Any Git command that reads the index can be the one that stalls (`status`,
`commit`,
`diff`,
`diff-tree`,
which reads the index before it walks anything,
editor and prompt integrations),
because the stall happens in the client's fsmonitor query when no daemon is listening.

Observed on this host on 2026-11-03 (Git 2.56.0,
Linux,
built-in fsmonitor daemon enabled by `core.fsmonitor = true`
in `~/.gitconfig` until 2026-11-03,
when it was set to `false`):

- A commit in the main checkout `/var/home/user/Monochromatic` took 7.7 s.
  From 0.5 s to 7.0 s the Git process sat in `wait4` (syscall 61) on its child `git fsmonitor--daemon start`,
  which ran beside a `git fsmonitor--daemon run --detach`.
  A Git process of another program (JetBrains) showed the same wait.
- Another commit at 09:19 took 2.5 min,
  with three separate Git processes each stalled about 50 s,
  consistent with the 60 s default start timeout.
- The main checkout's daemon (pid 1245627) held 2,269,377 inotify watches.
  223 or more linked worktrees live inside the main worktree under `.claude/worktrees/` (Claude Code's default location),
  each with `node_modules` and build output.
  At about 464 s old that daemon had used 117.7 s of CPU and 734 MB of resident memory.
- 261 daemons were alive for this user,
  holding 4,591,335 watches in total (the limits here,
  `max_user_watches` and `max_queued_events`,
  are 10,000,000 each),
  and 208 of them were about 3 h old and started together.
- With the monitor off,
  `git --no-optional-locks status --porcelain=v1 -z --untracked-files=no` on main took 16 ms;
  with a listening daemon,
  9 ms (five alternating runs each).

These host measurements came from scratch tools that are not kept in the repository
(`observe-main-commit.ts`,
`fsmonitor-watch-census.ts`,
`fsmonitor-served.ts` and `status-without-fsmonitor.ts`
under `/var/home/user/temp/agent/tools/`),
so this document cannot re-run them;
the harness in "Verification" reproduces the mechanism on a disposable tree.

Why the previous main daemon went away between 09:15 and 09:18 is unknown.
Finding it needs `GIT_TRACE_FSMONITOR` on a running daemon,
which was not run.
The sections from "Root cause" on explain why a missing daemon is expensive,
not why it went missing.

## Root cause

### The client starts a daemon and waits for it

`fsmonitor_ipc__send_query` finds no listening daemon,
runs `git fsmonitor--daemon start` with `run_command`
(so the client blocks until `start` returns),
then retries with `wait_if_not_found = 1`
([`fsmonitor-ipc.c:58-71`][src-ipc-spawn],
[`fsmonitor-ipc.c:105-124`][src-ipc-retry]):

```c
// fsmonitor-ipc.c:58-71
static int spawn_daemon(void)
{
	struct child_process cmd = CHILD_PROCESS_INIT;

	cmd.git_cmd = 1;
	cmd.no_stdin = 1;
	cmd.no_stdout = 1;
	cmd.no_stderr = 1;
	cmd.close_fd_above_stderr = 1;
	cmd.trace2_child_class = "fsmonitor";
	strvec_pushl(&cmd.args, "fsmonitor--daemon", "start", NULL);

	return run_command(&cmd);
}
```

```c
// fsmonitor-ipc.c:105-124
	case IPC_STATE__NOT_LISTENING:
	case IPC_STATE__PATH_NOT_FOUND:
		if (tried_to_spawn)
			goto done;

		tried_to_spawn++;
		if (spawn_daemon())
			goto done;

		/*
		 * Try again, but this time give the daemon a chance to
		 * actually create the pipe/socket.
		 *
		 * Granted, the daemon just started so it can't possibly have
		 * any FS cached yet, so we'll always get a trivial answer.
		 * BUT the answer should include a new token that can serve
		 * as the basis for subsequent requests.
		 */
		options.wait_if_not_found = 1;
		goto try_again;
```

By the code's own comment,
the answer the client waits for is always trivial (it means "treat every path as possibly changed").

### `start` waits for a listening socket, up to 60 s, without sleeping

`start` runs `fsmonitor--daemon run --detach` and waits until the IPC socket accepts connections
([`builtin/fsmonitor--daemon.c:1548-1549`][src-start-bg]).
The wait length is `fsmonitor.startTimeout`,
60 s by default
([`builtin/fsmonitor--daemon.c:39-40`][src-timeout-config]).
The readiness callback only reports "not yet" while the socket is missing
([`builtin/fsmonitor--daemon.c:1488-1511`][src-bg-wait-cb]):

```c
// builtin/fsmonitor--daemon.c:39-40
#define FSMONITOR__START_TIMEOUT "fsmonitor.starttimeout"
static int fsmonitor__start_timeout_sec = 60;
```

```c
// builtin/fsmonitor--daemon.c:1488-1511
static start_bg_wait_cb bg_wait_cb;

static int bg_wait_cb(const struct child_process *cp UNUSED,
		      void *cb_data UNUSED)
{
	enum ipc_active_state s = fsmonitor_ipc__get_state();

	switch (s) {
	case IPC_STATE__LISTENING:
		/* child is "ready" */
		return 0;

	case IPC_STATE__NOT_LISTENING:
	case IPC_STATE__PATH_NOT_FOUND:
		/* give child more time */
		return 1;

	default:
	case IPC_STATE__INVALID_PATH:
	case IPC_STATE__OTHER_ERROR:
		/* all the time in world won't help */
		return -1;
	}
}
```

The loop that calls it has no sleep,
and `fsmonitor_ipc__get_state` returns at once when the socket path does not exist yet
([`run-command.c:2052-2081`][src-bg-loop],
[`compat/simple-ipc/ipc-unix-socket.c:26-37`][src-ipc-state]):

```c
// run-command.c:2052-2081
	time(&time_limit);
	time_limit += timeout_sec;

wait:
	pid_seen = waitpid(cmd->pid, &wait_status, WNOHANG);

	if (!pid_seen) {
		/*
		 * The child is currently running.  Ask the callback
		 * if the child is ready to do work or whether we
		 * should keep waiting for it to boot up.
		 */
		ret = (*wait_cb)(cmd, cb_data);
		if (!ret) {
			/*
			 * The child is running and "ready".
			 */
			trace2_child_ready(cmd, "ready");
			sbgr = SBGR_READY;
			goto done;
		} else if (ret > 0) {
			/*
			 * The callback said to give it more time to boot up
			 * (subject to our timeout limit).
			 */
			time_t now;

			time(&now);
			if (now < time_limit)
				goto wait;
```

```c
// compat/simple-ipc/ipc-unix-socket.c:26-37
	options.wait_if_busy = 0;
	options.wait_if_not_found = 0;

	if (lstat(path, &st) == -1) {
		switch (errno) {
		case ENOENT:
		case ENOTDIR:
			return IPC_STATE__NOT_LISTENING;
		default:
			return IPC_STATE__INVALID_PATH;
		}
	}
```

So the `start` child,
which the client waits for,
spins on `waitpid` plus `lstat` for the whole wait.
The measurements in "Verification" agree:
user plus system CPU of the whole command (the client and its waited-for `start`) is about equal to its wall time
(1.85 s of CPU in 1.88 s of wall on the 100,000-directory tree).
That is a measurement of the client command,
which the detached daemon's own CPU does not enter,
so the spin is an inference from the `start_bg_command` loop plus the CPU equality,
not a direct count of `lstat` calls.

### The socket does not exist until every directory has a watch

On Linux the daemon registers an inotify watch on every directory of the worktree,
recursively,
before it creates its IPC socket ([`compat/fsmonitor/fsm-listen-linux.c:286-307`][src-register]):

```c
// compat/fsmonitor/fsm-listen-linux.c:286-307
	while ((de = readdir_skip_dot_and_dotdot(dir)) != NULL) {
		strbuf_reset(&current);
		strbuf_addf(&current, "%s/%s", path, de->d_name);
		if (lstat(current.buf, &fs)) {
			if (errno == ENOENT)
				continue; /* file was deleted */
			error_errno(_("lstat('%s') failed"), current.buf);
			goto failed;
		}

		/* recurse into directory */
		if (S_ISDIR(fs.st_mode)) {
			if (add_watch(current.buf, state->listen_data))
				goto failed;
			if (register_inotify(current.buf, state, batch))
				goto failed;
		} else if (batch) {
			rel = current.buf + state->path_worktree_watch.len + 1;
			trace_printf_key(&trace_fsmonitor, "explicitly adding '%s'", rel);
			fsmonitor_batch__add_path(batch, rel);
		}
	}
```

The registration runs inside `fsm_listen__ctor`
([`compat/fsmonitor/fsm-listen-linux.c:426-440`][src-ctor]):

```c
// compat/fsmonitor/fsm-listen-linux.c:426-440
	if (add_watch(state->path_worktree_watch.buf, data))
		ret = -1;
	else if (register_inotify(state->path_worktree_watch.buf, state, NULL))
		ret = -1;
	else if (state->nr_paths_watching > 1) {
		if (add_watch(state->path_gitdir_watch.buf, data))
			ret = -1;
		else if (register_inotify(state->path_gitdir_watch.buf, state, NULL))
			ret = -1;
	}

	if (!ret) {
		state->listen_error_code = 0;
		data->shutdown = SHUTDOWN_CONTINUE;
	}
```

`fsmonitor_run_daemon` calls the constructor first,
and only afterwards `fsmonitor_run_daemon_1`,
which is where `ipc_server_init_async` creates the socket
([`builtin/fsmonitor--daemon.c:1387-1394`][src-run-order],
[`builtin/fsmonitor--daemon.c:1224-1233`][src-ipc-init]):

```c
// builtin/fsmonitor--daemon.c:1387-1394
	/*
	 * Confirm that we can create platform-specific resources for the
	 * filesystem listener before we bother starting all the threads.
	 */
	if (fsm_listen__ctor(&state)) {
		err = error(_("could not initialize listener thread"));
		goto done;
	}
```

```c
// builtin/fsmonitor--daemon.c:1224-1233
	/*
	 * Start the IPC thread pool before the we've started the file
	 * system event listener thread so that we have the IPC handle
	 * before we need it.
	 */
	if (ipc_server_init_async(&state->ipc_server_data,
				  state->path_ipc.buf, &ipc_opts,
				  handle_client, state))
		return error_errno(
			_("could not start IPC thread pool on '%s'"),
```

Even after the socket exists the server does not serve until the listener loop starts
([`compat/fsmonitor/fsm-listen-linux.c:698-702`][src-loop]):

```c
// compat/fsmonitor/fsm-listen-linux.c:698-702
	/*
	 * Our fs event listener is now running, so it's safe to start
	 * serving client requests.
	 */
	ipc_server_start_async(state->ipc_server_data);
```

The cost is one watch per directory,
with no exclusion of nested worktrees,
`node_modules` or build output,
and the registration time grows with the number of directories (measured in "Verification").

### Why an early trivial answer would be correct

When a query fails,
the client already falls back to "invalidate everything" and stores a placeholder token
([`fsmonitor.c:558-569`][src-client-fail],
[`fsmonitor.c:691-707`][src-client-apply]):

```c
// fsmonitor.c:558-569
			} else {
				/*
				 * The builtin daemon is not available on this
				 * platform -OR- we failed to get a response.
				 *
				 * Generate a fake token (rather than a V1
				 * timestamp) for the index extension.  (If
				 * they switch back to the hook API, we don't
				 * want ambiguous state.)
				 */
				strbuf_addstr(&last_update_token, "builtin:fake");
			}
```

```c
// fsmonitor.c:691-707
	} else {
		/*
		 * We failed to get a response or received a trivial response,
		 * so invalidate everything.
		 *
		 * We only want to run the post index changed hook if
		 * we've actually changed entries, so keep track if we
		 * actually changed entries or not.
		 */
		int is_cache_changed = 0;

		for (i = 0; i < istate->cache_nr; i++) {
			if (istate->cache[i]->ce_flags & CE_FSMONITOR_VALID) {
				is_cache_changed = 1;
				istate->cache[i]->ce_flags &= ~CE_FSMONITOR_VALID;
			}
		}
```

And the daemon already answers a token it cannot parse with a trivial response that carries a real token
([`builtin/fsmonitor--daemon.c:652-675`][src-token-parse],
[`builtin/fsmonitor--daemon.c:745-754`][src-token-invalid]):

```c
// builtin/fsmonitor--daemon.c:745-754
	} else {
		/* We have "builtin:*" */
		if (fsmonitor_parse_client_token(command, &requested_token_id,
						 &requested_oldest_seq_nr)) {
			trace_printf_key(&trace_fsmonitor,
					 "fsmonitor: invalid V2 protocol token '%s'",
					 command);
			do_trivial = 1;
			do_cookie = 1;

		} else {
```

So a daemon that is still registering watches can answer at once with a trivial response and an unparseable token.
The client scans everything this time,
and its next query gets the real token.
A real token must not be handed out before registration ends,
because a change in a directory with no watch yet would never be recorded and a later query would miss it.

### Concurrent clients each start their own daemon

Nothing stops a second `fsmonitor--daemon run` from registering all watches while the first is still registering.
The only mutual exclusion is a lock file and a liveness probe taken when the IPC socket is created,
which is after registration ([`unix-stream-server.c:53-61`][src-lock]):

```c
// unix-stream-server.c:53-61
	/*
	 * If another server is listening on "<path>" give up.  We do not
	 * want to create a socket and steal future connections from them.
	 */
	if (is_another_server_alive(path, opts)) {
		rollback_lock_file(&lock);
		errno = EADDRINUSE;
		return -2;
	}
```

Every client that arrives while no daemon listens therefore starts a daemon that registers every watch,
and all but one fail at the end with `EADDRINUSE`.
Measured:
three clients produced three daemons at once,
and a lowered `fsmonitor.startTimeout` made ten clients produce eight
(see "What does not work").

### Nested linked worktrees are watched twice

A linked worktree inside the main worktree is an ordinary subdirectory to the main daemon,
so the main daemon watches every directory of it,
and a status run inside that worktree starts a second daemon that watches them again.
The daemon documentation says it watches "a single working directory",
and its Linux caveat names only the watch limit
([`Documentation/git-fsmonitor--daemon.adoc:49-51`][doc-single],
[`Documentation/git-fsmonitor--daemon.adoc:97-107`][doc-linux]).
`fsmonitor.startTimeout` and `fsmonitor.ipcThreads` have no entry in the configuration reference,
which lists only `fsmonitor.allowRemote` and
`fsmonitor.socketDir` ([`Documentation/config/fsmonitor--daemon.adoc:1-11`][doc-config]).
The `start` subcommand's option `--start-timeout` is defined at
[`builtin/fsmonitor--daemon.c:1582-1585`][src-start-timeout-option].

### Readings that were wrong

- Lowering `fsmonitor.startTimeout` looked like a way to shorten the wait.
  It does shorten each wait,
  but the daemon that the client abandoned keeps registering,
  and the next client starts another.
  Eight daemons at once were measured with ten clients 40 ms apart (see "What does not work").
- The first patched measurement of time-to-fully-registered read 6 s on a 100,000-directory tree,
  against 2.2 s for the unpatched build.
  That was the harness:
  it polled `/proc/<pid>/fdinfo` every 50 ms,
  and reading 100,000 watch lines contends with the daemon's own `inotify_add_watch` calls.
  With a one-second poll the patched build reads 2.6 s,
  the same order as the unpatched build,
  within the polling granularity.

## Verification

Version under test:
Git 2.56.0.
The host's `/usr/bin/git` reports `git version 2.56.0`.
The comparison builds are from the upstream tag `v2.56.0`,
commit `a018953688f1b10bddf91bff8747068f5f4746a4`,
cloned with `gh repo clone git/git <dir> -- --depth 1 --branch v2.56.0` into a fresh `mktemp --directory` directory
(origin `https://github.com/git/git.git`),
built with `make NO_GETTEXT=1 NO_TCLTK=1 NO_PERL=1 NO_CURL=1 NO_EXPAT=1 NO_PYTHON=1 git`.
The unpatched binary has sha256 `b6ca325b8339ed2a67b69077ed0c7f9993f1cf385a0e29492d188351cfc28bb3`.
The patched binary is the same tree plus [`git-fsmonitor-daemon-start-wait.patch`](git-fsmonitor-daemon-start-wait.patch).
The harness never starts a daemon in a real repository:
it makes a repository under `${HOME}/temp/agent/fsmonitor-doc`,
passes `-c core.fsmonitor=true` per command with `GIT_CONFIG_GLOBAL=/dev/null`,
finds its daemons by an environment marker (never by process name),
and stops every daemon it started before it deletes the tree.

### Harness

Save as `fsmonitor-start-wait.ts` and run it from its directory inside a capped scope:

```sh
# Bounds: four CPU threads, 4 GiB of memory, 512 tasks, disk weight 10.
FSM_GIT=/path/to/git systemd-run --user --scope --quiet --setenv=FSM_GIT \
  --property=CPUQuota=400% --property=MemoryMax=4G --property=TasksMax=512 --property=IOWeight=10 \
  node fsmonitor-start-wait.ts 100000 nested
```

The modes are the bare directory count (cold start and concurrent clients),
`nested`,
`correctness` and `starttimeout`.
A tree of 100,000 directories held about 100,000 watches,
a small share of the 10,000,000 limit on this host.

```ts
// fsmonitor-start-wait.ts
// Harness: how long a Git client blocks while a fresh fsmonitor daemon registers inotify watches.
// Usage: FSM_GIT=<git binary> node fsmonitor-start-wait.ts <directory-count> [nested | correctness | starttimeout]
// Builds a disposable repository, never touches a real one, and stops every daemon it started.
import { spawn, spawnSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readdirSync, readFileSync, readlinkSync, realpathSync, rmSync, writeFileSync } from 'node:fs';
import { homedir } from 'node:os';
import { dirname, join } from 'node:path';

// FSM_GIT names the Git binary under test; its directory is also its exec path so the daemon child is the same build.
const git = process.env.FSM_GIT ?? '/usr/bin/git';
const directoryCount = Number(process.argv[2] ?? 20000);
const mode = process.argv[3];
const topLevelDirectories = 100;
const concurrentClients = 3;
const nestedWorktrees = 3;
const pollSeconds = 1;
const startTimeoutAttempts = 10;
const startTimeoutGapSeconds = 0.04;
const patienceMs = 120000;

const scratch = join(homedir(), 'temp/agent/fsmonitor-doc');
mkdirSync(scratch, { recursive: true });
const root = realpathSync(mkdtempSync(join(scratch, 'tree.')));
// Every Git process the harness starts carries this marker, so daemons are found by environment, not by name.
const marker = `FSM_HARNESS=${root}`;
const env = {
  ...process.env,
  FSM_HARNESS: root,
  GIT_CONFIG_GLOBAL: '/dev/null',
  GIT_CONFIG_NOSYSTEM: '1',
  ...(process.env.FSM_GIT === undefined ? {} : { GIT_EXEC_PATH: dirname(git) }),
};
const on = ['-c', 'core.fsmonitor=true'];
const off = ['-c', 'core.fsmonitor=false'];
const quick = ['--no-optional-locks', 'status', '--porcelain=v1', '--untracked-files=no'];
const full = ['--no-optional-locks', 'status', '--porcelain=v1'];
const show = (label: string, value: unknown) => console.log(`${label}: ${JSON.stringify(value)}`);
const sleep = (seconds: number) => spawnSync('sleep', [String(seconds)]);

function run(cwd: string, args: readonly string[]) {
  const result = spawnSync(git, ['-C', cwd, ...args], { env, encoding: 'utf8' });
  if (result.status !== 0) throw new Error(`git ${args.join(' ')} failed: ${result.stderr}`);
  return result.stdout;
}

// Wall seconds (hrtime), user and system CPU seconds of one command, waited-for children included.
// The detached daemon is never waited for, so its CPU is not in user and system.
function timed(cwd: string, args: readonly string[], trace?: string) {
  const out = join(root, '.time');
  const childEnv = trace === undefined ? env : { ...env, GIT_TRACE2_PERF: trace };
  const started = process.hrtime.bigint();
  spawnSync('/usr/bin/time', ['--format=%U %S', '--output', out, git, '-C', cwd, ...args], { env: childEnv });
  const wall = Number(process.hrtime.bigint() - started) / 1e9;
  const [user, system] = readFileSync(out, 'utf8').trim().split(' ').map(Number);
  return { wall: Number(wall.toFixed(4)), user, system };
}

function makeDirectories(base: string, count: number) {
  for (let index = 0; index < count; index += 1) {
    mkdirSync(join(base, `d${index % topLevelDirectories}`, `e${index}`), { recursive: true });
  }
}

// Write one file into every `step`th of the first top-level directories.
function spread(name: string, text: string, step: number) {
  for (let index = 0; index < topLevelDirectories; index += step) {
    writeFileSync(join(root, `d${index}`, `e${index}`, name), text);
  }
}

// Daemons of this harness with the inotify watch count each one holds (zero when `withWatches` is false).
function census(withWatches = true) {
  const found: { pid: string; watches: number }[] = [];
  for (const pid of readdirSync('/proc').filter((name) => /^\d+$/.test(name))) {
    try {
      if (!readFileSync(`/proc/${pid}/cmdline`, 'utf8').includes('fsmonitor--daemon\0run')) continue;
      if (!readFileSync(`/proc/${pid}/environ`, 'utf8').split('\0').includes(marker)) continue;
    } catch {
      continue;
    }
    let watches = 0;
    for (const fd of withWatches ? readdirSync(`/proc/${pid}/fd`) : []) {
      try {
        if (readlinkSync(`/proc/${pid}/fd/${fd}`) === 'anon_inode:inotify') {
          const info = readFileSync(`/proc/${pid}/fdinfo/${fd}`, 'utf8');
          watches += info.split('\n').filter((line) => line.startsWith('inotify wd:')).length;
        }
      } catch {
        // The daemon closed this descriptor while it was read.
      }
    }
    found.push({ pid, watches });
  }
  return found;
}

// Seconds until `daemons` daemons each hold at least `minimum` watches, or -1 at the deadline.
// Polling reads /proc fdinfo, which contends with the daemon, so it is slow and coarse on purpose.
function secondsUntilWatching(minimum: number, since: number, daemons = 1) {
  const deadline = Date.now() + patienceMs;
  while (Date.now() < deadline) {
    if (census().filter((daemon) => daemon.watches >= minimum).length >= daemons) return (Date.now() - since) / 1000;
    sleep(pollSeconds);
  }
  return -1;
}

function stopAll(cwds: readonly string[]) {
  for (const cwd of cwds) spawnSync(git, ['-C', cwd, 'fsmonitor--daemon', 'stop'], { env, timeout: patienceMs });
  const deadline = Date.now() + patienceMs;
  while (census().length > 0 && Date.now() < deadline) sleep(pollSeconds);
  // A daemon that lost the socket race holds no socket, so `stop` cannot reach it: end it by pid.
  for (const { pid } of census()) process.kill(Number(pid), 'SIGTERM');
  const killDeadline = Date.now() + patienceMs;
  while (census().length > 0 && Date.now() < killDeadline) sleep(pollSeconds);
  if (census().length > 0) throw new Error(`daemons left running: ${JSON.stringify(census())}`);
}

function measureColdStart() {
  show('no monitor, status', [1, 2, 3].map(() => timed(root, [...off, ...quick])));
  const trace = join(root, '.trace2-cold');
  const coldStarted = Date.now();
  show('cold start, status', timed(root, [...on, ...quick], trace));
  show('cold start trace2 lines', readFileSync(trace, 'utf8').split('\n')
    .filter((line) => /child_start|child_ready|child_exit|region_(enter|leave).*fsm_client/.test(line)));
  show('status right after the cold start', timed(root, [...on, ...quick]));
  show('seconds from the cold start until the daemon holds every watch', secondsUntilWatching(directoryCount, coldStarted));
  show('daemons once registered', census());
  show('first status after registration (real token), then warm', [1, 2, 3].map(() => timed(root, [...on, ...quick])));
  stopAll([root]);
}

// Clients arrive while no daemon listens: how many daemons register watches?
async function measureConcurrentClients() {
  const clients = Array.from({ length: concurrentClients }, () => new Promise<number>((resolve) => {
    const started = Date.now();
    spawn(git, ['-C', root, ...on, ...quick], { env, stdio: 'ignore' })
      .on('exit', () => resolve((Date.now() - started) / 1000));
  }));
  show('concurrent cold clients, wall seconds', await Promise.all(clients));
  show('daemons right after the clients returned', census());
  sleep(pollSeconds);
  show('daemons one second later', census());
  stopAll([root]);
}

// The shorter-wait setting: clients stop waiting, but each one that finds no listener starts another daemon.
function measureStartTimeoutZero() {
  const args = [...on, '-c', 'fsmonitor.startTimeout=0', ...quick];
  show('cold start with fsmonitor.startTimeout=0', timed(root, args));
  let peak = 0;
  for (let attempt = 0; attempt < startTimeoutAttempts; attempt += 1) {
    run(root, args);
    peak = Math.max(peak, census(false).length);
    sleep(startTimeoutGapSeconds);
  }
  show('most daemons alive at once while the clients kept arriving', peak);
  stopAll([root]);
}

// Linked worktrees inside the main worktree, as Claude Code creates them under .claude/worktrees/.
function measureNestedWorktrees() {
  const worktrees = Array.from({ length: nestedWorktrees }, (_, n) => join(root, '.claude/worktrees', `wt${n}`));
  for (const path of worktrees) {
    run(root, ['worktree', 'add', '--quiet', '--detach', path]);
    makeDirectories(path, directoryCount);
  }
  const mainStarted = Date.now();
  run(root, [...on, ...quick]);
  secondsUntilWatching(directoryCount * (nestedWorktrees + 1), mainStarted);
  show('main daemon watches with nested worktrees', census());
  const nestedStarted = Date.now();
  run(worktrees[0], [...on, ...quick]);
  secondsUntilWatching(directoryCount, nestedStarted, 2);
  show('after one status inside a nested worktree (a second daemon, its directories counted twice)', census());
  stopAll([root, worktrees[0]]);
}

// Changes made while the daemon still registers watches must show up in every later status.
function checkCorrectness() {
  run(root, [...on, ...full]);
  const started = Date.now();
  spread('tracked.txt', 'y\n', 2);
  spread('untracked.txt', 'z\n', 2);
  show('watches held while the changes were made', census().map((daemon) => daemon.watches));
  // Each monitor status is compared with a no-monitor status taken at the same point in the file changes.
  const compare = () => run(root, [...on, ...full]) === run(root, [...off, ...full]);
  const during = compare();
  secondsUntilWatching(directoryCount, started);
  const afterOne = compare();
  const afterTwo = compare();
  spread('tracked.txt', 'w\n', 4);
  spread('later.txt', 'z\n', 4);
  const incremental = compare();
  show('lines in the no-monitor status', run(root, [...off, ...full]).split('\n').length - 1);
  show('status equals the no-monitor status: during, after one, after two, incremental', [during, afterOne, afterTwo, incremental]);
}

try {
  run(root, ['init', '--quiet']);
  makeDirectories(root, directoryCount);
  show('directories', directoryCount);
  spread('tracked.txt', 'x\n', 1);
  run(root, ['add', '.']);
  run(root, ['-c', 'user.name=h', '-c', 'user.email=h@example.invalid', 'commit', '--quiet', '--message', 'init']);
  if (mode === 'correctness') {
    checkCorrectness();
  } else if (mode === 'starttimeout') {
    measureStartTimeoutZero();
  } else {
    measureColdStart();
    await measureConcurrentClients();
    if (mode === 'nested') measureNestedWorktrees();
  }
} finally {
  try {
    stopAll([root]);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}
```

### Behaves as expected

All of these are on the unpatched Git 2.56.0 build,
recorded 2026-11-03T10:19:01-05:00 on the host (other sessions keep the load up here).

- A status with the monitor off,
  or with a listening daemon,
  takes a few milliseconds on a 100,000-directory tree
  with a hundred tracked files (under 20 ms in every run).
  The tree has almost nothing to scan,
  so this says nothing about the monitor's benefit;
  it shows that the stall is not in steady state.
- Once the daemon holds every watch,
  the first status carries a real token and later statuses are warm.
  An occasional first status took longer in both arms (unpatched 0.14 s,
  0.15 s and 0.03 s of five runs;
  patched 0.075 s of five runs,
  and 0.72 s once in a separate nested-mode run);
  the cause was not found.
- The correctness check passes:
  during registration,
  after it,
  and incrementally,
  the monitor status equals the no-monitor status
  (see the patched results).

### Fails: one cold client blocks for the registration

Unpatched build,
three runs per size,
wall seconds of the first `git status` with no daemon running
(user plus system CPU of that command in parentheses):

- 20,000 directories:
  0.3742 (0.10 + 0.26),
  0.3826 (0.08 + 0.29),
  0.3664 (0.09 + 0.26).
- 100,000 directories:
  1.8763 (0.44 + 1.41),
  1.9937 (0.47 + 1.50),
  1.881 (0.44 + 1.41).

Run-to-run spread on unchanged input is about 6 percent,
so only differences far larger than that mean anything.
A first pass of the 100,000-directory run,
made while other sessions were loading the host,
took 5.671 s (0.77 + 2.58),
so the wait also grows with host load.
Trace2 (`GIT_TRACE2_PERF`) on the cold run shows the client's own `git fsmonitor--daemon start` child started inside the `fsm_client` query region
and `child_ready` arriving at the end of it,
with the wall time of the region equal to the wall time of the command.

The rate is about 18 to 19 microseconds per directory on this host (20,000 to 100,000 directories scale linearly,
0.37 s to 1.88 s).
Extrapolating that rate to the main checkout's 2,269,377 watches gives about 43 s,
with large uncertainty
(real trees have files and uneven depth).
The stalls seen on the host were about 6.5 s and about 50 s.

### Fails: a tree-only step pays the wait too, and that is where the 60 s steps came from

Recorded 2026-11-10T14:40-05:00.

One commit on the main checkout on 2026-11-03 held two Git children of about 60 s each,
`diff-tree` and `reset -q --pathspec-from-file`.
`doc/handover/cli-git-rust-implementation.md` kept that open,
because only a command that reads the index was expected to ask the daemon,
and `diff-tree` was assumed not to.
Measured:
`git diff-tree` reads the index before it walks anything,
whatever its arguments,
so it asks the daemon and waits like every other index reader.

The chain in Git 2.56.0:

- `cmd_diff_tree` reads the index:
  `if (repo_read_index(the_repository) < 0) die(_("index file corrupt"));`
  ([`builtin/diff-tree.c:133`][src-diff-tree-index]);
  a `--stdin` walk with rename detection reads it again
  ([`builtin/diff-tree.c:212-213`][src-diff-tree-stdin-index]).
- The read ends in `post_read_index_from` (`read-cache.c:1962-1968`),
  which calls `tweak_fsmonitor` (`read-cache.c:1967`).
- When the index carries the monitor's bitmap,
  `tweak_fsmonitor` marks the saved entries dirty and calls `refresh_fsmonitor`
  (`fsmonitor.c:792-812`,
  the call at `fsmonitor.c:811`).
- `refresh_fsmonitor` sends the query (`fsmonitor.c:506` and `fsmonitor.c:537`,
  the `fsm_client`/`query` region at `fsmonitor-ipc.c:88`),
  which spawns the daemon and waits for it
  ([`fsmonitor-ipc.c:58-71`][src-ipc-spawn] and [`fsmonitor-ipc.c:105-124`][src-ipc-retry]),
  up to `fsmonitor.startTimeout`,
  60 s by default
  ([`builtin/fsmonitor--daemon.c:39-40`][src-timeout-config]).

The trace2 stream of one cold `git diff-tree` shows that order and nothing else of consequence:
`index`/`do_read_index` (`read-cache.c:2374`) at 19:40:46.826085Z,
`fsm_client`/`query` (`fsmonitor-ipc.c:88`) at 19:40:46.826525Z,
the daemon's `ipc-server`/`create-listener_socket`
(`compat/simple-ipc/ipc-unix-socket.c:814`) at 19:40:49.518686Z,
and `fsmonitor`/`apply_results` (`fsmonitor.c:658`) at 19:40:49.523991Z.

The same trace2 stream also answers why the bound held the whole time:
the daemon's socket is created only after its registration ends
(`compat/fsmonitor/fsm-listen-linux.c:426-440`,
then `builtin/fsmonitor--daemon.c:1387-1394`
and `builtin/fsmonitor--daemon.c:1224-1233`).

Measured with the harness at the end of this section,
Git 2.56.0,
a disposable repository of 60,000 directories,
`core.fsmonitor` given per command,
no daemon listening before each arm,
two runs of the same steps:

- `git diff-tree -r --name-only -z --no-renames HEAD~1 HEAD`:
  14.4 ms and 3.7 ms with the monitor off,
  and no daemon started;
  2,707.6 ms and 4,608.9 ms with the monitor on and the default bound;
  1,186.6 ms and 425.1 ms with `fsmonitor.startTimeout=1`.
- `git reset -q HEAD --pathspec-from-file=-`:
  23.8 ms and 4.0 ms with the monitor off;
  2,272.8 ms,
  2,628.4 ms,
  2,486.6 ms and 3,650.0 ms with the monitor on;
  5,974.6 ms and 1,001.6 ms with `fsmonitor.startTimeout=1`.
- With the monitor on,
  each step started `fsmonitor--daemon start` itself,
  and with the monitor off it started none.

The spread inside one arm is wide because every arm pays a full registration that competes with
the other sessions of the host for the disk;
the difference between the arms is three orders of magnitude,
so the conclusion does not rest on it.

A lowered bound ends the start's wait and not the step's:
the client then waits for the socket with `wait_if_not_found = 1`
([`fsmonitor-ipc.c:105-124`][src-ipc-retry]),
which appears only when the registration ends,
so an arm ends at the bound when the socket is already there (1,001.6 ms)
and at the bound plus the rest of the registration when it is not (5,974.6 ms).

So a cold step waits for the smaller of the registration and the bound,
and the 60 s pair is the bound rather than a registration of exactly 60 s:
this checkout held 2,269,377 directories,
about 43 s at the 19 microseconds per directory measured in
"Fails:
one cold client blocks for the registration",
and 261 daemons of the user shared the CPU,
so the registration outlasted 60 s and both steps gave up at the bound.

What the wrapper can do about it:
nothing without passing an fsmonitor setting,
which the owner's decision forbids.
The setting is the person's;
the wrapper passes no `core.fsmonitor` to a child and starts no daemon,
by the owner's decision of
2026-11-03 (`package/git-policy/cli/README.md`,
"Git's own fsmonitor"),
and `git monochromatic-git-policy-cli doctor` reports a configuration that turns it on.
What the wrapper does control is how many Git children it starts that read the index,
and a tree-only `diff-tree` child counts among them although it needs nothing from the index:
the landing runs two of them (`src/native/landing/service_land_follow.rs:128`,
`src/native/landing/service_land_git_steps_and_import_stream.rs:343`).
On 0.5.26 the landing's follow-up `git reset -q <commit> --pathspec-from-file=-`
held the landing's `index.lock` for 59 s;
that update is written in process now (`src/native/index/index_reset_file.rs`)
and keeps the monitor's bitmap itself (`src/native/index/index_full_write.rs:104-112`),
falling back to Git's own `reset --no-refresh`
(`src/native/landing/service_land_follow.rs:213`) when it cannot reproduce the index form.
The commit path's index readers are
`git read-tree` in the commit index build (`src/native/commit/commit_index.rs`)
and `git commit` in the shadow repository (`src/native/commit/commit_preparation.rs`).
Every one of them would still wait on a cold daemon with the monitor configured on a large checkout.

Harness,
saved as `step-waits.ts` and run from its own directory inside a capped scope;
the second run of the figures above is its own output:

```sh
# Bounds: six CPU threads, 4 GiB of memory, 512 tasks, disk weight 10.
systemd-run --user --scope --quiet --property=CPUQuota=600% --property=MemoryMax=4G \
  --property=TasksMax=512 --property=IOWeight=10 node step-waits.ts 60000
```

```ts
// step-waits.ts
// How long the two steps of the 120 s commit wait for a cold fsmonitor daemon, and where that wait ends.
// Disposable repository only; it stops the repository's daemon before and after every arm.
import { spawnSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readFileSync, realpathSync, rmSync, writeFileSync } from 'node:fs';
import { homedir } from 'node:os';
import { join } from 'node:path';

const git = '/usr/bin/git';
const directoryCount = Number(process.argv[2] ?? 60000);
const scratch = join(homedir(), 'temp/agent/fsmonitor-steps');
mkdirSync(scratch, { recursive: true });
const root = realpathSync(mkdtempSync(join(scratch, 'wait.')));
// A private global configuration: nothing of the person's own files decides this measurement.
const baseEnv = { ...process.env, GIT_CONFIG_GLOBAL: '/dev/null', GIT_CONFIG_NOSYSTEM: '1' };

const run = (args: readonly string[]) => spawnSync(git, ['-C', root, ...args], { env: baseEnv, encoding: 'utf8' });

function required(args: readonly string[]): string {
  const result = run(args);
  if (result.status !== 0) throw new Error(`git ${args.join(' ')} failed: ${result.stderr}`);
  return result.stdout;
}

// No daemon listens for the repository afterwards: `stop` reaches the socket, and a client's answer
// arrives only after the socket exists, so every arm leaves a daemon `stop` can reach.
const stopDaemon = () => { run(['fsmonitor--daemon', 'stop']); };

// One arm: wall milliseconds of the step, and whether its own trace2 stream shows the daemon start it waited for.
function arm(label: string, settings: readonly string[], args: readonly string[], input?: string) {
  stopDaemon();
  const trace = join(scratch, `wait.${label}.trace2.json`);
  const started = process.hrtime.bigint();
  const result = spawnSync(git, ['-C', root, ...settings, ...args], {
    env: { ...baseEnv, GIT_TRACE2_EVENT: trace },
    encoding: 'utf8',
    input,
  });
  const wallMs = Number(process.hrtime.bigint() - started) / 1e6;
  const daemonStarts = readFileSync(trace, 'utf8')
    .split('\n')
    .filter((line) => line.includes('child_start') && line.includes('fsmonitor--daemon')).length;
  console.log(JSON.stringify({ arm: label, status: result.status, wallMs: Number(wallMs.toFixed(1)), daemonStarts }));
}

try {
  required(['init', '--quiet']);
  for (let index = 0; index < directoryCount; index += 1) {
    mkdirSync(join(root, `d${index % 100}`, `e${index}`), { recursive: true });
  }
  writeFileSync(join(root, 'tracked.txt'), 'one\n');
  required(['add', '.']);
  required(['-c', 'user.name=h', '-c', 'user.email=h@example.invalid', 'commit', '--quiet', '--message', 'first']);
  writeFileSync(join(root, 'tracked.txt'), 'two\n');
  required(['add', '.']);
  required(['-c', 'user.name=h', '-c', 'user.email=h@example.invalid', 'commit', '--quiet', '--message', 'second']);
  const on = ['-c', 'core.fsmonitor=true'];
  const off = ['-c', 'core.fsmonitor=false'];
  console.log(JSON.stringify({ directories: directoryCount, git: required(['--version']).trim() }));
  arm('diff-tree-off', off, ['diff-tree', '-r', '--name-only', '-z', '--no-renames', 'HEAD~1', 'HEAD']);
  arm('reset-off', off, ['reset', '-q', 'HEAD', '--pathspec-from-file=-'], 'tracked.txt\n');
  arm('diff-tree-on', on, ['diff-tree', '-r', '--name-only', '-z', '--no-renames', 'HEAD~1', 'HEAD']);
  arm('reset-on', on, ['reset', '-q', 'HEAD', '--pathspec-from-file=-'], 'tracked.txt\n');
  arm('reset-on-again', on, ['reset', '-q', 'HEAD', '--pathspec-from-file=-'], 'tracked.txt\n');
  arm('diff-tree-bound-1s', [...on, '-c', 'fsmonitor.startTimeout=1'], ['diff-tree', '-r', '--name-only', '-z', '--no-renames', 'HEAD~1', 'HEAD']);
  arm('reset-bound-1s', [...on, '-c', 'fsmonitor.startTimeout=1'], ['reset', '-q', 'HEAD', '--pathspec-from-file=-'], 'tracked.txt\n');
} finally {
  stopDaemon();
  rmSync(root, { recursive: true, force: true });
}
```

Limits of this measurement:

- The 60 s pair itself was not re-run;
the daemon load behind it (261 daemons,
  4,591,335 watches) is not reproducible now that the
person's global configuration says `core.fsmonitor = false` by the owner's decision.
- One host and one Git release;
the macOS and Windows listeners differ.
- The steps were measured as plain Git commands;
an intermediate release ran the same two commands as children of the wrapper.

### Fails: concurrent clients start concurrent daemons

Three clients started together with no daemon listening,
unpatched build,
three runs per size:

- 20,000 directories:
  client wall times 0.41 s to 0.70 s;
  immediately after the clients returned,
  two or three daemons were alive in every run (for example one with 20,197 watches beside two with 12,241 and 13,392),
  and one second later exactly one remained.
- 100,000 directories:
  client wall times 2.47 s to 2.88 s;
  two or three daemons alive right after the clients returned,
  one a second later.
- A first pass under heavier host load gave client wall times up to 6.2 s.

The losing daemons spent their registration work and then exited at the socket lock
([`unix-stream-server.c:53-61`][src-lock]).

### Fails: nested worktrees are watched twice

Unpatched build,
100,000 directories in the main tree and in each of three linked worktrees under `.claude/worktrees/`:

- The main daemon held 400,512 watches (four trees of 100,000 directories,
  plus 512 other directories that were not itemized).
- One status inside a nested worktree started a second daemon with 100,106 watches,
  so those 100,000 directories were watched by two daemons.

### Fails: a shorter start timeout multiplies daemons

Unpatched build,
20,000 directories,
`-c fsmonitor.startTimeout=0`,
ten statuses 40 ms apart:
the first status returned in 0.0108 s,
and eight daemons were alive at once.

### Patched build results

With [`git-fsmonitor-daemon-start-wait.patch`](git-fsmonitor-daemon-start-wait.patch) applied to `v2.56.0`,
the same harness (three runs per size,
same capped scope):

- Cold client:
  20,000 directories took 0.0129 s,
  0.0125 s and 0.0129 s;
  100,000 directories took 0.0134 s,
  0.0134 s and 0.0118 s,
  against 0.37 s and 1.9 s unpatched.
  User plus system CPU of the client rounded to 0.00.
- A status issued while the daemon was still registering took 0.0043 s.
- Time until the daemon held every watch (one-second polling):
  1.1 s for 20,000 directories and 2.6 s for 100,000,
  against 0.47 s and 2.2 s unpatched,
  within the polling granularity.
  Registration costs the same;
  the client no longer pays for it.
- Concurrent clients:
  0.012 s,
  0.010 s and 0.009 s at 20,000 directories,
  0.013 s,
  0.012 s and 0.008 s at 100,000;
  exactly one daemon alive right after the clients returned,
  in every run.
- `fsmonitor.startTimeout=0` with ten clients 40 ms apart:
  one daemon at most.
- Correctness,
  100,000 directories,
  with changes made while the daemon held 66,063 of its watches:
  fifty tracked files modified and fifty untracked files created during registration,
  then twenty-five more of each kind after it.
  The monitor status equalled the no-monitor status in all four comparisons (during registration,
  after it,
  a second time,
  and after the further changes);
  the no-monitor status had 125 lines.
  The unpatched build passed the same check,
  as expected.
- The upstream test script `t/t7527-builtin-fsmonitor.sh` run on the patched build,
  in a capped scope
  (four CPU threads,
  4 GiB,
  512 tasks,
  disk weight 10):
  all 69 tests passed,
  6 of them skipped by prerequisites
  (`SHORTNAMES`,
  `MINGW`,
  `CASE_INSENSITIVE_FS`,
  `UNICODE_COMPOSITION_SENSITIVE`),
  built with `NO_PERL=1`.
  Leftover daemons from the test run exited on their own within about a minute.

## Verified workarounds

### Disable the built-in monitor globally (adopted)

The owner decided on 2026-11-03:
"If git's fsmonitor is fundamentally broken and cannot be easily or meaningfully worked around,
disable git's fsmonitor globally entirely and make doctor say so."

```sh
# Run once per user; the second command per worktree that has a daemon.
git config --global core.fsmonitor false
git -C <worktree> fsmonitor--daemon stop
```

Applied on this host 2026-11-03;
the running daemons were stopped.
The wrapper's doctor reporting is another agent's work and is not verified here.
Tradeoff:
no monitor anywhere,
so every Git command that needs the index state scans the worktree itself.
On the main checkout that cost about 7 ms more per status (16 ms against 9 ms),
and it grows with the number of tracked and untracked paths;
a repository far larger than this one pays more.

### Run a Git built with the prototype patch (not adopted)

[`git-fsmonitor-daemon-start-wait.patch`](git-fsmonitor-daemon-start-wait.patch) applies cleanly to `v2.56.0`
(`git apply --check` on a copy of the three files).
It removes the client's wait and the duplicate daemons,
as measured in "Verification".
Tradeoffs:
it is an unreviewed local patch to Git,
so the host would carry a custom Git build;
a client that gets the early answer scans everything that once and needs one more query before it holds a real token;
the nested-worktree double watching remains,
because the patch only changes when the daemon answers.

## What does not work

- Lowering `fsmonitor.startTimeout` (to 0 in the test).
  Each client returns quickly,
  but a client that finds no listener starts another daemon that registers every watch.
  Ten clients 40 ms apart produced eight live daemons on 20,000 directories.
  On the main checkout,
  whose registration lasts seconds,
  that is many daemons of millions of watches each.
- A client-only change.
  `spawn_daemon` could pass `--start-timeout=0` to `start`,
  but the `fsmonitor.startTimeout=0` measurement in "Verification" is exactly that behavior:
  the wait goes away and the daemons multiply,
  so the fix needs the daemon side (answer before registering,
  and refuse a second registration early).
- Raising `fs.inotify.max_user_watches`.
  The limit here is 10,000,000 and the total in use was 4,591,335,
  so the limit is not what stalls the client;
  the registration time per directory is.
- Moving the linked worktrees out of the main worktree's tree.
  It would remove the double watching,
  but it is not easy here:
  Claude Code creates them under `.claude/worktrees/`.
  Not tried.
- Reading `fsmonitor.ipcThreads` or any documented setting as a remedy.
  The configuration reference has no setting for startup behavior.

## Prototype

[`git-fsmonitor-daemon-start-wait.patch`](git-fsmonitor-daemon-start-wait.patch) changes three files of `v2.56.0`
(70 insertions,
15 deletions):

- `fsmonitor--daemon.h` adds `registering_watches` to the daemon state,
  protected by `main_lock`.
- `compat/fsmonitor/fsm-listen-linux.c` moves the recursive registration out of `fsm_listen__ctor` into a new
  `register_initial_watches`,
  which `fsm_listen__loop` calls right after `ipc_server_start_async`;
  the constructor sets `registering_watches` and the loop clears it under `main_lock` when registration succeeds,
  and a registration failure sets the existing `SHUTDOWN_ERROR`.
- `builtin/fsmonitor--daemon.c` makes `do_handle_client` answer,
  while `registering_watches` is set,
  with the token `builtin:registering` (which `fsmonitor_parse_client_token` rejects) and a trivial `/` payload,
  before the cookie wait.
  The `quit` command returns at the top of that function,
  before the lock,
  and is unaffected.

Because the IPC socket is now created within milliseconds,
the client's `start` sees it listening at once,
and a second `run` fails at the socket lock before it registers anything.
macOS and Windows listeners are unchanged because the new field stays zero there.

Limits and not run:

- No test was added to `t/t7527-builtin-fsmonitor.sh`;
  the harness is the evidence.
- No guard test was shown to fail by removing the early answer (a mutant that hands out a real token during registration would show the
  missed-change hazard the root-cause section describes);
  not run.
- `stop` during registration is not made faster:
  registration is not interrupted by the shutdown flag.
- Inotify queue overflow during a slow registration under heavy file churn was not tested.
- Only Linux and only 2.56.0 were built and run.
  The registration order is the same in the `v2.55.0` source and on `master` at `ebdc087dc9` (read from the hosted files,
  not built),
  and the Linux backend is absent from `v2.54.0`.
- Only `t7527` was run,
  not the rest of Git's test suite.
- The patch was written by an AI agent and has not been read by a Git developer.

## Upstream filing decision

Checked 2026-11-03T10:19:01-05:00.
Default policy is not to file.

### `.out-of-scope/` check

Read the file names in `.out-of-scope/` (`bun-install.md`,
`cargo-workspace.md`,
`claude-code-upstream-bugs.md`,
`codex-harness.md`,
`jsr.md`,
`lightningcss.md`,
`low-impact-typescript-formatting.md`,
`module-es-monolith.md`,
`terminal-title-fork-parity-tests.md`) and searched them for `fsmonitor` and `git`.
No file names Git or fsmonitor,
so no exemption applies.

### Duplicate search

`git/git` has GitHub issues disabled (`gh api repos/git/git` returned `has_issues: false`),
so an issue search there is empty by construction and proves nothing.
Bugs,
patches and review go through the mailing list `git@vger.kernel.org` ([contribution page][contrib]).

- `gh search prs --repo git/git` for `fsmonitor inotify`,
  `fsmonitor starttimeout`,
  `fsmonitor--daemon start`,
  `fsmonitor daemon slow start`,
  `register_inotify`,
  `fsmonitor_ipc__send_query`,
  `spawn_daemon fsmonitor`,
  `fsm_listen__ctor`,
  `fsmonitor nested worktree`,
  `fsmonitor linked worktrees`,
  `fsmonitor max_user_watches`,
  `fsmonitor daemon blocks`,
  `fsmonitor async start`,
  `inotify watches recursive`,
  `fsmonitor.startTimeout` and `fsmonitor Linux daemon`:
  hits were only the Linux backend series (pull requests 1352,
  1667 and 2147,
  the last closed after the series went through the list)
  and unrelated pull requests.
  Positive control:
  the `fsmonitor inotify` query did return the Linux series.
- `gh search issues` with the same queries:
  no results (issues are off).
- Mailing-list search on the `yhbt.net` public-inbox mirror of the `git` list (whose search feed was timestamped 2026-10-20) for
  `register_inotify`,
  `fsmonitor.startTimeout`,
  `fsmonitor--daemon start`,
  `fsmonitor inotify watches`,
  `fsmonitor daemon start blocks client`,
  `fsmonitor nested worktree watch`,
  `fsmonitor linked worktree daemon` and
  `fsmonitor daemon startup slow`:
  the hits were the Linux listener series (versions 1 to 15 by Paul Tarjan,
  and earlier attempts by Eric DeCosta and Marziyeh Esipreh),
  the 2021 and 2022 Windows and macOS series by Jeff Hostetler (which introduced the `start` command and its timeout),
  and a July to August 2026 macOS thread,
  "fsmonitor:
  flush pending FSEvents before cookie wait".
  None discusses the client wait for a registering daemon or concurrent starts.
  Positive control:
  the Linux series query returned the series' messages.
  Not used:
  `lore.kernel.org` (HTTP 403 for scripted requests) and `mail-archive.com`
  (a positive-control query for the Linux series returned no matches,
  so it does not cover recent mail).
  Not completed:
  a search for reactions to AI-assisted patches on the list (the mirror refused the connection),
  and two
  queries,
  `fsmonitor daemon startup wait slow large repository linux` and `fsmonitor inotify max_user_watches`,
  whose responses came back empty under the mirror's rate limit and were not repeated.

No matching report exists,
so a new report is not a duplicate of one;
there is nothing to comment on.

### Work in progress on the code path

- Open pull requests that mention fsmonitor on `git/git`:
  1352 (2024-08-17) and 1667 (2025-02-01),
  both the older Linux listener attempts,
  superseded by series 2147;
  neither touches the start wait.
- Recent commits on the files in the root cause (`gh api repos/git/git/commits?path=...`):
  `fsm-listen-linux.c`,
  last changed 2026-04-15 (the Linux listener's introduction);
  `fsmonitor-ipc.c`,
  2026-04-15 ("close inherited file descriptors and detach in daemon");
  `builtin/fsmonitor--daemon.c`,
  2026-07-05 (Johannes Schindelin,
  "plug token-data leak on early daemon-startup failures"),
  with earlier commits from 2026-04-15 by Paul Tarjan.
  The Linux backend and the daemon code are in recent maintainer flux,
  so the owners read these files now.
- `master` at `ebdc087dc9` (2026-11-01) has the same line numbers for the cited code in `fsmonitor-ipc.c`,
  `builtin/fsmonitor--daemon.c` and `compat/fsmonitor/fsm-listen-linux.c`.
- No claimed work:
  issues are off,
  so there are no "I'll take this" comments or assignees to read.
- Not run:
  branches on forks of `git/git` and `gitgitgadget/git`.
- No hit for this bug on the path.

### Constraint 1: is it really upstream's fault?

Soft yes.
The wait is a deliberate design (the code comment says the client waits for the socket to receive a trivial answer),
and a worktree with many directories is expected to need many watches (the Linux backend's cover letter names the per-directory watch as
inotify's limitation,
[pull request 2147][pr2147]).
What is not intended is that the client blocks for the whole registration to receive an answer that is trivial by the code's own comment,
spinning for it,
and that concurrent clients each pay for a full registration.
The nested-worktree amplification is this host's layout,
not Git's.

### Constraint 2: can upstream fix it?

Yes.
The prototype shows the daemon can answer before registering with a token the existing parser already rejects,
and the client's failure path already does a full scan.

### Constraint 3: do they support this use case?

Soft yes.
A large worktree is the stated purpose of the daemon (the manual says the daemon exists so that commands such as `git status` can avoid scanning the disk),
the Linux caveats name large repositories,
and the Linux listener is new in the 2.55 cycle.
No document covers nested linked worktrees or the start wait;
`fsmonitor.startTimeout` is undocumented.

### Constraint 4: would the repository welcome our contribution?

Soft fail.
Git does not use the hosted tracker;
reports and patches go to the mailing list,
and pull requests go through GitGitGadget
([contribution page][contrib],
[pull request template][pr-template]).
No trust or blocklist system was found (see "Risk to the poster").
The project's own text on AI says,
quoted from [`Documentation/SubmittingPatches:571-587`][doc-ai]:

> To avoid these issues,
> we will reject anything that looks AI
> generated,
> that sounds overly formal or bloated,
> that looks like AI
> slop,
> that looks good on the surface but makes no sense,
> or that
> senders don't understand or cannot explain.

It does not ban AI assistance,
and it recommends careful use.
But this patch and this analysis were written by an agent,
and a patch sent as-is would be the thing that text says is rejected.
The same file requires a human `Signed-off-by` certifying the origin of the contribution,
which an agent cannot give.
A human who rewrote the change and can explain every line could meet the text,
but that is a different artifact from this prototype.

### Constraint 5: will they likely fix it?

Soft yes,
by absence of a signal against it.
No documented won't-fix or stated non-goal was found,
and no maintainer has declined a comparable request in the searches under "Duplicate search".
The code owners are active in this code (commits on 2026-04-15 and 2026-07-05,
a macOS fsmonitor patch under review in July and August 2026).

### Constraint 6: have we prototyped a minimal fix?

Yes (auto-prototype ran because constraints 1 to 5 held or soft-held).
Result:
[`git-fsmonitor-daemon-start-wait.patch`](git-fsmonitor-daemon-start-wait.patch) in a disposable clone of `v2.56.0`
(commit `a018953688f1b10bddf91bff8747068f5f4746a4`,
origin `https://github.com/git/git.git` confirmed before editing).
Verification command and output are in "Verification",
baseline against patched,
with the limits listed in "Prototype".

### Risk to the poster

Filing is the poster's decision,
and this document does not recommend it.

- The Git list is a public,
  permanent archive under the poster's name and address.
  A patch that "looks AI generated" is rejected by the project's stated policy,
  and the rejection would be public.
- The patch needs a human `Signed-off-by`,
  a legal certification the poster alone can make.
- A trust or blocklist system:
  searched `git/git` on 2026-11-03 at `master` commit `ebdc087dc9d68bb340c282b2ba5f722def28d19`.
  The repository tree has no `VOUCHED`,
  denouncement or blocklist file,
  and its workflows are `check-style.yml`,
  `check-whitespace.yml`,
  `coverity.yml`,
  `l10n.yml` and `main.yml`.
  The contribution policy text read was `.github/CONTRIBUTING.md`,
  `.github/PULL_REQUEST_TEMPLATE.md` and `Documentation/SubmittingPatches`.
  No such system was found.
  The poster should check the live repository again on the day of posting.
  Mailing-list reputation is outside what a repository search can see.
- A report or patch the poster cannot explain line by line is the case the project says it rejects.

### Readiness check

Recorded 2026-11-03T10:19:01-05:00.

1.  Latest release and other maintained lines.
    The tag list (`git ls-remote --tags --refs https://github.com/git/git`) shows `v2.54.0`,
    `v2.55.0` and `v2.56.0` as the latest three,
    with no maintenance releases for older lines among them.
    Reproduced on `v2.56.0` (the host's `/usr/bin/git`,
    and a build of the tag).
    `v2.55.0` was not built;
    its hosted `compat/fsmonitor/fsm-listen-linux.c` registers watches before `ipc_server_start_async` at the same
    lines as `v2.56.0`.
    `v2.54.0` has no Linux backend (the file returns 404).
    `master` at `ebdc087dc9` has the same cited lines.
2.  Prior art.
    The cover letter of pull request 2147 was read (inotify design,
    the 30 s stop timeout,
    the watch-limit message).
    The series' review messages were not read in full,
    so a maintainer reason for the blocking start,
    if one was given there,
    was not found.
    The only stated reason found is the code comment in `fsmonitor-ipc.c:114-122` (the daemon "can't possibly have any FS cached yet").
3.  Intake.
    Mailing list only;
    a pull request goes through GitGitGadget and the pull request template asks for an empty description on a
    single-commit request.
    Fields only the poster can answer:
    name and address for the mail,
    a `Signed-off-by`,
    and what they personally reproduced.
    There is no issue form and issues are off.
4.  Automation.
    GitGitGadget turns pull requests into mail;
    its approval step for new contributors was not read.
    Not run.
5.  Duplicate search,
    work-in-progress check and trust-system search were run today (queries under "Duplicate search");
    the cited code exists on `master` at `ebdc087dc9`.
6.  Reproduction as written:
    the harness in this document was run as written,
    on both binaries,
    today.
    The draft in "Upstream filing artifact" uses that harness.
7.  Runnable by others:
    not run.
    A private reproduction repository with continuous integration would be a GitHub mutation outside this repository,
    and the target takes mail,
    not issues;
    the harness here is the self-contained reproduction.
    It needs the poster's authorization before anything is created.
8.  Verdict:
    not recommended.
    The blocker is a rule that can be quoted,
    the project's own text in `Documentation/SubmittingPatches:571-587`,
    plus the human `Signed-off-by` in the same file.
    What would clear it:
    the poster rewrites the change themselves,
    can explain it,
    and signs it off.
    Cross-vendor review of the root cause and patch was not run.

## Upstream filing artifact

Do not file as-is.
Constraint 4 is a soft fail for an agent-authored patch,
and the readiness check has no recommendation.
The draft in this section is kept as an auditable record for a mailing-list post that a human would rewrite and own.
Recipients to check with `contrib/contacts/git-contacts` (not run):
`git@vger.kernel.org`,
and the authors of recent commits on these files (Paul Tarjan,
Patrick Steinhardt,
Johannes Schindelin).

~~~md
Subject: [BUG] fsmonitor--daemon (Linux): the first command after daemon start blocks until every directory has an inotify watch

Hi,

[Poster: state in your own words what you reproduced yourself, and whether and how AI assistance was used.]

What happens: with core.fsmonitor=true on Linux and no daemon running, the first Git command that reads the
index waits for the new daemon to register an inotify watch on every directory, then continues.
The wait is about 19 microseconds per directory here (0.37 s for 20,000 directories, 1.9 s for 100,000),
and the waiting `git fsmonitor--daemon start` process spins (waitpid and lstat, no sleep) for all of it.
Clients that arrive while the daemon is still registering each start another daemon, which registers every
watch again and then fails at the IPC socket lock.

Why: fsmonitor_ipc__send_query() runs `git fsmonitor--daemon start` and waits for the socket
(fsmonitor-ipc.c:58-71, 105-124). start waits up to fsmonitor.startTimeout (60 s) in
start_bg_command() (run-command.c:2052-2081). In the daemon, fsm_listen__ctor() registers all watches
(compat/fsmonitor/fsm-listen-linux.c:426-440) before fsmonitor_run_daemon_1() creates the socket
(builtin/fsmonitor--daemon.c:1387-1394, 1224-1233), and the server only starts serving in fsm_listen__loop()
(fsm-listen-linux.c:698-702). The comment at fsmonitor-ipc.c:114-122 says the answer is always trivial
anyway. The only exclusion between daemons is at socket creation (unix-stream-server.c:53-61), after registration.

Reproduction (Git 2.56.0, Linux): the attached harness builds a disposable repository with N directories and runs
`git -c core.fsmonitor=true status` with no daemon: [Poster: attach fsmonitor-start-wait.ts and its output].

Possible fix: start serving before registering, and while registering answer every query with a trivial response and a
token that fsmonitor_parse_client_token() rejects, so the client does a full scan now and gets a real token on its next
query (a real token must not be issued before every directory has a watch). That also makes a second `run` fail at the
socket lock before it registers anything. With such a change the cold client took about 12 ms for both sizes, one daemon
was alive for three concurrent clients, status output matched `core.fsmonitor=false` during and after registration,
and t7527 passed on Linux. [Poster: attach a patch you wrote, understand and signed off.]

Not tested: macOS and Windows listeners (unchanged by the idea), inotify queue overflow during a long registration,
the rest of the test suite.
~~~

[contrib]: https://github.com/git/git/blob/a018953688f1b10bddf91bff8747068f5f4746a4/.github/CONTRIBUTING.md?plain=1#L1-L6
[pr-template]: https://github.com/git/git/blob/a018953688f1b10bddf91bff8747068f5f4746a4/.github/PULL_REQUEST_TEMPLATE.md?plain=1#L1-L6
[pr2147]: https://github.com/git/git/pull/2147#:~:text=While%20inotify%20has%20the%20limitation%20of%20requiring%20a%20separate%20watch%20on%20every%20directory
[src-diff-tree-index]: https://github.com/git/git/blob/a018953688f1b10bddf91bff8747068f5f4746a4/builtin/diff-tree.c#L133
[src-diff-tree-stdin-index]: https://github.com/git/git/blob/a018953688f1b10bddf91bff8747068f5f4746a4/builtin/diff-tree.c#L212-L213
[src-ipc-spawn]: https://github.com/git/git/blob/a018953688f1b10bddf91bff8747068f5f4746a4/fsmonitor-ipc.c#L58-L71
[src-ipc-retry]: https://github.com/git/git/blob/a018953688f1b10bddf91bff8747068f5f4746a4/fsmonitor-ipc.c#L105-L124
[src-timeout-config]: https://github.com/git/git/blob/a018953688f1b10bddf91bff8747068f5f4746a4/builtin/fsmonitor--daemon.c#L39-L40
[src-bg-wait-cb]: https://github.com/git/git/blob/a018953688f1b10bddf91bff8747068f5f4746a4/builtin/fsmonitor--daemon.c#L1488-L1511
[src-start-bg]: https://github.com/git/git/blob/a018953688f1b10bddf91bff8747068f5f4746a4/builtin/fsmonitor--daemon.c#L1548-L1549
[src-bg-loop]: https://github.com/git/git/blob/a018953688f1b10bddf91bff8747068f5f4746a4/run-command.c#L2052-L2081
[src-ipc-state]: https://github.com/git/git/blob/a018953688f1b10bddf91bff8747068f5f4746a4/compat/simple-ipc/ipc-unix-socket.c#L26-L37
[src-register]: https://github.com/git/git/blob/a018953688f1b10bddf91bff8747068f5f4746a4/compat/fsmonitor/fsm-listen-linux.c#L286-L307
[src-ctor]: https://github.com/git/git/blob/a018953688f1b10bddf91bff8747068f5f4746a4/compat/fsmonitor/fsm-listen-linux.c#L426-L440
[src-loop]: https://github.com/git/git/blob/a018953688f1b10bddf91bff8747068f5f4746a4/compat/fsmonitor/fsm-listen-linux.c#L698-L702
[src-run-order]: https://github.com/git/git/blob/a018953688f1b10bddf91bff8747068f5f4746a4/builtin/fsmonitor--daemon.c#L1387-L1394
[src-ipc-init]: https://github.com/git/git/blob/a018953688f1b10bddf91bff8747068f5f4746a4/builtin/fsmonitor--daemon.c#L1224-L1233
[src-lock]: https://github.com/git/git/blob/a018953688f1b10bddf91bff8747068f5f4746a4/unix-stream-server.c#L53-L61
[src-client-fail]: https://github.com/git/git/blob/a018953688f1b10bddf91bff8747068f5f4746a4/fsmonitor.c#L558-L569
[src-client-apply]: https://github.com/git/git/blob/a018953688f1b10bddf91bff8747068f5f4746a4/fsmonitor.c#L691-L707
[src-token-parse]: https://github.com/git/git/blob/a018953688f1b10bddf91bff8747068f5f4746a4/builtin/fsmonitor--daemon.c#L652-L675
[src-token-invalid]: https://github.com/git/git/blob/a018953688f1b10bddf91bff8747068f5f4746a4/builtin/fsmonitor--daemon.c#L745-L754
[src-start-timeout-option]: https://github.com/git/git/blob/a018953688f1b10bddf91bff8747068f5f4746a4/builtin/fsmonitor--daemon.c#L1582-L1585
[doc-single]: https://github.com/git/git/blob/a018953688f1b10bddf91bff8747068f5f4746a4/Documentation/git-fsmonitor--daemon.adoc?plain=1#L49-L51
[doc-linux]: https://github.com/git/git/blob/a018953688f1b10bddf91bff8747068f5f4746a4/Documentation/git-fsmonitor--daemon.adoc?plain=1#L97-L107
[doc-config]: https://github.com/git/git/blob/a018953688f1b10bddf91bff8747068f5f4746a4/Documentation/config/fsmonitor--daemon.adoc?plain=1#L1-L11
[doc-ai]: https://github.com/git/git/blob/a018953688f1b10bddf91bff8747068f5f4746a4/Documentation/SubmittingPatches#L571-L587
