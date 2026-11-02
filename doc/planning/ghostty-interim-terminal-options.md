# Interim terminals for Claude Code on Bazzite testing

## Scope and evidence limits

The user asked for alternatives to the crashing Ghostty installation,
then explicitly waived the full technology-selection skill for this session.
The follow-up asks for explanations,
ranking,
and additional candidates.
This is a documentation and package comparison,
not a completed source audit or a measured stability ranking.
No terminal was installed,
configured,
or made the default.
No candidate's full Claude workflow was runtime-tested.

Ranking prioritizes the current Linux/KDE desktop,
documented Claude keyboard support,
terminal features useful for coding,
and available package channels.
The relative ordering is judgment,
not a benchmark score.
User preferences for a permanent terminal remain open.

## Local facts checked on 2026-11-01

- Bazzite `testing-45.20261101`,
  Fedora 45,
  KDE Wayland.
- `konsole --version`:
  `26.08.2`.
- `kitty --version`:
  `0.48.0`.
  The launcher resolves to the user's `.local/kitty.app/bin/kitty`.
- Installed Contour RPM:
  `0.6.3.8249-2.fc45`.
- Claude executable resolves to version `2.1.316`.
- Ghostty preferences include font features,
  increased cell height,
  transparency,
  command-finish notifications,
  and disabled scrolling to the bottom on input and output.
  Feature parity for those settings has not been tested.

The query used the enabled `fedora`,
`updates`,
and `terra` repositories:

```sh
# Repository inventory only; this installs nothing.
dnf --assumeyes --repo=fedora --repo=updates --repo=terra repoquery --available \
  --queryformat='%{name} %{evr} %{arch} %{repoid}\n' \
  konsole contour-terminal kitty foot wezterm alacritty ptyxis rio gnome-terminal \
  cosmic-term xfce4-terminal tilix terminator terminology qterminal xterm st cool-retro-term
```

Available x86-64 package versions included:
kitty `0.48.2`,
Konsole `26.08.2`,
foot `1.28.0`,
Ptyxis `50.1`,
Alacritty `0.17.0`,
GNOME Terminal `3.60.0`,
Xfce Terminal `1.2.0`,
COSMIC Terminal `1.10.0`,
Rio `0.5.29`,
Terminator `2.1.5`,
Tilix `1.9.6`,
QTerminal `2.4.0`,
and Terminology `1.13.0`.
WezTerm did not appear in these repositories.
That does not establish absence from upstream,
Copr,
or Flatpak channels.

The GitHub latest-release API reports WezTerm's latest non-prerelease tag as
`20240203-110809-5046fc22`,
published `2024-02-03`.
This is a release-channel observation,
not proof that development has stopped.
Contour's latest upstream tag is `v0.8.0.9335`,
published `2026-10-23`;
the configured Fedora package remains `0.6.3.8249`.

## Requirements added after the initial comparison

The user added:
"Must have a CoC explicitly banning harrassment."
When asked whether detailed prohibitions also count,
the user answered:
"Accept too."
An applicable community code that explicitly prohibits personal attacks,
discrimination,
stalking,
intimidation,
or unwanted sexual attention may therefore qualify;
a generic request to be nice is insufficient.

The user then added:
"Must accept LLM-generated bug reports.
Requiring a human to look at them is fine too."
Human review is acceptable.
The user subsequently clarified:
"Requiring LLM output to be rewritten in the reporter's own words is fine."
Human rewriting therefore also meets the reporting requirement.
This restores Konsole and Yakuake's eligibility under KDE's reporting rule.
A project's use of AI in its product or code does not establish acceptance of generated bug reports.
Absence of a ban is recorded as unconfirmed,
not as affirmative acceptance.

The user also reported:
"I couldn't answer the question because Contour seems to be swallowing modifiers."
This is an unverified input incident,
separate from the reproduced Ghostty font crash.
The subsequent [Contour input investigation](../troubleshooting/contour-modifier-bindings-codex.md)
identified tab-key interception,
an absent clipboard shortcut,
and partial-region scrollback loss.
The authorized tab and copy changes passed runtime probes.
Codex's full-transcript viewer was verified as a workaround for missing native scrollback.

The historical ranking in this document is superseded by these hard requirements.

### Conduct-policy results

- Foot `CODE_OF_CONDUCT.md:33` explicitly lists public or private harassment as unacceptable.
- Contour `CODE_OF_CONDUCT.md:34` does likewise and provides a reporting contact.
- Rio `.github/CODE_OF_CONDUCT.md:28` does likewise,
  but its enforcement address at line 58 is still the literal placeholder `{{ email }}`.
