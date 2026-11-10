# IntelliJ IDEA 2026.2.1 on Bazzite 44: en-US fallback lets Droid Sans Fallback or Noto Sans CJK JP win

## Symptom

The desired behavior is to retain the current Latin fonts while using a chosen
Noto Sans CJK regional face for CJK characters throughout IntelliJ IDEA.
No error is emitted.
The visible failure is an unexpected CJK family or regional glyph form.

The trigger is an unset application-specific fallback combined with the
`en_US.UTF-8` startup locale.
The measured surfaces differ:

- `Editor | Font` has no configured fallback in
  `$HOME/.config/JetBrains/IntelliJIdea2026.2/options/editor-font.xml`.
  IntelliJ therefore reaches its logical monospaced fallback.
  The current JBR cache records `Noto Sans CJK JP` for that fallback.
- Swing UI fonts can reach JBR's sans-serif composite.
  Its current cache records `Droid Sans Fallback` before
  `Noto Sans CJK JP`.
  The active theme's concrete UI-family hop was not probed directly.
- The terminal has its own empty `SECONDARY_FONT_FAMILY` in
  `$HOME/.config/JetBrains/IntelliJIdea2026.2/options/terminal-font.xml`.

The installed Noto collection exposes regional families rather than one
region-neutral `Noto Sans CJK` family:

- `Noto Sans CJK SC`:
   Simplified Chinese
- `Noto Sans CJK TC`:
   Traditional Chinese for Taiwan
- `Noto Sans CJK HK`:
   Traditional Chinese for Hong Kong
- `Noto Sans CJK JP`:
   Japanese
- `Noto Sans CJK KR`:
   Korean

The same suffixes exist for `Noto Sans Mono CJK`.
Noto's upstream README describes these as language-specific fonts and says
that switching regional glyphs through OpenType `locl` requires an application
that supplies language tags.
Plain source text in an editor does not reliably supply such tags.
One fallback therefore needs one deliberate regional default.

## Root cause

### Fedora's CJK aliases are language-gated

Bazzite 44 enables Fedora's
`/usr/share/fontconfig/conf.avail/65-1-google-noto-sans-cjk-fonts.conf`.
Its rules prepend a regional family only when the font pattern carries the
matching language.
For Simplified Chinese,
 lines 101 to 109 contain:

```xml
<match>
  <test name="lang">
    <string>zh-cn</string>
  </test>
  <test name="family">
    <string>sans-serif</string>
  </test>
  <edit name="family" mode="prepend">
    <string>Noto Sans CJK SC</string>
  </edit>
</match>
```

The same file maps `ja` to JP at lines 21 to 30,
`ko` to KR at lines 55 to 64,
`zh-tw` to TC at lines 189 to 198,
and `zh-hk` to HK at lines 289 to 298.

The current process locale is `en_US.UTF-8`.
Measured matches show the consequence:

```text
sans-serif:lang=en-us:charset=4e2d -> Noto Sans CJK JP
sans-serif:lang=zh-cn:charset=4e2d -> Noto Sans CJK SC
sans-serif:lang=zh-tw:charset=4e2d -> Noto Sans CJK TC
sans-serif:lang=zh-hk:charset=4e2d -> Noto Sans CJK HK
sans-serif:lang=ja:charset=4e2d    -> Noto Sans CJK JP
sans-serif:lang=ko:charset=4e2d    -> Noto Sans CJK KR
```

### JBR passes its startup locale into fontconfig

The installed runtime is JBR `25.0.3+9-b508.16-nomod`.
Its `release` file reports source commit
`c624f1bd958763cf442320ee570b5ad468b226bb+`.
The `+` means the shipped build may contain uncommitted build-tree changes beyond
that public commit.
The public commit carries tag `jbr-release-25.0.3b508.16`,
and the cited fontconfig path matches the installed behavior.

`JetBrainsRuntime@c624f1bd:src/java.desktop/unix/classes/sun/font/FontConfigManager.java:142-149`
builds the fontconfig locale from JBR's startup locale:

```java
private static String getFCLocaleStr() {
    Locale l = SunToolkit.getStartupLocale();
    String localeStr = l.getLanguage();
    String country = l.getCountry();
    if (!country.isEmpty()) {
        localeStr = localeStr + "-" + country;
    }
    return localeStr;
}
```

