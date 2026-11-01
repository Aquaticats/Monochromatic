# rpm-ostree v2026.2: install --apply-live stages Chromium but refuses a live diff containing removals

## Symptom

On Bazzite 44,
this command resolves and composes a deployment,
prints `Staging deployment... done`,
and then exits with an error:

```sh
rpm-ostree install chromium --apply-live
```

```text
error: packages would be removed: 3, allow replacement to override
```

Trying to put the suggested option on `install` fails during argument parsing:

```sh
rpm-ostree install chromium --apply-live --allow-replacement
```

```text
error: Unknown option --allow-replacement
```

The observed transcript did not include `rpm-ostree --version`.
The implementation trace in this document uses upstream rpm-ostree v2026.2,
commit `d5ef5f0cb5ace53f40a8d92933cc87e69ad8c7f4`.

## Root cause

### The interactive command deliberately resolves twice

`src/app/rpmostree-pkg-builtins.cxx:199-225` first performs a dry-run transaction when
`--apply-live` is used interactively without `--assumeyes`,
then asks for confirmation and runs the real transaction:

```cpp
const bool unconfirmed_live = !opt_assumeyes && opt_apply_live;
if (unconfirmed_live && !opt_dry_run)
  {
    // ...
    if (!pkg_change (invocation, sysroot_proxy, TRUE, (const char *const *)argv,
                     (const char *const *)opt_uninstall, cancellable, error))
      return FALSE;
    CXX_TRY (rpmostreecxx::confirm_or_abort (), error);
    // Fall through, the operation is confirmed.
  }
```

This explains the repeated checkout,
repository,
and dependency-resolution output around the `Continue? [y/N]` prompt.
It does not mean that two deployments were installed.

### The large install list is the complete layering transaction

`src/libpriv/rpmostree-rpm-util.cxx:1016-1044` asks libdnf for every package in the
install,
reinstall,
downgrade,
or update goal and prints that complete list:

```cpp
packages = dnf_goal_get_packages (dnf_context_get_goal (dnfctx), DNF_PACKAGE_INFO_INSTALL,
                                  DNF_PACKAGE_INFO_REINSTALL, DNF_PACKAGE_INFO_DOWNGRADE,
                                  DNF_PACKAGE_INFO_UPDATE, -1);

if (packages->len > 0)
  {
    empty = FALSE;
    rpmostree_output_message ("Installing %u packages:", packages->len);
    print_pkglist (packages);
  }
```

The separately printed `Will download: 5 packages` is only the cache-miss count.
The transcript's larger list includes the packages being layered into the newly composed deployment.
It may include pre-existing host package requests and their dependencies,
but only `rpm-ostree status --verbose` on the affected host can identify those requests.
The list is not evidence that Chromium unexpectedly requested all of those packages as new dependencies.

### Staging succeeds before live application begins

`src/daemon/rpmostreed-transaction-types.cxx:1655-1670` stages the deployment first,
then calls the live-apply operation:

```cpp
g_autoptr (OstreeDeployment) new_deployment = NULL;
if (!rpmostree_sysroot_upgrader_deploy (upgrader, &new_deployment, cancellable, error))
  return FALSE;

// ...

if (deploy_has_bool_option (self, "apply-live"))
  {
    g_autoptr (GVariantDict) dictv = g_variant_dict_new (NULL);
    g_autoptr (GVariant) live_opts = g_variant_ref_sink (g_variant_dict_end (dictv));
    ROSCXX_TRY (transaction_apply_live (*sysroot, *live_opts), error);
  }
```

The user's `Staging deployment... done` line therefore precedes a separate live-apply failure.
The pending deployment normally remains queued for the next boot.
`rpm-ostree status --verbose` is the authoritative check on the affected host.

### Live apply rejects removals by default

`rust/src/live.rs:420-445` compares the package databases for the running source commit and target pending commit.
Before it writes the `/usr` diff,
it rejects package removals or changes unless the live operation explicitly allows replacements:

```rust
let pkgdiff = {
    cxx::let_cxx_string!(from = source_commit);
    cxx::let_cxx_string!(to = &*target_commit);
    crate::ffi::rpmdb_diff(repo.reborrow_cxx(), &from, &to, false)
        .map_err(anyhow::Error::msg)?
};
if !allow_replacement {
    if pkgdiff.n_removed() > 0 {
        return Err(anyhow!(
            "packages would be removed: {}, allow replacement to override",
            pkgdiff.n_removed()
        )
        .into());
    }
    if pkgdiff.n_modified() > 0 {
        return Err(anyhow!(
            "packages would be changed: {}, allow replacement to override",
            pkgdiff.n_modified()
        )
        .into());
    }
}
```

Consequently,
the error is a safety refusal to mutate the running deployment.
The transcript stops before `Computing /etc diff to preserve` and `Updating /usr`,
so it does not show package files being live-applied.
The three removals belong to the complete booted-to-pending diff,
which can include changes already accumulated in a pending deployment,
not necessarily Chromium's direct dependency transaction.

### The suggested option belongs to apply-live, not install

`src/app/rpmostree-pkg-builtins.cxx:94-103` defines `--apply-live` for `install`,
but does not define `--allow-replacement` there:

```cpp
static GOptionEntry install_option_entry[]
    = { { "uninstall", 0, 0, G_OPTION_ARG_STRING_ARRAY, &opt_uninstall,
          "Remove overlayed additional package", "PKG" },
        // ...
        { "apply-live", 'A', 0, G_OPTION_ARG_NONE, &opt_apply_live,
          "Apply changes to both pending deployment and running filesystem tree", NULL },
        { "force-replacefiles", 0, 0, G_OPTION_ARG_NONE, &opt_force_replacefiles,
          "Allow package to replace files from other packages", NULL },
        { NULL } };
```

`rust/src/builtins/apply_live.rs:12-24` defines `--allow-replacement` on the separate
`apply-live` command:

```rust
#[derive(Debug, Parser)]
#[clap(name = "apply-live")]
#[clap(rename_all = "kebab-case")]
struct Opts {
    /// Target provided commit instead of pending deployment
    #[clap(long)]
    target: Option<String>,
    /// Reset back to booted commit
    #[clap(long)]
    reset: bool,

    /// Allow replacement of packages/files (default is pure additive)
    #[clap(long)]
    allow_replacement: bool,
}
```

The syntactically valid command is therefore
`rpm-ostree apply-live --allow-replacement`,
not another `install` invocation.
It should not be run before identifying the three removed packages.

## Verification

Source verification used rpm-ostree v2026.2 at
`d5ef5f0cb5ace53f40a8d92933cc87e69ad8c7f4`.
The upstream destructive test at `tests/kolainst/destructive/apply-live:85-96`
asserts the same default refusal and then verifies that the option succeeds on the separate live command:

```sh
rpm-ostree install bar
if rpm-ostree apply-live 2>err.txt; then
    fatal "live-removed foo"
fi
assert_file_has_content_literal err.txt 'packages would be removed: 1, allow replacement to override'
rpm-ostree ex livefs --allow-replacement | tee out.txt
```

Run these read-only commands on the affected Bazzite host:

```sh
rpm-ostree --version
rpm-ostree status --verbose
rpm-ostree db diff --format=diff
```

With no revisions,
the v2026.2 manual specifies that `db diff` compares the booted commit with the pending commit.
Its `--format=diff` output prefixes removed packages with `-`,
added packages with `+`,
and updates with `!` and `=`.

Working catalog:

- On a fresh state without an existing Chromium request,
  `rpm-ostree install chromium` normally composes a pending deployment for the next boot.
  This is not a recovery command after Chromium has already been staged.
