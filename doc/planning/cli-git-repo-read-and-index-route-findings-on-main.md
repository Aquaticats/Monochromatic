# Repo-read and index route findings on `main`

Status:
audit,
2026-11-09.
Audited revision:
`main` at `94664520deb0`,
in the worktree
`/var/mnt/encrypted/repository/Monochromatic/.claude/worktrees/pi-repo-read-and-index-route-findings-on-main`.
No production code was changed and no test was committed.
The differential controls were a throwaway file,
`package/git-policy/cli/src/native/repo_read/audit_probe_tests.rs`,
deleted after the run,
with its output quoted where each result appears,
and the shell fixtures in `${HOME}/temp/agent/repo-read-index-audit`
(`stale-cache-tree.sh`,
`dot-component.sh`,
`diff-lib.c`,
`read-cache.c`,
`refs.c`,
`cache-tree.c`,
`unpack-trees.c`).
Git's own source is cited from the clone
`${HOME}/temp/agent/git-v2.56.0-2026-11-03`
(tag `v2.56.0`,
the release the gate image carries).
Scope:
`package/git-policy/cli/src/native/repo_read/` (61 files)
and `package/git-policy/cli/src/native/index/` (41 files),
about 22,600 lines with their test sidecars,
the SPEC sections "Module seams",
"Other processes changing files" and "Compatibility and degradation",
and the callers that act on the answers.
Linux only.
Purpose:
these two folders are the place where the wrapper reads and writes Git's own on-disk
data instead of asking Git.
Each reader is a claim that the wrapper understands a Git format as well as Git does,
so this page records,
format by format,
which shapes are answered and what happens on one that is not,
what a concurrent writer can be seen doing,
whether the bytes written differ from Git's,
and which answers feed a decision that changes or removes data.
Two findings came out,
both reproduced,
one of them a disagreement with Git's output and one a capability the route loses on
every repository that carries a scoped tag,
including this one.

## Where the route is entered

A read-only Git start happens in three places,
and each one tries the in-process answer first:

- `repo_read/repo_read_hook.rs:127`,
  `try_metadata`,
  the captured queries a policy or a commit reads.
- `repo_read/repo_read_hook.rs:151`,
  `try_complete`,
  the auto-push coordination's own environment.
- `repo_read/repo_read_hook.rs:178`,
  `try_request`,
  the transaction requests of the landing and the merge isolation.

All three lead to `repo_read/repo_read_query.rs:324`,
`answer_with`,
which parses the command line,
reads the layout and the configuration,
and dispatches on eleven command words
(`repo_read/repo_read_query.rs:355`).
Every reason to decline is the same `None`,
so the caller starts Git and nothing about the result changes.

Two guards hold around that:

- `repo_read/repo_read_hook.rs:73`,
  `guarded`,
  turns a panic inside a reader into one Git start.
  The release profile keeps `panic = "unwind"` for it.
  A control shows a panic costs the answer and not the command
  (`repo_read/repo_read_hook_tests.rs`,
  `a_panicking_reader_is_left_to_git`).
- `repo_read/repo_read_hook.rs:28`,
  the variable `CLI_GIT_IN_PROCESS_READS=off`
  (`:31`,
  `enabled`),
  which turns every answer off for a measurement or a diagnosis.

The answers are bounded by these limits,
each of which is a decline and not a truncation:

- 4096 bytes for one loose ref file or `HEAD` (`repo_read/repo_read_refs.rs:39`).
- 256 MiB for `packed-refs` (`repo_read/repo_read_refs.rs:53`).
- 5 symbolic hops (`repo_read/repo_read_refs.rs:67`),
  the same limit Git's `SYMREF_MAXDEPTH` sets.
- 1 GiB for one index file
  (`repo_read/repo_read_query_request_answer_and_context.rs:163`).
- 1 MiB for one configuration file
  (`repo_read/repo_read_config_text.rs`,
  `CONFIG_TEXT_LIMIT`).
- 10 nested includes
  (`repo_read/repo_read_config_list.rs`,
  `INCLUDE_DEPTH_LIMIT`),
  Git's own limit.
- 1024 environment-supplied configuration entries
  (`repo_read/repo_read_layout_environment_rules.rs:180`).