The manager passes that value into native font setup at line 191:

```java
setupFontConfigFonts(getFCLocaleStr(), fcInfo, fontArr, includeFallbacks);
```

`JetBrainsRuntime@c624f1bd:src/java.desktop/unix/native/libawt/awt/fontconfigmanager.c:548-558`
adds the locale to each pattern and asks fontconfig for an ordered set:

```c
if (locale != NULL) {
    (*fcPatternAddString)(pattern, FC_LANG, (unsigned char*)locale);
}
(*fcConfigSubstitute)(NULL, pattern, FcMatchPattern);
(*fcDefaultSubstitute)(pattern);
fontset = (*fcFontSort)(NULL, pattern, FcTrue, NULL, &result);
```

JBR builds its logical composite from that order.
`JetBrainsRuntime@c624f1bd:src/java.desktop/share/classes/sun/font/CompositeGlyphMapper.java:114-122`
checks slots from the start and returns the first non-missing glyph:

```java
for (int slot = 0; slot < font.numSlots; slot++) {
    if (!hasExcludes || !font.isExcludedChar(slot, unicode)) {
        CharToGlyphMapper mapper = getSlotMapper(slot);
        glyphCode = mapper.charToVariationGlyphRaw(unicode, variationSelector);
        if (glyphCode != mapper.getMissingGlyphCode()) {
            glyphCode = font.compositeGlyphCode(slot, glyphCode);
            if (variationSelector == 0) setCachedGlyphCode(unicode, glyphCode);
            return glyphCode;
        }
    }
}
```

The baseline JBR cache at
`$HOME/.java/fonts/25.0.3/fcinfo-1-bazzite-RedHat-44-en-US.properties`
records these relevant plain-style entries:

```text
monospaced.0.24.fullName=Noto Sans CJK JP
sansserif.0.14.fullName=Droid Sans Fallback
sansserif.0.32.fullName=Noto Sans CJK JP
```

The cache's property order is lexical,
 so the numeric suffix is the component
index,
 not the file line's visual position.

### Mixed CJK plain text has no automatic regional answer

Unicode assigns one CJK unified ideograph code point to character identities
that can have different conventional glyphs in Chinese,
Japanese,
 and Korean typography.
The Unicode CJK FAQ says that determining a character's language from its code
point is largely impossible.
It recommends language context or heuristics instead.

Noto Sans CJK contains glyphs for all supported regional conventions.
Noto's deployment guide says switching conventions requires text language tags
and an application that applies OpenType `locl` substitutions.
The regional family selects the default convention when no such tag reaches the
shaper.

JBR does not carry a text language through this layout path.
`JetBrainsRuntime@c624f1bd:src/java.desktop/share/classes/sun/font/GlyphLayout.java:395-415`
hardcodes its language value to the default before splitting text into script
and font runs:

```java
int lang = -1; // default for now
// ...
while (_scriptRuns.next()) {
    int limit = _scriptRuns.getScriptLimit();
    int script = _scriptRuns.getScriptCode();
    while (_fontRuns.next(limit)) {
        // ...
        nextEngineRecord(start, pos, script, lang, pfont, slot, slotShift);
    }
}
```

The exact JBR HarfBuzz bridge also sets a default language.
`JetBrainsRuntime@c624f1bd:src/java.desktop/share/native/libfontmanager/HBShaper.c:226-234`
contains:

```c
hb_buffer_t *create_buffer(int script, int ltrDirection) {
    hb_buffer_t *buffer = hb_buffer_create();
    // ...
    hb_buffer_set_script(buffer, getHBScriptCode(script));
    hb_buffer_set_language(buffer, hb_ot_tag_to_language(HB_OT_TAG_DEFAULT_LANGUAGE));
    // ...
}
```

A fontconfig alias can therefore choose a better regional default,
but it cannot make one editor line use JP forms and another use SC forms.
An ordered list of regional faces also cannot solve this:
each face covers the shared Han code points,
so JBR's first matching face wins.

### IntelliJ's editor override is separate from Linux fallback

