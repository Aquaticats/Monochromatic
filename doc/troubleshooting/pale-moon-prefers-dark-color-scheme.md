# Pale Moon 34.3.1 exposes light colors until its web-page color scheme is set to dark

## Symptom

Web pages using the CSS `prefers-color-scheme` media feature receive a light preference,
even when Pale Moon itself uses a dark theme.
The page-visible probe is:

```javascript
matchMedia('(prefers-color-scheme: dark)').matches;
```

Pale Moon controls this result independently from its application theme and operating-system theme.
The setting is available at:

`Preferences > Content > Fonts & Colors > Colors > Web Page Color Scheme > Use dark colors`

The corresponding profile preference is:

```javascript
user_pref("browser.display.prefers_color_scheme", 2);
```

## Root cause

This is intentional product behavior,
 not a failure to detect the desktop theme.
Pale Moon defaults the preference to light and gives the user explicit control.

UXP commit `659c690d5b34cc3e46c5ba8a6e00f134d8d20c35`,
used by Pale Moon's `34.3.1_Release` tag,
defines the values at `modules/libpref/init/all.js:278-282`:

```javascript
// 0 = feature disabled
// 1 = default: light theme preferred
// 2 = dark theme preferred
// 3 = match ui.color_scheme
pref("browser.display.prefers_color_scheme", 1);
```

Pale Moon commit `0d869b85feca1409f5aadb55e6eaabb08db134ad`,
tagged `34.3.1_Release`,
binds the Colors dialog's dark radio button to value `2` at
`palemoon/components/preferences/colors.xul:102-112`:

```xml
<radiogroup id="prefersColorSchemeSelection"
            preference="browser.display.prefers_color_scheme">
  <radio value="1"
         label="&prefersColorSchemeLight.label;"
         accesskey="&prefersColorSchemeLight.accesskey;"/>
  <radio value="2"
         label="&prefersColorSchemeDark.label;"
         accesskey="&prefersColorSchemeDark.accesskey;"/>
```

The media-query evaluator reads that preference at
`layout/style/nsMediaFeatures.cpp:577-588` in the same UXP commit:

```cpp
static nsresult
GetPrefersColorScheme(nsPresContext* aPresContext, const nsMediaFeature* aFeature,
          nsCSSValue& aResult)
{
  switch(Preferences::GetInt("browser.display.prefers_color_scheme", 1)) {
    case 1:
      aResult.SetIntValue(NS_STYLE_PREFERS_COLOR_SCHEME_LIGHT,
                          eCSSUnit_Enumerated);
      break;
    case 2:
      aResult.SetIntValue(NS_STYLE_PREFERS_COLOR_SCHEME_DARK,
                          eCSSUnit_Enumerated);
      break;
```

The call chain is direct:
the Colors dialog stores integer value `2`,
`GetPrefersColorScheme` reads it,
and the CSS media feature receives the dark enumerated value.

## Verification

Verified with installed Pale Moon 34.3.1,
build ID `20260622224538`.
The release source was checked at Pale Moon tag `34.3.1_Release`
(commit `0d869b85feca1409f5aadb55e6eaabb08db134ad`),
whose `platform` submodule points to UXP commit
`659c690d5b34cc3e46c5ba8a6e00f134d8d20c35`.

A disposable profile contained:

```javascript
user_pref("browser.display.prefers_color_scheme", 2);
```

The test page set its title from both media queries:

```html
<!doctype html>
<meta charset="utf-8">
<title>CHECKING-PREFERS-COLOR-SCHEME</title>
<script>
const dark = matchMedia("(prefers-color-scheme: dark)").matches;
const light = matchMedia("(prefers-color-scheme: light)").matches;
document.title = dark && !light
  ? "PALEMOON-DARK-PREFERENCE-PASS"
  : `PALEMOON-DARK-PREFERENCE-FAIL-dark-${dark}-light-${light}`;
</script>
```

It was launched separately from the user's profile:

```bash
MOZ_ENABLE_WAYLAND=0 GDK_BACKEND=x11 palemoon \
  --no-remote \
  --profile "$scratch/profile" \
  "file://$scratch/check.html"
xdotool search --sync --name 'PALEMOON-DARK-PREFERENCE' getwindowname %@
```

Observed output:

```text
PALEMOON-DARK-PREFERENCE-PASS - Pale Moon
```

### Settings that satisfy the request

- Colors dialog:
   `Use dark colors`.
- Direct profile value:
   `browser.display.prefers_color_scheme = 2`.
- Page result:
   dark query `true`,
   light query `false`,
   confirmed by the live title probe.

### Settings that do not satisfy the request

- An absent preference uses the source-defined default value `1`,
   which exposes light.
- `browser.display.prefers_color_scheme = 1` explicitly exposes light.
- Selecting a dark Pale Moon application theme alone does not set the web-page preference.
  The inspected profile selected the `blackmoon` theme but had no
  `browser.display.prefers_color_scheme` user value before this change.

## Verified workarounds

No workaround is needed because Pale Moon provides the setting directly.
When GUI automation is unavailable and Pale Moon is closed,
adding this line to the active profile's `prefs.js` has the same semantics as selecting the radio button:

```javascript
user_pref("browser.display.prefers_color_scheme", 2);
```

Tradeoff:
manual `prefs.js` editing bypasses the preference dialog and is overwritten if performed while Pale Moon is running.
The GUI is safer when the browser is open.

A persistent `user.js` entry also works on browser startup:

```javascript
user_pref("browser.display.prefers_color_scheme", 2);
```

Tradeoff:
`user.js` reapplies its value at every startup,
so later GUI changes do not remain effective after the next restart.

## What does not work

- Editing `prefs.js` while Pale Moon is running.
  Its own file header warns that the application overwrites such changes when it exits.
- Changing only the browser theme.
  Application appearance and page-visible color preference are separate controls.
- Assuming Pale Moon mirrors the operating-system scheme by default.
  The default is explicitly light.
  The project describes the independent choice as a fingerprinting safeguard in the
  [Pale Moon forum explanation][forum-explanation].

## Upstream filing artifact

### Upstream filing decision

The repository's `.out-of-scope/` directory contains no Pale Moon exemption.
Searches of Pale Moon and UXP issues and pull requests for
`prefers-color-scheme`,
`prefers_color_scheme`,
and `Use dark colors` found no matching defect.
The [existing forum thread][forum-explanation] already documents the intended control and exact GUI path.

1.  **Is it really upstream's fault?**
     No.
    The source,
     GUI,
     and runtime result agree.
2.  **Can upstream fix it?**
     Not applicable because there is no defect.
3.  **Are they supporting this use case?**
     Yes.
    The Colors dialog exposes the required choice.
4.  **Would the repo welcome our contribution?**
     Yes in general.
    `docs/CONTRIBUTING.md` in UXP requests focused issues and associated pull requests,
    and no AI-assistance prohibition was found.
5.  **Will they likely fix it?**
     No fix is needed.
    Changing the default would reverse an explicit privacy and user-choice decision.
6.  **Have we prototyped a minimal fix compatible with their architecture?**
     No.
    Constraints 1 and 5 fail,
    so the prototype requirement does not apply.

Nothing should be filed upstream.
There is no missing behavior or additive evidence for the existing forum explanation.

[forum-explanation]: https://forum.palemoon.org/viewtopic.php?f=62&t=28448
