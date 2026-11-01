# Bundled application fonts

The application embeds the official variable roman and real italic fonts for both families.
JetBrains Mono is the source font.
Inter Variable is the UI font.
System fonts provide fallback outside their glyph coverage.

## Assets and provenance

Inter files come from [Inter v4.1][inter]:

- `InterVariable.ttf`:
  archive path `InterVariable.ttf`.
- `InterVariable-Italic.ttf`:
  archive path `InterVariable-Italic.ttf`.

Both advertise family name `Inter Variable`,
with `Regular` and `Italic` styles respectively.
The release archive is v4.1;
the files' internal version string is `Version 4.001;git-9221beed3`.
The archive's `LICENSE.txt` is preserved as `Inter-LICENSE.txt`.

JetBrains Mono files come from [v2.304][mono]:

- `JetBrainsMono-Variable.ttf`:
  archive path `fonts/variable/JetBrainsMono[wght].ttf`.
- `JetBrainsMono-VariableItalic.ttf`:
  archive path `fonts/variable/JetBrainsMono-Italic[wght].ttf`.

Both advertise family `JetBrains Mono`,
with `Regular` and `Italic` styles respectively.
Only the filesystem names were normalized;
font contents and internal names are unmodified.
The archive's `OFL.txt` is preserved as `JetBrainsMono-OFL.txt`.

Both families use SIL Open Font License 1.1.
No file is subsetted or internally renamed.

## Variable axes and genuine styles

Inspection of the packaged `fvar` tables found:

- JetBrains Mono roman and italic:
  `wght` 100 to 800,
  default 400.
- Inter Variable roman and italic:
  `wght` 100 to 900,
  default 400;
  `opsz` 14 to 32,
  default 14.

Italic is a separate real font design for both families,
not a slant transform of the roman font.
Tests request intermediate weights such as 527.5 and 537.5,
check the exact selected font bytes and normalized coordinates,
and reject synthesized emboldening or skew for these primary faces.
The source raster cache is tested across weights with the same font blob identity.

Native widget snapshots verify repainting for weight/style requests
and distinguish the bundled families from an unregistered-family fallback control.
Pixel differences alone do not establish which font bytes were selected;
exact face and synthesis assertions belong to the font-stack tests.
An additional native test exercises the actual source-image and selection bindings,
changes weight and italic state,
and retains the middle character selected within `===`.
The ordinary source view currently requests roman weight 400;
syntax classifications currently change colors,
not weight or italic style.
The test injects typography through the existing shaper boundary,
without adding user-facing font controls.

Inter's optical-size axis is verified with explicit 14 and 32 controls.
The current Slint `FontRequest` interface does not carry arbitrary variation settings:
its ordinary UI request leaves `opsz` at the font default 14.
Variable weight and real italic selection are supported through its weight/style properties.
Do not label the current UI behavior automatic optical sizing.
The explicit-axis tests distinguish available font capability from toolkit policy.

## Shaping and feature policy

JetBrains Mono's programming ligatures are in `calt`,
not `liga`,
according to the [upstream feature catalog][mono-features].
The source shaper explicitly enables `calt`.
Tests compare actual glyph IDs against a disabled control,
retain operator advances,
and verify caret/hit/copy boundaries inside ligatures.
Optional stylistic sets and character variants are not indiscriminately enabled.

Monochrome glyph foreground is clipped against source-character selection geometry.
It is not a text brush applied to an entire glyph:
that previously left unselected-colored ink inside a partially selected ligature.
The observed-failing regression selected the middle character of `===`.
The corrected path preserves glyph identities and checks clipping,
alpha,
and real mixed-color boundary pixels at fractional scales and origins.
Color-font glyphs retain their intrinsic RGBA rather than being tinted by selection foreground;
real color-font selection is not covered by these monochrome-family acceptance tests.

The [Inter project][inter-site] and [v4.1 feature samples][inter-features] distinguish
contextual alternates,
discretionary ligatures,
kerning,
number forms,
and character/style variants.
Tests verify default `kern` and `calt`,
opt-in `dlig`,
and tabular/proportional numeral controls.
No font-settings panel was added.

An observed-failing native window-event test verifies bitmap regeneration after idle scale changes.
It covers factors 2,
1.25,
and a return to 1 on the headless backend.
Live physical-output scale migration remains unverified;
the nested compositor currently advertises a fixed output scale.

## SHA-256

- `InterVariable.ttf`:
  `4989b125924991b90d05b2d16e0e388c48f7d5bb8b30539bbf9c755278d0ccaf`.
- `InterVariable-Italic.ttf`:
  `d6f1f6a172d9e588438db9f986fd5cfad7b30f644374080a8a9d4d91e344586f`.
- `JetBrainsMono-Variable.ttf`:
  `662a196d58f1183bf2d77428b6d5283fe3f45161ab021bea4036bc98e5cac016`.
- `JetBrainsMono-VariableItalic.ttf`:
  `f115aaa12113718c02ce72864fe6823b87241bc23d3e44cf1220155f861063f2`.
- `Inter-LICENSE.txt`:
  `262481e844521b326f5ecd053e59b98c8b2da78c8ee1bdbb6e8174305e54935a`.
- `JetBrainsMono-OFL.txt`:
  `30f0c136e3c88e422d0791acd97238870f9054a9729bc34cf2ff0d4ed8cac4ad`.

## Superseded static subset

`Inter-Regular.ttf` and `Inter-SemiBold.ttf` were replaced on 2026-10-04
by `InterVariable.ttf` and the newly included `InterVariable-Italic.ttf`.
`JetBrainsMono-Regular.ttf` was replaced on the same date
by `JetBrainsMono-Variable.ttf` and the newly included `JetBrainsMono-VariableItalic.ttf`.
Those static files are no longer bundled.
The previous statement that current UI weights were covered did not justify omitting variable axes and real italics.

Historical checksums:

- `Inter-Regular.ttf`:
  `40d692fce188e4471e2b3cba937be967878f631ad3ebbbdcd587687c7ebe0c82`.
- `Inter-SemiBold.ttf`:
  `78a843fade9d4612a5567302fb595b56976eb5fcebf4fea5a5912d638bafcde3`.
- `JetBrainsMono-Regular.ttf`:
  `a0bf60ef0f83c5ed4d7a75d45838548b1f6873372dfac88f71804491898d138f`.

[inter]: https://github.com/rsms/inter/releases/download/v4.1/Inter-4.1.zip
[mono]: https://github.com/JetBrains/JetBrainsMono/releases/download/v2.304/JetBrainsMono-2.304.zip
[mono-features]: https://github.com/JetBrains/JetBrainsMono/wiki/OpenType-features
[inter-site]: https://rsms.me/inter/
[inter-features]: https://github.com/rsms/inter/blob/v4.1/docs/_data/feature_samples.yml