IntelliJ IDEA's exact source tag is `idea/2026.2.1`,
commit `b75ab523e6adbe1d26112219729eacbcfd24daa0`.
The source citations use these aliases under that commit:

- `editor-fonts/`:
  `platform/platform-impl/src/com/intellij/application/options/editor/fonts/`
- `editor-colors-impl/`:
  `platform/editor-ui-ex/src/com/intellij/openapi/editor/colors/impl/`
- `editor-impl/`:
  `platform/platform-impl/src/com/intellij/openapi/editor/impl/`

`editor-fonts/AppFontOptionsPanel.kt:225-230`
creates the editor's explicit fallback control:

```kotlin
val secondaryFont = JLabel(ApplicationBundle.message("secondary.font"))
setSecondaryFontLabel(secondaryFont)
row(secondaryFont) {
  cell(secondaryCombo)
    .widthGroup("TypographySettingsCombo")
    .comment(ApplicationBundle.message("label.fallback.fonts.list.description"))
}
```

The selected value persists as `SECONDARY_FONT_FAMILY`.
`editor-colors-impl/AppFontOptions.java:84-85`
registers it after the primary family:

```java
if (state.SECONDARY_FONT_FAMILY != null) {
  fontPreferences.register(state.SECONDARY_FONT_FAMILY, fontSize);
}
```

`editor-impl/ComplementaryFontsRegistry.java:168-191`
tries registered families in order,
 then the logical monospaced fallback:

```java
List<String> fontFamilies = preferences.getEffectiveFontFamilies();
for (int i = 0, len = fontFamilies.size(); i < len; ++i) {
  final String fontFamily = fontFamilies.get(i);
  result = doGetFontAbleToDisplay(codePoint, preferences.getSize2D(fontFamily), style, fontFamily,
                                  i == 0 ? preferences.getRegularSubFamily() : null,
                                  i == 0 ? preferences.getBoldSubFamily() : null,
                                  useLigatures, variants, context, true, true);
  if (result != null) {
    return result;
  }
}
// ...
result = doGetFontAbleToDisplay(codePoint, size, style, DEFAULT_FALLBACK_FONT,
                                null, null, useLigatures, Collections.emptySet(),
                                context, false, false);
```

The UI is a different path.
`intellij-community@b75ab523:platform/util/ui/src/com/intellij/util/ui/StartupUiUtil.kt:215-225`
uses Swing's fallback-aware `StyleContext` on Linux:

```kotlin
val fontWithFallback = if (OS.CURRENT == OS.macOS || GraphicsEnvironment.isHeadless()) {
  Font(familyName, style, size.toInt()).deriveFont(size)
}
else {
  StyleContext().getFont(familyName, style, size.toInt()).deriveFont(size)
}
return fontWithFallback as? FontUIResource ?: FontUIResource(fontWithFallback)
```

This split explains why an editor fallback fixes editor text but not every
UI or terminal path.
The verified Linux rule changes JBR's generic sans and monospace composites;
it does not change the serif composite.

## Verification

Verified on 2026-08-11 with:

- Bazzite `44.20260721.0`,
   based on Fedora 44
- IntelliJ IDEA Ultimate `2026.2.1`,
   build `262.9437.185`
- JBR `25.0.3+9-b508.16-nomod`
- fontconfig `2.17.0`
- Noto Sans CJK `2.004`

### Baseline harness

```bash
locale

fc-match --format='%{family[0]} | %{file}\n' \
  'sans-serif:lang=en-us:charset=4e2d'
fc-match --format='%{family[0]} | %{file}\n' \
  'sans-serif:lang=zh-cn:charset=4e2d'

for pattern in 'sans:regular:roman' 'monospace:regular:roman'; do
  printf '\n%s\n' "$pattern"
  fc-match --sort --format='%{family[0]} | %{file}\n' "$pattern" \
    | awk '!seen[$0]++ { print ++count ": " $0 }' \
    | grep -E '^(1:|[0-9]+: (Noto Sans CJK|Noto Sans Mono CJK|Droid Sans Fallback))'
done
```

Baseline output from the installed font set:

```text
sans:regular:roman
1: Noto Sans
17: Droid Sans Fallback
52: Noto Sans CJK JP

monospace:regular:roman
1: Noto Sans Mono
43: Noto Sans CJK JP
```

