# Equibop 3.3.2 has no setting to keep the guild member list hidden, so the right sidebar comes back

## Symptom

The user's request,
verbatim:

> I want to always hide this ugly right sidebar in Equibop,
> but I couldn't find the settings to do it.
> What should I do?

An annotated screenshot marks two things:
the right-hand column beside chat (the guild
member list,
a stack of avatar rows) and the toolbar button that toggles it.
The want is
"always hidden",
not "hidden in the channel I am looking at".

There are no error strings.
This is a capability gap,
not a crash:
the settings search for
a member-list hide option finds nothing because no such option exists in the settings
surfaces this build ships.

## Root cause

The call chain runs from Discord's own layout surface through every settings surface of
Equibop and Equicord,
and ends at the injection point that can actually hide the panel.

### Step 1: the marked sidebar is a tracked Discord layout surface

Equicord locates the member list by landmark label and tracks it as the `membersList`
surface,
in `src/equicordplugins/_api/surfaceClasses.ts:46` ([patch excerpt][equicord-surface-classes]):

```ts
// src/equicordplugins/_api/surfaceClasses.ts
{
    find: "#{intl::MEMBERS_LIST_LANDMARK_LABEL}",
    replacement: {
        match: /"aria-labelledby":.{0,130}#{intl::MEMBERS_LIST_LANDMARK_LABEL}/,
        replace: '...(Vencord.Api.SurfaceClasses._trackSurfaceInstance("membersList",this),Vencord.Api.SurfaceClasses._getSurfaceProps("membersList")),$&'
    }
},
```

The `MemberCount` plugin renders into that same surface by matching a className
containing `members`,
in `src/plugins/memberCount/index.tsx:62` ([patch excerpt][equicord-member-count]):

```ts
// src/plugins/memberCount/index.tsx
find: "{isSidebarVisible:",
// ...
match: /children:\[(\i\.useMemo[^}]+"aria-multiselectable")(?<=className:(\i),.+?)/,
replace: "children:[$2?.includes('members')?$self.render():null,$1",
```

### Step 2: the running build's class map names the wrapper classes

The wrapper class families were read out of the running client's compiled chunk cache,
which is where this build's hashed class names live:

```sh
# read the hashed class families from the running build's chunk cache
rg -a -o 'membersWrap_[A-Za-z0-9_-]+|members_[A-Za-z0-9_-]+|memberListWrapper__[A-Za-z0-9_-]+|memberList__[A-Za-z0-9_-]+' \
  ~/.var/app/org.equicord.equibop/config/equibop/sessionData/Code\ Cache/js/
```

Two wrapper families came back.
`membersWrap_c8ffbb` sits in a module with `members_c8ffbb`,
`member_c8ffbb`,
`membersGroup_c8ffbb` (the classic member list module).
`memberListWrapper__5e1e2`
sits in a module with `channelInfo__5e1e2`,
`voiceUserRow__5e1e2`,
`joinButtonRegion__5e1e2`
(a member-list-bearing panel that also carries voice user rows).

### Step 3: Discord ships a toggle action, not a preference

The circled toolbar button is Discord's own "Toggle Channel Member List" action,
bound to
`Control + U` ([hotkeys article][discord-hotkeys]).
The hotkey table documents an action
("Toggle Channel Member List | Control + U"),
not a settings row.
This investigation found
no always-hidden preference in any settings surface of Equibop or Equicord (steps 4 and 5).
Discord's own per-channel persistence semantics were not measured here.

### Step 4: Equibop's app settings schema has no member-list field

`src/shared/settings.d.ts` defines the whole Equibop settings surface;
the interface at
line 9 enumerates `tray`,
`staticTitle`,
audio,
updater and window fields and nothing else
([schema excerpt][equibop-settings-schema]):

```ts
// Equibop src/shared/settings.d.ts
export interface Settings {
    // tray, staticTitle, audio, updater, window fields only
```

The absence claim is measured,
not read:
`rg -c -i 'member'` over the full 88-line file
returns `0`.
This is the reason "I couldn't find the settings" succeeds at nothing:
the
row does not exist to be found.

### Step 5: Equicord has no hide plugin, and its policy bans the fix shape

The plugin inventory (both `src/plugins` and `src/equicordplugins`) has no plugin that
hides the member list.
The three near-misses all edit list contents,
never the panel:

- `src/equicordplugins/declutter/index.tsx:67` ([option][equicord-declutter]):
  `Remove animated/styled display names in the member list.`
- `src/equicordplugins/equiTweaks/index.tsx:93` ([option][equicord-equitweaks]):
  `Removes the activity section above member list`
- `src/plugins/silentTyping/index.tsx:52` ([option][equicord-silenttyping]):
  `Hide other users' typing indicators from the members list.`

Upstream explicitly assigns this use case to CSS.
`CONTRIBUTING.md:66` ([rule][equicord-contributing]):

```md
<!-- Equicord CONTRIBUTING.md -->
6. No plugins that only hide or redesign UI elements (use CSS for that). This rule may be negotiable.
```

So the missing setting is missing by policy.
A "hide member list" plugin request runs into
the plugin rules;
the sanctioned mechanism for removing UI is user CSS.

### Step 6: CollapsibleUI collapses, it does not hide

Equicord's `CollapsibleUI` plugin has a `membersList` panel with a persisted collapsed
flag,
in `src/equicordplugins/collapsibleUi/settings.ts:23` and `:114`
([panel registry][equicord-cui-settings],
[persisted flag][equicord-cui-persist]):

```ts
// src/equicordplugins/collapsibleUi/settings.ts
membersList: {
    id: "membersList",
    classId: "members-list",
    label: "Members List",
    collapsedKey: "membersListCollapsed",
},
membersListCollapsed: {
    type: OptionType.BOOLEAN,
    description: "Persist the members list as collapsed.",
    default: false,
    hidden: true,
    onChange: onCollapseSettingChanged("membersList"),
},
```

But "collapsed" here is a peek strip,
not a hide.
`src/equicordplugins/collapsibleUi/style.css:71`
([collapse rule][equicord-cui-style-peek]) pins the panel to a 12px strip,
and `:139`
([hover rule][equicord-cui-style-hover]) expands it back on hover or focus:

```css
/* src/equicordplugins/collapsibleUi/style.css */
[data-vc-collapsible-ui-members-list-collapsed] {
    width: var(--vc-cui-peek-size) !important;
    min-width: var(--vc-cui-peek-size) !important;
    max-width: var(--vc-cui-peek-size) !important;
    flex: 0 0 var(--vc-cui-peek-size) !important;
    opacity: 0.01;
}
[data-vc-collapsible-ui-members-list-collapsed]:is(:hover, :focus-within) {
    width: var(--vc-cui-members-list-expanded-width) !important;
    /* re-expands to the full panel */
}
```

`--vc-cui-peek-size` is `12px` at `style.css:9`.
So a persisted collapse always leaves a
12px strip and hover brings the panel back,
which contradicts "always hide".

### Step 7: QuickCSS is the working injection point

Equicord loads `settings/quickCss.css` live and hot-reloads it on change,
in
`src/api/Themes.ts:44` ([loader][equicord-themes]):

```ts
// src/api/Themes.ts
VencordNative.quickCss.addChangeListener(css => {
    // ...
    style.disabled = !Settings.useQuickCss;
```

The settings UI exposes it as the "Edit QuickCSS" button
(`src/components/settings/tabs/vencord/index.tsx:248`,
[button][equicord-edit-quickcss]) and
the themes tab describes it as the tool for this job
(`src/components/settings/tabs/themes/index.tsx:441`,
[copy][equicord-themes-copy]):
`use QuickCSS for quick style tweaks`.

Local state,
measured 2026-10-30:
`settings.json:4` already has `"useQuickCss": true`,
and
`~/.var/app/org.equicord.equibop/config/equibop/settings/quickCss.css` is 0 bytes.
The
injection point is live and empty.

### Readings that were wrong and what disproved them

- The `members__6e500` class family was first read as a third sidebar wrapper.
  The cache
  byte context puts it next to `header__6e500`,
  `memberCount__6e500`,
  `dot__6e500`,
  which
  is the member-count header module,
  not a wrapper.
  Do not target it.
- The first failing-catalog run reported every pattern as succeeding.
  A prior
  `agent-browser eval` had left its `<style>` element in the fixture page,
  so nothing could
  fail.
  The catalogs below come from a run that reloads the fixture and resets the injected
  style between patterns.

## Verification

### Versions under test

