# IntelliJ platform: a scheme colour of `0` is `000000`, and the `Islands Dark` parent theme exists only from 2025.2

Two assumptions a third-party theme built on the IntelliJ Islands family can get wrong,
both of them silent:

- a colour literal shorter than six digits is not off-format:
  it is a hex integer,
  so `0` is `000000`,
  and `FFF` is blue;
- the parent theme a plugin names in `parentTheme` is a bundled theme,
  so the plugin's declared `since-build` decides whether that parent exists at all.

## Symptom

A theme plugin written against the bundled Islands theme shows both shapes:

- `scheme/Islands_Black.xml` writes `BLOCK_TERMINAL_BLACK` `FOREGROUND` as `value='0'`,
  while every other colour in the same file is six digits,
  and a review reads that as a malformed literal that either fails to load or means black;
- `META-INF/plugin.xml` declares `since-build='233'` (2023.3),
  and the theme JSON says `"parentTheme": "Islands Dark"`,
  which asks an IDE older than that theme family for a parent it cannot have.

Neither failure prints anything:
the scheme loads,
and the theme either resolves its parent or does not.

## Root cause

### A scheme colour value is a hex integer, not a six-digit string

`AttributesFlyweightSerDe.readColor` reads each colour option.
An eight-character value is offered to `ColorHexUtil.fromHexOrNull` for its alpha channel,
and everything else falls through to `Integer.parseInt(value, 16)`:

```java
// platform/core-api/src/com/intellij/openapi/editor/markup/AttributesFlyweightSerDe.java
private static @Nullable Color readColor(@NotNull Element element, @NotNull @NonNls String fieldName) throws InvalidDataException {
  String value = JDOMExternalizerUtil.readField(element, fieldName);
  if (value == null) {
    return null;
  }
  if (value.length() == RGBA_HEX_LENGTH) {
    Color color = ColorHexUtil.fromHexOrNull(value);
    if (color != null) {
      return color;
    }
  }
  try {
    return new Color(Integer.parseInt(value, 16));
  } catch (NumberFormatException e) {
    throw new InvalidDataException("Wrong color value: " + value, e);
  }
}
```