- 256 MiB for one object (`repo_read/repo_read_object.rs:34`),
  256 for tree nesting and 4,000,000 for one flattened tree
  (`repo_read/repo_read_tree.rs:31`,
  `:34`),
  100 for a delta chain (`repo_read/repo_read_pack.rs:30`),
  5 for alternates (`repo_read/repo_read_odb.rs`),
  64 MiB for the `packed-refs` scan behind a sequencer marker
  (`repo_read/sequencer_marker.rs`),
  and 8 for tag peeling (`repo_read/repo_read_query_refs.rs:38`).

Each of those is classed in
`doc/planning/cli-git-shipped-limits-audit.md`,
"Exempt limits",
as a reader fallback.

## The formats read natively, and what an unknown shape does

The one answer every reader gives to a shape it does not know is "start Git",
with these exceptions,
each named in the finding where it is wrong or in the list where it is right:

### `HEAD`, loose refs and `packed-refs`

Read by `repo_read/repo_read_refs.rs`.

- `HEAD` (`:234`,
  `lookup_head`):
  `ref: <name>` or one lowercase hexadecimal name of the repository's length,
  with one trailing line feed.
  A detached `HEAD` is a direct value.
  Anything else,
  a symlink,
  an oversized file or a link-shaped `HEAD` is "start Git".
- A loose ref (`:241`,
  `lookup`):
  the same two spellings,
  read only for a name of the plain alphabet (`:102`,
  `is_plain_ref_name`,
  since 2026-11-09 named `is_listable_ref_name` and widened to Git's own rule by
  `fix(git-policy/cli): the ref naming alphabet is Git's own refname rule`,
  commit `64e8bd5102`):
  `refs/` followed by components of ASCII letters,
  digits,
  `.`,
  `_` and `-`,
  none empty,
  none starting with `.`,
  none ending in `.lock`,
  no `..`.
  A loose file is fetched by path,
  so a file where a directory is expected is absence and a link is "start Git".
- `packed-refs` (`:192`,
  `packed_named`):
  an optional `#` header line first,
  then `<hex> <name>` lines and `^<hex>` peeled lines directly after a ref line.
  A name holding a control byte,
  a damaged object name or a peeled line out of place ends the read.
  A well-formed line whose name is outside the plain alphabet is skipped for a
  single lookup and ends the listing.
  A missing file is an empty file.
- A symbolic chain is followed at most five hops (`:278`,
  `resolve`),
  and a missing or unsure hop is "start Git".

What this costs:
`list` (`:294`) is all or nothing,
so one name outside the plain alphabet in either storage ends the whole listing.
Finding 2 measures the consequence.

### The repository location and the Git directory

Read by `repo_read/repo_read_layout.rs`.

- `.git` as a directory,
  or as a file holding `gitdir: <path>` (`:129`),
  the path relative to the worktree,
  with trailing carriage returns and line feeds stripped as Git strips them.
- `commondir` (`:295`),
  the same trailing-strip rule.
  A symlinked or unreadable one is "start Git".
- `reftable`,
  a `reftable` directory in either Git directory
  (`:301`),
  `extensions.refstorage` with any value other than `files` (`:201`),
  any extension key other than `noop`,
  `preciousobjects`,
  `objectformat` and `worktreeconfig` at format version 1,
  and a format version other than `0` or `1` all decline (`:171`,
  `format_of`).
- `HEAD` must parse as `ref: refs/...` or as a hexadecimal name of the right
  length (`:115`,
  `head_is_valid`).
- Every path the walk up passes must be owned by the running user,
  and the walk stops at a device boundary.
- `extensions.worktreeConfig` turns on the per-worktree `config.worktree`,
  which is read after the common file,
  and only if the common file names a repository format version.
- `core.bare` must not be true in any scope and `core.worktree` must be unset
  in every scope.

A linked worktree,
a bare repository,
a `GIT_DIR`-style environment,
a repository reached across a file system boundary and an unknown extension all
decline,
which the SPEC records as the design and not as a gap.

### The plain-text configuration

Read by `repo_read/repo_read_config_text.rs`,
merged by `repo_read/repo_read_config_list.rs`.

- Sections:
  `[section]`,
  `[section "subsection"]` with `\x` escapes,
  and the old `[section.sub]` spelling,
  section and name lowercased,
  the quoted subsection kept in case.
- Values:
  the escapes `\n`,
  `\t`,
  `\b`,
  `\\` and `\"`,
  a backslash before a line feed as a continuation,
  a pending run of white space kept only when more text follows,
  `#` and `;` comments outside quotes,
  a bare name as no value,
  and a byte-order mark at the start of the file.
  A NUL,
  an unknown escape or an open quote at the end of the file declines.
