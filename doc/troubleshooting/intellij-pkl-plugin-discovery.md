# Pkl IntelliJ Plugin 0.36.1 is not discoverable in JetBrains Marketplace and is hidden from IDEA before 2024.3

Note 2026-11-09T16:56:27-05:00:
`.idea/pklSettings.xml` left the tree on 2026-07-11 in `122c5dbd2a`.

## Repository status

Issue `#357` removed this repository's sole Pkl input,
root `hk.pkl`,
and obsolete `.idea/pklSettings.xml` after cli-git replaced hk.
No Pkl editor integration is now required for Monochromatic.
The external plugin findings remain durable evidence for any future Pkl use;
they are not current repository setup instructions.

Pkl support for IntelliJ IDEA is real,
 but it is not bundled into IDEA and the
official Apple plugin is not published in the ordinary JetBrains Marketplace
feed.
 The supported install path is a custom GitHub plugin repository.
 The
latest official plugin metadata also requires IntelliJ build `243` or newer,
which maps to IDEA 2024.3 or newer,
 so older IDEA installs can look as though
Pkl is unsupported even after the custom repository is added.

## Symptom

Opening a `.pkl` file,
 for example this repo's `hk.pkl`,
 in IntelliJ IDEA can
show plain text or unknown-file behavior.
 Searching only the built-in plugin
Marketplace can also fail to find the official Apple plugin.

Two common variants produce the same user-facing impression:

- The official custom plugin repository has not been added,
   so IDEA never sees
  Apple's plugin feed.
- The latest plugin feed is visible,
   but IDEA is older than 2024.3,
   so the
  current plugin is incompatible and the plugin manager hides or rejects it.

At diagnosis time,
the repository's only Pkl input was `./hk.pkl`.
Issue `#357` later removed it.

## Root cause

The Pkl IntelliJ support lives in Apple's separate `pkl-intellij` plugin,
 not in
IDEA itself.
 In the cloned upstream repository
`apple/pkl-intellij@0f9ddad9354484d42bf3d8a6ec8295484b52eafe`,
 the README says:

```adoc
// README.adoc:12-13
Pkl language support for the IntelliJ Platform.
Compatible with **IntelliJ 2023.1** or higher and **Pkl 0.25.0** or higher.
```

However,
 the install instructions do not send users to the default Marketplace
alone.
 They first add Apple's GitHub-hosted update repository:

```adoc
// doc/modules/ROOT/pages/installation.adoc:3-6
. Select menu:Settings[Plugins > ⚙ > Manage Plugin Repositories].
. Add the url `https://github.com/apple/pkl-intellij/releases/latest/download/updatePlugins.xml`
  to the list of custom plugin repositories.
