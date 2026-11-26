# desktop-file-utils 0.29 cannot replace user caches when AppArmor omits their temporary files

## Status

The real `update-desktop-database --verbose ~/.local/share/applications` command exits 0. Both
cache files were replaced, Pale Moon is registered, and the complete 812-profile cache rebuild
passed. This is an evidence record, not approval of unrelated desktop-entry warnings.

## Symptom

After installing a Pale Moon desktop entry, `update-desktop-database` reported:

```text
Could not create cache file in "/home/user/.local/share/applications":
Failed to create file “/home/user/.local/share/applications/mimeinfo.cache.BS4OZ3”:
Permission denied
The databases in [/home/user/.local/share/applications] could not be updated.
```

The directory and existing cache were owned by `user`, so ordinary filesystem ownership did not
explain the failure. The kernel audit showed the enforcing boundary directly:

```text
apparmor="DENIED" operation="mknod" class="file"
profile="update-desktop-database"
name="/home/user/.local/share/applications/mimeinfo.cache.BS4OZ3"
requested_mask="c" denied_mask="c"
```

An earlier attempt produced the same denial for `mimeinfo.cache.5VDJZ3`.

## Root cause

desktop-file-utils 0.29's `update_mime_cache_database()` builds `mimeinfo.cache` in the selected
directory and calls `g_file_set_contents_full()` with `G_FILE_SET_CONTENTS_CONSISTENT` in
`src/update-desktop-database.c:459-477`:

```c
mime_cache_file = g_build_filename (dir, MIMEINFO_CACHE_FILENAME, NULL);
if (!g_file_set_contents_full (mime_cache_file,
                               mime_cache_contents,
                               -1,
                               G_FILE_SET_CONTENTS_CONSISTENT,
                               0666,
                               error))
  return FALSE;
```