- `include.path` (`repo_read/repo_read_config_scan.rs:89`),
  absolute,
  relative to the including file,
  or under `~`,
  followed to Git's own nesting depth.
  `include.path` files Git would list but not honour are accounted for separately
  (`repo_read/repo_read_config_list.rs`,
  the `extensions.worktreeConfig` check).
- Every `includeIf.<condition>.path` declines (`:89`,
  `:141`),
  because Git evaluates the condition.
- `GIT_CONFIG_COUNT` with its `GIT_CONFIG_KEY_<n>` and `GIT_CONFIG_VALUE_<n>`
  pairs is listed under the `command` scope,
  with the key canonicalized (`repo_read/repo_read_config_list.rs`),
  and `GIT_CONFIG_PARAMETERS` declines.
- The listing is `scope\0key\nvalue\0` per entry
  in the order system,
  global,
  local,
  worktree,
  command,
  which is what `git config --list -z --show-scope` prints
  (reproduced below).

### The index

Read by `repo_read/repo_read_index.rs` for listings and diffs,
by `index/index_full.rs` for the whole file,
and written by `index/index_full_write.rs`.

- Header `DIRC`,
  versions 2,
  3 and 4,
  entries sorted by path and stage,
  the version 2 and 3 padding and the version 4 prefix varint.
- Only the four canonical modes (`index/index_full.rs:162`),
  only the intent-to-add and skip-worktree extended flags,
  and an extended flag only at version 3 or 4.
  Git canonicalizes other modes and refuses other bits,
  so both sides here are a decline.
- Extensions:
  `TREE` is read for the cache-tree.
  Every other signature must be uppercase and alphanumeric,
  and a lower-case one is a decline
  (`repo_read/repo_read_index.rs:184-185`),
  which is Git's rule read the other way round:
  Git ignores an unknown capitalised extension with a message and dies on an
  unknown lower-case one
  (`read-cache.c:1773-1781`).
  So `link` (a split index) and `sdir` (a sparse index) decline,
  and so does any unknown required extension.
- A split index and a sparse index are declined,
  and both are also built by their own controls
  (`repo_read/repo_read_query_tests.rs`,
  `a_real_sparse_index_is_declined`;
  the split index is measured in the reproduction below).
- The trailer checksum is not verified.
  That is Git's own behaviour outside `git fsck`:
  `verify_index_checksum` is "Allow fsck to force verification of the index
  checksum" (`read-cache.c:1710`),
  and an all-zero trailer is accepted outright (`:1735`),
  so a repository that once set `index.skipHash` reads the same on both sides.
- The index the update writes is Git's own shape:
  the version Git would pick,
  the entries,
  `TREE`,
  `REUC`,
  `UNTR` and `FSMN` in that order,
  and the checksum
  (`index/index_full_write.rs`).
  It is published by rename beside the file under `core.sharedRepository`'s mode
  (`index/index_reset_publish.rs`).
  The landing's own read of a private index does verify the checksum
  (`index/index_reset_file.rs:170`),
  because there a wrong trailer means the file Git is about to read is damaged.

### The index extensions

- `TREE` (the cache-tree):
  read,
  written back,
  and invalidated along a path exactly as `do_invalidate_path` does,
  ancestors included
  (`index/index_cache_tree.rs`,
  `invalidate`).
- `REUC` (resolve-undo):
  read,
  recorded into when an unmerged entry leaves the index,
  written in Git's order
  (`index/index_resolve_undo.rs`).
- `UNTR` (the untracked cache):
  read and written in Git's layout,
  preorder directories then the three bitmaps then the stat and name areas
  (`index/index_untracked_io.rs`).
- `FSMN` (fsmonitor):
  both the timestamp version and the text-token version are
  read,
  and the text-token version is written (`index/index_fsmonitor.rs`).
- `EOIE` and `IEOT` are skipped without being read,
  as Git skips them once it has used them,
  and a repository whose configuration asks for them still answers
  (reproduced below).
- The configuration that decides what an update may reproduce is read first
  (`index/index_settings.rs`):
  split index,
  sparse index,
  `index.skipHash`,
  the two offset tables,
  `index.threads`,
  a hook path,
  `core.worktree`,
  `core.sparseCheckout`,
  `core.ignoreCase` and `feature.manyFiles` each decline the in-process update,
  and `core.untrackedCache` and `core.fsmonitor` pick its behaviour.

