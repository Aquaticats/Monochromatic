# libFuzzer (libfuzzer-sys 0.4.13) dictionaries reject `\n`, `\r` and `\t` escapes
and the target exits before any execution

## Symptom

A fuzz target started with `-dict=<file>` exits with status 1 after zero executions,
and its standard error holds only:

```text
ParseDictionaryFile: error in line 30
		"\n"
```

The line number names the first entry with an escape libFuzzer does not know.
In this repository it stopped the `dependent_version` target of `package/git-policy/cli.fuzz`
during `mise run //package/git-policy/cli.fuzz:smoke`;
the runner reported
`dependent_version exited 1 after 0 verified executions`
(evidence `package/git-policy/cli.fuzz/target/verification/campaign-8fWevh/dependent_version`,
`exit.json` `{"status":1,"signal":null,"executedUnits":0}`).

Entries that trigger it are any backslash not followed by `\`, `"` or `x` and two hexadecimal digits,
for example `"\n"`, `"\r\n"`, `"\t"`, `"\0"` and `"A"`.

## Root cause

The driver parses the dictionary before fuzzing and returns 1 when parsing fails,
`libfuzzer/FuzzerDriver.cpp:751` to `753` in `libfuzzer-sys` 0.4.13:

```cpp
  if (Flags.dict)
    if (!ParseDictionaryFile(FileToString(Flags.dict), &Dictionary))
      return 1;
```

`ParseDictionaryFile` stops at the first entry that does not parse and prints the line,
`libfuzzer/FuzzerUtil.cpp:143` to `148`:

```cpp
    if (ParseOneDictionaryEntry(S, &U)) {
      Units->push_back(U);
    } else {
      Printf("ParseDictionaryFile: error in line %d\n\t\t%s\n", LineNo,
             S.c_str());
      return false;
    }
```

`ParseOneDictionaryEntry` knows two escapes,
a backslash before `\` or `"`,
and `\x` before two hexadecimal digits;
every other backslash fails the entry,
`libfuzzer/FuzzerUtil.cpp:101` to `118`:

```cpp
    if (V =='\\') {
      // Handle '\\'
      if (Pos + 1 <= R && (Str[Pos + 1] == '\\' || Str[Pos + 1] == '"')) {
        U->push_back(Str[Pos + 1]);
        Pos++;
        continue;
      }
      // Handle '\xAB'
      if (Pos + 3 <= R && Str[Pos + 1] == 'x'
           && isxdigit(Str[Pos + 2]) && isxdigit(Str[Pos + 3])) {
        ...
        continue;
      }
      return false;  // Invalid escape.
```

This is the documented syntax.
`llvm/docs/LibFuzzer.md`, section "Dictionaries"
(LLVM `main` at `af2d76922b54fc347faac9727b0748387fea043f`),
lists only these escapes:

```text
# Use \\ for backslash and \" for quotes.
kw2="\"ac\\dc\""
# Use \xAB for hex values
kw3="\xF7\xF8"
```

C-style escapes such as `\n` look valid because the entries are written in double quotes,
but they are not part of the format.

## Verification

Version under test:
`libfuzzer-sys` 0.4.13 from crates.io,
as locked in `package/git-policy/cli.fuzz/Cargo.lock`;
`sha256sum` of its `libfuzzer/FuzzerUtil.cpp` is
`cf0e9217539c77117a739e47e9e9f35e43141db291ab6167deae365b3ef496be`.

The failing run is the smoke run named in section "Symptom".
After the dictionary was changed as in section "Verified workarounds",
the same target loads it;
the smoke evidence of that run is recorded in `doc/handover/cli-git-native-dependent-version.md`,
section "Fuzzing".

This check restates `ParseOneDictionaryEntry`'s escape rule and lists the lines of every dictionary that it rejects:

```python
# check-dictionaries.py, run from package/git-policy/cli.fuzz with python3 -I
import glob, string

def parses(line):
    text = line.strip()
    if not text or text.startswith('#'):
        return True
    if not (text.endswith('"') and '"' in text[:-1]):
        return False
    body = text[text.index('"') + 1:-1]
    index = 0
    while index < len(body):
        if body[index] == '\\':
            if body[index + 1:index + 2] in ('\\', '"') and index + 1 < len(body):
                index += 2
                continue
            digits = body[index + 2:index + 4]
            if body[index + 1:index + 2] == 'x' and len(digits) == 2 and all(c in string.hexdigits for c in digits):
                index += 4
                continue
            return False
        index += 1
    return True

for path in sorted(glob.glob('dictionary/*.dict')):
    print(path, [number for number, line in enumerate(open(path, encoding='latin1'), 1) if not parses(line)])
```

On the dictionary before the fix it reports lines 30, 31 and 32,
the first of which is the line libFuzzer reported;
after the fix it reports no line in any of the five dictionaries.

Entries that parse:

- `"\x0a"`, `"\x0d\x0a"`, `"\x09"`, `"\x00"`
- `"\""`, `"\\"`, `"\\\""`
- `"\"vers\\u0069on\""` (a backslash escaped as `\\`, then plain `u0069`)
- `" "` (spaces inside the quotes are kept)

Entries that fail:

- `"\n"`, `"\r\n"`, `"\t"`
- `"\0"`, `"A"`
- `"\x0"` (one hexadecimal digit)

## Verified workarounds

Spell every byte that is not printable as `\xAB`:
`"\x0a"` for a line feed,
`"\x0d\x0a"` for CRLF,
`"\x09"` for a tab.
The entries mean the same bytes;
the only cost is readability.
This is what `package/git-policy/cli.fuzz/dictionary/dependent_version.dict` now does,
matching `batch_reply.dict`, which already wrote `"\x0a"`.

## What does not work

- C escapes (`\n`, `\t`, `\r`, `\0`):
  rejected,
  per section "Root cause".
- A literal line feed inside the quotes:
  the file is split into entries at line feeds before any entry is parsed
  (`std::getline(ISS, S, '\n')`, `libfuzzer/FuzzerUtil.cpp:137`),
  so an entry cannot contain one.

## Upstream filing decision

`.out-of-scope/` has no entry for libFuzzer or cargo-fuzz.

1.  Is it really upstream's fault?
    No.
    The behavior matches the documented dictionary syntax,
    which lists only `\\`, `\"` and `\xAB`.
2.  Can upstream fix it?
    Yes:
    more escapes could be accepted.
3.  Are they supporting this use case?
    Not as documented:
    the format is AFL's,
    and `\xAB` already covers every byte.
4.  Would the repo welcome our contribution?
    Not evaluated,
    since constraint 1 fails.
5.  Will they likely fix it?
    Not evaluated,
    since constraint 1 fails.
6.  Have we prototyped a minimal fix?
    No;
    with constraint 1 failing there is nothing to fix.

Duplicate search:
`gh search issues --repo llvm/llvm-project "libfuzzer dictionary escape"`,
`gh search issues --repo llvm/llvm-project "ParseDictionaryFile"`
and `gh search issues --repo llvm/llvm-project "libFuzzer dictionary"` returned no issue;
`gh search issues --repo rust-fuzz/cargo-fuzz "dictionary"` returned none about escapes.

Decision:
nothing to file.