- `rpm-ostree install chromium --apply-live` works when the complete booted-to-pending package diff is additive.
- `rpm-ostree apply-live --allow-replacement` accepts package changes and removals after a deployment is pending.

Failing catalog:

- `rpm-ostree install chromium --apply-live` rejects a complete booted-to-pending diff containing removals.
- `rpm-ostree install chromium --apply-live --allow-replacement` rejects the option because `install` does not own it.

## Verified workarounds

### Inspect the pending deployment and reboot

First inspect the pending deployment:

```sh
rpm-ostree status --verbose
rpm-ostree db diff --format=diff
```

Use this path only after the removed-package lines are understood and acceptable.
The following command activates Chromium and every other queued pending change at boot:

```sh
systemctl reboot
```

The tradeoff is that all queued changes become active at boot,
not immediately.
This is the normal transactional package-layering path documented by both rpm-ostree and Bazzite.

### Discard the pending deployment

```sh
rpm-ostree status --verbose
rpm-ostree cleanup --pending
rpm-ostree status --verbose
```

The tradeoff is that `cleanup --pending` discards the complete pending deployment,
including unrelated queued changes,
not only the Chromium request.
It does not uninstall packages from the currently booted deployment.

### Force the complete pending diff live

```sh
rpm-ostree db diff --format=diff
rpm-ostree apply-live --allow-replacement
```

The tradeoff is that the complete pending diff,
including the three removals and any package changes,
is applied to the running system.
The v2026.2 manual states that live apply synchronizes the filesystem but does not restart systemd units.
A reboot removes the transient live overlay,
but then boots the pending deployment unless that deployment was discarded first.

## What does not work

- Re-running `install` with `--allow-replacement` does not work because that subcommand does not define the option.
- Treating `Installing 148 packages` as 148 new Chromium dependencies is incorrect;
  the output is the complete libdnf layering goal,
  while the download line reports only uncached payloads.
- Assuming the command did nothing because it ended with `error:` is incorrect;
  deployment staging completed before live apply failed.
- Assuming the three removals are safe is also incorrect without reading `rpm-ostree db diff --format=diff`.

## Upstream filing decision

`.out-of-scope/` was checked for rpm-ostree and Bazzite exemptions.
No matching exemption exists.
Open and closed rpm-ostree issues and pull requests were searched for the exact diagnostic,
pending-deployment accumulation,
and `install --apply-live` option behavior.

[coreos/rpm-ostree#3728](https://github.com/coreos/rpm-ostree/issues/3728)
already tracks the misleading stage-then-error result for `install -A`.
A maintainer explains there that live apply always applies accumulated changes
from the pending deployment to the current one.
The current investigation adds no installed-version result or minimal reproduction absent from that issue,
so there is nothing additive to post.

1. Is it really upstream's fault?
   The default refusal and option scope are intentional,
   documented safety behavior.
   The confusing stage-then-error presentation is an upstream wording issue already tracked by issue #3728.
2. Can upstream fix it?
   Yes,
   upstream can clarify that staging succeeded and print the exact follow-up subcommand.
3. Are they supporting this use case?
   Yes,
   the manual documents additive `install --apply-live` and replacement-capable `apply-live` separately.
4. Would the repo welcome our contribution?
   `CONTRIBUTING.md`,
   `.github/ISSUE_TEMPLATE/bug-report.yml`,
   and `.github/PULL_REQUEST_TEMPLATE` accept external reports and patches.
   No AI-assistance restriction was found.
5. Will they likely fix it?
   Unknown.
   Issue #3728 remains open without a linked patch,
   and the relevant source path has no post-2024 commit in the shallow v2026.2 source history.
6. Have we prototyped a minimal fix compatible with their architecture?
   No.
   The observed safety behavior is correct,
   and the remaining wording concern is already tracked without new incident version data.
   Constraints 1 and 5 do not trigger an upstream prototype.

Nothing to add to issue #3728:
no comment draft is retained because repeating the same behavior
without the affected binary version would not advance the thread.