### Objects, packs and the object database

Read by `repo_read/repo_read_odb.rs`,
`repo_read/repo_read_pack.rs` and `repo_read/repo_read_object.rs`.

- Loose objects are read as zlib streams under `objects/<2 hex>/<38 hex>`.
- A pack index is read only at version 2
  (`repo_read/repo_read_pack.rs:207`),
  with the large-offset table,
  and the pack header's version and count must agree with it.
  A version 1 index declines.
- A delta chain is followed to depth 100 by offset,
  and by name through the database.
- `objects/info/alternates` is followed to depth 5 and a line that is quoted or
  holds a backslash declines the whole database
  (`repo_read/repo_read_odb.rs:114`).
- Loose `refs/replace/*`,
  a `shallow` file,
  `info/grafts` and any ` refs/replace/` line in `packed-refs`
  decline the database (`:146`),
  because Git then answers about different objects than the names say.

### Sequencer markers

`repo_read/sequencer_marker.rs` proves that a marker names nothing,
which is the common answer,
without a Git process.
It looks at the six places Git looks (`lookup_paths`),
where any entry of any kind counts as "maybe",
and scans `packed-refs` for the marker's name.
Under reftable it answers "maybe" for every name (`:153`).

## Findings ranked by what they risk

### 1. An answer's standard error is not Git's, and an unknown optional index extension shows where

Fixed on 2026-11-09 (commit `421b692b9a`,
its control in `7ec3cca27c`):
the reader declines on every signature whose silence Git does not keep,
and still answers `TREE`,
`UNTR`,
`EOIE`,
`IEOT` and the `REUC` and `FSMN` bodies that read as Git reads them.

Amended on 2026-11-09 (commit `2c347b79c8`,
its controls in `cfaa62bb64`):
`IEOT` is not silent on any body.
Git reads the table only when `index.threads` turns threaded loading on and a valid `EOIE`
is present,
and the cross-vendor review's case holds.
Reproduced on Git 2.56.0:
an index written with `index.threads=4` and `index.recordOffsetTable=true`,
its `IEOT` version byte changed to 2 and the trailing checksum resealed,
makes Git print `error: invalid IEOT version 2` and answer,
where the reader answered with empty standard error.
Two further bodies Git words or dies on came out of the same reproduction:
a first block one byte off ends Git without a word,
and a first block pointing into the header prints
`fatal: unknown index entry format 0xc0110000`,
so the reader now answers an `IEOT` body only when it is version 1 and every block begins where
this reader parsed an entry and together they cover all of them.

`repo_read/repo_read_query.rs:324` returns an `Answer`
whose `stderr` is empty for every query except the two the readers word
themselves
(`repo_read/repo_read_query_refs.rs:180`,
`:294`).
Git writes one diagnostic line for a shape the readers answer:
an index carrying an optional extension the reader does not know.

Sequence,
in a fixture repository with one commit:

1.  Build an index with an unknown capitalised extension:
    take the index's bytes,
    append `ZZZZ`,
    a four-byte length and four bytes of content before the trailer,
    and recompute the trailer.
2.  Ask both sides for `ls-files --stage -z --full-name`.

The reproduction's own output,
`stdout` abbreviated to the entry names:

```text
--- unknown extension :: ["ls-files", "--stage", "-z", "--full-name"]
    git    code=Some(0) out="100644 5626abf0... 0\ta.txt\0100644 af9c6fd1... 0\td/b.txt\0100644 558ad609... 0\td/e/c.txt\0" err="ignoring ZZZZ extension\n"
    reader code=0 out="100644 5626abf0... 0\ta.txt\0100644 af9c6fd1... 0\td/b.txt\010644 558ad609... 0\td/e/c.txt\0" err=""
    DISAGREE
```

Wrong result:
the answer reports the same standard output and exit code as Git and a standard
error Git did not write.
The reader's own documentation states the claim narrowly,
"the same argument list the caller would give Git and returns Git's standard output and exit code"
(`repo_read/repo_read_query.rs`,
module doc),
so the code keeps its promise and the type does not:
`Answer` carries `stderr` beside them and every caller that shows or compares it
gets a silence where Git speaks.
No caller shows this line today,
because the one that formats Git's standard error does so only for a failed
command (`candidate/candidate_store.rs:273-279`),
which is why the finding ranks below the second one.
The extension itself is the forward-compatibility case the format is designed for:
a repository whose index was last written by a newer Git than 2.56.0 carries one,
and this repository raises its Git with the gate image.