- Equibop 3.3.2,
  Flatpak `org.equicord.equibop`,
  flathub commit
  `1f9bb303d08a61cbe644cf9af03dfdfa00f7ba6e8da97d407baf684f86122261`
- Bundled Equicord `8d0ebf3549890be2ecb1e1d602b1695f584c71cc` (read from `equicord.asar`)
- Measured 2026-10-30

### Harness A: the settings-absence claim

```sh
# ${HOME}/temp/agent/equicord-20261030 (Equicord at 8d0ebf35) and the Equibop schema fetch
rg -c -i 'member' equibop-settings.d.ts
# 0
rg -n --no-heading -e 'Remove animated/styled display names in the member list' \
  -e 'Removes the activity section above member list' \
  -e "typing indicators from the members list" src/plugins src/equicordplugins
# only the three content options listed in step 5, no panel-hiding option
```

### Harness B: the selector fixture

The fixture is a flex row of chat plus four wrappers carrying the real class names from
step 2,
including a rotated hash and an `aside`-tagged wrapper.

```html
<!-- ${HOME}/temp/agent/ml-fixture.html -->
<!doctype html>
<meta charset="utf-8">
<title>member-list hide selector harness</title>
<style>
  #app { display: flex; width: 1200px; }
  #chat { flex: 1 1 auto; background: #ccc; min-height: 400px; }
  .wrap { width: 240px; background: #888; }
</style>
<div id="app">
  <div id="chat">chat</div>
  <div id="wrapA" class="wrap membersWrap_c8ffbb"><div class="members_c8ffbb">rows</div></div>
  <div id="wrapB" class="wrap memberListWrapper__5e1e2"><div class="memberList__5e1e2">list</div></div>
  <div id="wrapC" class="wrap membersWrap_9ab32"><div class="members__1ee8c">rotated</div></div>
  <aside id="wrapD" class="wrap membersWrap_77aa1"><div class="members__211d1">aside</div></aside>
</div>
```

Measure with `agent-browser`,
resetting the injected style between patterns so no run
inherits the previous rule:

```sh
# open the fixture and measure each pattern in one eval, clearing style between runs
agent-browser open "file://${HOME}/temp/agent/ml-fixture.html"
agent-browser eval '<per-pattern measurement script: set style.textContent, read
getBoundingClientRect().width of #chat and each wrapper, clear style, next pattern>'
```

Baseline (no rule):
chat 240px,
each wrapper 240px of the 1200px row.

### Patterns that work cleanly

Measured widths after the rule,
chat first,
then the four wrappers:

- `[class^="membersWrap_"],[class*=" membersWrap_"],[class^="memberListWrapper_"],[class*=" memberListWrapper_"]{display:none!important}`
  gives chat 1200px,
  wrappers 0,
  0,
  0, 0.
  This is the recommended rule:
  it hides both
  families,
  survives hash rotation,
  and does not care which HTML tag the wrapper uses.
- `[class^="membersWrap_"],[class*=" membersWrap_"]{display:none!important}`
  gives chat 960px,
  wrappers 0,
  240px,
  0, 0.
  Narrow variant:
  hides the classic member
  list everywhere including rotated hashes,
  leaves the `memberListWrapper__` family alone.

### Patterns that fail

Measured widths after the rule,
chat first,
then the four wrappers:

- `div[class^="members_"]{display:none!important}` gives chat 240px,
  wrappers 240px all
  round.
  Hiding the inner scroller leaves the wrapper column occupying its width,
  so chat
  does not reclaim the space and an empty gray strip remains.
- `.membersWrap_c8ffbb{display:none!important}` gives chat 480px,
  wrappers 0,
  240px,
  240px,
  240px.
  Exact-hash pinning works for one family only and stops matching when
  Discord rotates the hash suffix.
- `.membersWrap{display:none!important}` gives chat 240px,
  wrappers 240px all round.
  Bare
  class names match nothing,
  because every real class is hash-suffixed.
- `div[class^="membersWrap_"],div[class*=" membersWrap_"],div[class^="memberListWrapper_"],div[class*=" memberListWrapper_"]{display:none!important}`
  gives chat 960px,
  wrappers 0,
  0,
  0,
  240px.
  Tag-qualifying with `div` misses the
  `aside`-tagged wrapper,
  so one sidebar survives.

### Limits of this harness

The fixture proves selector mechanics against the class names of the running 3.3.2 build.
The live client's DOM was not queried directly (the running Electron process has no debug
port open),
so the live end-to-end result is recorded with the workaround only after it is
applied to the real config.