[`AttributesFlyweightSerDe.java#L140-L156`](https://github.com/JetBrains/intellij-community/blob/38a0ffad46cbcaa799bcd90a20142a007f4a5fe5/platform/core-api/src/com/intellij/openapi/editor/markup/AttributesFlyweightSerDe.java#L140-L156)

`Integer.parseInt("0", 16)` is `0`,
so the literal is a valid colour value meaning `000000`,
and a neighbouring comment states the intent:
fully opaque colours are stored as an unpadded `RRGGBB` number.

The same fallback makes short literals misleading rather than invalid:
`FFF` parses as `0x000FFF`,
a blue,
because the integer is left-padded,
not right-padded.

`ColorHexUtil` only serves the alpha-channel branch,
and its accepted lengths are why a short literal looks rejected if you stop at that file:

```java
// platform/util/src/com/intellij/ui/ColorHexUtil.java
if (len == 3) return new Color(fromHex1(str, pos), fromHex1(str, pos + 1), fromHex1(str, pos + 2), 255);
...
if (len == 6) return new Color(fromHex2(str, pos), fromHex2(str, pos + 2), fromHex2(str, pos + 4), 255);
...
return null;
```

[`ColorHexUtil.java#L28-L37`](https://github.com/JetBrains/intellij-community/blob/38a0ffad46cbcaa799bcd90a20142a007f4a5fe5/platform/util/src/com/intellij/ui/ColorHexUtil.java#L28-L37)

Rendering then follows whichever slot the parse filled.
A terminal ANSI colour uses the attribute's foreground,
then its background,
then the terminal default foreground:

```kotlin
// platform/terminal/src/com/intellij/terminal/TerminalColorPalette.kt
override fun getForegroundByColorIndex(colorIndex: Int): Color {
  val attributes = getAttributesByColorIndex(colorIndex)
  val color = attributes?.foregroundColor ?: attributes?.backgroundColor
  return if (color != null) {
    AwtTransformers.fromAwtColor(color)!!
  }
  else {
    ...
    defaultForeground
```

[`TerminalColorPalette.kt#L16-L28`](https://github.com/JetBrains/intellij-community/blob/38a0ffad46cbcaa799bcd90a20142a007f4a5fe5/platform/terminal/src/com/intellij/terminal/TerminalColorPalette.kt#L16-L28)

So an unset colour is visible,
because it inherits the default foreground,
and a colour that parses as black is invisible on a black console background.
`0` is the second case,
which makes the black-on-black report correct for the wrong reason:
it is not the literal's shape,
it is the value the literal really means.

JetBrains writes the same value in its own schemes,
which is the strongest evidence that ANSI black is deliberately black:

```xml
<!-- platform/platform-resources/src/DefaultColorSchemesManager.xml, scheme "Darcula" -->
<option name="BLOCK_TERMINAL_BLACK">
  <value>
    <option name="FOREGROUND" value="0" />
    <option name="BACKGROUND" value="1e1f22" />
  </value>
</option>
```

[`DefaultColorSchemesManager.xml#L1888-L1893`](https://github.com/JetBrains/intellij-community/blob/38a0ffad46cbcaa799bcd90a20142a007f4a5fe5/platform/platform-resources/src/DefaultColorSchemesManager.xml#L1888-L1893),
the same file's light scheme does it at line 1264,
and the visible shade sits in the bright slot:

```xml
<option name="BLOCK_TERMINAL_BLACK_BRIGHT">
  <value>
    <option name="FOREGROUND" value="4E5157" />
    <option name="BACKGROUND" value="4E5157" />
```

[`DefaultColorSchemesManager.xml#L1936-L1940`](https://github.com/JetBrains/intellij-community/blob/38a0ffad46cbcaa799bcd90a20142a007f4a5fe5/platform/platform-resources/src/DefaultColorSchemesManager.xml#L1936-L1940)

ANSI black is black by definition,
and bright black carries the readable dark grey.

### The Islands theme family is younger than a 2023.3 `since-build`

A `parentTheme` names a theme some other provider registers,
so it resolves only on IDE lines that ship that provider.
Release branches `233`,
`241`,
`243` and `251` have no
`platform/platform-resources/src/themes/islands/` directory at all,
while branch `252` has it,
with the dark member naming itself:

```json
// platform/platform-resources/src/themes/islands/ManyIslandsDark.theme.json
{
  "name": "Islands Dark",
  "dark": true,
```

[`ManyIslandsDark.theme.json#L1-L6`](https://github.com/JetBrains/intellij-community/blob/4f7a2fa69cf0d5ec94e7baab73ace3d2ec19a06e/platform/platform-resources/src/themes/islands/ManyIslandsDark.theme.json#L1-L6)

The theme id is real on that line too,
not only the file name:

```kotlin
// platform/platform-impl/src/com/intellij/openapi/application/impl/islands/IslandsThemeRemapper.kt
"Many Islands Dark", "One Island Dark" -> "Islands Dark"
```

[`IslandsThemeRemapper.kt#L9-L11`](https://github.com/JetBrains/intellij-community/blob/4f7a2fa69cf0d5ec94e7baab73ace3d2ec19a06e/platform/platform-impl/src/com/intellij/openapi/application/impl/islands/IslandsThemeRemapper.kt#L9-L11)

What 2025.2 does not do is make the Islands family the platform default;
2025.3 does,
which is what the filing issue cites as IJPL-212710:

```kotlin
// platform/platform-impl/src/com/intellij/ide/ui/laf/LafManagerImpl.kt, branch 252
private data object DefaultNewUiThemeStrategy : DefaultThemeStrategy {
  override fun getPlatformDefaultId(isDark: Boolean) = if (isDark) "ExperimentalDark" else "ExperimentalLight"
...
private data object DefaultClassicThemeStrategy : DefaultThemeStrategy {
  override fun getPlatformDefaultId(isDark: Boolean) = if (isDark) "Darcula" else "JetBrainsLightTheme"
```

[`LafManagerImpl.kt#L1465-L1473`](https://github.com/JetBrains/intellij-community/blob/4f7a2fa69cf0d5ec94e7baab73ace3d2ec19a06e/platform/platform-impl/src/com/intellij/ide/ui/laf/LafManagerImpl.kt#L1465-L1473)

So the honest floor for a theme whose parent is `Islands Dark` is the 2025.2 line,
not 2023.3.
The registration is what decides that,
because a directory of theme files is not a provider:
branch `252` declares the provider,
and branches `251` and `243` declare no provider with that id at all:

```xml
<!-- platform/platform-resources/src/META-INF/PlatformExtensions.xml, branch 252 -->
<themeProvider id="Islands Dark" path="/themes/islands/ManyIslandsDark.theme.json" targetUi="islands"/>
```

[`PlatformExtensions.xml#L701`](https://github.com/JetBrains/intellij-community/blob/4f7a2fa69cf0d5ec94e7baab73ace3d2ec19a06e/platform/platform-resources/src/META-INF/PlatformExtensions.xml#L701),
and branch `253` keeps the same entry at line 716.

`targetUi="islands"` does not narrow that to Islands UI sessions.
A parent is resolved by id across every registered provider,
with no `targetUi` filter:

```kotlin
// platform/platform-impl/src/com/intellij/ide/ui/laf/UiThemeProviderListManager.kt, branch 252
private fun findParentTheme(themes: Collection<LafEntry>, parentId: String?): UITheme? {
  return if (parentId == null) null else themes.firstOrNull { it.id == parentId }?.theme?.get()?.theme
}
```

[`UiThemeProviderListManager.kt#L180-L182`](https://github.com/JetBrains/intellij-community/blob/4f7a2fa69cf0d5ec94e7baab73ace3d2ec19a06e/platform/platform-impl/src/com/intellij/ide/ui/laf/UiThemeProviderListManager.kt#L180-L182)

The same file filters the theme *chooser* by `targetUi`,
so a plugin theme that declares none is grouped with the classic themes and shown
under either UI.
That is why a third-party theme can inherit `Islands Dark` on 2025.2 and still
appear in the list of an Islands session on 2025.3.

## An earlier reading was wrong

The 2026-10-17 review that filed these findings read the literal as a defect,
asking for a six-digit literal,
and reasoned that `0` either fails to load in a reader that expects `RRGGBB`,
or silently means `#000000`.
The first branch is false:
`readColor` accepts it and returns black.
The second branch is true,
and is the whole point of the key.
The plugin's `README` now states the deliberate black instead of leaving it implicit.

## Verification

Version under test:
intellij-community `master` at `38a0ffad46cbcaa799bcd90a20142a007f4a5fe5`,
release branch `252` at `4f7a2fa69cf0d5ec94e7baab73ace3d2ec19a06e`,
release branch `253` at `6865954b26c532021319aab770abdda0b218ef9b`.
No JetBrains IDE is installed on this host,
so the colour results below are read from the parser and palette source,
not from a running IDE.

Presence of the Islands themes per release line,
reproducible with one call per line:

```bash
# presence probe for the Islands theme directory, one release line per call
gh api "repos/JetBrains/intellij-community/contents/platform/platform-resources/src/themes/islands?ref=251"   # 404
gh api "repos/JetBrains/intellij-community/contents/platform/platform-resources/src/themes/islands?ref=252"   # 3 files
# provider registration, one release line per call
gh repo clone JetBrains/intellij-community "$HOME/temp/agent/intellij-community-252" -- --depth 1 --branch 252 --single-branch
rg -n 'themeProvider id="Islands Dark"' "$HOME/temp/agent/intellij-community-252/platform/platform-resources/src/META-INF/PlatformExtensions.xml"
```

Colour literals,
by what `Integer.parseInt(value, 16)` does with them:

- `0` and `000000` parse to black,
  and are indistinguishable at render time;
- `1e1e1e` and `4E5157` parse to the greys they name;
- `FFF` parses to `0x000FFF`,
  a blue rather than white;
- `#FFF`,
  `#000000`,
  an empty string and any non-hex text throw `InvalidDataException` with
  `Wrong color value: <value>`,
  so a `#` prefix never reaches a colour in a scheme file;
- an absent colour option is not an error:
  the slot stays unset,
  and the parent scheme or the terminal default foreground supplies the colour.

The scheme-shape half of this is now enforced in this repository by
`package/intellij-plugin/islands-black/src/scheme.unit.test.ts`,
which reads the shipped `scheme/Islands_Black.xml`,
and fails with the offending `NAME=value` when a colour is not six hex digits.

## Verified workarounds

Write every scheme colour as six hex digits.
There is no correctness tradeoff,
because six digits is the only length that means exactly the red,
green and blue bytes named;
the gain is that a reader never has to reconstruct `Integer.parseInt`
to know what a value is.
The plugin applies this in `package/intellij-plugin/islands-black/scheme/Islands_Black.xml`.

State the theme's real floor in `since-build`,
or provide a second theme JSON with a parent older lines do have.
The plugin took the first:
`META-INF/plugin.xml` declares build `252`,
and its `README` says 2025.2+.
Tradeoff of the first:
users on 2023.3 through 2025.1 can no longer install the plugin,
which is a support decision,
not a mechanical one.
Tradeoff of the second:
two theme files and two palettes to keep in step,
and the older one cannot inherit the Islands colour system it overrides.

## What does not work

- Reading `ColorHexUtil` alone and concluding that a one-digit colour value is rejected:
  that helper is reached only for eight-character values.
- Writing `#RRGGBB` in a scheme XML:
  the `#` is not stripped by this parser.
- Treating a black ANSI foreground as a defect:
  the platform's own dark scheme does the same,
  and bright black is the ANSI slot for readable dark text.
- Expecting an unparseable colour to be dropped rather than repaired:
  it throws at load,
  so the scheme file fails loudly instead of silently falling back.
- Assuming 2025.3 is the first line with the theme files:
  2025.2 already ships them.

## Upstream filing artifact

Nothing to add.
Both behaviours are JetBrains' own shipped behaviour,
exercised by `DefaultColorSchemesManager.xml` in the same repository,
and the Islands theme line is a documented product rollout rather than a defect.
The defect is in how a consuming plugin declares compatibility,
which is ours to fix.
