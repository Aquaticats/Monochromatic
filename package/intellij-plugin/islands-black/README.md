# Islands Black

OLED black variant of the JetBrains Islands Dark theme for IntelliJ-based IDEs.

Extends Islands Dark by overriding its four base gray tokens (`gray-10` through `gray-40`) to `#000000`,
producing true black backgrounds across all UI panels,
 toolbars,
 borders,
 and editor areas.
Warnings retain their yellow underline but lose the olive background fill.

![Islands Black theme](https://monochromatic-lfs.aquaticat.workers.dev/06d9081f0b8d7fdfeda6cf4332717973ef315a91752e69660745db8668d45e00/package/intellij-plugin/islands-black/screenshot/islands-black.png)

## Installation

1. From this directory,
    build the JAR:
    `mise run build`
2. In your IDE,
    open **Settings > Plugins > gear icon > Install Plugin from Disk...**
3. Select the `islands-black.jar` file
4. Restart the IDE
5. Go to **Settings > Appearance & Behavior > Appearance > Theme** and select **Islands Black**

## Compatibility

Requires IntelliJ platform build 233 or later (2023.3+).
Tested with WebStorm but works with any JetBrains IDE that ships the Islands Dark theme.

## How it works

The theme JSON overrides four color tokens in the Islands Dark color system:

- `gray-10`:
   base background for editors and tool windows
- `gray-20`:
   tool window headers and island borders
- `gray-30`:
   main window background between panels and panel borders
- `gray-40`:
   toolbar,
   status bar,
   and stripe backgrounds

The editor color scheme extends Darcula and overrides:

- Editor and gutter backgrounds to `#000000`
- Disables caret row highlight
- Removes warning background fill (keeps underline)
- Removes injected language fragment background

## Terminal colors

The block terminal's ANSI black foreground stays `#000000`,
which is what the bundled Darcula scheme uses and what the ANSI palette means by
black: on this theme's `#000000` console background such text is invisible,
and a program that wants readable dark text prints bright black instead,
which this scheme leaves at Darcula's visible `#4E5157`.
