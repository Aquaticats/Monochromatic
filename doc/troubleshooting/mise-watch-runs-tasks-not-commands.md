# Mise 2026.7.0 `watch -- node` treats `node` as a task and aborts startup

## Symptom

Both Done development tasks used:

```toml
run = "mise watch -w src -r -- node src/server.ts"
```

Running `mise run //package/done/app:dev:site` failed before starting Node:

```text
mise ERROR no task //package/done/app:node found
```

`--watch` and `--restart` worked as options.
The failure was specific to placing an arbitrary command after `mise watch`.

## Root cause

Mise `watch` watches and reruns mise tasks,
not arbitrary executables.
The command documentation says this directly at `src/cli/watch.rs:20-23`:

```rust
/// Run task(s) and watch for changes to rerun it
///
/// This command uses the `watchexec` tool to watch for changes to files and rerun the specified task(s).
```

The first positional value is parsed as `task`,
and trailing values become task arguments at `src/cli/watch.rs:29-42`:

```rust
pub struct Watch {
    /// Tasks to run
    task: Option<String>,

    /// Task and arguments to run
    args: Vec<String>,
}
```

`Watch::run` collects those values and resolves all of them through the task list at `src/cli/watch.rs:80-88`:

```rust
let args = once(self.task)
    .flatten()
    .chain(self.task_flag.iter().cloned())
    .chain(self.args.iter().cloned())
    .collect::<Vec<_>>();
let tasks = crate::task::task_list::get_task_lists(&config, &args, false, false).await?;
```

The `--` separator did not switch `watch` into arbitrary-command mode.
It made `node src/server.ts` positional task input,
so mise searched for a task named `node`.

## Verification

The installed binary and audited upstream tag were `mise 2026.7.0`.
Tag `v2026.7.0` resolves to commit `857b73f6a6b39a3bc90c44119a1e86ee11bd7273`.

### Failing pattern

```sh
mise watch --watch src --restart -- node src/server.ts
```

Result:

```text
mise ERROR no task //package/done/app:node found
```

### Working patterns

Mise's own documentation uses a named task at `doc/tasks/running-tasks.md:119-125`:

```sh
mise watch build
```

The verified package configuration follows the same shape:

```toml
[tasks."serve:site"]
hide = true
run = "node src/server.ts"

[tasks."dev:site"]
run = "mise watch --watch src --restart serve:site"
```

Verify each package with disposable database state:

```sh
DB_PATH=:memory: mise run //package/done/app:dev:site
DB_PATH=:memory: mise run //package/done/variant-postcss:dev:site
```

Each watch task started its hidden `serve:site` task and printed a listening URL.
Touching that package's `src/server.ts` caused mise to stop the server gracefully,
rerun the same `serve:site` task,
and print a second listening URL.
Neither run attempted to resolve a `node` task.

## Verified workarounds

### Wrap the command in a hidden mise task

Define the executable command once as `serve:site`,
then pass that task name to `mise watch`.

Advantages:

- preserves mise-managed tools and environment;
- keeps watch semantics in mise;
- gives browser verification a non-watching `serve:site` entry point;
- uses documented task selection.

Tradeoff:
the package gains one hidden task.
The watcher still depends on `watchexec` and follows mise task argument parsing.

### Call `watchexec` directly

`watchexec --restart --watch src -- node src/server.ts` accepts an arbitrary command.

Tradeoff:
this bypasses the repository rule that build and development commands run through mise tasks,
and duplicates mise's tool and environment setup.
It was not adopted.

## What does not work

### Put `--` before the executable

`mise watch -- node src/server.ts` does not delegate to a shell command.
`node` remains the task positional.

### Pass the source path as a task argument

Once `node` fails task lookup,
`src/server.ts` is never interpreted by Node.
Changing its position cannot create the missing task.

### Use short options to change parsing

`-w` and `-r` are aliases for watcher options only.
They do not change positional task resolution.
Long options make that distinction explicit.

## Upstream filing decision

No `.out-of-scope/` entry matched mise watch task parsing.
Open and closed mise issue and pull-request searches
for `watch arbitrary command task not found` returned no matching report.

1.  **Upstream fault**
    No.
    Mise documents `watch` as task-oriented,
    and the source implements that contract.
2.  **Upstream fixability**
    Mise could add an arbitrary-command mode,
    but no defect requires it.
3.  **Supported use case**
    Watching named mise tasks is documented and supported.
    Watching arbitrary commands through `mise watch` is not documented.
4.  **Contribution policy**
    The audited repository has `CONTRIBUTING.md` and an issue template.
    No policy review changes the failed fault criterion.
5.  **Fix likelihood**
    Not applicable because no upstream defect is claimed.
6.  **Compatible prototype**
    The project-side hidden-task composition is implemented and verified.
    An upstream prototype is not appropriate because criterion 1 fails.

There is no upstream issue or comment to file,
so the filing artifact is intentionally empty.

## Sources

- [Mise watch implementation][mise-watch-source]
- [Mise task watching documentation][mise-watch-docs]

[mise-watch-source]: https://github.com/jdx/mise/blob/857b73f6a6b39a3bc90c44119a1e86ee11bd7273/src/cli/watch.rs
[mise-watch-docs]: https://github.com/jdx/mise/blob/857b73f6a6b39a3bc90c44119a1e86ee11bd7273/docs/tasks/running-tasks.md
