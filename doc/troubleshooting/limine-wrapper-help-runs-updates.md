# `limine-mkinitcpio` and `limine-update` 1.40.0 treat `--help` as a request to rebuild boot artifacts

## Symptom

Running either command with `--help` does not print wrapper help and does not stop without changing the system:

- `limine-mkinitcpio --help` printed `Building initramfs for linux-cachyos-lts` and `Building initramfs for linux-cachyos`, regenerated both initramfs images, and updated `/boot/limine.conf`.
- `limine-update --help` printed `Limine EFI update completed successfully.` before producing the same initramfs build output.
- `limine-entry-tool --help`, in contrast, printed `Usage: limine-entry-tool [options] [--quiet]` without starting an initramfs build.

The initial assumption that both invocations would be read-only capability checks was wrong. Their outputs prove that they changed boot artifacts.

## Root cause

The wrapper `limine-update` forwards every argument to `limine-mkinitcpio` after running `limine-install --no-efi-register`. There is no option parser before the update calls, as shown in [`limine-update` lines 17 to 25][limine-update-source]:

```bash
# install/arch-linux/limine-mkinitcpio-hook/usr/bin/limine-update
if command -v limine-install &>/dev/null; then
	limine-install --no-efi-register
fi
if command -v limine-mkinitcpio &>/dev/null; then
	limine-mkinitcpio "$@"
fi
```

The downstream wrapper treats all arguments other than the empty argument list as kernel package names. It validates them through `pacman -Q`, then forwards them on standard input, as shown in [`limine-mkinitcpio` lines 16 to 32][limine-mkinitcpio-source]:

```bash
# install/arch-linux/limine-mkinitcpio-hook/usr/bin/limine-mkinitcpio
validate_kernel_packages() {
	local kernel_package
	for kernel_package in "$@"; do
		if ! pacman -Q "$kernel_package" &>/dev/null; then
			return 1
		fi
	done
}

if [[ $# -eq 0 ]]; then
	echo rebuild | /usr/share/libalpm/scripts/limine-mkinitcpio-install
else
	if ! validate_kernel_packages "$@"; then
		exit 1
	fi
	printf '%s\n' "$@" | /usr/share/libalpm/scripts/limine-mkinitcpio-install
fi
```

`pacman -Q --help` is itself a successful help query, so `--help` passes validation. It does not name a kernel package, but no diagnostic distinguishes that case.

The collector then calls `add_kernel_dir_by_package` for the string. The fallback sets `rebuild_all=true` when that call fails, as shown in [`limine-mkinitcpio-install` lines 55 to 66][limine-collector-package-source] and [`limine-mkinitcpio-install` lines 95 to 96][limine-rebuild-all-source]:

```bash
# install/arch-linux/limine-mkinitcpio-hook/usr/share/libalpm/scripts/limine-mkinitcpio-install
add_kernel_dir_by_package() {
	local package="$1" path
	local found=false
	[[ "$package" != */* ]] || return 2
	pacman -Q "$package" &>/dev/null || return 3
	while read -r _ path; do
		[[ "$path" == /usr/lib/modules/*/modules.builtin ]] || continue
		KERNEL_DIRS+=("${path%/modules.builtin}")
		found=true
	done < <(pacman -Ql "$package" 2>/dev/null)
	$found
}

# Inside collect_kernels
		elif ! add_kernel_dir_by_package "$line"; then
			rebuild_all=true
		fi
```

When `rebuild_all` is true, the collector replaces the list with every installed kernel directory containing `modules.builtin`. This explains both the missing help output and the full rebuild.

## Verification

The installed package is `limine-mkinitcpio-hook 1.40.0-1`. Source tag 1.40.0 resolves to commit `35dab4b732861f5e8d307a1a7873856a877913aa`. The installed wrapper scripts and downstream install script exactly match that tag as checked with `diff --unified`.

The current source checkout is version 1.41.0 at commit `5e6be0cf978edc511786fddd662d1f50280eafad`. Its two wrapper scripts also match the installed files. The downstream script has unrelated later changes, while the argument acceptance and full-rebuild branches cited here remain present.

Failing patterns, observed on CachyOS with both `linux-cachyos` and `linux-cachyos-lts` installed:

```sh
# These commands mutate boot artifacts. Do not use them as read-only help probes.
sudo limine-mkinitcpio --help
sudo limine-update --help
```

The first printed `Building initramfs for` for each installed kernel. The second additionally printed `Limine EFI update completed successfully.`.

Working patterns:

```sh
# Read-only entry-manager help
limine-entry-tool --help

# Non-mutating confirmation that pacman itself accepts --help and exits successfully
pacman --query --help
```

`limine-entry-tool --help` printed its usage text. `pacman --query --help` exited successfully and produced 42 output lines on this installation.

After intentional regeneration, a local verifier checked every `path` and `module_path` with a BLAKE2b suffix in `/boot/limine.conf` against the actual ESP files. All four current artifacts matched. The LTS image also contained `usr/lib/modules/6.18.64-1-cachyos-lts/extramodules/zfs.ko.zst`.

## Verified workarounds

Use `limine-entry-tool --help` when working with entry-manager options. It is read-only and printed the expected usage text. Its output documents entry operations, not the update wrappers' package-selection behavior, so it is not a substitute for wrapper-specific documentation.