## Verified workarounds

### A. QuickCSS wrapper hide (recommended)

Append this to `~/.var/app/org.equicord.equibop/config/equibop/settings/quickCss.css`
(flatpak path;
the file sits next to `settings/settings.json` in the Equibop config dir),
or open it with the settings UI button "Edit QuickCSS":

```css
/* ~/.var/app/org.equicord.equibop/config/equibop/settings/quickCss.css */
/* Hide the guild member list (right sidebar) everywhere. */
[class^="membersWrap_"],
[class*=" membersWrap_"],
[class^="memberListWrapper_"],
[class*=" memberListWrapper_"] {
  display: none !important;
}
```

Tradeoffs:

- The panel is gone in every channel and every thread view that reuses those wrappers.
  There are no per-channel exceptions;
  the rule is global.
- Discord's toolbar toggle and `Control + U` keep flipping the view state,
  but nothing
  visibly changes.
  The toggle button looks broken while the rule is active.
  Comment the
  rule out to get the panel back.
- Prefix matching survives class-hash rotation (measured against `membersWrap_9ab32`).
  It does not survive a full rename of the wrapper classes;
  if a Discord update renames
  them,
  the rule silently stops matching and the sidebar returns.
  Re-run Harness B after
  big client updates.
- The `memberListWrapper__` family carries voice user rows and a join button in this
  build.
  The full rule hides that panel too.
  Drop the last two selectors to keep it and
  still hide the classic member list (the narrow variant,
  measured above).
- The fix lives in user CSS,
  so it survives plugin changes and Equicord updates that do
  not rename the wrapper classes.

### B. CollapsibleUI persistent collapse

Enable the `CollapsibleUI` plugin (it is not enabled in the current `settings.json`),
toggle "Members List" off in its toolbar menu,
and the `membersListCollapsed` flag
persists the collapse across sessions.

Tradeoffs:

- This is not "always hidden".
  A 12px peek strip stays visible and the panel re-expands
  on hover or focus (measured in source at `collapsibleUi/style.css:71` and `:139`).
- The upside over A:
  hover or keyboard focus gives you the member list back instantly
  without editing any file.
- The plugin's persisted flag is `hidden: true` in its settings,
  so the toolbar menu is
  the only toggle surface.

### C. Discord's own toggle (Control + U)

The toolbar button or `Control + U` flips the member list for the current view.

Tradeoffs:

- A toggle action,
  not a preference.
  There is no always-hidden row in any settings
  surface of this build,
  which is the whole reason this doc exists.
- Nothing to install and nothing to maintain,
  but the sidebar comes back whenever the
  view state says so.

Ranking:
A > B > C,
because A is the only one that matches "always hide" with zero
residue (B leaves a hover-expandable 12px strip that contradicts "always"),
and B > C
because B at least persists its state across sessions while C is a per-view action with
no persistence surface at all.

## What does not work

- Looking for the option in Equibop or Equicord settings.
  The app schema has zero
  `member` occurrences (Harness A) and the plugin inventory has no hide plugin.
- Asking upstream for a "hide member list" plugin.
  `CONTRIBUTING.md:66` bans plugins that
  only hide or redesign UI and points at CSS.
- Hiding only the inner `members_` scroller.
  The wrapper keeps its width,
  chat does not
  expand,
  and an empty strip stays (Harness B).
- Pinning the exact hash class (`.membersWrap_c8ffbb`).
  Hash rotation breaks it and the
  other wrapper family is missed (Harness B).
- Bare `.membersWrap`.
  Real classes are hash-suffixed,
  so it matches nothing (Harness B).
- `div`-qualified attribute selectors.
  An `aside`-tagged wrapper survives (Harness B).
- CollapsibleUI alone,
  when the goal is "always hidden".
  Its collapsed state is a peek
  strip with hover re-expansion,
  measured in its own stylesheet.

## Upstream filing artifact

### Duplicate and work-in-progress search

Run 2026-10-30,
results recorded verbatim:

- `gh search issues "hide member list" --repo Equicord/Equicord --repo Equicord/Equibop --limit 5`
  returned nothing.
  `gh search prs` with the same terms returned nothing.
