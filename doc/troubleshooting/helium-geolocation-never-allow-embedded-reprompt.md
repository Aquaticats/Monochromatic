# Helium 0.22.5.1 crashes on Never allow when an embedded geolocation prompt is recreated

## Status

Disabling `GeolocationElement` avoids the affected embedded-prompt path. The owner confirmed that
Helium now denies Best Buy's location request without crashing. Coredump extraction and debugger
inspection work through narrowly scoped AppArmor additions. The 812-profile cache rebuild is in
progress. This is an evidence record, not approval of unrelated browser behavior.

## Symptom

Clicking **Never allow** for Best Buy's location prompt terminated Helium with `SIGTRAP` at
2026-11-27T19:39:03-05:00. The retained core included:

```text
www.bestbuy.ca wants to: Know your location
Permission requested, press F6 to respond
Never allow
```

The crashing thread began with `ud2`, not an AppArmor denial or an ordinary segmentation fault. The
process had Helium `0.22.5.1`, which reports Chromium `158.0.8127.88`.

## Root cause

Chromium's embedded `<permission>` flow can recompute a prompt variant after the request's decision
callback has already been consumed. If a higher-priority settings provider shadows the stored user
grant, the variant can return to a site-level ask or previously-denied prompt and recreate UI whose
callback is no longer valid.

Current Chromium's `EmbeddedPermissionPromptFlowModel::DeterminePromptVariant()` avoids that state
by treating a request that is already granted as site-allowed:

```cpp
const bool is_site_permission_allowed =
    request->IsPermissionGranted() ||
    permission_info->delegate().IsAnyPermissionAllowed(setting);
```