### Language-tagged shaping harness

Direct HarfBuzz shaping confirms that the installed TTC contains the regional
forms and that language metadata can select them:

```bash
font=/usr/share/fonts/google-noto-sans-cjk-fonts/NotoSansCJK-Regular.ttc
for face in 0 2; do
  for lang in en ja zh-cn; do
    printf 'face=%s lang=%s: ' "$face" "$lang"
    hb-shape --shapers=ot --face-index="$face" \
      --language="$lang" --script=Hani --direction=ltr \
      "$font" '直骨令辻'
  done
done
```

Face `0` is JP and face `2` is SC.
The output changes glyphs when a non-default language is supplied:

```text
face=0 lang=en: [gid27873=0+1000|gid45132=1+1000|gid9808=2+1000|gid40043=3+1000]
face=0 lang=ja: [gid27873=0+1000|gid45132=1+1000|gid9808=2+1000|gid40043=3+1000]
face=0 lang=zh-cn: [gid27874=0+1000|gid45133=1+1000|gid9809=2+1000|gid40044=3+1000]
face=2 lang=en: [gid27874=0+1000|gid45133=1+1000|gid9809=2+1000|gid40044=3+1000]
face=2 lang=ja: [gid27873=0+1000|gid45132=1+1000|gid9808=2+1000|gid40043=3+1000]
face=2 lang=zh-cn: [gid27874=0+1000|gid45133=1+1000|gid9809=2+1000|gid40044=3+1000]
```

The distinct glyph IDs are a positive control for the regional difference.
They also show the better mechanism:
one Pan-CJK font plus language-tagged runs.
The JBR source trace shows why IntelliJ's plain editor cannot use that mechanism
in this version.

### Candidate fontconfig harness

The verified candidate keeps the current generic Latin families first and
places one regional CJK family immediately after each.
The sample selects Simplified Chinese.
Replace both `SC` suffixes together for another region.

```xml
<?xml version="1.0"?>
<!DOCTYPE fontconfig SYSTEM "urn:fontconfig:fonts.dtd">
<fontconfig>
  <alias>
    <family>sans-serif</family>
    <prefer>
      <family>Noto Sans</family>
      <family>Noto Sans CJK SC</family>
    </prefer>
  </alias>
  <alias>
    <family>monospace</family>
    <prefer>
      <family>Noto Sans Mono</family>
      <family>Noto Sans Mono CJK SC</family>
    </prefer>
  </alias>
</fontconfig>
```

With that file loaded from a disposable XDG config root,
the same `fc-match --sort` harness produced:

```text
sans:regular:roman
1: Noto Sans
2: Noto Sans CJK SC
18: Droid Sans Fallback

monospace:regular:roman
1: Noto Sans Mono
5: Noto Sans Mono CJK SC
```

A disposable JBR home then generated this cache:

```text
monospaced.0.2.fullName=Noto Sans Mono CJK SC
sansserif.0.1.fullName=Noto Sans CJK SC
```

The matching bold and italic composites also selected the same regional
families.
The baseline and candidate harnesses are positive controls for each other:
the same installed fonts produce JP or Droid ordering without the candidate
and SC ordering with it.

### Catalog that works

- An exact family query such as `Noto Sans CJK SC:charset=4e2d` resolves to
  the SC face at TTC index `2`.
- A language-bearing query such as
  `sans-serif:lang=zh-cn:charset=4e2d` resolves to SC.
- Direct HarfBuzz shaping selects JP or SC forms from the installed Pan-CJK TTC
  when the text run carries `ja` or `zh-cn`.
- The candidate alias chain preserves `Noto Sans` and `Noto Sans Mono` as
  the first generic families and moves the chosen CJK families ahead of
  Droid and JP.
- IntelliJ `Editor | Font | Fallback font` stores and tries the selected
  editor fallback before logical Linux fallback.

### Catalog that does not meet the goal

- A bare `Noto Sans CJK` query resolves to Latin `Noto Sans` on this system
  because no installed family has that exact region-neutral name.
- An `en-us` generic CJK query resolves to JP.
- Leaving the editor and terminal fallback fields empty delegates those
  surfaces to logical fallback order.