- `gh api repos/Equicord/Equicord/commits?path=src/equicordplugins/collapsibleUi` shows
  maintainer churn in this exact code path as recently as 2026-10-19 (`fe393c1c`,
  `collapsibleui: fix banner`),
  but nothing adding a full-hide mode.
  The owner is active
  here;
  any future proposal should read their open work first.
- `.out-of-scope/` in this repository was listed before filing.
  It holds
  `bun-install.md`,
  `cargo-workspace.md`,
  `claude-code-upstream-bugs.md`,
  `codex-harness.md`,
  `jsr.md`,
  `lightningcss.md`,
  `low-impact-typescript-formatting.md`,
  `module-es-monolith.md`,
  `pi-gpt55-long-context.md`,
  `terminal-title-fork-parity-tests.md`,
  `typescript-project-references.md`.
  None covers Equibop,
  Equicord,
  or Discord UI,
  so no
  exemption applies.

### Upstream filing decision

Walked per constraint,
2026-10-30,
against Equicord `8d0ebf35`:

1. **Is it really upstream's fault?**
   No. Nothing misbehaves.
   Discord renders a member
   list and offers a toggle action;
   Equibop and Equicord expose no always-hidden row.
   This is a missing preference,
   and upstream's own wording assigns hiding UI to CSS
   (`CONTRIBUTING.md:66`).
   The behavior is by design,
   so this fails at the first gate.
2. **Can upstream fix it?**
   Technically yes (a plugin or a settings row),
   but see 1 and
   4:
   the fix shape their rules allow is the CSS the user can already write.
3. **Are they supporting this use case?**
   Yes,
   through QuickCSS.
   The themes tab says
   `use QuickCSS for quick style tweaks` (`themes/index.tsx:441`) and CollapsibleUI ships
   the weaker collapse variant for users who want a peek strip.
   The use case is served.
4. **Would the repo welcome our contribution?**
   A hide-only plugin is banned by
   `CONTRIBUTING.md:66`,
   and their AI policy requires disclosure and forbids AI-generated
   text in issues.
   A settings-row patch might be negotiable per that rule's last sentence,
   but the case for it is weak while CSS solves the want completely.
5. **Will they likely fix it?**
   No signal of an always-hidden mode in flight.
   Fresh
   maintainer commits on `collapsibleUi` (2026-10-19) show the collapse feature is
   maintained as-is,
   not extended toward hiding.
6. **Have we prototyped a minimal fix?**
   Not run.
   Constraints 1,
   3,
   and 4 already fail,
   so the auto-prototype trigger (constraints 1-5 hold or sorta-hold) does not fire.

Decision:
**do not file.
Nothing to add upstream.**
The want is fully served by the
consumer-side QuickCSS workaround,
which upstream's own policy names as the right
mechanism.
The draft below is kept only as an auditable record.

### Risk to the poster

Filing is the poster's decision,
and it carries real risk whatever the draft says.
It
puts a named account in front of strangers and of maintainer trust or blocklist systems,
some of which are shared between projects;
a denouncement can follow a person beyond one
repository.
Equicord's `CONTRIBUTING.md` additionally bans AI-generated text in issues and
requires disclosure of AI assistance,
so any filing here would need a human rewrite and a
disclosure statement.
Nothing in this doc recommends filing,
and the recommendation to
add nothing is what the six-constraint walk supports.

### Readiness check before anyone files

Recorded 2026-10-30 against Equibop 3.3.2 and Equicord `8d0ebf35`.
An item that was not
run is written as not run.

1. Reproduce on the latest release and other maintained lines:
   not run.
   Only 3.3.2 is
   installed here,
   and the decision is do-not-file.
2. Prior art in the touched code:
   run.
   CollapsibleUI history,
   the plugin inventory,
   and
   `CONTRIBUTING.md:66` were read (root cause steps 5 and 6).
3. Intake fit (issue forms,
   templates,
   contribution policy):
   run in part.
   `CONTRIBUTING.md` was read and its AI policy and plugin rules recorded.
   Issue and PR
   templates were not read,
   because no filing is planned.
4. Automation on new filings (triage bots,
   auto-labelers,
   trust lists):
   not run.
   No
   filing is planned.
5. Same-day duplicate,
   WIP,
   and trust-system search:
   run.
   Queries and results are in the
   search record above.
6. Run the draft's reproduction exactly as written:
   run.
   Harness A and Harness B are the
   reproduction and were run as written on 2026-10-30.
