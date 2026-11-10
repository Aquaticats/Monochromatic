# pnpm store links: a workspace dependency symlink resolves 14 levels up where 15 are needed, so `node` reports the package as missing

Tool:
pnpm 11 store (`~/.local/share/pnpm/store/v11/links`),
observed 2026-11-09.
Surface:
any `node` invocation that imports a dependency whose store variant links
to a workspace package.
Failure mode:
`ERR_MODULE_NOT_FOUND` naming a package that is installed,
with a `readlink -f` of the store's `node_modules` entry resolving to nothing.

## Symptom

A script that imports through the MDX pipeline dies before running a line of its
own code:

```text
Error [ERR_MODULE_NOT_FOUND]: Cannot find package '@ungap/structured-clone' imported from
/var/home/user/.local/share/pnpm/store/v11/links/@/mdast-util-to-hast/13.2.1/731ac1605df95da2c6460400a76f8ee3deb27ff37053d6ffe561f13d59708ba4/node_modules/mdast-util-to-hast/lib/footer.js
```

The package is present as a symlink in exactly the `node_modules` directory node
searched,
so the report reads as a missing install when the link is what is
broken.
`mise exec -- pnpm install --offline --frozen-lockfile` completes with
exit 0 and does not repair it.

The same failure recurs per dependency:
repairing the `@ungap/structured-clone`
link moved the error to `rehype-autolink-headings`,
whose variant carries the
same defect.

## Root cause

Measured,
not traced to pnpm source (see "Source trace status"):

- The store entry
  `links/@/mdast-util-to-hast/13.2.1/731ac1605df95da2c6460400a76f8ee3deb27ff37053d6ffe561f13d59708ba4/node_modules/@ungap/structured-clone`
  is a symlink whose target begins with 14 `../` segments followed by
  `var/mnt/encrypted/repository/.../package/pnpm-override/shim-ungap-structured-clone`.
- The symlink lives in a directory 15 levels deep (`/var/home/user/.local/share/pnpm/store/v11/links/@/mdast-util-to-hast/13.2.1/<hash>/node_modules/@ungap`),
  so 14 segments land in `/var/home/user`,
  and the target resolves to
  `/var/home/user/var/mnt/...`,
  which does not exist.
- With 15 segments the link resolves,
  and `node` imports the workspace shim.

`readlink -f` on the entry prints nothing,
`ls` on it reports `No such file or directory`,
and `find . -maxdepth 6 -xtype l` under
`~/.local/share/pnpm/store/v11/links` lists 1801 dangling links,
so this is one
class across many variants,
not a single bad entry.

The workspace packages involved are the `package/pnpm-override/shim-*` and other
workspace-linked dependencies that pnpm materialises inside a store variant
directory rather than in the consuming package's own `node_modules`.

## Verification

Observed 2026-11-09 on this repository's `package/ssg/aquati.cat`:

- `mise run //package/ssg/aquati.cat:build:site` fails with the error above
  before writing any page.
- `ls -la <store>/links/@/mdast-util-to-hast/13.2.1/731ac.../node_modules/@ungap/`
  lists `structured-clone -> ../../...`,
  and `readlink -f` of it yields the empty
  string.
- Counting the leading `..` segments with
  `readlink "$LINK" | tr '/' '\n' | rg -c '\.\.'` returns 14.
- The count that resolves:
  15,
  verified by recreating the link with 15 segments
  and reading `readlink -f` back to the existing
  `package/pnpm-override/shim-ungap-structured-clone` directory in this worktree.
- Workspaces that never touch the affected dependency graph,
  such as this
  package's unit tests and typecheck,
  run normally,
  which is why the breakage
  surfaces only in the site build.

## Verified workarounds

Recreate the one link with a target that resolves,
and record that it is a
hand-made link:

```bash
LINK=<store>/links/@/<pkg>/<version>/<variant>/node_modules/<dep-name>
TARGET=$(printf '../%.0s' $(seq 1 15))var/mnt/encrypted/repository/<repo>/package/<workspace-package>
ln --symbolic --force --no-dereference "$TARGET" "$LINK"
readlink -f "$LINK"   # must print the workspace package directory
```

Tradeoffs:

- It repairs exactly one link.
  Every other variant that links a workspace
  package keeps failing,
  and the site build may hit the next one (it hit
  `rehype-autolink-headings` immediately after `mdast-util-to-hast`).
- The target embeds this worktree's path,
  so a different worktree needs a
  different target;
  a later `pnpm install` in any worktree may rewrite the link
  and reintroduce the off-by-one.
- It edits the shared pnpm store,
  which every session and worktree reads.
- It does not change this repository's files,
  so nothing records the repair:
  the store is machine state,
  and the next agent inherits either a witness or
  the same failure.

Removal condition:
a `pnpm install` in this repository that leaves
`find ~/.local/share/pnpm/store/v11/links -maxdepth 6 -xtype l` empty makes the
workaround obsolete;
delete nothing else,
and re-run that `find` after any
install that reports success but leaves a build failing on `ERR_MODULE_NOT_FOUND`.

## What does not work

- `mise exec -- pnpm install --offline --frozen-lockfile`,
  run from the worktree:
  exits 0,
  leaves the dangling link untouched.
- Re-running the failing build after that install:
  same error,
  same link.
- Repairing only the first reported dependency:
  moves the failure to the next
  store variant with the same defect instead of fixing the build.

## Source trace status

The depth arithmetic and the failing resolution are measured;
the pnpm code that
computes the relative target has not been read,
so this doc makes no claim about
which function miscounts,
and no upstream filing artifact is prepared:
the
6-constraint filing check cannot pass its source-trace requirement without that
reading.
A future session that wants to file upstream should clone pnpm,
find the
relative-symlink computation behind `links/@/<pkg>/<version>/<variant>/
node_modules/`,
cite it with a commit-pinned line range,
and record the
duplicate search here before drafting anything.