- Tabby `CODE_OF_CONDUCT.md:21` does likewise and provides a reporting contact.
- KDE's community code explicitly prohibits personal attacks,
  racism,
  sexism,
  and other discrimination.
  It applies to Konsole and Yakuake.
  This qualifies under the user's acceptance of detailed prohibitions.
- GNOME's code explicitly prohibits intimidation,
  stalking,
  unwanted sexual attention,
  discriminatory language,
  and other specified misconduct.
  Its scope covers GNOME community spaces and `gitlab.gnome.org`.
  This qualifies for Ptyxis,
  GNOME Terminal,
  and GNOME Console under the accepted detailed-prohibitions interpretation.
- No qualifying policy was verified for kitty,
  Alacritty,
  WezTerm,
  Xfce Terminal,
  COSMIC Terminal,
  Terminator,
  Tilix,
  QTerminal,
  or Terminology.
  This is a bounded search result,
  not a claim that every possible policy location was exhausted.
  The Pop!_OS umbrella code found during the COSMIC check gives general collaboration principles,
  without an explicit harassment or comparable misconduct prohibition.

### LLM-report results

- Foot fails:
  its README's No AI / LLM policy forbids generative AI for all contributions,
  explicitly including bug reports and tracker comments.
- Konsole and Yakuake pass under the clarified requirement:
  KDE's current Issue Reporting guide says not to paste LLM output,
  and requires the reporter to synthesize it and write the conclusion in their own words.
  The user explicitly accepts that rewriting step.
- Alacritty independently fails:
  its contribution guide prohibits LLM-generated content in issues and comments.
- Rio has a positive observed acceptance case:
  issue `1753` discloses Claude-assisted diagnosis and a fix;
  maintainer `raphamorim` credited that diagnosis and fix in PR `1789`,
  merged it on `2026-08-01T20:33:21Z`,
  and closed the reported issue.
  Merge commit:
  `74966176290c7e3f8d977da997ec836997e8292c`.
  The original PR `1754` was superseded because the relevant code had moved,
  not rejected for its disclosed AI assistance.
  This establishes an accepted AI-assisted report and fix,
  not a blanket published permission statement for every generated report.
- Wave Terminal is an additional candidate:
  its CoC explicitly bans public or private harassment,
  and its contribution guide welcomes AI-assisted contributions with understanding and supervision.
  Its bug-report template asks for behavior,
  reproduction steps,
  version,
  and environment;
  it adds no authorship restriction.
  The permission is general contribution guidance rather than a report-specific guarantee.
- Contour,
  Tabby,
  and the named GNOME applications have no confirmed report-acceptance result from this check.
  GNOME's general handbook does not settle individual projects' AI policies.
  A GNOME maintainer's article arguing against report bans is advocacy,
  not a policy granting permission across GNOME projects.
  Tabby issue `9146` is disputed as AI-generated by another participant;
  that dispute does not establish a disclosed,
  accepted generated bug report or a general permission policy.

### Shortlist after accepting human rewriting

Konsole returns to the first-choice position for an immediate fallback:
it is already installed,
its applicable KDE CoC meets the accepted detailed-prohibitions standard,
and its human-rewriting rule now meets the user's reporting preference.
Yakuake qualifies through the same community policies if a drop-down interface is wanted.
The requested Contour investigation is recorded separately,
including the applied tab and clipboard changes and the transcript-view workaround.

### Historical shortlist before accepting human rewriting

1.  Wave Terminal:
    clearest written permission for supervised AI-assisted contributions,
    an explicit harassment ban,
    and a filled-in CoC reporting contact.
    Its terminal also includes file,
    browser,
    and AI workspace features,
    so it is a broader application than the conventional terminals first considered.
    No package matched `waveterm*` or `wave-terminal*` in the configured Fedora/updates/Terra repositories.
    Upstream release `v0.14.5` provides Linux x86-64 RPM and AppImage assets.
2.  Rio:
    accepted-report precedent and an explicit harassment ban,
    with a conventional terminal workflow and `0.5.29` available through Terra.
    Ranked after Wave because acceptance is supported by precedent rather than an explicit general policy,
    and its CoC reporting address remains `{{ email }}`.

This is an evidence-qualified recommendation:
neither project promises that every valid generated report will be acted on,
and no operational stability test was run for either application.
Foot and Alacritty prohibit LLM contributions outright.
Konsole/Yakuake's rewriting requirement is acceptable after the user's clarification.
No recommendation is made for the remaining candidates without a verified applicable acceptance policy or precedent.
Contour's modifier-key report has since been diagnosed and its relevant bindings changed;
report-acceptance policy remains unconfirmed in this comparison.

