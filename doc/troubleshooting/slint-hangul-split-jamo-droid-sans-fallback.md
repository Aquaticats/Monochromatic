# Slint 1.18.1 (parley 0.11.1, fontique 0.11.1): a Hangul syllable such as `소` renders as two separate jamo
when fontconfig has no CJK preference and picks Droid Sans Fallback

The desktop music player draws the folder `방탄소년단` as `방탄ㅅㅗ년단`: the syllable `소` (U+C18C) shows as the two jamo
`ㅅ` and `ㅗ` side by side with a gap. Both Slint renderers (femtovg and software) draw it the same way, so it is not a
renderer bug. It is a font selection result that depends on the fontconfig configuration of the process.

## Symptom

- Finding OV4 of the second desktop polish review: `소` in the folder picker "renders as two separate jamo with a gap"
  (`s1--10-picker-end.png`, `s1--crops-hangul.png` in the review folder).
- The neighbouring syllables `방`, `탄`, `년`, `단` render correctly, so a reader sees a half-broken word.
- The on-disk name is precomposed (NFC): the bytes of the first character are `eb b0 a9`, and `소` is `ec 86 8c`
  (`ls | head -1 | od -An -tx1`). The cause is not Unicode normalization of the name.
- The same text renders correctly in the same binary when the process sees the user's own fontconfig configuration
  (`~/.config/fontconfig/conf.d/50-noto-cjk-fallback.conf`). The review sessions ran the app with scratch `HOME` and XDG
  directories, which hid that file.

## Root cause

1. The app sets `default-font-family: "Inter Variable"` (`ui/app_window.slint`). Inter has no Hangul glyphs.
2. Slint builds the font stack of every text as the named family followed by two generic families,
so it cannot be given a
   fallback list of its own
   (`i-slint-core-1.18.1/textlayout/sharedparley/shaping.rs:86-107`, `i-slint-common-1.18.1/sharedfontique.rs:188`):

   ```rust
   // shaping.rs:86
   let mut fallback_family_iter = sharedfontique::FALLBACK_FAMILIES
       .into_iter()
       .map(parley::style::FontFamilyName::Generic);
   // sharedfontique.rs:188
   pub const FALLBACK_FAMILIES: [fontique::GenericFamily; 2] = [
       fontique::GenericFamily::SansSerif,
       fontique::GenericFamily::SystemUi,
   ];
   ```

3. For a cluster that no family of the stack covers, fontique asks fontconfig for one font for the cluster's script,
with
   a pattern made from a sample of the script's characters
   (`fontique-0.11.1/src/backend/fontconfig.rs:735-760`):

   ```rust
   let script_char_set = self.script_charsets.charset_for_script(key.script());
   ...
   if let Some(set) = script_char_set {
       pattern.add_charset(FC_CHARSET, set);
   }
   config.substitute(&mut pattern, FcMatchPattern);
   let font = config.font_match(&pattern).ok()?;
   ```

