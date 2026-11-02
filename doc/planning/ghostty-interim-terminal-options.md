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

## Proposed ranking

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