- Setting only the editor fallback does not change Swing UI text.
- Listing several regional CJK faces does not select among them by text
  language.
  Shared Han code points are present in each face,
  so the first face wins.
- Installing a Pan-CJK OTC or variable font does not make plain editor text
  language-aware.
  The installed TTC already has `locl` forms,
  but JBR supplies the default language to HarfBuzz.

## Verified workarounds

There is no automatic mixed-Japanese-and-Chinese workaround for plain text in
this IntelliJ/JBR version.
Choose a regional default only when one convention is preferable for untagged
text.
For content with language metadata,
use a renderer that preserves tags such as `ja`,
`zh-Hans`,
`zh-Hant`,
 or `ko` so the font's `locl` forms can work.

### Per-launch font language default

Fontconfig documents `FC_LANG` as the query's weak default language.
For a monolingual IDEA session,
it can activate Fedora's installed regional rules without changing `LANG` or
writing custom XML:

Fully quit existing IDEA processes,
then launch:

```bash
FC_LANG=ja \
  "$HOME/.local/share/JetBrains/Toolbox/apps/intellij-idea/bin/idea"
```

Use `zh-cn`,
`zh-tw`,
`zh-hk`,
 or `ko` instead when appropriate.
A disposable exact-JBR probe with `LANG=en_US.UTF-8` measured:

```text
FC_LANG=ja
sans-serif -> Noto Sans CJK JP
monospaced -> Noto Sans Mono CJK JP

FC_LANG=zh-cn
sans-serif -> Noto Sans CJK SC
monospaced -> Noto Sans Mono CJK SC
```

Tradeoffs:

- This still selects one whole-process regional default.
  It cannot distinguish adjacent Japanese and Chinese runs.
- Fedora prepends the regional Noto family at slot `0`.
  Because that family also covers Latin,
  it can change Latin rendering rather than acting only as CJK fallback.
- Launching through Toolbox without the environment variable returns to the
  normal `en-US` ordering.

### Forced regional default for JBR sans and monospace fallback

Use the candidate XML as
`$HOME/.config/fontconfig/conf.d/50-noto-cjk-fallback.conf`.
Fontconfig's installed `50-user.conf:5-12` explicitly loads both
`$XDG_CONFIG_HOME/fontconfig/conf.d` and
`$XDG_CONFIG_HOME/fontconfig/fonts.conf`,
 so no system file needs editing.

After creating the file,
 refresh fontconfig and restart IntelliJ IDEA:

```bash
fc-cache --force --verbose
```

Verify the order with the baseline `fc-match --sort` harness.
After IntelliJ restarts,
 open a file containing representative CJK text and
run `Show Fonts Used by Editor` from `Find Action`.
The expected regional family is `Noto Sans CJK SC` in this sample.

Tradeoffs:

- This does not correctly typeset mixed untagged Japanese and Chinese.
  An SC default gives shared Han characters SC forms even inside Japanese text;
  a JP default does the converse for Chinese text.
- This is user-wide Linux configuration,
   not IntelliJ-only configuration.
  Other fontconfig clients inherit the generic family chain.
- It intentionally pins the current generic Latin choices,
  `Noto Sans` and `Noto Sans Mono`,
   before the CJK fallbacks.
- One selected region becomes the default for shared Han code points.
  Text from another CJK language can use that region's glyph form unless
  the rendering surface supplies a language tag.
- Noto Sans CJK also covers arrows,
   box drawing,
   geometric shapes,
  fullwidth forms,
   and other symbols.
  Moving it near the start of the sans chain can make it render those characters
  before dedicated symbol fonts.
- The rule does not alter JBR's `serif` composite.
  A surface that explicitly requests serif retains its previous CJK fallback.
- Existing processes and JBR's generated font cache require a restart after
  `fc-cache` changes the cache directory timestamp.

Rollback:

```bash
rm -- "$HOME/.config/fontconfig/conf.d/50-noto-cjk-fallback.conf"
fc-cache --force --verbose
```

Restart IntelliJ IDEA after rollback.

### IntelliJ editor fallback only

Open `Settings | Editor | Font`,
 expand `Typography Settings`,
 and set