Suggested fix (suggestion):
decline when the index carries an extension signature that is neither `TREE`,
`EOIE` nor `IEOT`,
since the reader then knows Git would have said something it does not reproduce
(`repo_read/repo_read_index.rs:184-190`).
The cost is one Git start for a repository with a future optional extension,
which is the cost the readers already pay for a required one.

### 2. One scoped tag removes the ref listing, and this repository carries them

Fixed on 2026-11-09 (commit `64e8bd5102`,
its controls in `16c6a1fab1`):
the predicate is `is_listable_ref_name` and follows Git's own rule.
Measured on this repository in the release profile,
the answer costs more than the Git start it replaces,
so the widening fixes what the reader can answer,
not what it saves.

Answered on 2026-11-09 (commits `6f8d629abe` for the object check,
`b2481ec2a4` for the limit):
the measured cost of the listing is now bounded instead of reduced to nothing.
The answer took 13.7 ms at the median against 3.8 ms for `git for-each-ref`,
and 10.8 ms of it was the object check;
asking the packs before the loose paths removed 6.6 ms of that,
leaving 7.1 ms for the answer on this repository.
Because the layout read and the ref walk already take 1.8 ms,
the two meet near 490 refs,
so `REF_LISTING_LIMIT` (256) declines a longer listing before the object database is opened
and Git answers it.
A repository past the limit pays the layout read (0.5 ms) and the count of `packed-refs`
before Git starts,
where the answer cost 9.9 ms more than that start.

