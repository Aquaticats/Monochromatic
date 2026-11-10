# ImageMagick 7.1.2-27 warns about an absent font but returns success during capture

## Symptom

The Fold Search screenshot sanitizer requested `-font DejaVu-Sans` on a host
where `fc-match 'DejaVu Sans'` resolved instead to
`/usr/share/fonts/google-noto/NotoSans-Regular.ttf`.
ImageMagick emitted this diagnostic for every capture:

```text
magick: unable to read font `DejaVu-Sans' @ warning/annotate.c/RenderType/1026.
```

The files were written and the `magick` process still exited with status 0.
Adding `-regard-warnings` before the input or before the output did not change
that result.
 A capture pipeline that checks only the exit status can therefore
publish a substituted clock font without noticing.

## Root cause

This is documented warning behavior,
**not an established ImageMagick bug**.
The [ImageMagick exception guide](https://imagemagick.org/exception/) says a
warning can accompany usable output;
a type warning means a font was
unavailable and substitution may have occurred.
The [command-line option reference](https://imagemagick.org/command-line-options/#regard-warnings)
says `-regard-warnings` treats **some warnings in some image formats** as errors,
not every warning.

The matching upstream source tag `7.1.2-27` resolves to commit `b661ac9`.
At `MagickCore/annotate.c:1015-1027`,
a font argument that is not an
accessible path is looked up by name;
a failed lookup emits `TypeWarning`:

```c
if (IsPathAccessible(draw_info->font) != MagickFalse)
  {
    status=RenderFreetype(image,draw_info,draw_info->encoding,offset,
      metrics,exception);
    return(status);
  }
type_info=GetTypeInfo(draw_info->font,exception);
if (type_info == (const TypeInfo *) NULL)
  (void) ThrowMagickException(exception,GetMagickModule(),TypeWarning,
    "UnableToReadFont","`%s'",draw_info->font);
```

At `MagickCore/annotate.c:1084-1114`,
rendering searches for a default
type and calls FreeType with the fallback if available:

```c
  if (type_info == (const TypeInfo *) NULL)
    {
      ExceptionInfo
        *sans_exception;

      /*
        Search for a default font.
      */
      sans_exception=AcquireExceptionInfo();
      if (type_info == (const TypeInfo *) NULL)
        type_info=GetTypeInfoByFamily((const char *) NULL,draw_info->style,
          draw_info->stretch,draw_info->weight,sans_exception);
      if (type_info == (const TypeInfo *) NULL)
        type_info=GetTypeInfo("*",sans_exception);
      sans_exception=DestroyExceptionInfo(sans_exception);
    }
  if (type_info == (const TypeInfo *) NULL)
    {
      status=RenderFreetype(image,draw_info,draw_info->encoding,offset,metrics,
        exception);
      return(status);
    }
  annotate_info=CloneDrawInfo((ImageInfo *) NULL,draw_info);
  annotate_info->face=type_info->face;
  if (type_info->metrics != (char *) NULL)
    (void) CloneString(&annotate_info->metrics,type_info->metrics);
  if (type_info->glyphs != (char *) NULL)
    (void) CloneString(&annotate_info->font,type_info->glyphs);
  status=RenderFreetype(image,annotate_info,type_info->encoding,offset,metrics,
    exception);
```

The CLI catches a nonfatal warning at `MagickWand/wandcli.c:234-240`:

```c
status=cli_wand->wand.exception->severity > ErrorException ? MagickTrue :
  MagickFalse;
if ((status == MagickFalse) || (all_exceptions != MagickFalse))
  CatchException(cli_wand->wand.exception); /* output and clear exceptions */
