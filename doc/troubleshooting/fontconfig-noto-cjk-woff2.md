# Fontconfig 2.17.0 returns no patterns for Noto CJK WOFF2 collections that exceed FreeType's output cap

## Symptom

On Fedora 44 with Fontconfig 2.17.0 and FreeType 2.14.3,
`fc-scan` rejects both WOFF2 collections from the `noto-cjk` main-branch archive:

```text
NotoSansCJK-wght-400-900.ttf.woff2: exit 1, no output
NotoSerifCJK-wght-400-900.ttf.woff2: exit 1, no output
```

The matching `.ttf.ttc` files scan successfully.
Each TTC yields 35 Fontconfig patterns,
including a variable pattern for each of the five regional families.

`file` identifies both rejected inputs as WOFF2 files whose inner flavor is `ttcf`.
A normal WOFF2 variable font scans successfully on the same host,
so the failure is specific to these large WOFF2 collections rather than all WOFF2 support.

## Root cause

Fontconfig delegates font parsing to FreeType.
FreeType accepts the WOFF2 header and collection directory,
then fails during output reconstruction.
The trace position,
source control flow,
and expanded sizes identify the separate 30 MiB output cap as the Sans trigger
and as the supported explanation for the Serif failure.
The exact Serif guard branch was not instrumented.

### Fontconfig hands the file to FreeType

`fc-scan/fc-scan.c:162-166` in Fontconfig tag `2.17.0`
passes every non-directory input to `FcFileScan`:

```c
for (; i < argc; i++) {
    const FcChar8 *file = (FcChar8 *)argv[i];

    if (!FcFileIsDir (file))
        FcFileScan (fs, NULL, NULL, NULL, file, FcTrue);
```

`src/fcdir.c:84-91` selects `FcFreeTypeQueryAll` by default
and reports failure when the query adds no patterns:

```c
unsigned int (*query_function) (const FcChar8 *, unsigned int, FcBlanks *, int *, FcFontSet *) = FcFreeTypeQueryAll;
/* ... */
if (!query_function (file, -1, NULL, NULL, set))
    return FcFalse;
```

`src/fcfreetype.c:2175-2179` calls `FT_New_Face` and leaves with zero patterns when FreeType rejects the file:

```c
if (FT_Init_FreeType (&ftLibrary))
    return 0;

if (FT_New_Face (ftLibrary, (const char *)file, face_num, &face))
    goto bail;
```

`fc-scan` does not print the FreeType error.
With no patterns to iterate,
`fc-scan/fc-scan.c:199-204` returns status 1:

```c
FcFontSetDestroy (fs);
/* ... */
return i > 0 ? 0 : 1;
```

### FreeType stops reconstructed output at 30 MiB

FreeType tag `VER-2-14-3` defines two expanded-output limits.
`src/sfnt/sfwoff2.h:37-38` defines the limit used by output-buffer writes:

```c
/* Suggested maximum size for output. */
#define WOFF2_DEFAULT_MAX_SIZE  30 * 1024 * 1024
```

`src/sfnt/sfwoff2.c:212-228` applies that limit to cumulative output,
not to one table:

```c
static FT_Error
write_buf( FT_Byte**  dst_bytes,
           FT_ULong*  dst_size,
           FT_ULong*  offset,
           FT_Byte*   src,
           FT_ULong   size,
           FT_Memory  memory )
{
  /* ... */
  if ( ( *offset + size ) > WOFF2_DEFAULT_MAX_SIZE  )
    return FT_THROW( Array_Too_Large );
```

The same file also defines `MAX_SFNT_SIZE` as 64 MiB at
`src/sfnt/sfwoff2.c:40-41` and uses it to reject allocation bombs later in
`woff2_open_font`.
The older 30 MiB write cap remains the first effective ceiling.
Git history shows `WOFF2_DEFAULT_MAX_SIZE` has remained unchanged since 2019,
while commit `336503df` added the 64 MiB allocation-bomb cap in 2023.
Both caps remain in FreeType `origin/master` at commit `9e9d3b73f31367dbb4261f93c727a277f6632c77`.

For untransformed tables,
`src/sfnt/sfwoff2.c:1632-1638` routes each write through that capped helper:

```c
checksum = compute_ULong_sum( transformed_buf + table.src_offset,
                              table.src_length );
/* ... */
if ( WRITE_SFNT_BUF( transformed_buf + table.src_offset,
                     table.src_length ) )
  goto Fail;
```

Transformed `glyf` reconstruction follows the same output path through
`reconstruct_glyf` at `src/sfnt/sfwoff2.c:1644-1657`.
The `WRITE_SFNT_BUF` macro expands directly to `write_buf`,
but the caller tests its return value without assigning it to the local `error`.
When that write fails,
`src/sfnt/sfwoff2.c:1747-1749` therefore maps the still-unset local error to `Invalid_Table`:

```c
Fail:
  if ( !error )
    error = FT_THROW( Invalid_Table );
```

`include/freetype/fterrdef.h:75-80` renders error `0x08` as `broken table`.
A traced FreeType 2.14.3 build therefore reports:

```text
NotoSansCJK-wght-400-900.ttf.woff2 8 broken table
NotoSerifCJK-wght-400-900.ttf.woff2 8 broken table
```

The trace accepts both WOFF2 headers and prints
`WOFF2 collection directory is valid` before reconstruction.
For Noto Sans CJK,
the last successful trace event computes the checksum of the 14,254,066-byte `gvar` table.
The next source operation writes that table;
its cumulative end offset exceeds 30 MiB,
so `write_buf` rejects it before the trace can continue to `head`.
For Noto Serif CJK,
the trace stops inside transformed `glyf` reconstruction.
That table's expanded length is 34,358,908 bytes,
which is consistent with the same cumulative cap,
but the precise inner write was not separately instrumented.

This is not ZIP corruption.
Both files pass the source ZIP's CRC check,
and their installed SHA-256 values are:

```text
4907a1c69fcfc2d81640c9ecd8fadcad2956cf462c70e6ae2730e2ec27219f2a  NotoSansCJK-wght-400-900.ttf.woff2
e08e55f2b1e3d54e5b54cac2dfe6e608b695270d70b3ed9dba6870e91fd4147b  NotoSerifCJK-wght-400-900.ttf.woff2
```

## Verification

The behavior was verified against:

- Fedora package `fontconfig-2.17.0-4.fc44.x86_64`;
  source tag `2.17.0`,
  commit `a7a03d6ce093042d1ad4c5cf68746f44b03d8bda`.
- Fedora package `freetype-2.14.3-1.fc44.x86_64`;
  source tag `VER-2-14-3`,
  commit `0a0221a1347e2f1e07c395263540026e9a0aa7c7`.
- `noto-cjk-main.zip` files copied byte-for-byte into the user font directory.

The direct probe is:

```bash
# doc/troubleshooting/fontconfig-noto-cjk-woff2.md
font_dir="$HOME/.local/share/fonts/noto-complete/noto-cjk-main/noto-cjk-main/android"

for stem in NotoSansCJK-wght-400-900 NotoSerifCJK-wght-400-900; do
  fc-scan --format='%{family[0]}\t%{variable}\n' "$font_dir/$stem.ttf.ttc"
  printf 'TTC status=%s\n' "$?"

  fc-scan --format='%{family[0]}\t%{variable}\n' "$font_dir/$stem.ttf.woff2"
  printf 'WOFF2 status=%s\n' "$?"
done
```

### Inputs that scan cleanly

- `NotoSansCJK-wght-400-900.ttf.ttc`:
  status 0,
  35 patterns,
  5 variable patterns.
- `NotoSerifCJK-wght-400-900.ttf.ttc`:
  status 0,
  35 patterns,
  5 variable patterns.
- A 47,208-byte Roboto WOFF2 variable font from the installed ublue documentation assets:
  status 0,
  10 patterns.

### Inputs that fail

- `NotoSansCJK-wght-400-900.ttf.woff2`:
  status 1,
  zero patterns.
- `NotoSerifCJK-wght-400-900.ttf.woff2`:
  status 1,
  zero patterns.

Setting `FC_FONTATIONS=1` produces the same status 1 and zero-pattern result in this Fontconfig build.
The installed build does not expose the optional Fontations fallback for these files.

The source trace was reproduced in a Fedora 44 container limited to 2 GiB of memory and 2 CPUs.
FreeType was built from tag `VER-2-14-3` with Brotli and `FT_DEBUG_LEVEL_TRACE` enabled.
`FT_New_Face` returned error 8 for both CJK WOFF2 files after the trace accepted each collection directory.

## Verified workarounds

### Use the paired variable TTC files for desktop font discovery

Keep the WOFF2 files for format completeness,
but let Fontconfig use the matching `.ttf.ttc` files:

```bash
# doc/troubleshooting/fontconfig-noto-cjk-woff2.md
font_dir="$HOME/.local/share/fonts/noto-complete/noto-cjk-main/noto-cjk-main/android"
fc-scan --brief "$font_dir/NotoSansCJK-wght-400-900.ttf.ttc"
fc-scan --brief "$font_dir/NotoSerifCJK-wght-400-900.ttf.ttc"
fc-cache --force "$HOME/.local/share/fonts/noto-complete"
```

This is the installed workaround.
Both TTCs are variable fonts and provide the same Sans and Serif CJK regional families.

Tradeoff:
the WOFF2 copies consume disk space but remain unavailable to desktop applications through Fontconfig.
They are retained because the font installation requested every archive format.

## What does not work

- **Refreshing the cache.**
  `fc-cache --force` cannot index a file that `FT_New_Face` rejects.
- **Setting `FC_FONTATIONS=1`.**
  The installed Fontconfig build still returns status 1 with no pattern.
- **Re-downloading the ZIP.**
  The ZIP passes CRC validation,
  and the failure is deterministic after FreeType accepts the WOFF2 collection directory.
- **Treating the WOFF2 files as corrupt because error 8 says `broken table`.**
  The traced call crosses the fixed output cap;
  the generic failure label comes from the `Fail` block's fallback error.

## Upstream filing artifact

### Upstream filing decision

