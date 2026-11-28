# What the wrapper's `0.6.1-alpha.48` release passes found

Round of 2026-11-27,
unapproved and non-final.

Two passes read the release `0.6.1-alpha.48`:
a behaviour pass,
which exercised alpha.47 and alpha.48 side by side,
and a splits pass,
which read what the 344 refactor commits of the range did.
This record keeps the findings and the measurements that no other file holds,
in the passes' own words where they had them.
Every figure is the pass's own measurement,
taken on 2026-11-27;
nothing here was re-measured for this record.

The behaviour the release rests on is the code,
the gate,
and the service-manager install proof named under
"No change of behaviour is not demonstrated" below.

## The behaviour pass found no High finding

Neither pass raised a High finding.
The behaviour pass's two Mediums and six Lows were all fixed by rounds merged on 2026-11-27,
each with a control that fails without the fix;
what each said is history,
and the controls pin the fixed words.

The splits pass's Mediums and Lows are below.

## The behaviour pass: what held

These held,
exercised with both binaries:

- the slot refusals,
  for the spellings they name;
- the merge pass's union of deletions;
- the self-check's memory,
  62.7 to 66.1 MB in alpha.47 and 9.3 to 11.0 MB in alpha.48;
- named pipes,
  answered and not waited on;
- the typed ssh relay at a terminal,
  which ended about 100 ms sooner.

## The behaviour pass: read and not run

Three surfaces were read only,
because nothing outside the process can start them:
the prune race,
the conversion environment's precedence,
and the service-only changes.

## The splits pass: no change of behaviour is not demonstrated

Its verdict is that "no change of behaviour" is not demonstrated.
Of the items compared at both ends,
97.9 percent have identical text,
no test-only switch is in either release binary,
no static became two and no trait impl was lost;
but 318 of the 344 refactor commits carry changed lines that are not moves,
and the pass read three of them in full.

Neither pass ran `git commit` in a release binary,
because without a session bus every commit form stops at one refusal.
So for the service paths the evidence is the reading,
the gate,
and the service-manager install proof,
which issue 6587's second round ran to its end (`fdd9ce0e83`):
alpha.48 installed over alpha.47 in a container of the gate image,
and the proof printed

```text
THE PROOF PASSED: the instance the older release loaded started as this release, and the pair of commits each landed once
```

## The splits pass: reach

At least 31 definitions became nameable from outside the crate,
seven of them private helpers a split made bare `pub`,
one of which removes directories.
The list of published names is
`src/native/source_guard/items_nameable_from_outside_release.txt`,
and its control is
`src/native/source_guard/nothing_becomes_nameable_from_outside_tests.rs`,
which fails on a published name that has no line there.

## The splits pass: refactor commits read and not read

The extract-function refactors are unread but for ten commits.
A second read of the seven the pass named first,
all on the client's and the service's commit paths,
found each equivalent by reading
(`e90482f100`,
`dde5c4be3b`,
`bad45d0420`,
`01c2f32e2c`,
`6a24c6342b`,
`f4e8d2fb9b`,
`8af8f56255`).
One guard covers less than its name says:
`src/native/source_guard/hook_self_invocation_tests.rs` reads 263 of the 699 code lines
the commit preparation spans,
which the list-guards round took first.
About three hundred refactor commits stay unread.

## The splits pass: guards that name moved files

The pass found 71 pairs of a source guard and a file it names,
where code left the file in a split and the guard does not name where it went.
Two were checked,
and neither was a hole.
The list-guards round holds the pairs now.

## The splits pass: low findings

Eight points,
as the pass put them:

- the sweep's warnings carry four new log tags;
- a commit labelled `docs` changed the `shutdown-unmount` help row
  (`066e33a47d`,
  noted on the commit);
- the panic notice's source locations moved;
- 34 items lost a platform gate on split-off files,
  with no effect on x86_64 Linux;
- tag constants are copied into more files;
- the README's allow-list counts are stale;
- two commits in the range cannot build alone;
- the linter the binary carries moved four alphas.

The platform gates are fixed:
a split-off file keeps the platform gate of the file it left (`4d6f09f1f2`).

## The splits pass: a split that was not a pure move

Not every split is a pure move.
The landing split's last commit (`3b9ada357b`) boxed the commit inside the service's answers
and made the commit command pass its own copy of its client configuration,
both for Clippy after the move,
which the release pass that reads the range reads as a change.