### Conduct and contribution sources checked

- Foot's tagged README and CoC:
  <https://codeberg.org/dnkl/foot/src/tag/1.28.0/README.md#no-ai-llm-policy>,
  <https://codeberg.org/dnkl/foot/src/tag/1.28.0/CODE_OF_CONDUCT.md#our-standards>.
- KDE community CoC and report-writing rule:
  <https://kde.org/code-of-conduct/#be-respectful>,
  <https://community.kde.org/Get_Involved/Issue_Reporting#Don%27t_paste_LLM_output>.
- GNOME CoC and issue guidance:
  <https://conduct.gnome.org/>,
  <https://handbook.gnome.org/issues/reporting.html>.
- Contour CoC:
  <https://github.com/contour-terminal/contour/blob/0ef0f74f9cbb6680522b9b65dd2f035f9b34077f/CODE_OF_CONDUCT.md#L34>.
- Rio CoC and acceptance trail:
  <https://github.com/raphamorim/rio/blob/41a22f165cc3e008a4b953e8a1c39171289b5bd2/.github/CODE_OF_CONDUCT.md#L28>,
  <https://github.com/raphamorim/rio/issues/1753>,
  <https://github.com/raphamorim/rio/pull/1754>,
  <https://github.com/raphamorim/rio/pull/1789>.
- Tabby CoC:
  <https://github.com/Eugeny/tabby/blob/42059e3bc698f1e998f6da33de678cdaf17d61df/CODE_OF_CONDUCT.md#L21>.
- Wave contribution guidance and CoC:
  <https://github.com/wavetermdev/waveterm/blob/011ca146df09e7828aa8e300f5df66e06237352a/CONTRIBUTING.md>,
  <https://github.com/wavetermdev/waveterm/blob/ee396a480a7a51c19a7172eac1cc647bfc29db7a/CODE_OF_CONDUCT.md>.
- Wave bug template and release assets:
  <https://github.com/wavetermdev/waveterm/blob/main/.github/ISSUE_TEMPLATE/bug-report.yml>,
  <https://github.com/wavetermdev/waveterm/releases/tag/v0.14.5>.

The negative-policy search included the GitHub community-profile API,
untruncated recursive trees for relevant project and organization-default repositories,
contribution/readme documents,
issue templates,
and parent-community policies.
Search results and an absent conventional file were not treated as proof of an absolute absence.
No maintainers were contacted and no issue was posted.

## Historical ranking before the conduct and report requirements

The subsequent [Monochromatic daily-use proposal](monochromatic-terminal-daily-use.md)
addresses the user's Rio font failure,
reported Konsole visual artifacts,
and question about developing the existing terminal prototype.
It recommends a bounded reliability milestone while retaining a working terminal.
It does not establish that Monochromatic is currently ready to replace these applications.

1.  Kitty:
    installed;
    documented Claude Shift+Enter support;
    font features,
    tabs,
    splits,
    images,
    shell integration,
    and script control.
    Preferred to Konsole for these terminal capabilities;
    configuration and shortcut conventions need acclimatization.
2.  Konsole:
    installed KDE application with profiles,
    tabs,
    splits,
    output search,
    and settings UI.
    Preferred to foot for complete desktop workflow without adding a multiplexer.
3.  Foot:
    Wayland terminal with font fallbacks,
    synchronized updates,
    emoji,
    and documented Claude Shift+Enter support.
    Preferred to Ptyxis for direct terminal focus;
    normally one window per process.
    Its optional shared server explicitly shares crash fate across windows.
4.  Ptyxis:
    VTE/GTK4 terminal with container integration,
    searchable tab overview,
    and preferences UI.
    Preferred to Alacritty for integrated host/container navigation;
    the GNOME interface is a desktop-style change on KDE.
5.  Alacritty:
    OpenGL terminal with search,
    vi navigation,
    configurable hints,
    and documented Claude Shift+Enter support in version 0.16 or later.
    Preferred to GNOME Terminal for that input support;
    no built-in tabs,
    splits,
    or graphical configuration editor.
6.  GNOME Terminal:
    VTE terminal with tabs,
    profiles,
    search,
    and conventional settings.
    Preferred to Xfce Terminal as the direct GNOME/VTE baseline;
    Claude docs require Ctrl+J or backslash followed by Enter for multiline input.
7.  Xfce Terminal:
    VTE terminal with tabs,
    transparency,
    scrollback configuration,
    and a drop-down mode.
    Preferred to WezTerm for the current distro package path;
    no distinct benefit from another desktop's integration on KDE.
