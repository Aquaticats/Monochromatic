# Git 2.56.0 rejects `ls-tree --recursive` before inventory collection

## Symptom

On 2026-10-18,
the research inventory command used `git ls-tree --recursive --name-only`.
Git 2.56.0 exited 129 with this exact diagnostic:

```text
error: unknown option `recursive'
```

That failed invocation supplied no path inventory or absence evidence.
The initial assumption that recursion had a spelled-out long option was wrong.
`git ls-tree -h` lists `-r` without a corresponding long spelling.

## Root cause

The same-release source tag `v2.56.0` resolves to
`a018953688f1b10bddf91bff8747068f5f4746a4`
in the read-only `git/git` clone.
In `builtin/ls-tree.c:354`,
the recursion option has a short name and a null long name:

```c
// builtin/ls-tree.c:354
OPT_BIT('r', NULL, &options.ls_options, N_("recurse into subtrees"),
        LS_RECURSIVE),
```

`parse-options.h:89` documents what null means:

```c
// parse-options.h:89
 * `long_name`::
 *   the long option (without the leading dashes) or subcommand name,
 *   NULL if none.
```

`builtin/ls-tree.c:383` passes that option array to the parser:

```c
// builtin/ls-tree.c:383
argc = parse_options(argc, argv, prefix, ls_tree_options,
                     ls_tree_usage, 0);
```

`Documentation/git-ls-tree.adoc:44` documents `-r` as recursion into subtrees.
The observed failure matches the supported option surface;
it is not a Git defect or a reason to treat the source tree as empty.

## Verification and verified handling

The bounded read-only controls run through the private research controller's
`verify-git-ls-tree-recursion-options` Mise task.
Both commands target immutable repository source
`e02be6c8b8fe9efb41a7f8f1a0b311c17796588a`:

```sh
# From the repository root; Git's recursion flag has no long form.
git ls-tree -r --name-only e02be6c8b8fe9efb41a7f8f1a0b311c17796588a
git ls-tree --recursive --name-only e02be6c8b8fe9efb41a7f8f1a0b311c17796588a
```

The working catalog contains `-r --name-only`:
exit zero,
13,860 paths and the known nested owner `package/linter/kotlin/build.gradle.kts`.
The failing catalog contains `--recursive --name-only`:
exit 129 and the exact diagnostic in Symptom.

Use `-r` while retaining long forms for options that actually have them.
This changes only the invalid spelling;
it preserves recursive Git-tree enumeration.
The inventory still concerns tracked Git paths,
not ignored files,
LFS-materialized bytes or every possible manifest name.
No source patch or alias is required.

## What does not work

Replacing every short flag mechanically with an imagined long spelling fails before collection.
Treating the resulting empty output as a successful negative search would invalidate applicability claims.
Plain nonrecursive `ls-tree --name-only` does not exercise nested owner discovery.

## Upstream filing decision

The `.out-of-scope/` inventory contains no Git-specific exemption.
The GitHub mirror search for `"ls-tree" "recursive"`,
including issues and pull requests,
returned no result.
That is not a claim about the separate Git mailing-list archive.

1.  Fault:
    no upstream fault is established;
    the command used an unsupported option.
2.  Fixability:
    adding an alias could be a feature request,
    but it is unnecessary for the intended inventory.
3.  Supported use:
    recursive enumeration is explicitly supported as `-r`.
4.  Contribution policy:
    `.github/CONTRIBUTING.md` and `.github/PULL_REQUEST_TEMPLATE.md`
    direct contributions and bug reports to Git's mailing list,
    with GitGitGadget as an optional submission bridge.
5.  Maintainer disposition:
    no maintainer response about adding this alias was collected;
    the documented spelling already satisfies this task.
6.  Prototype:
    not applicable because the upstream-fault premise fails.

There is nothing to file or add upstream.
No third-party source was modified and no external message was sent.