. Select menu:Settings[Plugins > Marketplace].
. Search for and install the _pkl_ plugin.
```

The latest published update XML advertises a stricter current compatibility
range than the README headline.
 The release feed says build `243` minimum and
describes the current plugin as IntelliJ 2024.3 or newer:

```xml
<!-- release/updatePlugins.xml:7-12 -->
<idea-version since-build="243" until-build="263.*" />
<name>Pkl</name>
<description><![CDATA[
<p>
Language support for <a href="https://github.com/apple/pkl">Pkl</a>.
Compatible with <strong>IntelliJ 2024.3</strong> or higher and <strong>Pkl 0.25</strong> or higher.
```

The changelog explains when that minimum moved:

```adoc
// doc/modules/ROOT/pages/changelog.adoc:99-101
=== Breaking Changes

* The minimum version of IntelliJ and other JetBrains editors has been bumped to 2024.3
  (https://github.com/apple/pkl-intellij/pull/125[#125]).
```

The plugin itself does register Pkl file extensions,
 so once a compatible plugin
is installed,
 `.pkl` and `.pcf` files are supposed to be associated with the Pkl
language:

```xml
<!-- src/main/resources/META-INF/plugin.xml:144-145 -->
<fileType name="Pkl" implementationClass="org.pkl.intellij.PklFileType"
          fieldName="INSTANCE" language="Pkl" extensions="pkl;pcf" fileNames="PklProject"/>
```

The missing support is therefore a plugin publication,
 repository,
 or IDE-version
compatibility problem,
 not absence of a Pkl file-type registration in the plugin.

The Marketplace publication gap is already tracked upstream in
`apple/pkl-intellij#4`.
 `gh issue view 4 --repo apple/pkl-intellij` showed:

- The issue title is `Publish plugin to JetBrains Marketplace (https://plugins.jetbrains.com/)`.
- The issue remains `OPEN`.
- A collaborator wrote on 2024-02-09,
   `Agree. We are working on it.`
- A maintainer wrote on 2024-02-27 that Marketplace plugin `23729-pkl` was not
  from Apple and they were asking that publisher to remove it.
- A contributor wrote on 2026-03-29,
   `We are still working on this and are closer
  to it than we were, but we still have no ETA.`

## Verification

Version under test:

- `apple/pkl-intellij@0f9ddad9354484d42bf3d8a6ec8295484b52eafe`,
   cloned 2026-06-08.
- Official current docs page for IntelliJ Plugin `0.36.1`.
- Official latest plugin feed at
  `https://github.com/apple/pkl-intellij/releases/latest/download/updatePlugins.xml`.

The diagnosis-time repository inventory found only `./hk.pkl`.
After issue `#357`,
`rg --files . --glob '*.pkl'` returns no tracked path.

The latest official feed returns plugin `0.36.1` with IntelliJ build `243`
minimum:

```shell
curl --silent --show-error --location \
  https://github.com/apple/pkl-intellij/releases/latest/download/updatePlugins.xml \
  | head --lines 24
```

```xml
<plugin
    id="org.pkl-lang"
    url="https://github.com/apple/pkl-intellij/releases/download/0.36.1/pkl-intellij-0.36.1.zip" version="0.36.1"
>
  <idea-version since-build="243" until-build="263.*" />
  <name>Pkl</name>
  <description><![CDATA[
<p>
Language support for <a href="https://github.com/apple/pkl">Pkl</a>.
Compatible with <strong>IntelliJ 2024.3</strong> or higher and <strong>Pkl 0.25</strong> or higher.
```

Older release metadata shows why some old IDEA versions need an older plugin
instead of the current feed:

```shell
for version in 0.33.0 0.34.0 0.36.1; do
  printf '%s ' "$version"
  curl --silent --show-error --location \
    "https://github.com/apple/pkl-intellij/releases/download/$version/updatePlugins.xml" \
    | grep --only-matching '<idea-version[^>]*/>'
done
```

```text
0.33.0 <idea-version since-build="231" until-build="252.*" />
0.34.0 <idea-version since-build="243" until-build="252.*" />
0.36.1 <idea-version since-build="243" until-build="263.*" />
```

Duplicate search for the Marketplace gap found the existing upstream issue and
no closed replacement:

```shell
gh search issues --repo apple/pkl-intellij "JetBrains Marketplace" \
  --state open --json number,title,state,url,createdAt --limit 10
gh search issues --repo apple/pkl-intellij "JetBrains Marketplace" \
  --state closed --json number,title,state,url,createdAt --limit 10
```

```json
[
  {
    "createdAt":"2024-02-05T17:55:41Z",
    "number":4,
    "state":"open",
    "title":"Publish plugin to JetBrains Marketplace (https://plugins.jetbrains.com/)",
    "url":"https://github.com/apple/pkl-intellij/issues/4"
  }
]
[]
```

Patterns that work cleanly:

- IDEA 2024.3 or newer can use the official latest custom repository URL and
  install current plugin `0.36.1`.
- IDEA builds `231` to `252.*` can use release `0.33.0` metadata,
   but this is an
  older plugin release.
- A compatible installed plugin should recognize `.pkl`,
   `.pcf`,
   and `PklProject`
  because `plugin.xml` registers those names.

Patterns that fail or confuse:

- Searching only the default JetBrains Marketplace does not find the official
  Apple plugin because upstream issue `#4` is still open.
- Installing current plugin `0.36.1` into IDEA before 2024.3 fails the plugin's
  `since-build="243"` constraint.
- Treating Marketplace plugin `23729-pkl` as official conflicts with Apple's own
  maintainer comment that it was not from Apple.

## Verified workarounds

For IDEA 2024.3 or newer,
 use Apple's official custom plugin repository URL:

```text
https://github.com/apple/pkl-intellij/releases/latest/download/updatePlugins.xml
```

Tradeoff:
 this is not the default Marketplace feed,
 so the plugin appears only
after adding that repository.
 The plugin requires restart according to
`src/main/resources/META-INF/plugin.xml:1-2`:

```xml
<!-- See comments in `org.pkl.intellij.psi.PklStdLib` for why a restart is required. -->
<idea-plugin require-restart="true">
```

For IDEA 2023.1 to 2024.2,
 use an older compatible plugin release such as
`0.33.0`,
 whose metadata says `since-build="231" until-build="252.*"`:

```text
https://github.com/apple/pkl-intellij/releases/download/0.33.0/pkl-intellij-0.33.0.zip
```

Tradeoff:
 old plugin releases lack later fixes and newer Pkl fallback standard
library support.
 For example,
 changelog `0.34.0` added Pkl `0.30` fallback
standard library support after `0.33.0`,
 and `0.36.0` added Pkl `0.31.0`
fallback standard library support.

## What does not work

- Do not rely on default Marketplace search alone for the official Apple plugin.
- Do not assume the README's `IntelliJ 2023.1 or higher` sentence applies to the
  latest release feed;
   the current feed and changelog show the latest plugin now
  requires IDEA 2024.3 or newer.
- Do not treat the old Marketplace listing mentioned in upstream issue `#4` as
  official.
   Apple's maintainer said it was not from Apple.
- Do not expect a `.pkl` file to become recognized merely because the Pkl CLI is installed.
  The CLI evaluates Pkl;
  IDEA editor support comes from the separate JetBrains plugin.

## Upstream filing artifact

`.out-of-scope/` was checked on 2026-06-08.
 No entry covers `pkl-intellij`,
 Pkl,
JetBrains Marketplace,
 or this plugin-publication gap.

### Upstream filing decision

1.  Is it really upstream's fault?
     Yes for the discoverability gap.
     Apple's
    official install path requires a custom plugin repository,
     and upstream issue
    `apple/pkl-intellij#4` tracks Marketplace publication.
2.  Can upstream fix it?
     Yes,
     by publishing the official plugin to JetBrains
    Marketplace.
3.  Are they supporting this use case?
     Yes.
     The docs and README explicitly say the
    plugin provides Pkl language support for IntelliJ Platform IDEs.
4.  Would the repo welcome our contribution?
     Not for this specific gap.
     The
    contributor guide accepts issues and pull requests generally,
     but maintainer
    comment `https://github.com/apple/pkl-intellij/issues/4#issuecomment-2317876133`
    says Marketplace publication is `all internal`.
5.  Will they likely fix it?
     Indeterminate.
     Maintainers said they are working on
    it,
     but the 2026-03-29 comment says there is still no ETA.
6.  Have we prototyped a minimal fix compatible with their architecture?
     No. The
    missing step is publication to JetBrains Marketplace,
     not a code change in the
    public repository.
     The public repository already contains functional file-type
    registration and release metadata.

Duplicate search found `apple/pkl-intellij#4`,
 and the findings above add no new
technical content beyond the existing thread:
 it already records that Marketplace
publication would improve discoverability,
 that Apple is working on it,
 that one
Marketplace listing was not official,
 and that there is no ETA.
 Nothing should be
filed or commented upstream as-is.
