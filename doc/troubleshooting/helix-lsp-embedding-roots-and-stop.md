# helix-lsp embedded outside Helix: `Registry::stop` blocks later starts, and a `PWD`-spelled workspace loses canonical document roots

Two behaviors of `helix-lsp` at revision `ba40e547426b0f9896c8bdc699a4ab11f2b37dbc`
that are correct inside the Helix editor
but break an embedder that keeps its own server lifecycle and uses resolved (canonical) paths,
as the IDE's Language module in `package/ide/app/src/language` does.
Neither is recorded in [the language-intelligence design](../planning/slint-ide-language-intelligence.md).

## Symptom

### `Registry::stop` makes a server name unstartable

After `Registry::stop(name)`,
every later `Registry::get` for a language that lists that server yields nothing for it:
no client,
no error,
and no `ExecutableNotFound`.
The design proposed `Registry::stop` for a server whose root turned out to be outside the project;
used that way,
the server can never be started again in the same process,
not even for a later file whose root is inside the project.
The Language module then reports the server as not started,
and every position request answers "no server".

### Canonical document paths lose their root when `PWD` names a link

On this host `/home` is a symbolic link to `var/home`.
When the application is started from a shell whose `PWD` spells the project through the link
(for example `cd ~/project`),
and the application makes the project its working directory with the resolved path
(`/var/home/user/project`),
Helix still spells its workspace through the link (`/home/user/project`).
A document passed to `Registry::get` with its resolved path
then gets no LSP root:
`initialize` carries `rootUri: null` and no workspace folders,
although the document is inside the project.

## Root cause

### The stop tombstone

`Registry::stop` drains the client list for the name and keeps the empty list on purpose
(`helix-lsp/src/lib.rs:690`):

```rust
// helix-lsp/src/lib.rs:690
pub fn stop(&mut self, name: &str) {
    if let Some(clients) = self.inner_by_name.get_mut(name) {
        // Drain the clients vec so that the entry in `inner_by_name` remains
        // empty. We use the empty vec as a "tombstone" to mean that a server
        // has been manually stopped with :lsp-stop and shouldn't be automatically
        // restarted by `get`. :lsp-restart can be used to restart the server
        // manually.
        for client in clients.drain(..) {
```

`Registry::get` then skips the name entirely (`helix-lsp/src/lib.rs:714`):

```rust
// helix-lsp/src/lib.rs:714
if let Some(clients) = self.inner_by_name.get(name) {
    // If the clients vec is empty, do not automatically start a client
    // for this server. The empty vec is a tombstone left to mean that a
    // server has been manually stopped and shouldn't be started automatically.
    // See `stop`.
    if clients.is_empty() {
        return None;
    }
```

Only `Registry::restart_server` removes the tombstone (`helix-lsp/src/lib.rs:661`).
`Registry::remove_by_id` (`helix-lsp/src/lib.rs:604`) removes one client
and deletes the name's list when it becomes empty,
 leaving no tombstone.

### The `PWD` spelling of the workspace

Helix reads the working directory once and prefers `PWD` when it resolves to the same directory,
which is `pwd -L` behavior (`helix-stdx/src/env.rs:17`):

```rust
// helix-stdx/src/env.rs:17
pub fn current_working_dir() -> PathBuf {
    if let Some(path) = &*CWD.read().unwrap() {
        return path.clone();
    }

    // implementation of crossplatform pwd -L
    // we want pwd -L so that symlinked directories are handled correctly
    let mut cwd = std::env::current_dir().expect("Couldn't determine current working directory");

    let pwd = std::env::var_os("PWD");
    // ...
    if let Some(pwd) = pwd.map(PathBuf::from) {
        if pwd.canonicalize().ok().as_ref() == Some(&cwd) {
            cwd = pwd;
        }
    }
```

`std::env::set_current_dir` does not change `PWD`,
so a shell-exported `/home/user/project` survives a change to `/var/home/user/project`.
`start_client` derives the workspace from that value (`helix-lsp/src/lib.rs:900`),
and `find_lsp_workspace` compares the document path with it as text,
without resolving links (`helix-lsp/src/lib.rs:996`):

```rust
// helix-lsp/src/lib.rs:996
if !file.starts_with(workspace) {
    return None;
}
```

`/var/home/user/project/src` does not start with `/home/user/project`,
so the root is `None` and `start_client` falls back to the workspace directory
with no root address (`helix-lsp/src/lib.rs:914`).
The `root_dirs` entries are compared the same way (`helix-lsp/src/lib.rs:1015`),
so passing the resolved project root there does not help either.

## Verification

Version under test:
`helix-lsp`,
`helix-core`,
`helix-loader`,
and `helix-stdx` at `ba40e547426b0f9896c8bdc699a4ab11f2b37dbc`,
locked in `package/ide/app/Cargo.lock`.
Harness:
the package task `inspect:language-guards`
copies the package to a disposable directory,
applies one mutation,
and runs one named test in the bounded build container
against the scripted language server `ide-scripted-lsp`.

```sh
# package/ide/app; CACHE is a disposable copy of target/ below ~/temp/agent
mise run //package/ide/app:inspect:language-guards "$CACHE" '' root-spelling,stop-tombstone
```

Patterns that work:

- Removing an ended or refused server with `Registry::remove_by_id` plus `Client::force_shutdown`:
  the next file of the language starts a new process
  (`lifecycle::crash_fails_the_pending_request_and_the_next_open_restarts`,
  three processes in one session).
- Passing the document path to Helix in Helix's spelling of the project root
  (resolved project path re-rooted at the resolved workspace,
   then joined to the workspace as Helix spells it):
  `rootUri` is the project in the link spelling,
  and pushed diagnostics for that address are recognized as the displayed file by resolved path
  (`roots::project_reached_through_a_linked_working_directory_is_rooted_at_the_project`).

Patterns that fail:

- `stop-tombstone`:
  replacing `remove_by_id` with `Registry::stop` in the removal step.
  The restart after a crash never happens;
  the test fails with `an exited server could not be started again`.
- `root-spelling`:
  passing the resolved document path unchanged.
  The test fails with
  `the server was not rooted at the project reached through the linked working directory`
  (`rootUri` is `null`).

Results are listed under "Evidence".

## Verified workarounds

- Never call `Registry::stop` for a server that may be needed again.
  Remove it with `Registry::remove_by_id(id)` and stop it with `Client::force_shutdown()`
  (`package/ide/app/src/language/attach.rs`,
   function `retire`).
  Tradeoff:
  nothing prevents the next `get` from starting the same server again,
  so a refusal that must persist (a root outside the project) is re-checked before every start
  instead of being remembered by Helix.
- Ask Helix for its own workspace with `helix_core::find_workspace()`,
  resolve it,
  and respell every path below the resolved project root with Helix's spelling
  before handing it to `Registry::get` or building a document address
  (`package/ide/app/src/language/root.rs`,
   type `RootView`).
  Compare documents and targets by resolved path everywhere else.
  Tradeoff:
  server-facing addresses use the link spelling,
  so any comparison with a server address must resolve it first;
  the Language module already classifies every server address by resolved path.
- Before starting a server,
  compute the root with the public `helix_lsp::find_lsp_workspace` using the same arguments `start_client` uses,
  and refuse a root outside the project.
  Tradeoff:
  it duplicates three lines of `start_client` that must be kept in step with the pinned revision.

## What does not work

- Clearing `PWD` before startup:
  it would have to happen before any thread starts,
  because `std::env::remove_var` is unsafe once other threads exist in the 2024 edition,
  and it changes how every child process sees its directory.
- Calling `helix_stdx::env::set_current_working_dir`:
  it fixes the spelling but adds a direct dependency on `helix-stdx`,
  which the design avoided,
  and it still runs process-wide.
- Passing the resolved project root in `root_dirs`:
  `find_lsp_workspace` joins it to the link-spelled workspace and compares text,
  so it never matches.

## Upstream filing decision

`.out-of-scope/` was checked:
no entry covers Helix or `helix-lsp`.
Tracker searches on 2026-10-05 (`gh search issues --repo helix-editor/helix`):
`lsp-stop` returned #15776 (stale editor state after a stop),
#7405,
and #14584,
none about restarting after `stop`;
`symlink` returned #13520 (duplicate references in a symlinked directory),
a different symptom.
A positive control for the search:
the `lsp-stop` query did return results.

1. Is it really upstream's fault?
   No.
   The tombstone is documented intent for `:lsp-stop` (`helix-lsp/src/lib.rs:692` to `:696`),
   and the `PWD` preference is documented `pwd -L` intent (`helix-stdx/src/env.rs:22` to `:23`).
   Inside Helix both are consistent,
    because Helix never mixes resolved and link-spelled paths.
   The mismatch comes from this embedder.
2. Can upstream fix it?
   Not applicable after constraint 1.
3. Are they supporting this use case?
   No evidence of support for embedding `helix-lsp` outside Helix was found;
   the crate's interfaces mirror the editor's needs.
4. Would the repo welcome our contribution?
   Not evaluated,
    because constraint 1 fails.
5. Will they likely fix it?
   Not evaluated.
6. Prototype:
   none,
    because there is no upstream defect to fix.

Decision:
do not file.

~~~md
Title: (do not file as-is) helix-lsp: Registry::stop leaves a permanent tombstone; find_lsp_workspace compares PWD-spelled workspace with resolved paths as text

Both behaviors are intended inside Helix. Recorded only so a future embedder-support discussion has a source trace:
- helix-lsp/src/lib.rs:690 `stop` keeps an empty client list; `get` (lib.rs:714) then never starts the name again.
- helix-stdx/src/env.rs:17 prefers `PWD`; helix-lsp/src/lib.rs:996 compares paths textually, so resolved document paths get no root.
~~~

## Evidence

Measured on 2026-10-05 with
`mise run //package/ide/app:inspect:language-guards "$CACHE" '' root-spelling,stop-tombstone`.
Each control passed its unmodified baseline,
failed with the mutation,
and passed again after restoring
(`~/temp/agent/ide-language-guard-uWmw4T/results.json`,
 scratch that can vanish):

- `root-spelling` removed:
  `rootUri` was `Null` where `"file:///tmp/.tmp9SuHqi/link/project"` was expected.
- `stop-tombstone` removed:
  the request after the crash answered `NoServer` where `Starting` was expected.