Source: [Chromium `embedded_permission_prompt_flow_model.cc`, finalized-grant handling](https://github.com/chromium/chromium/blob/c00e3e4c171cabbd92ebcf0c91ba9b19b8a35096/components/permissions/embedded_permission_prompt_flow_model.cc#L112-L150).

The corresponding `PermissionRequestManager::CanContinueEmbeddedPromptFlow()` prevents advancement
to a site-decision variant when any active request has already been finalized:

```cpp
if (is_site_decision_variant &&
    std::ranges::any_of(requests_, &PermissionRequest::IsDecisionFinalized)) {
  return false;
}
```

Source: [Chromium `permission_request_manager.cc`, no re-prompt after finalization](https://github.com/chromium/chromium/blob/c00e3e4c171cabbd92ebcf0c91ba9b19b8a35096/components/permissions/permission_request_manager.cc#L1050-L1067).

Upstream's commit message identifies the exact failure mode: accepting an embedded prompt while a
higher-priority provider shadows the user grant recreates the site prompt and fails a null
`OnceCallback` check on the next interaction. The fix is commit
`c00e3e4c171cabbd92ebcf0c91ba9b19b8a35096`, Bug 585118316.

The installed Helium source does not contain `CanContinueEmbeddedPromptFlow` or its finalized-request
guard. Best Buy had multiple location-related objects in the retained core, so the embedded
geolocation path and its callback lifetime are the relevant boundary. The retained stack and `ud2`
are consistent with that fatal check, but the stripped binary does not provide enough internal
symbols to name the exact source line from the core alone.

## Verification

Installed package is `helium-browser-bin 0.22.5.1-1`. Installed Chromium source tag
`158.0.8127.88` resolves to `0cefb18236a23db1ebb6b2bc040fbbaf8cbe759e`. The fixed implementation
was inspected separately at `c00e3e4c171cabbd92ebcf0c91ba9b19b8a35096`; it was not mistaken for the
installed version.

### Patterns that fail

Before the workaround, the real Best Buy action crashed with `SIGTRAP`. GDB showed:

```text
=> 0x5631ae6efedd: ud2
   0x5631ae6efedf: int3
```

`coredumpctl dump` initially failed because the `coredumpctl` profile denied creation and truncation
of the extracted report. A root-run attempt then failed on `dac_override`, which was not added. The
working approach was to grant only the incident scratch directory and run `coredumpctl` as the user.
GDB also needed read access to `/opt/helium-browser-bin` before it could map the trap instruction.

### Patterns that work

The flag file `/home/user/.config/helium-browser-flags.conf` contains:

```text
# Avoid the embedded-permission re-prompt crash until Helium includes the upstream fix.
--disable-features=GeolocationElement
```

`PermissionUtil::ShouldCurrentRequestUsePermissionElementSecondaryUI()` checks
`blink::features::kGeolocationElement`, so this disables the affected embedded flow rather than every
permission prompt. After a full Helium restart, the owner retried Best Buy's location prompt and
confirmed that **Never allow** completed without a crash.

The installed wrapper reads `/etc/helium-browser-flags.conf` and then the user file, so this is a
real supported configuration path, not an environment-only experiment.

## Verified workarounds

### Use classic geolocation prompts until Helium includes the fix

Keep `--disable-features=GeolocationElement` in `/home/user/.config/helium-browser-flags.conf` until
the installed Helium version includes `c00e3e4c171cabbd92ebcf0c91ba9b19b8a35096`. Fully restart
Helium after editing it.

The tradeoff is that `<permission>` geolocation requests use the older prompt path instead of the
embedded element UI. Normal location allow/deny behavior remains available. This does not disable
JavaScript geolocation, AppArmor, Chromium's renderer sandbox, or other permission types.

The flag should be removed after upgrading to a Helium build that contains the upstream fix and
revalidating the same user action. No global `--no-sandbox`, `apparmor=unconfined`, or permanent
browser downgrade is needed.

`verified-selected-coredumpctl-debug-access-2026-11-27T19-48-06-0500.json` records all 812 expected
cache artifacts, exact live feature bytes, selection, and reload. A subsequent parser invocation
reported `Cache hit` for the selected `coredumpctl` artifact.

### Permit retained-core analysis only where needed

The local AppArmor additions are:

```apparmor
# /etc/apparmor.d/local/coredumpctl
/home/user/temp/agent/helium-bestbuy-never-allow-crash-2026-11-27/{,**} rw,

# /etc/apparmor.d/local/coredumpctl_gdb
/home/user/temp/agent/helium-bestbuy-never-allow-crash-2026-11-27/{,**} rw,
/opt/helium-browser-bin/{,**} r,
```

The first rule permits extraction and truncation of this incident's report. The second permits the
nested debugger to map the extracted core and the installed Helium executable. `coredumpctl` already
had `/var/lib/systemd/coredump/{,**} r` and `dac_read_search`, so no new capability was added. Running
as root against the user-owned report produced `capability dac_override` and was rejected rather
than broadened; running as the user succeeded.

## What does not work

- Treating the DMI AVCs as the crash cause: they are unrelated to the location decision callback.
- Treating `ud2` as a filesystem permissions failure: it is a fatal Chromium trap, whose embedded
  prompt lifetime bug is documented upstream.
- Adding `dac_override`: unnecessary when `coredumpctl` runs as the user with the exact scratch
  grant. The compressed core was already readable.
- Using the current Chromium source as if it were installed: the finalized-request guard arrived
  after the installed Helium build and must be compared explicitly.
- Disabling all permission prompts or the browser sandbox: unnecessary for this UI-specific bug.

## Upstream filing decision

The exact failure is already fixed in upstream Chromium. No duplicate issue is recommended.

1.  **Is it really upstream's fault?** Yes, this is a Chromium embedded-permission lifetime defect,
    not a local AppArmor denial. The installed Helium predates the fix.
2.  **Can upstream fix it?** It already did, in `c00e3e4c171cabbd92ebcf0c91ba9b19b8a35096`.
3.  **Are they supporting this use case?** The fix explicitly handles an accepted embedded prompt
    whose higher-priority provider shadows the user grant, including the next interaction.
4.  **Would the repo welcome our contribution?** No new contribution is needed. Helium only needs to
    incorporate an existing upstream fix; no acceptance claim is made for a future release.
5.  **Will they likely fix it?** Chromium has fixed it. A Helium release containing that commit is
    not assumed or scheduled here.
6.  **Have we prototyped a compatible fix?** The feature-specific workaround passed the real Best
    Buy action. It avoids the affected flow without modifying upstream source.

No new upstream issue or additive comment is recommended. This record does not claim that the fixed
Chromium build was run locally; it identifies the workaround that passed on the installed version.
