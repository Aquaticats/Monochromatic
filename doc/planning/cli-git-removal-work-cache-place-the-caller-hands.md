# A removal work's cache place, and who hands it over

This proposal is unapproved and non-final.
It was written on 2026-11-27 by the round for issue 6571.
The round built the fix that hands a fixture's cache place to the work a fixture context starts,
and this file holds the one writer that fix does not reach,
with the options for it and the questions the owner answers.
Nothing here is built:
this branch changes no signature these options touch,
and no release code was changed for them.
What was done and checked is in the round's report;
what is open is the choice below.

## What the branch fixed

The wrapper records what it learns about the real Git in a cache under `XDG_CACHE_HOME` or the home directory:
the text `git version` prints,
where that Git's system configuration file is,
and where its system attributes file is.
A control's work must use disposable fixtures.
It must never use the person's home
(`AGENTS.md`'s THR and HOM).

`for_context` and `check_clean_by_content` now read the variables a child of their context runs with,
this process's own with the context's overlay over them,
so a fixture names `XDG_CACHE_HOME` in its overlay and its work records in a folder of its fixture.
The measurements are in the report:
the landing control that wrote `git-system-attributes-<digest>` under a throwaway home writes it under
`native-unit-<process id>-<number>-cache-home/monochromatic-git-policy-cli` now,
and a fresh home stays empty.

## The writer that remains

Measured on this host by running the `worktree::worktree_remove` module with the process's home pointed
at a throwaway folder:
one file appeared,
`git-system-config-<digest>` in `$HOME/.cache/monochromatic-git-policy-cli`,
with the same 16 hexadecimal digits as the attributes file the issue measured.
A stack captured at the write named the route:

- control `worktree::worktree_remove_hand_over::tests::a_half_deleted_worktree_holding_a_persons_file_is_kept_and_said`,
- `delete_subvolume_of_with` -> `remove_the_worktree_of`,
- `worktree_remove_hand_over_service_git_and_pool::the_service_git`,
  which builds a context of its own with `overlay: Vec::new()`,
- `transaction_git::run_git` -> `repo_read_hook::try_request` -> `repo_read_hook::attempt`,
- `ScopeEnvironment::from_variables` -> `system_config_for` -> `store_cached_text`.

The control runs the removal in its own process,
so `the_service_git` resolves a Git from the test binary's process environment,
and the cache place is the home that process has.
No variable a control names reaches that place through a context:
the work makes a context instead of taking one,
because a record may be attempted in a service that has just restarted,
and the Git is resolved at that attempt.

`repo_read_hook::attempt` has a second defect on this route:
it puts this process's entries before the caller's variables,
and `environment_value` answers with the first match,
so a variable the caller named does not win.
Nothing in this proposal depends on that being fixed first,
but an option that hands the work variables needs it.

## Options

### Option A: the removal work runs under the context its handlers hold

`delete_subvolume_of_with`
and `remove_the_worktree_of`
and `the_service_git`
take the `&GitContext` their caller runs every other Git child in,
and `the_service_git`'s own resolution of Git goes.

```rust
// worktree_remove_hand_over_service_removal.rs
pub(crate) fn delete_subvolume_of_with(
    context: &GitContext,
    item: &WorkItem,
    proc_root: &Path,
) -> Verdict;

// the service's work: the context the handlers run every child in
let verdict = delete_subvolume_of_with(&handlers.context, item, Path::new(PROCESS_TABLE));

// a control: the context of its fixture, whose overlay names its cache place
let verdict = delete_subvolume_of_with(&git_context(), &item, table.as_path());
```

Pros:

- the cache place comes from the caller,
  which is the shape issue 6571 asks for;
- the work runs the same Git the service's other children run,
  which is what `GitHandlers.context` already says it is for;
- one read of this process's environment for resolving Git leaves the removal path.

Cons:

- a signature the release code's callers see changes in three functions,
  and the queue and wait faces that call them change with them;
- a service that restarted between the hand-over and the removal removes with the Git its handlers resolved
  at their start,
  not one resolved at the record's attempt;
- every test that calls these faces passes a context.

### Option B: the work keeps resolving its own Git, and its caller hands it the variables

`the_service_git` takes the variables to resolve with,
and the release passes this process's own,
which is what it reads today.

```rust
pub(crate) fn the_service_git(
    item: &WorkItem,
    environment: &[(OsString, OsString)],
) -> Result<GitContext, String>;

// the release: the service's own environment, as the removal reads it today
let context = the_service_git(item, std::env::vars_os().collect::<Vec<_>>().as_slice())?;

// a control: its fixture's environment, whose overlay names its cache place
let context = the_service_git(&item, environment_with_overlay(&process, &isolated_overlay()).as_slice())?;
```

Pros:

- the work keeps its own Git resolution,
  so a record attempted after a restart still finds the Git of that moment;
- the place arrives through the variables every other reader takes,
  with no second kind of value to learn.

Cons:

- a signature the release code's callers see changes,
  as in Option A;
- one fact has two channels (a context's overlay and a handed environment),
  so a reader must know which one a call takes;
- it needs `repo_read_hook::attempt` to let the caller's variables win,
  or the handed place is outranked by the process's own.

### Option C: a build of the controls refuses the process's own home cache

No signature changes:
the writer refuses a place that is this process's own home cache,
in a build whose `cfg!(test)` holds,
and its words name the variable to add.

```rust
// invocation/git_version_cache.rs, in a test build only
if Some(place) == cache_directory(std::env::vars_os().collect::<Vec<_>>().as_slice()).as_deref() {
    panic!(
        "a control's work must name its own cache place: put `XDG_CACHE_HOME` in the fixture's overlay"
    );
}
```

Pros:

- no signature changes at all,
  and no release path behaves differently in a release;
- every test that reaches any writer fails at the line that would write outside its fixtures,
  whatever route it took,
  so the guard the branch adds would cover routes no context reaches.

Cons:

- the refusal is release code that behaves differently under `cfg!(test)`,
  which this crate does once already (`invocation/git_version.rs`)
  for the process-wide version ask,
  and calls a limit;
- several callers catch a panic and turn it into a declined answer
  (`call_catching_panics`
  and `repo_read_hook::guarded`),
  so a control can pass while the refusal fired,
  which is exactly how this route would hide its own finding;
- a control that reads a cache a person's own session wrote is refused too,
  although reading it is what makes a control's answer depend on the machine,
  and the message would name a write.

## Ranking

The ranking is A > B > C.

### Why A ranks above B

Both hand the place in from the caller.
A hands the one value the service already holds for every child it runs,
so there is one channel instead of two,
and the removal stops resolving a second Git of its own.
B's only gain over A is the Git of the attempt,
and a service that restarts mid-record is the rarer case.

### Why B ranks above C

B fixes the place for every route that reaches the work.
C refuses the write,
and two callers already catch a panic and turn it into a declined answer,
so C can leave the file written and the control green.
C stays listed because it is the only option that costs no signature,
and a round that cannot change release code may reach for it.

## Questions for the owner

1. Which option is built?
   The options are A,
   B and C.
2. Where a removal record outlives a service restart,
   should its attempt resolve Git itself (B) or use the Git the service started with (A)?
3. Should the shape chosen also hand a control's own child process the fixture's cache place,
   since a body a control runs in a child of its own inherits the parent's home today,
   or is that a separate defect?