4. On a Fedora system without a user fontconfig file, `font_match` returns `Droid Sans Fallback` (inferred: the explicit
   family `Droid Sans Fallback` draws the identical split, and fontique's choice was not logged)
   (`/usr/share/fonts/google-droid-sans-fonts/DroidSansFallbackFull.ttf`).
   That font has the jamo and many syllables but not
   U+C18C (`fc-list ':charset=c18c'` lists only Noto Sans CJK and Noto Serif CJK). The shaper then decomposes the
   syllable into its jamo and draws them from Droid's jamo glyphs, which are not designed to join.
5. The user's `50-noto-cjk-fallback.conf` makes `sans-serif` prefer `Noto Sans` and `Noto Sans CJK SC`,
so the same match
   returns a Noto CJK face, which has every syllable.

Earlier reading that was wrong: "the Hangul glyph is split by the femtovg renderer".
The software renderer draws the same
split (`SLINT_BACKEND=software`, see below), and so does `slint-viewer`, which has no renderer in common with the app's
femtovg path.

## Verification

Versions: `slint 1.18.1`, `i-slint-core 1.18.1`, `fontique 0.11.1`, `parley 0.11.1` (from `Cargo.lock`), Fedora fonts
`google-droid-sans-fonts` and `google-noto-sans-cjk-fonts`.

Harness (`t2.slint` is a window with `import "InterVariable.ttf"`,
`default-font-family: "Inter Variable"` and texts that
hold `방탄소년단 소 손 선`; one text names `Noto Sans CJK KR` explicitly):

```sh
# with the real home: fine
SLINT_BACKEND=software slint-viewer --screenshot real.png t2.slint
# with a scratch home and no user fontconfig: split
HOME=$S XDG_DATA_HOME=$S SLINT_BACKEND=software slint-viewer --screenshot scratch.png t2.slint
# scratch home, but only the user's one fontconfig fragment copied in: fine again
HOME=$S XDG_DATA_HOME=$S XDG_CONFIG_HOME=$S/cfgonly SLINT_BACKEND=software slint-viewer --screenshot cfg.png t2.slint
```

Works cleanly:

- Real home and real XDG directories: `소` is one syllable.
- Scratch home with only `50-noto-cjk-fallback.conf` in `$XDG_CONFIG_HOME/fontconfig/conf.d`: one syllable.
- Any text with `font-family: "Noto Sans CJK KR"` (also `JP`, `SC`, `HK`) and with `font-family: "Noto Sans"`:
one syllable.
- `HOME` real and `XDG_DATA_HOME` scratch: one syllable (the fonts are system fonts; the user's config decides).

Fails:

- Scratch `HOME` (even with the real `XDG_DATA_HOME`): `소` as `ㅅㅗ`, for font weights 400, 500 and 700, in `slint-viewer`
  and in the app under the nested compositor, with the femtovg renderer (the app's default) and with
  `SLINT_BACKEND=software` (the app honors only a value that contains `software`, `src/main.rs`, `force_software`).
- `font-family: "Droid Sans Fallback"` set explicitly: the same split, which names the font that the fallback picked.

## Verified workarounds

- Test and review environments: keep the user's `XDG_CONFIG_HOME` (or copy the one fontconfig fragment into the scratch
  config home) so that fontconfig sees the same preference as a real session. Tradeoff: the screenshots then depend on
  one machine's fontconfig.
- A machine without Droid Sans Fallback, or with a Noto CJK preference, never shows the defect; installing a
  `sans-serif` preference such as the user's fragment is the user-side fix.

## What does not work

- Declaring a fallback family in the `.slint` files: `font-family` takes one name and Slint appends only the two generic
  families (`shaping.rs:86-107`), so a Hangul-capable family cannot be added next to `Inter Variable`.
- Naming `Noto Sans CJK KR` as the app's default family: it fixes Hangul only where that family exists, and it replaces
  Inter for every Latin text.
- Shaping settings: Slint 1.18.1 exposes no OpenType feature or locale property on `Text`, and fontique is given no
  locale, so the `lang=ko` match (which returns `Noto Sans CJK KR`) cannot be requested.
- Choosing a renderer through `SLINT_BACKEND`: both renderers show the same split.
- Bundling a Hangul font in the app: it would be used only if named as the text's family, which brings back the first
  point, and it adds megabytes for one fallback case.

## Upstream filing decision

1. Really upstream's fault? Partly. The selection takes the first fontconfig match for a script sample, and a font that
   lacks some syllables of the script can win; the missing glyph is then synthesized from jamo. Whether that is a defect
   of fontique or of the system's font set is arguable.
2. Can upstream fix it? Yes, for example by checking the chosen font against the cluster's characters
and trying the next
   match.
3. Do they support the use case? Per-script fallback is a stated feature of fontique; no document mentions partial
   syllable coverage.
4. Would the repo welcome a contribution? Not checked against `CONTRIBUTING.md` of either repository in this session.
5. Will they likely fix it? Not checked in the trackers in this session.
6. Prototype? None was made.

`.out-of-scope/` was read: it lists no Slint, fontique or font entry. Because constraints 4 to 6 were not worked,
nothing is
drafted for filing; the work stopped at the diagnosis, as the review task required for a defect that cannot be fixed in
the app.