7. Make the reproduction runnable by others (minimal repo plus CI workflow):
   not run.
   No
   filing is planned.
8. Verdict:
   do not file.
   The blocker that stops a recommendation to file is the target's
   own policy (`CONTRIBUTING.md:66`) together with a complete consumer-side workaround,
   not an unmet checklist item.

### Draft (do not file as-is)

~~~md
Title: Add an "always hidden" mode for the member list

Equibop 3.3.2 (bundled Equicord 8d0ebf35) ships no settings row that keeps the guild
member list hidden. CollapsibleUI's `membersListCollapsed` leaves a 12px peek strip that
re-expands on hover (collapsibleUi/style.css:71, :139). Discord's own action is a
per-view toggle.

Repro: open any guild text channel; the member list shows; no settings row hides it.

Suggested fix: a plugin setting or a CollapsibleUI option that sets
`display: none` on the wrapper families `membersWrap_` and `memberListWrapper_`.
Note CONTRIBUTING.md rule 6, which the request collides with; the workaround is user CSS.
~~~

Do not file as-is:
constraint 1 fails (no defect),
constraint 3 fails (the use case is
already served by QuickCSS),
and constraint 4 fails (plugin rule 6 bans the fix shape).

[discord-hotkeys]: https://support.discord.com/hc/en-us/articles/225977308--Windows-Discord-Hotkeys#:~:text=Toggle%20Channel%20Member%20List
[equibop-settings-schema]: https://github.com/Equicord/Equibop/blob/57406f627221bc6054deb9834c18fa9cd914d96a/src/shared/settings.d.ts#L9-L30
[equicord-contributing]: https://github.com/Equicord/Equicord/blob/8d0ebf3549890be2ecb1e1d602b1695f584c71cc/CONTRIBUTING.md#L66
[equicord-cui-persist]: https://github.com/Equicord/Equicord/blob/8d0ebf3549890be2ecb1e1d602b1695f584c71cc/src/equicordplugins/collapsibleUi/settings.ts#L114-L120
[equicord-cui-settings]: https://github.com/Equicord/Equicord/blob/8d0ebf3549890be2ecb1e1d602b1695f584c71cc/src/equicordplugins/collapsibleUi/settings.ts#L23-L28
[equicord-cui-style-hover]: https://github.com/Equicord/Equicord/blob/8d0ebf3549890be2ecb1e1d602b1695f584c71cc/src/equicordplugins/collapsibleUi/style.css#L134-L145
[equicord-cui-style-peek]: https://github.com/Equicord/Equicord/blob/8d0ebf3549890be2ecb1e1d602b1695f584c71cc/src/equicordplugins/collapsibleUi/style.css#L69-L80
[equicord-declutter]: https://github.com/Equicord/Equicord/blob/8d0ebf3549890be2ecb1e1d602b1695f584c71cc/src/equicordplugins/declutter/index.tsx#L67
[equicord-edit-quickcss]: https://github.com/Equicord/Equicord/blob/8d0ebf3549890be2ecb1e1d602b1695f584c71cc/src/components/settings/tabs/vencord/index.tsx#L248
[equicord-equitweaks]: https://github.com/Equicord/Equicord/blob/8d0ebf3549890be2ecb1e1d602b1695f584c71cc/src/equicordplugins/equiTweaks/index.tsx#L93
[equicord-member-count]: https://github.com/Equicord/Equicord/blob/8d0ebf3549890be2ecb1e1d602b1695f584c71cc/src/plugins/memberCount/index.tsx#L62-L66
[equicord-silenttyping]: https://github.com/Equicord/Equicord/blob/8d0ebf3549890be2ecb1e1d602b1695f584c71cc/src/plugins/silentTyping/index.tsx#L52
[equicord-surface-classes]: https://github.com/Equicord/Equicord/blob/8d0ebf3549890be2ecb1e1d602b1695f584c71cc/src/equicordplugins/_api/surfaceClasses.ts#L46-L51
[equicord-themes-copy]: https://github.com/Equicord/Equicord/blob/8d0ebf3549890be2ecb1e1d602b1695f584c71cc/src/components/settings/tabs/themes/index.tsx#L441
[equicord-themes]: https://github.com/Equicord/Equicord/blob/8d0ebf3549890be2ecb1e1d602b1695f584c71cc/src/api/Themes.ts#L44-L53