8.  WezTerm:
    Lua configuration,
    local/remote multiplexing,
    tabs,
    panes,
    font shaping,
    and images.
    Preferred to Contour for programmable and remote workflows;
    the stable-release gap and external package channel reduce its interim appeal.
9.  Contour:
    installed GPU terminal;
    upstream emphasizes Unicode,
    font shaping,
    VT functionality,
    and keyboard navigation.
    Preferred to COSMIC Terminal for immediate availability;
    current website features must not be assumed present in the installed 0.6.3 build.
10. COSMIC Terminal:
    Alacritty terminal core with a separate cosmic-text renderer,
    bidirectional text,
    and ligatures.
    GPU rendering has a software fallback.
    Preferred to Rio for a Fedora-provided alternative focused on text;
    COSMIC interface integration brings less benefit on KDE.
11. Rio:
    GPU terminal with documented keyboard and graphics protocols,
    split panels,
    vi mode,
    and shell integration.
    Preferred to Terminator for modern terminal-protocol features;
    this comparison has no local evidence of its failure recovery or Claude behavior.
12. Terminator:
    nested terminal splits,
    grouping,
    and broadcast input.
    Preferred to Tilix when grouping simultaneous sessions is the aim;
    those features add less value for a single Claude session.
13. Tilix:
    GTK3 tiling terminal with draggable panes,
    saved layouts,
    profiles,
    and finish notifications.
    Preferred to QTerminal for persisted multi-pane workspaces;
    duplicates features available in the top-ranked terminals.
14. QTerminal:
    Qt/QTermWidget application with tabs,
    splits,
    drop-down mode,
    search,
    and shortcut editor.
    Preferred to Tabby for a local Qt terminal workflow;
    Konsole already supplies the KDE alternative here.
15. Tabby:
    Electron application with terminal panes,
    SSH/SFTP/serial connection management,
    profiles,
    and plugins.
    Preferred to Terminology when managing remote connections;
    adds an application framework and connection-management scope not required by local Claude.
16. Terminology:
    EFL terminal with tabs,
    splits,
    inline media,
    and graphical settings.
    Its distinctive media features have little relevance to the reported problem.

Yakuake is a useful form-factor alternative to Konsole:
it uses Konsole technology in a drop-down window,
so it is not a separate terminal engine to diversify against Konsole defects.
GNOME Console is a simpler GNOME candidate,
but the customization already present in the user's Ghostty settings favors the ranked options.

## Primary sources read

- Kitty overview and feature documentation:
  <https://sw.kovidgoyal.net/kitty/>.
- Konsole app and command reference:
  <https://apps.kde.org/konsole/>,
  <https://docs.kde.org/stable_kf6/en/konsole/konsole/commandreference.html>.
- Foot tagged `1.28.0` README and Fedora package description:
  <https://codeberg.org/dnkl/foot/src/tag/1.28.0/README.md>,
  <https://packages.fedoraproject.org/pkgs/foot/foot/>.
- Ptyxis:
  <https://devsuite.app/ptyxis/>.
- Alacritty README and configuration:
  <https://github.com/alacritty/alacritty>,
  <https://alacritty.org/config-alacritty.html>.
- GNOME Terminal:
  <https://help.gnome.org/gnome-terminal/>.
- Xfce Terminal:
  <https://docs.xfce.org/apps/xfce4-terminal/start>.
- WezTerm features and Linux distribution instructions:
  <https://wezterm.org/features.html>,
  <https://wezterm.org/install/linux.html>.
- Contour:
  <https://contour-terminal.org/features/>.
- COSMIC Terminal README:
  <https://github.com/pop-os/cosmic-term>.
- Rio:
  <https://rioterm.com/docs/features>.
- Terminator:
  <https://gnome-terminator.readthedocs.io/en/latest/gettingstarted.html>.
- Tilix:
  <https://gnunn1.github.io/tilix-web/>.
- QTerminal README and `qterminal.metainfo.xml`:
  <https://github.com/lxqt/qterminal>.
- Tabby README and `package.json`:
  <https://github.com/Eugeny/tabby>.
- Terminology:
  <https://www.enlightenment.org/about-terminology>.
- Yakuake:
  <https://apps.kde.org/yakuake/>.
- GNOME Console:
  <https://apps.gnome.org/Console/>.
- Claude keyboard compatibility:
  <https://code.claude.com/docs/en/terminal-config>.

These sources establish documented features,
not a comparative performance or reliability result.