return(status);
```

`MagickWand/magick-cli.c:169-184` scans for `-regard-warnings` and checks
exceptions after the command returns:

```c
    if (LocaleCompare("-regard-warnings",option) == 0)
      regard_warnings=MagickTrue;
  }
  if (iterations == 1)
    {
      char
        *text;

      text=(char *) NULL;
      status=command(image_info,argc,argv,&text,exception);
      if (exception->severity != UndefinedException)
        {
          if ((exception->severity > ErrorException) ||
              (regard_warnings != MagickFalse))
            status=MagickFalse;
          CatchException(exception);
```

The caught warning no longer survives as a fatal error.
The command-level return at `MagickWand/magick-cli.c:1480` also checks only
current exception severity:

```c
  return(exception->severity < ErrorException ? MagickTrue : MagickFalse);
```

These source paths explain the observed warning plus successful exit.
They do **not** show that ImageMagick promises an all-warnings-fatal switch.

## Verification

- Installed binary:
  `magick -version` reported ImageMagick
  `7.1.2-27 (Beta) Q16-HDRI`.
  Source audit used `ImageMagick/ImageMagick` tag `7.1.2-27`,
  commit `b661ac9`,
  in a read-only clone.
- Working catalog:

  ```sh
  fc-match 'sans-serif' --format '%{file}\n'
  magick -size 64x64 xc:white \
    -font /usr/share/fonts/google-noto/NotoSans-Regular.ttf \
    -pointsize 12 -annotate +1+15 test /home/user/temp/agent/font-ok.png
  ```

  The sanitizer uses the installed font resolved by `fc-match` and completed
  its full Fold screenshot batch without a font diagnostic.
- Warning catalog:

  ```sh
  magick -size 64x64 xc:white -font DejaVu-Sans \
    -pointsize 12 -annotate +1+15 test -regard-warnings \
    /home/user/temp/agent/font-warning.png
  ```

  On this host the quoted `UnableToReadFont` warning appeared while the
  command returned status 0.
- Consumer guard positive control:
  `MUSIC_PLAYER_REVIEW_FONT=DejaVu-Sans` with the throwaway task
  `mise run //package/music-player/design:prototype:sanitize:search`
  failed on its first image with
  `ImageMagick status 0; magick: unable to read font ...`.
  That task exists only on branch `prototype/music-player-theme-compose`
  (commit `948894d3c4` on 2026-09-24),
  and `main`'s `package/music-player/design/mise.toml` declares no `prototype:*` task.
  The output pointed at a disposable directory,
  not the user's review assets.
  Repeating the task with the installed font succeeded.

`package/music-player/design/sanitize-search-review.mjs` exists only on an unmerged branch (newest
commit `7fda864849` (fix(music-player-design):
reject warning-tainted review images) on 2026-09-24)
and is not in the working tree.
`mise run //package/music-player/design:prototype:sanitize:search` is not declared today;
git last
carries it in `948894d3c4` on 2026-09-24,
and `package/music-player/design/mise.toml` has no
sanitize task now.

## Verified workarounds

- Resolve a real font file with `fc-match sans-serif --format '%{file}'`,
  then pass that absolute path to `magick -font`.
  This may use a different
  family on another host,
  so it preserves legibility but not pixel-identical
  typography across machines.
- In a capture pipeline where **any warning** invalidates visual evidence,
  capture ImageMagick stderr and reject nonempty output even if the exit
  status is 0.
  `package/music-player/design/sanitize-search-review.mjs` on branch
  `prototype/music-player-theme-compose` implements this with `spawnSync`.
  The tradeoff is deliberate:
  harmless warnings also stop capture.

## What does not work

- `-regard-warnings` as an all-warnings-fatal switch:
  both tested option positions still emitted the font warning and exited 0.
  The official option reference restricts its promise to some warnings in
  some image formats.
- Checking only `execFileSync` success:
  it observes process status 0,
  so it does not reject the substituted-font output.
- Naming an uninstalled family as though it were a portable font file:
  `fc-match` fell back to Noto Sans on the tested host.

## Upstream filing decision

No `.out-of-scope/` entry matches ImageMagick font warnings.
`gh search issues` and `gh search prs` with `regard-warnings font` in
`ImageMagick/ImageMagick` found no matching GitHub thread.
No upstream filing is warranted:

1.  Upstream fault:
    No established fault.
    The documented behavior allows
    usable output with a warning and narrows `-regard-warnings` to some cases.
2.  Fixability:
    The CLI could offer a new all-warnings-fatal mode,
    but that is a separate feature request,
    not a demonstrated regression.
3.  Supported use case:
    Font fallback is documented;
    an all-warning failure
    guarantee is not.
4.  Contribution policy:
    `.github/CONTRIBUTING.md`,
    `.github/ISSUE_TEMPLATE/bug-report.yml`,
    `.github/PULL_REQUEST_TEMPLATE.md` and tracker results were checked.
    They allow ordinary external reports and do not state an AI-assisted ban,
    but there is no bug to report.
5.  Expected upstream response:
    No matching issue or maintainer position
    establishes demand for an all-warnings-fatal option.
6.  Tested upstream fix:
    None.
    Constraints 1 and 3 fail;
    the source audit supports a consumer-side guard,
    not an upstream bug patch.

### Draft, do not file as-is

~~~md
ImageMagick font warning during image annotation

ImageMagick 7.1.2-27 emits a TypeWarning for an unavailable named font and
may substitute another installed family while returning success.
The documentation describes usable output with warnings,
and `-regard-warnings` does not claim to make every warning fatal.
Our capture pipeline instead uses an installed font path and rejects stderr.
No upstream action is requested.
~~~