Source: [desktop-file-utils 0.29 `update-desktop-database.c`, MIME cache replacement](https://gitlab.freedesktop.org/xdg/desktop-file-utils/-/blob/0.29/src/update-desktop-database.c#L459-477).

The adjacent `update_intent_cache_database()` performs the same operation for `intent.cache` in the
same file at lines 579-598.

Source: [desktop-file-utils 0.29 `update-desktop-database.c`, intent cache replacement](https://gitlab.freedesktop.org/xdg/desktop-file-utils/-/blob/0.29/src/update-desktop-database.c#L579-598).

GLib 2.90.1 implements consistent replacement by appending `.XXXXXX`, creating that temporary file,
and renaming it over the destination in `glib/gfileutils.c:1330-1374`:

```c
tmp_filename = g_strdup_printf ("%s.XXXXXX", filename);

errno = 0;
fd = g_mkstemp_full (tmp_filename, O_RDWR | O_BINARY | O_CLOEXEC, mode);
```

Source: [GLib 2.90.1 `gfileutils.c`, temporary file creation](https://gitlab.gnome.org/GNOME/glib/-/blob/2.90.1/glib/gfileutils.c#L1330-L1374).

GLib's `get_tmp_file()` replaces the six `X` characters with uppercase letters or digits in the same
file at lines 1537-1597, matching the audited names and AppArmor's `@{rand6}` pattern.

The stock `update-desktop-database` profile already covers these patterns for system applications
directories. For the user applications directory, it covered `mimeinfo.cache` and hidden
`.mimeinfo.cache.*`, but omitted:

- `mimeinfo.cache.@{rand6}`
- `intent.cache`
- `intent.cache.@{rand6}`

The first local rule allowed the MIME temporary file. The command then reached the independent
intent-cache stage and failed on `intent.cache.PYQGZ3`. The source-derived destination and temporary
rules for intent cache were therefore also required.

## Verification

Installed versions were desktop-file-utils `0.29-1.1` and GLib `2.90.1-1.1`. Source was fetched from
the matching `0.29` and `2.90.1` tags, not their default branches.

### Patterns that fail

Before the local rules, two real invocations failed to create different six-character MIME cache
temporary files. After adding only the MIME temporary rule, the next invocation failed on:

```text
Failed to create file “/home/user/.local/share/applications/intent.cache.PYQGZ3”:
Permission denied
```

This sequential result is not evidence that the first rule failed. It proves the command advanced
to the second cache write, whose destination and temporary name were still blocked.

### Patterns that work

After all three rules were loaded, the real command completed:

```sh
update-desktop-database --verbose /home/user/.local/share/applications
```

It exited 0 and produced fresh `mimeinfo.cache` and `intent.cache` files. The MIME cache contains
`palemoon.desktop` for these registrations:

```text
application/xhtml+xml
text/html
x-scheme-handler/http
x-scheme-handler/https
```

The remaining `lacks MimeType key` and `lacks Implements key` messages are nonfatal desktop-entry
warnings. They are distinct from the former fatal cache-creation error. `desktop-file-validate`
had already accepted the Pale Moon entry.

`verified-selected-desktop-database-user-cache-fix-2026-11-26T17-26-15-0500.json` records all 812
expected cache artifacts, exact live feature bytes, selection, reload, and rollback preservation.
A subsequent parser invocation reported `Cache hit` for the selected `update-desktop-database`
artifact. No denial or six-character temporary cache file remained after the final run.

## Verified workarounds

### Permit the exact user cache destination and temporary names

Create `/etc/apparmor.d/local/update-desktop-database` with:

```apparmor
# GLib writes each cache.XXXXXX before atomically renaming it.
owner @{user_share_dirs}/applications/mimeinfo.cache.@{rand6} rw,
owner @{user_share_dirs}/applications/intent.cache w,
owner @{user_share_dirs}/applications/intent.cache.@{rand6} rw,
```

Then reload the profile and run the real command:

```sh
sudo apparmor_parser --replace --write-cache /etc/apparmor.d/update-desktop-database
update-desktop-database --verbose "$HOME/.local/share/applications"
```

The rule allows creation and replacement only for the two cache names in the user's `applications`
directory. It does not grant arbitrary `.local/share` writes, desktop-file modification, executable
transitions, capabilities, or access to another user's files. `mimeinfo.cache` already had its
destination grant in the stock profile; `intent.cache` needed both destination and temporary grants.

The pre-change state, an absent local file, is recorded privately at
`/home/user/temp/agent/apparmor-full-system-cachyos-2026-11-17/update-desktop-database-local.before-user-cache-temp`.

## What does not work

- Changing directory ownership or mode: the existing directory and cache were already owned by
  `user`, and the AVC named AppArmor as the denial source.
- Adding only `mimeinfo.cache.@{rand6}`: this repairs the first stage but leaves both intent-cache
  paths blocked.
- Granting write access to all of `.local/share/applications`: broader than the two cache outputs
  required by the measured workload.
- Treating every verbose warning as a fatal error: missing optional desktop-entry keys remain
  warnings when the command exits 0 and the caches are replaced.
- Validating only `mimeinfo.cache`: the complete command must also create `intent.cache` and exit 0.

## Upstream filing decision

Issue and pull-request searches for `update-desktop-database`, `mimeinfo.cache`, and `intent.cache`
under `roddhjav` returned no matching records on 2026-11-26.

1.  **Is it really upstream's fault?** The demonstrated omission is in the packaged `apparmor.d`
    profile, which models the system-directory pattern but not its user-directory equivalent.
2.  **Can upstream fix it?** Yes, the same three source-derived user rules could be added to the
    stock profile. The local file provides the immediate repair without modifying managed source.
3.  **Are they supporting this use case?** desktop-file-utils explicitly supports updating a chosen
    user applications directory, and the stock profile already grants parts of that workflow.
4.  **Would the repo welcome our contribution?** Not evaluated because no external report was
    authorized. No contribution-policy claim is made.
5.  **Will they likely fix it?** Not predicted without maintainer input. The local evidence is
    sufficient for a future patch but does not require public filing.
6.  **Have we prototyped a compatible fix?** The three-rule local profile passed the real command
    and produced both caches without broadening unrelated access.

No upstream issue is filed. This record preserves a possible upstream patch and its verification,
without claiming maintainer acceptance or requiring a public report.