`Fallback font` to the desired `Noto Sans CJK <region>` family.
For a terminal,
 set the separate fallback under
`Settings | Tools | Terminal | Font Settings` to
`Noto Sans Mono CJK <region>`.

Tradeoffs:

- This is deterministic and limited to the configured IntelliJ surface.
- It does not fix CJK in menus,
   tabs,
   tool windows,
  or every console path.
- `Noto Sans CJK` is proportional for Latin.
  That does not matter when it supplies only missing CJK glyphs,
  while the Mono family is safer for terminal cell geometry.

## What does not work

- **Changing the UI custom font to Noto Sans CJK.**
  This changes Latin UI text too and is not a fallback-only setting.
- **Appending the CJK family weakly or with an `accept` alias.**
  The tested sans-serif order still placed `Droid Sans Fallback` before SC.
- **Strongly prepending only the CJK family.**
  The tested order made Noto Sans CJK the first generic family,
  so it also became the Latin font.
- **Changing `LANG` as a mixed-language fix.**
  It successfully selects Fedora's family for that locale,
  but still supplies one whole-process default.
  It can also affect localization,
  collation,
   formatting,
   and child processes.
  `FC_LANG` is narrower when a monolingual launch is useful.
- **Editing `/usr/share/fontconfig/conf.avail`.**
  That path is package-managed and is not a durable Bazzite customization.
- **Deleting only `$HOME/.java/fonts/25.0.3`.**
  It forces JBR to regenerate the same order if fontconfig itself is unchanged.

## Upstream filing artifact

### Upstream filing decision

No `.out-of-scope/` entry covers IntelliJ,
 JBR,
 fontconfig,
 or CJK fallback.
GitHub searches across open and closed JetBrainsRuntime issues and pull requests
for `fontconfig CJK fallback` returned no match.
Equivalent searches in `JetBrains/intellij-community` for
`fallback font CJK` returned no match.
JetBrains YouTrack searches found font and input-method reports but no duplicate
for this configuration behavior.

The six constraints are:

1.  **Is it really upstream's fault?**
    No.
    Fedora supplies language-gated regional aliases,
    JBR correctly forwards its `en-US` startup locale and consumes fontconfig's
    ordered result,
    and IntelliJ offers an explicit editor override.
    The undesired result comes from an unspecified regional preference across
    those supported layers.
2.  **Can upstream fix it?**
    Upstream could add more policy or UI controls,
    but there is no correctness defect to fix without choosing a regional
    preference on the user's behalf.
3.  **Are they supporting this use case?**
    Yes.
    Fontconfig documents user aliases,
    JBR consumes fontconfig,
    and IntelliJ exposes editor and terminal fallback controls.
4.  **Would the repositories welcome a contribution?**
    JetBrainsRuntime's `CONTRIBUTING.md` points contributors to the OpenJDK guide.
    IntelliJ's `CONTRIBUTING.md` welcomes reproducible bug fixes and requires a
    YouTrack ticket.
    Neither checked policy contains an AI-assistance ban.
5.  **Will they likely fix it?**
    No upstream change is indicated because constraint 1 fails.
    The supported user configuration already expresses the missing preference.
6.  **Has a minimal upstream fix been prototyped?**
    No.
    The auto-prototype gate does not trigger because constraints 1 and 5 fail.
    A consumer-side fontconfig configuration was prototyped and verified instead.

Nothing should be filed or added upstream.
There is no issue or comment draft because the measured behavior is expected,
the supported configuration solves it,
and the duplicate searches found no thread that needs additional evidence.

## Sources

- JetBrains IDEA 2026.2 font settings:
  <https://www.jetbrains.com/help/idea/settings-editor-font.html>
- JetBrains IDEA 2026.2 terminal settings:
  <https://www.jetbrains.com/help/idea/settings-tools-terminal.html>
- Fontconfig `fonts-conf` reference:
  <https://fontconfig.pages.freedesktop.org/fontconfig/fontconfig-user>
- Noto CJK deployment and regional-family guide:
  <https://github.com/notofonts/noto-cjk/blob/main/Sans/README.md>
- Unicode CJK FAQ on regional glyph conventions:
  <https://unicode.org/faq/han_cjk.html>
