# pnpm store link for a `link:` override dangles and `pnpm install` will not repair it

Observed on 2026-11-10 in this repository's `.claude/worktrees` checkout,
Node v26.13.0,
`pnpm --version` reporting `pnpm-shadow 0.1.3-alpha.1` over pnpm 12.18.0,
store root `/home/user/.local/share/pnpm/store/v11`.

This record states only what was probed.
Which program writes the bad link is not established:
the pnpm in `PATH` here is the repository toolchain's shadow build,
so the writer may be that shadow rather than upstream pnpm.

## Symptom

Loading any module of `@earendil-works/pi-coding-agent` from a worktree fails:

```text
Error [ERR_MODULE_NOT_FOUND]: Cannot find package 'proper-lockfile' imported from
/var/home/user/.local/share/pnpm/store/v11/links/@earendil-works/pi-coding-agent/1.5.0/<hash>/node_modules/@earendil-works/pi-coding-agent/dist/core/settings-manager.js
```

`proper-lockfile` is the dependency `pnpm-workspace.yaml` substitutes through a
`link:` override:

```yaml
# pnpm-workspace.yaml
overrides:
  'proper-lockfile': 'link:package/pnpm-override/shim-proper-lockfile'
```

## Mechanism

Inside the resolving store link directory,
`node_modules/proper-lockfile` is a relative symlink whose target assumed the
logical `$HOME` (`/home/user` is a symlink to `/var/home/user` here).
Resolving it from the real parent path yields a path that does not exist:

```sh
# Resolving from the real parent overshoots the filesystem root.
node --input-type=module -e "const fs=await import('node:fs'); const path=await import('node:path'); \
const l='<store-link-dir>/node_modules/proper-lockfile'; \
console.log(path.resolve(fs.realpathSync(path.dirname(l)), fs.readlinkSync(l)))"
# /var/var/mnt/encrypted/repository/Monochromatic/.claude/worktrees/<worktree>/package/pnpm-override/shim-proper-lockfile
```

Node resolves a package symlink against the real path,
so the module never loads.
The same `@earendil-works/pi-coding-agent` version resolved from the main
checkout's store link (`1.5.0/6997a68d...`) was intact,
which is why the breakage appeared only inside the worktree.

`pnpm install --offline --frozen-lockfile` does not repair it:

```text
Lockfile is up to date, resolution step is skipped
Already up to date
```

## Repair

Repoint that single store entry at a checkout that outlives the worktree,
then re-probe resolution:

```sh
ln --symbolic --force --no-dereference \
  /var/mnt/encrypted/repository/Monochromatic/package/pnpm-override/shim-proper-lockfile \
  /home/user/.local/share/pnpm/store/v11/links/@earendil-works/pi-coding-agent/1.5.0/<hash>/node_modules/proper-lockfile

node --input-type=module -e "import { createRequire } from 'node:module'; \
const r = createRequire('/home/user/.local/share/pnpm/store/v11/links/@earendil-works/pi-coding-agent/1.5.0/<hash>/node_modules/@earendil-works/pi-coding-agent/dist/core/settings-manager.js'); \
console.log(r.resolve('proper-lockfile'))"
```

## Limits

- The store link directory is machine-global state,
  not repository state,
  so the repair is outside version control and another install may rewrite it.
- The `<hash>` segment depends on the resolved dependency set;
  a lockfile change produces a different directory that has to be re-probed on
  its own.
- Pointing the link into a worktree makes it dangle when that worktree is removed,
  so prefer a path that survives,
  such as the main checkout.