## The read of `0.6.1-alpha.52`: the surface control and three source guards

Round of 2026-11-28,
unapproved and non-final.
The sections above are the `0.6.1-alpha.48` passes;
this one is the read of `0.6.1-alpha.52`
that the session then weighed,
and the rounds that answered two of its Low findings.
Only the owner declares it complete.

The read is a second pass by another agent of the same vendor as some of the authors,
so it is no independent review.
Its limits are the ones it states:
it did not build the crate,
it ran the reader functions copied byte for byte in two tiny `rustc` programs,
and it searched for each shape in the tree rather than planting one.

### What the read found on the surface control

`nameable_items_of_a_source_tree_tests.rs` compared the whole of an attribute with the text
`#[macro_export]`,
took every line starting with `#[` for an attribute,
and read an `impl`'s type from the `impl` line alone.
So it answered nothing for three shapes,
none of them in the crate on 2026-11-28,
and its own doc,
which says what the reading cannot see,
named none of them:

- `#[macro_export(local_inner_macros)] macro_rules! name`;
- an attribute and a `pub` item on one line, `#[inline] pub fn f() {}`;
- an `impl<` whose generics run over several lines, which is the header a long generic list is
  formatted with.

Each shape published a name with no line in `items_nameable_from_outside_release.txt`,
which is what that list exists to stop.
The read's suggestion was to plant each shape in a fixture and see the control red,
or to list the three in the reader's doc.

### What was planted

Commit `359aaf32e1` adds one control beside `surface_shapes_a_split_writes_tests.rs` per shape,
each writing a fixture tree and asking for the name the shape publishes.
Threaded,
with the guide's flags,
they failed on that commit:

```text
a_macro_export_with_parentheses_is_a_name
  left: ["outer macro_rules plain"]
 right: ["outer macro_rules local", "outer macro_rules plain"]
an_item_on_its_attributes_line_is_a_name
  left: ["outer fn plain"]
 right: ["outer fn inlined", "outer fn plain", "outer macro_rules exported", "outer struct Held"]
an_impl_whose_generics_run_over_several_lines_publishes_its_members
  left: ["outer field Published::held", "outer struct Published"]
 right: ["outer field Published::held", "outer fn Published::seen", "outer struct Published"]
```

`test result: FAILED. 18 passed; 3 failed`.

### What the reading reads now

Commit `819bc743ee` reads all three, so the reader's doc lists exactly the shapes it still
cannot see and none of these.
An attribute on the declaration's line is split from it by `leading_attributes`,
which counts square brackets and skips string literals;
`is_a_macro_export` reads the attribute with or without its parentheses;
and an `impl` whose own generics run over several lines is held as one header text until its
`{` arrives, and the type is read from all of it.
`nothing_becomes_nameable_from_outside_tests` compares the whole real surface against the list
green, so the widening published nothing and the list needs no new line.

The same split moved into `source_scan_declaration_tests.rs` in commit `57d381c4cc`,
because that file owns the attributes a declaration line carries.
It found a hole of the same shape one layer down:
`#[cfg(test)] pub mod support;` was swallowed as an attribute,
so the module and every name it publishes were outside the module tree,
and outside every guard whose scope that tree decides,
with nothing said.
The control in `source_scan_checks_tests.rs` was shown red without the split:
`left` held no `gated_on_its_attribute_line` and no `renamed_on_its_attribute_line`,
`right` held both,
and the same run failed `declarations_refuse_what_they_cannot_place` on a `mod` visibility the
reading does not place, written on its attribute's line.

### What the read found on the three guards

The read planted each spelling by running the guards' own reading functions over the release
sources,
and found the guards green for six spellings:

- the link guard, `hook_self_invocation_tests.rs`, let `std::fs::soft_link` through:
  the deprecated standard-library spelling of a symlink was in no list of link names;
- the note guard, `the_commit_path_writes_its_notes_off_its_thread_tests.rs`, let `eprintln!`,
  a file opened at `/dev/stderr`, and a write to `/dev/fd/2` through:
  it read the call, the type, the rename and the descriptor constant, and no macro or path;
- the login guard, `no_start_at_login_tests.rs`, let a `Command` imported under another name
  (`use std::process::Command as Cmd;` then `Cmd::new("systemctl")`) and a `Command::new(`
  whose literal stands on the next line through:
  it read one line at a time, for the text `Command::new`.

### The green that had to be red

Planted in covered release files of the tree,
in a scope,
with the switches the guide's flags pass,
each guard's own cell stayed green:

```text
hook_self_invocation_tests                           test result: ok. 8 passed; 0 failed
the_commit_path_writes_its_notes_off_its_thread_tests test result: ok. 6 passed; 0 failed
no_start_at_login_tests                              test result: ok. 10 passed; 0 failed
```

The plants were a `std::fs::soft_link(..)` call in `commit/commit_command.rs`,
an `eprintln!`, a `File::create("/dev/stderr")` and a `std::fs::write("/dev/fd/2", ..)`
in `commit/commit_command_words.rs`,
and the rename plus the next-line literal in `service/service_instance.rs`.
They were reverted and never committed.

### What the guards read now

Commit `342d3d21fd` makes each guard read its spellings,
and each guard's own self-test plants them:

- the link guard reads `soft_link` as one more name of a link,
  where no longer name touches it;
- the note guard reads `eprintln!` and `eprint!` as the terse ways to write standard error,
  and `/dev/stderr`, `/dev/fd/2` and `/proc/self/fd/2` as the paths that open the same handle;
- the login guard refuses the import that renames the type of a start,
  and reads a start whose literal stands on the code line after its `(`.

The same plants, in the same files, are findings afterwards:

```text
commit/commit_command.rs:109 (soft_link) let _ = std::fs::soft_link("planted-source", "planted-target");
commit/commit_command_words.rs:69 (eprintln!) eprintln!("planted");
commit/commit_command_words.rs:70 (/dev/stderr) let _ = std::fs::File::create("/dev/stderr");
commit/commit_command_words.rs:71 (/dev/fd/2) let _ = std::fs::write("/dev/fd/2", b"planted");
service/service_instance.rs:118: Command as
service/service_instance.rs:120: a start of the manager's own program: let _ = std::process::Command::new( "systemctl",
```

The last two are quoted with their runs of spaces closed up, since the finding
prints the line as the file writes it, indentation included,
and the start's is the call's line joined with the literal's line.
The `Command as` finding's banned text ends with the space between the two words.

`test result: FAILED. 7 passed; 1 failed`, `5 passed; 1 failed` and `11 passed; 1 failed`.
With the plants reverted the three modules answer
`ok. 8 passed`, `ok. 6 passed` and `ok. 12 passed`.

### Which other rule refuses a spelling

Only `eprintln!` is refused elsewhere:
`source_guard/diagnostic_door_tests.rs` fails any release file whose code carries `eprintln!(`,
because `eprintln!` is the one terse way to say a diagnostic without reaching the door.
The note guard's top doc says so and reads it in the note lane besides,
so a note file that spells it fails there too.

Nothing else refuses any of the six.
`clippy.toml` bans `std::env::temp_dir`, three `tempfile` constructors and `Result::unwrap`;
the crate's `[lints.clippy]` turns on `disallowed_methods`, `implicit_return`, `shadow_reuse`,
`shadow_same` and `shadow_unrelated` and no group,
so `clippy::print_stderr` stays as allow-by-default as it ships;
and the repository linter's Rust rules are
`avoidable-closure`,
`file-order-in-folder`,
`folder-uses-links`,
`max-lines`,
`no-anonymous-functions`,
`require-explicit-types`,
`require-rustdoc` and
`tests-in-sidecars`,
none of which names a link, a stream, a path or a start.

### What is still unread

The reader's own top doc keeps the list,
and each line of it is still a shape no line covers:

- a `pub` field in a tuple-like struct's parentheses;
- a member of a type a facade names through an inline `mod` body or a module the tree cannot
  resolve;
- a `pub trait`'s methods and a `pub enum`'s variants;
- an item inside an inline `mod name { ... }`;
- the names a glob carries out of a module no `pub mod` chain reaches
  (the guide said "a glob's own names", which is broader than the reading, and now says what the
  reading does);
- the second path a re-export adds when it keeps the name it re-exports;
- a signature or a value that changes under a name that stays.

The guards' own top docs keep theirs:
a call a macro of another file expands to,
a name a glob import brings into scope,
a call whose name and `(` a line break separates,
a start whose program is a value rather than a literal,
a `member:` whose literal a line break separates from its key,
a write to the plain descriptor number (`libc::write(2, ..)`),
a path built from parts or held in a variable,
and, for the login guard, a literal further than the next code line from its call's `(`.

The guide's section on the published surface and its section on a guard's scope, excuses and
anchor now say the same things as those docs.