`repo_read/repo_read_refs.rs:102` accepts a ref name only when every component is
made of ASCII letters,
digits,
`.`,
`_` and `-`.
Git's own rule (`refs.c`,
`check_refname_component`)
forbids the bytes `~`,
`^`,
`:`,
`?`,
`*`,
`[`,
`\`,
space and the control bytes,
the two-character sequences `..` and `@{`,
a leading `.` on a component and a trailing `.lock` on one,
and allows every other byte,
`@` and `+` included.

Sequence,
in a fixture repository with one commit and a tag named
`@scope/pkg@1.0.0`,
which is the spelling this repository's own release tags use:

1.  `git tag @scope/pkg@1.0.0`
    (measured in the reproduction below;
    the tag is in `packed-refs` after a
    `git pack-refs --all`).
2.  Ask both sides for
    `for-each-ref --format=%(refname)%00%(objectname)%00%(symref)`.

The reproduction's output:

```text
--- for-each-ref with outside-alphabet names :: ["for-each-ref", "--format=%(refname)%00%(objectname)%00%(symref)"]
    reader declined
```

and the same command without those tags answered,
byte for byte,
loose and packed:

```text
--- for-each-ref loose :: ["for-each-ref", "--format=%(refname)%00%(objectname)%00%(symref)"]
    git    code=Some(0) out="refs/heads/feat/nested/deep\0bba8a0e4...\0\nrefs/heads/main\0bba8a0e4...\0\nrefs/remotes/origin/HEAD\0bba8a0e4...\0refs/remotes/origin/main\nrefs/remotes/origin/main\0bba8a0e4...\0\nrefs/tags/annotated\0e477892bb...\0\nrefs/tags/light\0bba8a0e4...\0\n" err=""
    reader code=0 out="refs/heads/feat/nested/deep\0bba8a0e4...\0\nrefs/heads/main\0bba8a0e4...\0\nrefs/remotes/origin/HEAD\0bba8a0e4...\0refs/remotes/origin/main\nrefs/remotes/origin/main\0bba8a0e4...\0\nrefs/tags/annotated\0e477892bb...\0\nrefs/tags/light\0bba8a0e4...\0\n" err=""
--- for-each-ref packed :: ["for-each-ref", "--format=%(refname)%00%(objectname)%00%(symref)"]
    git    code=Some(0) out="...the same six refs..." err=""
    reader code=0 out="...the same six refs..." err=""
```

Wrong result:
on any repository that carries a scoped tag,
`for-each-ref` is never answered,
because `list()` (`repo_read/repo_read_refs.rs:294`) requires every packed and
loose name to be plain,
while `lookup()` (`:241`) already skips the unplain **packed** names for exactly this
reason
("a repository with scoped tags would otherwise have no branch the in-process
reader can resolve",
`:192`).
The listing is what `shadow/shadow_refs.rs:85` takes as the merge's ref snapshot
and what `lfs/lfs_object_plan.rs:172` takes as the set of refs to upload,
so a merge and an LFS-planned push each pay one Git process on such a
repository.
This repository is such a repository:
`rg --count 'refs/tags/@monochromatic-dev/' "$(git rev-parse --git-common-dir)/packed-refs"`
finds eight lines,
and no tag lives loose,
so `packed()` returns `None` and the listing declines on the main checkout too.
The answer is never wrong,
only absent,
and the alphabet is narrower than Git's for no reason the reader needs:
a name outside it could be skipped from the listing only by giving a wrong
answer,
which is why declining is right today.

Suggested fix (suggestion):
widen `is_plain_ref_name` to Git's `refname_disposition` rules
(`refs.c`),
which keeps the property the reader needs,
namely that it can prove the name is a valid ref name Git would also list,
and lets the listing answer in the repositories that carry scoped tags.
Adopted on 2026-11-09 as `is_listable_ref_name`
(commit `64e8bd5102`),
which declines on the bytes above ASCII and on the control bytes as well;
the measurement that came with it is in the report of that branch and shows the
answer is slower than the Git start on this repository.
The smaller change is to teach `list()` to skip a packed name that is not plain
the way `packed_named(true)` already does,
but that would answer with a listing Git does not print,
so it is the wrong shape.

## Reproductions

Each was a control in the throwaway file,
run with
`mise run //package/git-policy/cli:native:test:host -- --no-capture <filter>`.
The controls compare the reader's `stdout`,
`stderr` and exit code with real
Git's in the same fixture,
and print `DISAGREE` when any of the three differs.

### The shapes that agreed

- `config --list -z --show-scope`,
  byte for byte,
  for a configuration holding an `include.path` with its included file,
  a quoted value with escapes,
  an empty value,
  a bare name,
  a quoted subsection with a dot,
  the old `[section.sub]` spelling,
  a backslash continuation,
  a comment after a value,
  a mixed-case section,
  a carriage return at the end of a line,
  and a value with a trailing space.
- The same command for an `include` of a missing file (answered,
  matching),
  and
  for `includeIf` and for a value Git types but cannot read (declined).
- `for-each-ref` with a nested branch,
  a symbolic `refs/remotes/origin/HEAD`,
  an annotated and a lightweight tag,
  loose and packed and mixed.
- `rev-parse --verify --quiet <name>^{commit}` for a branch and for an annotated
  tag,
  `symbolic-ref --quiet HEAD`,
  and `rev-parse --abbrev-ref --symbolic-full-name @{upstream}` for a
  differently named upstream.
- `remote get-url <name>`,
  and the two failures the readers word themselves:
  `error: No such remote 'nowhere'` with exit 2,
  and `fatal: no upstream configured for branch 'main'` with exit 128,
  both byte for byte.
- `ls-files --stage -z` on an index at versions 2 and 4,
  with `EOIE` written,
  with a zeroed trailer,
  with a trailer that no longer matches its body,
  and with assume-unchanged and skip-worktree marks.
- A linked worktree (all five queries declined,
  which is the design).

### The forms left to Git

- A split index,
  measured to carry the `link` signature (`split index carries link: true`)
  and declined.
- A sparse index,
  in the existing control
  (`repo_read/repo_read_query_tests.rs`,
  `a_real_sparse_index_is_declined`),
  with and without the configuration keys that announce it.
- An unknown required (lower-case) extension,
  which Git dies on and the reader declines.
- A linked worktree,
  a reftable repository,
  a conditional include,
  a `packed-refs` or loose ref name outside the plain alphabet,
  a ref whose object is missing,
  and a repository whose configuration holds a key that moves the answer.

### The cache-tree, and why trusting it is Git's own behaviour

The readers do not read the directories a valid cache-tree proves equal
(`repo_read/repo_read_tree_cached.rs`,
`unequal_parts`):
a node counts only when its object name equals the tree's entry there and its
entry count equals the index entries below that directory.
That looked like a trust the reader places and Git does not,
so the control edited one entry of an index in place,
recomputed the trailer,
and asked both sides.
Git answered `0` for `diff-index --cached --quiet HEAD` and printed nothing for
`--name-only`,
while `ls-files --stage` showed the planted blob:

```text
    cache-tree probe: HEAD blob b90d918c6196dd2f1c4094089f36d30e5bbd0678, planted 417b483fa07367d6b57d8860d6acc5b69dc7d8de, at byte 124
    index now lists: "100644 5626abf0... 0\ta.txt\n100644 417b483f... 0\td/b.txt\n100644 558ad609... 0\td/e/c.txt"
    HEAD's blob:    b90d918c...
--- stale cache-tree, quiet :: ["diff-index", "--cached", "--quiet", "HEAD"]
    git    code=Some(0) out="" err=""
    reader code=0 out="" err=""
--- stale cache-tree, name-only :: ["diff-index", "--cached", "--name-only", "-z", "7d6dc129adcec054e3d05fe3d63353e1c31ae8ac"]
    git    code=Some(0) out="" err=""
    reader code=0 out="" err=""
```

Git's own `diff-index --cached` walks the trees through `unpack_trees`,
which skips a directory the cache-tree matches
(`cache-tree.c:945`,
`cache_tree_matches_traversal`,
reached from `unpack-trees.c`,
`all_trees_same_as_cache_tree`).
Git's test is weaker than the reader's in two places,
and in the safe direction:
it accepts a node whose entry count is greater than zero without checking it
against the index,
and it bounds the walk only by
"corrupted cache-tree has entries not present in index"
(`unpack-trees.c`,
`traverse_by_cache_tree`),
whereas the reader declines unless the count equals the index entries below.
So the reader answers exactly where Git does and leaves the rest to Git.
A stale cache-tree that Git trusts is a state the wrapper is not wrong about.

### A component that ends with a dot

`repo_read/repo_read_refs.rs:102` also refuses a whole name that ends with `.`.
Git's `check_refname_component` does not,
so a ref such as `refs/heads/v1./main` exists for Git:

```text
--- git for-each-ref sees:
refs/heads/main
refs/heads/v1./main
--- packed-refs with that line, read by Git:
refs/heads/main
refs/heads/v1./main
--- git status exit with that packed-refs:
status exit=0
```

The reader declines that listing,
which is the safe direction and the same shape as finding 2:
a valid Git name can end a reading the reader would otherwise answer.
It is not a wrong answer,
so it is here and not in the findings.

## What I checked and found sound

- The entry points and the switch:
  one call per runner,
  one panic guard,
  and a variable that turns the answers off for measurement
  (`repo_read/repo_read_hook.rs`).
- Every answered command line is checked by exact flags and operands
  (`repo_read/repo_read_query_request_answer_and_context.rs`,
  `flags_are`),
  so a flag the wrapper never issues starts Git instead of being half understood.
- A content command additionally requires that no configuration file and no
  environment entry sets a key that changes its output
  (`repo_read/repo_read_query.rs:89`,
  `repo_read/repo_read_config_scan.rs`,
  `content_key`).
  A configuration value Git types but the reader cannot prove declines the
  layout (`repo_read/repo_read_config_typed.rs`),
  except for the callers that follow a command that already succeeded.
- The configuration listing matches Git byte for byte on every shape listed in
  "The shapes that agreed",
  including the scope names and the order.
- The two messages the readers word (`No such remote`,
  `no upstream configured`) match Git's text and exit codes,
  and are answered only when the locale is English
  (`repo_read/repo_read_query_refs.rs:112`,
  `messages_are_english`).
- `for-each-ref` matches Git for loose,
  packed and mixed storage,
  including `%(symref)` for a symbolic ref and `%(objectname)` for an annotated
  tag.
  A name outside the plain alphabet and a ref with a missing object decline.
- `rev-parse --verify --quiet <name>^{commit}` peels tags to a commit or declines,
  and `symbolic-ref --quiet HEAD` answers the branch or the silent failure a
  detached `HEAD` gives.
- `@{upstream}` follows the fetch refspec,
  refuses a remote of `.`,
  refuses more than one `branch.<name>.merge`,
  and checks every name Git's short-name rules could prefer
  (`repo_read/repo_read_query_refs.rs:238`,
  `shadowed`).
- The layout refuses reftable,
  a bare repository,
  a linked worktree,
  an unknown extension and any `GIT_*` variable that moves discovery,
  which is the list in
  `repo_read/repo_read_layout_environment_rules.rs`.
- The index reader and writer agree with Git on the shapes the reproductions
  cover,
  and the writer's output is proved equal to Git's own bytes for the private
  index the landing writes:
  round trips of files real Git wrote
  (`index/index_full_tests.rs`,
  versions 2 and 4 with `TREE` and `UNTR`),
  and a differential against `git reset` over the same scene with `TREE`,
  `REUC`,
  `UNTR` and `FSMN` present,
  a configuration that removes the untracked cache,
  and a SHA-256 repository
  (`index/index_reset_file_tests.rs`,
  `kept_extensions_equal_git`,
  `a_configuration_that_removes_the_untracked_cache_equals_git`,
  `a_sha256_repository_equals_git`).
- The index checksum is not verified on the read paths,
  which is Git's behaviour outside `git fsck`
  (`read-cache.c:1729`),
  and the one path that does verify it,
  the landing's private index,
  declines a
  wrong trailer (`index/index_reset_file.rs:170`).
- The cache-tree invalidation is `do_invalidate_path`'s,
  ancestors included,
  so the index the wrapper writes never hands Git a tree name that contradicts
  its entries (`index/index_reset.rs`,
  `remove` and `put`,
  `index/index_cache_tree.rs`).
- `REUC`,
  `UNTR`,
  `EWAH` and `FSMN` are read and written in Git's layouts,
  `FSMN` in both versions it has had,
  and the reference to Git's own functions is in each module's doc.
- The object database declines replace refs,
  a shallow file,
  grafts,
  quoted alternates and a version 1 pack index,
  and answers from loose objects and version 2 packs otherwise.
- A concurrent writer cannot be read half done for the two formats where it
  matters,
  because both sides replace by rename:
  Git writes a loose ref through `refs/**.lock` and `packed-refs` through
  `packed-refs.lock`,
  and both orders Git uses leave the reader's preference correct
  (a loose file wins over a packed line;
  Git commits the loose update before it deletes any packed version,
  "Perform updates first so live commits remain referenced",
  `refs/files-backend.c:3345-3430`,
  and `pack-refs` writes the packed file before it prunes the loose ones,
  `refs/files-backend.c:1565-1573`).
  The one window that remains is the listing's own:
  a `pack-refs` that runs between the `packed-refs` read and the directory walk
  can hide a ref from `for-each-ref`,
  which is the race the owner's decision of 2026-11-03T00:15:33-05:00 leaves
  outside the wrapper's duty
  (`package/git-policy/cli/SPEC.md`,
  "Other processes changing files").
- The sequencer-marker proof cannot say "absent" when a marker might exist:
  it treats any entry of any kind and any occurrence of the name in
  `packed-refs` as "maybe",
  and reftable as "maybe" for every name.

## What I could not determine

- Whether finding 1's silence has a caller today.
  The crate's `Answer` carries standard error and the query docs promise only
  standard output and the exit code,
  so the fix is cheap either way,
  but I did not trace every caller that could print it.
- Whether any of the answered queries is asked in a repository whose index was
  written by a Git newer than 2.56.0.
  The gate image's Git is the only one the tests use,
  so the shape finding 1 measures is built by hand.
- How often the listing in finding 2 matters in practice:
  the two callers I found pay one Git process each,
  and I did not measure what that process costs on the loaded host,
  which would be a timing claim this audit does not need.
- Whether the Windows and macOS halves of these modules behave the same.
  The owner's pause ("Ignore Win and mac until I say so") covers them,
  every reader here is `#[cfg(unix)]`,
  and the two `#[cfg(not(unix))]` entry points return "start Git".

## What to inspect and how to respond

Inspect,
in this order:

1.  finding 1 against
    `repo_read/repo_read_index.rs:184-190`
    and `read-cache.c:1773-1781` in
    `${HOME}/temp/agent/git-v2.56.0-2026-11-03`,
    then decide whether the answer should decline or word the line itself;
2.  finding 2 against
    `repo_read/repo_read_refs.rs:102`,
    `:192` and `:294`,
    with `shadow/shadow_refs.rs:85` and `lfs/lfs_object_plan.rs:172` as the
    callers that pay for it,
    and the `rg --count` over the common directory's `packed-refs` as the
    measurement that this repository is affected;
3.  the reproduction scripts in
    `${HOME}/temp/agent/repo-read-index-audit`
    (`stale-cache-tree.sh`,
    `dot-component.sh`),
    which rebuild the two index and ref shapes without the throwaway test.

Respond with:
whether finding 1 should be fixed now or recorded as an accepted limit,
and whether widening the ref-name alphabet to Git's own rules is wanted,
which would let the listing answer on this repository.
The audit changed no production code,
so no other action is pending.