No `.out-of-scope/` entry covers Fontconfig,
FreeType,
WOFF2,
or font installation.
The check covered every file currently in `.out-of-scope/`.

A web search over indexed FreeType GitLab issue URLs,
the issue-disabled GitHub mirror,
and the Noto CJK tracker found no exact report.
The authoritative GitLab search page presented an anti-bot challenge,
so this is not claimed as an exhaustive tracker query.
The search was widened to peer decoders:
[google/woff2#141][google-woff2-141] remains open for failed WOFF2 collection decompression,
and [khaledhosny/ots#219][ots-219] records the same inherited 30 MiB family of limits for large CJK fonts.
Neither thread contains this FreeType call chain or these Noto CJK files.

The filing constraints resolve as follows:

1.  **Is it really upstream's fault?**
    No for the capability behavior.
    FreeType intentionally applies a documented suggested output maximum as a decoder safety boundary.
    Fontconfig correctly delegates parsing and cannot create patterns after `FT_New_Face` fails.
    The misleading `broken table` fallback is a diagnostic-quality issue,
    but it does not cause the rejection.
2.  **Can upstream fix it?**
    Yes.
    FreeType could reconcile the 30 MiB write cap with its later 64 MiB allocation-bomb cap,
    make the boundary configurable,
    or preserve `Array_Too_Large` through the reconstruction failure path.
3.  **Are they supporting this use case?**
    Partly.
    FreeType documents WOFF2 as a supported compressed SFNT format,
    and its source parses WOFF2 collection directories.
    The fixed cap explicitly excludes larger expanded output.
4.  **Would the repository welcome a contribution?**
    Yes.
    FreeType's `README` directs detailed bug reports to its GitLab tracker and larger patches to merge requests.
    `README`,
    `README.git`,
    the repository policy files,
    and the current developer page contain no ban on external or AI-assisted contributions.
5.  **Will they likely fix it?**
    No positive signal was found.
    The 30 MiB cap has remained unchanged from its 2019 introduction through current `origin/master`.
    The comparable Google WOFF2 collection issue has remained open since 2021.
6.  **Have we prototyped a minimal upstream fix compatible with their architecture?**
    No.
    The auto-prototype condition does not apply because constraints 1 and 5 fail.
    Raising a decoder safety limit without a memory-risk review would not be a verified fix.

The decision is to file nothing upstream.
The paired TTC files solve the desktop installation at the consumer boundary.
The following draft is retained only if FreeType's size-limit policy changes or a maintainer requests a reproducer.

### Draft, do not file as-is

~~~md
Title: `FT_New_Face` reports `broken table` for valid Noto CJK WOFF2 collections after crossing the 30 MiB output cap

## Description

FreeType 2.14.3 accepts the headers and collection directories of these Noto CJK files:

- `NotoSansCJK-wght-400-900.ttf.woff2`
- `NotoSerifCJK-wght-400-900.ttf.woff2`

It then returns error 8 (`Invalid_Table`, rendered as `broken table`) while reconstructing face 0.
Fontconfig's `fc-scan` consequently returns status 1 with no patterns.

The source path is:

1. Fontconfig `FcFreeTypeQueryAll` calls `FT_New_Face`
   (`fontconfig/src/fcfreetype.c:2175-2179`).
2. FreeType `write_buf` rejects cumulative output over
   `WOFF2_DEFAULT_MAX_SIZE`, fixed at 30 MiB
   (`src/sfnt/sfwoff2.h:37-38`, `src/sfnt/sfwoff2.c:226-228`).
3. The reconstruction `Fail` block maps the unset local error to `Invalid_Table`
   (`src/sfnt/sfwoff2.c:1747-1749`).

The Sans trace stops at the `gvar` write whose cumulative end offset exceeds 30 MiB.
The Serif trace stops inside reconstruction of a 34,358,908-byte `glyf` table,
consistent with the same cap;
the exact inner Serif write was not instrumented.
Both WOFF2 headers declare expanded SFNT sizes below FreeType's separate 64 MiB `MAX_SFNT_SIZE` allocation-bomb limit.

## Reproduction

```bash
fc-scan --brief NotoSansCJK-wght-400-900.ttf.woff2
echo "$?" # 1

fc-scan --brief NotoSerifCJK-wght-400-900.ttf.woff2
echo "$?" # 1
```

A trace-enabled FreeType 2.14.3 build reports:

```text
WOFF2 collection directory is valid.
NotoSansCJK-wght-400-900.ttf.woff2 8 broken table
WOFF2 collection directory is valid.
NotoSerifCJK-wght-400-900.ttf.woff2 8 broken table
```

## Suggested investigation

Please clarify whether the 30 MiB cumulative `write_buf` cap should remain lower than
the later 64 MiB `MAX_SFNT_SIZE` cap.
If rejection is intended,
preserve `Array_Too_Large` so callers do not receive the misleading `broken table` diagnostic.
If larger WOFF2 collections are supported,
reconcile the limits with tests covering these two files and allocation-bomb bounds.
~~~

[google-woff2-141]: https://github.com/google/woff2/issues/141
[ots-219]: https://github.com/khaledhosny/ots/issues/219
