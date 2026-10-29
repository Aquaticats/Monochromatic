# rgffplay

Music player that finds files by case-insensitive name via ripgrep and plays them with ffplay.

## Usage

```sh
rgffplay <name...>
```

Positional arguments form the search query.
Each word's first letter is converted to a case-insensitive bracket expression,
then joined with `*` wildcards to produce a ripgrep glob.

```sh
rgffplay sweet devil
# glob: *[Ss]weet*[Dd]evil*

rgffplay "Sweet Devil"
# glob: *[Ss]weet*[Dd]evil*
```

## How it works

1. Builds a glob pattern from the name words
2. Resolves the music directory from `$XDG_MUSIC_DIR` or `xdg-user-dir MUSIC`
3. Runs `rg --files -g <glob> <music_dir> --null` to find matching files
4. Plays matched files with `ffplay -loop 0 -nodisp` (loops indefinitely,
    no video window)

## Why not a bash function?

The equivalent bash function is ~6 lines:

```bash
rgffplay() {
  local glob="*" c
  for w in "$@"; do
    c="${w:0:1}"
    glob+="[${c^^}${c,,}]${w:1}*"
  done
  rg --files -g "$glob" "${XDG_MUSIC_DIR:-$(xdg-user-dir MUSIC)}" --null | xargs -0 ffplay -loop 0 -nodisp
}
```

The bracket-case loop with substring extraction and parameter expansion
(`${c^^}`,
 `${c,,}`,
 `${w:1}`) reads like line noise to anyone not fluent in bash-isms.
Add error handling for zero or ambiguous matches and it gets worse fast.

Bash `${c^^}` and `${c,,}` also rely on the C locale's `toupper`/`tolower`,
so accented characters (`é`,
 `ç`,
 `ü`) may not case-fold correctly depending on locale settings.
JavaScript's `toUpperCase()`/`toLowerCase()` uses Unicode-aware case mapping by default.

The TypeScript version is more verbose,
but `bracketFirst`,
 `buildGlob`,
 `findFiles` are self-documenting.
You can skim the function names and understand the pipeline
without parsing cryptic sigils.

## Known limitations

ffplay's in-place progress line (the `\r`-updating time position) does not display.
Bun's `child_process.spawn` does not preserve TTY characteristics even with `stdio: 'inherit'`,
so ffplay sees a pipe instead of a terminal and suppresses the progress output.
The banner,
 metadata,
 and audio playback work normally.

## Dependencies

- **ripgrep** (`rg`) for fast file search
- **ffplay** (from FFmpeg) for audio playback
- **xdg-user-dir** (optional) as fallback when `$XDG_MUSIC_DIR` is unset