Read the wrapper source before invoking an unfamiliar option. The relevant installed paths are `/usr/bin/limine-mkinitcpio`, `/usr/bin/limine-update`, and `/usr/share/libalpm/scripts/limine-mkinitcpio-install`. This avoids mutations, but requires reading implementation rather than consulting a help screen.

Invoke an update only when its side effects are intended:

```sh
# Intentionally regenerate entries and artifacts for all installed kernels
sudo limine-mkinitcpio

# Read back the generated entries afterward
sudo limine-entry-tool --tree 3
```

After any intended or accidental update, verify the boot entries and artifact hashes before rebooting. The tradeoff is that `limine.conf` alone is not a complete rollback because regenerated images and their hashes may have changed.

## What does not work

Using `--help` as a capability probe on either update wrapper fails because it performs updates. The resulting successful exit status cannot be interpreted as help support or as evidence that no mutation occurred.

Checking only `pacman -Q <argument>` does not establish that `<argument>` is a kernel package. Pacman's own `--help` is successful and is not a package existence check.

Restoring only an old `/boot/limine.conf` after these mutations does not restore older kernel or initramfs contents. Its embedded hashes also refer to the previous artifacts. Artifact-level backups or a separately configured recovery kernel are needed.

## Upstream filing decision

Do not file as-is. A new report or comment would put a real person's account and reputation in front of external maintainers, so filing remains the poster's decision.

The repository-local `.out-of-scope/` search found no Limine exemption. A web search for the exact quoted pair `"limine-mkinitcpio"` and `"--help"` found source and unrelated full-rebuild reports, but no matching help-handling report. A GitLab API search returned HTTP 401, so it supplied no usable duplicate-search evidence.

1. **Is it really upstream's fault?** Yes, for the wrappers' argument acceptance and rebuild fallback. This is not caused by Limine's boot menu or by the kernel.
2. **Can upstream fix it?** Yes. The wrappers can recognize `--help` or reject option-looking package names before starting updates.
3. **Are they supporting this use case?** Not from the evidence gathered. The README documents direct update commands and `limine-entry-tool --help`, but no help surface for these wrappers. This constraint fails and is the reason not to file as-is.
4. **Would the repo welcome our contribution?** No contribution, issue-template, pull-request-template, or trust-list files appeared in the source checkout. That is an absence of found files, not proof of policy. Intake expectations and trust-system risks remain unverified.
5. **Will they likely fix it?** No active refusal was found. The current 1.41.0 wrapper source retains the behavior. Duplicate and work-in-progress evidence is incomplete because the authenticated tracker search was unavailable and the clone was shallow.
6. **Have we prototyped a minimal fix compatible with their architecture?** No. The supported-use-case constraint already fails, and no patch was written. The reproduction and source trace are sufficient for local avoidance, not for a filing recommendation.

Readiness check at 2026-11-23T06:14:07-05:00:

- Behavior was reproduced on installed version 1.40.0. Current 1.41.0 source was read, but its binaries were not rebuilt or executed.
- Prior merge requests about full rebuild behavior were found, but they do not match this help-handling root cause.
- Intake files and new-filing automation were not fully assessed.
- The exact mutating invocations were run on the affected installation. No clean-runner workflow was created because this record does not recommend filing and wraps real boot-artifact mutations rather than a disposable unit-test fixture.
- No draft is ready to post. The remaining blockers are supported-use-case evidence, authenticated duplicate review, intake and trust-system review, and a disposable reproduction with a minimal fix prototype.

Draft artifact, not for filing as-is:

~~~md
Title: Update wrappers treat --help as a request to rebuild all kernels

`limine-mkinitcpio --help` rebuilds all installed kernels, and `limine-update --help` additionally updates the EFI loader. Version 1.40.0 is affected, and the same wrapper logic is present in current source.

The wrapper accepts `--help` as a package name because `pacman -Q --help` succeeds. `limine-mkinitcpio-install` cannot map the string to a kernel package, sets `rebuild_all=true`, and rebuilds all kernels.

Suggested fix: handle `--help` before the update path and reject unsupported option-looking package names before validation. Verification should prove no boot files change for help and invalid-option cases while intentional package updates still run.
~~~

[limine-update-source]: https://gitlab.com/Zesko/limine-entry-tool/-/raw/35dab4b732861f5e8d307a1a7873856a877913aa/install/arch-linux/limine-mkinitcpio-hook/usr/bin/limine-update#:~:text=limine-install%20--no-efi-register
[limine-mkinitcpio-source]: https://gitlab.com/Zesko/limine-entry-tool/-/raw/35dab4b732861f5e8d307a1a7873856a877913aa/install/arch-linux/limine-mkinitcpio-hook/usr/bin/limine-mkinitcpio#:~:text=validate_kernel_packages()
[limine-collector-package-source]: https://gitlab.com/Zesko/limine-entry-tool/-/raw/35dab4b732861f5e8d307a1a7873856a877913aa/install/arch-linux/limine-mkinitcpio-hook/usr/share/libalpm/scripts/limine-mkinitcpio-install#:~:text=add_kernel_dir_by_package()
[limine-rebuild-all-source]: https://gitlab.com/Zesko/limine-entry-tool/-/raw/35dab4b732861f5e8d307a1a7873856a877913aa/install/arch-linux/limine-mkinitcpio-hook/usr/share/libalpm/scripts/limine-mkinitcpio-install#:~:text=elif%20!%20add_kernel_dir_by_package
