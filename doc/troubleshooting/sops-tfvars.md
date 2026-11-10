# SOPS 3.13.2: encrypting `*.tfvars` uses binary storage and direct Terraform loading fails

## Symptom

SOPS can encrypt a file named `secrets.tfvars`,
but it does not parse the file as HCL.
The encrypted file becomes a JSON envelope whose HCL source is one encrypted `data` value:

```json
{
  "data": "ENC[AES256_GCM,...]",
  "sops": {}
}
```

Passing that encrypted file directly to OpenTofu 1.12.5 as HCL produced:

```text
Error: Argument or block definition required

  on secrets.tfvars line 1:
   1: {

An argument or block definition is required here.
```

Forcing the Dotenv store on a file containing an HCL object produced:

```text
Error unmarshalling file: invalid dotenv input line: }
```

The practical distinction is:

- SOPS can manage native HCL `*.tfvars` as an opaque binary document.
- SOPS cannot encrypt native HCL values individually or preserve HCL structure in the encrypted document.
- SOPS can manage `*.tfvars.json` as structured JSON and encrypt its leaf values individually.

Native HCL support remains open as
[getsops/sops#292](https://github.com/getsops/sops/issues/292).

## Root cause

### Unknown extensions select the binary store

SOPS recognizes YAML,
JSON,
Dotenv,
and INI suffixes.
Every other suffix defaults to `Binary`,
so `.tfvars` takes the binary path.
`cmd/sops/formats/formats.go:54-66` at SOPS tag `v3.13.2` contains:

```go
// FormatForPath returns the correct format given the path to a file
func FormatForPath(path string) Format {
	format := Binary // default
	if IsYAMLFile(path) {
		format = Yaml
	} else if IsJSONFile(path) {
		format = Json
	} else if IsEnvFile(path) {
		format = Dotenv
	} else if IsIniFile(path) {
		format = Ini
	}
	return format
}
```

The selected `Binary` format maps to the JSON-backed binary store.
`cmd/sops/common/common.go:62-68` contains:

```go
var storeConstructors = map[Format]storeConstructor{
	Binary: newBinaryStore,
	Dotenv: newDotenvStore,
	Ini:    newIniStore,
	Json:   newJsonStore,
	Yaml:   newYamlStore,
}
```

### Binary storage encrypts the complete HCL source as one value

The binary store wraps all plaintext bytes in one `data` item and emits the encrypted tree through the JSON store.
`stores/json/store.go:55-70` contains:

```go
// LoadPlainFile loads a plaintext json file onto a sops.Tree encapsulated
// within a sops.TreeBranches object
func (store BinaryStore) LoadPlainFile(in []byte) (sops.TreeBranches, error) {
	return sops.TreeBranches{
		sops.TreeBranch{
			sops.TreeItem{
				Key:   "data",
				Value: string(in),
			},
		},
	}, nil
}

// EmitEncryptedFile produces an encrypted json file's bytes from its corresponding sops.Tree object
func (store BinaryStore) EmitEncryptedFile(in sops.Tree) ([]byte, error) {
	return store.store.EmitEncryptedFile(in)
}
```

Decryption returns the original bytes rather than JSON.
`stores/json/store.go:75-85` contains:

```go
// EmitPlainFile produces plaintext json file's bytes from its corresponding sops.TreeBranches object
func (store BinaryStore) EmitPlainFile(in sops.TreeBranches) ([]byte, error) {
	if len(in) != 1 {
		return nil, fmt.Errorf("%w: there must be exactly one tree branch", BinaryStoreEmitPlainError)
	}
	// JSON stores a single object per file
	for _, item := range in[0] {
		if item.Key == "data" {
			if value, ok := item.Value.(string); ok {
				return []byte(value), nil
```

This explains why the SOPS round trip works while Terraform cannot consume the encrypted file directly.

### `exec-file` bridges decrypted bytes to Terraform

`exec-file` creates a private temporary directory,
uses either a FIFO or a regular file,
and substitutes its path for `{}` in the child command.
`cmd/sops/subcommand/exec/exec.go:71-105` contains:

```go
dir, err := os.MkdirTemp("", ".sops")
if err != nil {
	return err
}
defer os.RemoveAll(dir)

// ...

if opts.Fifo {
	// fifo handling needs to be async, even opening to write
	// will block if there is no reader present
	filename = opts.Filename
	if filename == "" {
		filename = FallbackFilename
	}
	filename, err = GetPipe(dir, filename)
	if err != nil {
		return err
	}
	go WritePipe(filename, opts.Plaintext)
} else {
	// GetFile handles opts.Filename == "" specially, that's why we have
	// to pass in opts.Filename without handling the fallback here
	handle, err := GetFile(dir, opts.Filename)
	if err != nil {
		return err
	}
	handle.Write(opts.Plaintext)
	handle.Close()
	filename = handle.Name()
}
```

`cmd/sops/subcommand/exec/exec.go:115-127` performs the substitution and waits for the child:

```go
placeholdered := strings.Replace(opts.Command, "{}", filename, -1)
cmd := BuildCommand(placeholdered)
cmd.Env = env

if opts.Background {
	return cmd.Start()
}

cmd.Stdin = os.Stdin
cmd.Stdout = os.Stdout
cmd.Stderr = os.Stderr

return cmd.Run()
```

The `--filename` option is important for `*.tfvars.json` because Terraform chooses JSON parsing from the temporary
file's suffix.
It is also useful for keeping a diagnostic filename for HCL `*.tfvars`.
The filename must be a local relative path.
On Windows,
SOPS disables FIFO delivery and uses a regular temporary file.
`cmd/sops/subcommand/exec/exec.go:66-80` contains:

```go
if runtime.GOOS == "windows" && opts.Fifo {
	log.Warn("no fifos on windows, use --no-fifo next time")
	opts.Fifo = false
}

// ...

if opts.Filename != "" {
	if filepath.IsAbs(opts.Filename) || !filepath.IsLocal(opts.Filename) {
		return fmt.Errorf("The provided filename is not a local path.")
	}
}
```

Passing `--no-fifo` also selects the regular-file branch.
`cmd/sops/main.go:368-374` contains:

```go
if err := exec.ExecWithFile(exec.ExecOpts{
	Command:    command,
	Plaintext:  output,
	Background: c.Bool("background"),
	Fifo:       !c.Bool("no-fifo"),
	User:       c.String("user"),
	Filename:   c.String("filename"),
```

## Verification

Verified on 2026-08-01 with:

- SOPS 3.13.2,
tag `v3.13.2`,
commit `15e36f97a8641769712da76b64aec4ea566be81e`.
- OpenTofu 1.12.5 on `linux_amd64`.
- A disposable age identity and a provider-free OpenTofu root module.

The native HCL fixture included a string,
a list,
and an object:

```hcl
secret_value = "tfvars probe secret"
ports        = [8080, 9090]
settings = {
  enabled = true
  mode    = "strict"
}
```

The round-trip harness was:

```bash
age-keygen --output identity.txt
cp -- secrets.tfvars secrets.tfvars.plain
sops encrypt \
  --age "$(age-keygen -y identity.txt)" \
  --in-place \
  secrets.tfvars
SOPS_AGE_KEY_FILE=identity.txt \
  sops decrypt --output secrets.tfvars.decrypted secrets.tfvars
cmp --silent secrets.tfvars.plain secrets.tfvars.decrypted
```

`cmp` succeeded,
proving a byte-identical binary round trip.

The consumer-boundary harness was:

```bash
SOPS_AGE_KEY_FILE=identity.txt \
  sops exec-file \
  --filename secrets.tfvars \
  secrets.tfvars \
  'tofu plan -input=false -lock=false -var-file={}'
```

The plan succeeded and rendered the list and object values from the decrypted HCL fixture.
The same harness succeeded for structured JSON after changing both source and temporary names to
`secrets.tfvars.json`.

### Patterns that work

- `secrets.tfvars` inferred as binary,
encrypted in place,
decrypted byte-identically,
and passed through `exec-file` with `--filename secrets.tfvars`.
- `secrets.tfvars.json` inferred as JSON,
with root keys preserved and leaf values encrypted,
then passed through `exec-file` with `--filename secrets.tfvars.json`.
- A `.sops.yaml` creation rule matching `\.tfvars(\.json)?$` selected the disposable age recipient for both forms.
- Default `exec-file` FIFO delivery worked with OpenTofu 1.12.5.
- Fake `_comment` properties remained valid JSON.
  SOPS encrypted them by default,
  while `--unencrypted-regex '^_comment$'` kept both root and nested comment values readable.
- OpenTofu discarded a nested `_comment` when converting into a fixed-shape `object` variable,
  but retained it as actual data when the destination variable was `map(string)`.

### Patterns that fail or mislead

- Passing encrypted `secrets.tfvars` directly to `-var-file` fails at the opening JSON `{`.
- Naming the encrypted file `probe.auto.tfvars` makes OpenTofu auto-load it and fail at the same opening `{`.
- `--input-type dotenv` rejects native HCL objects with
  `Error unmarshalling file: invalid dotenv input line: }`.
- `--input-type hcl` does not enable HCL parsing.
  An unknown type string maps to `Binary`.
- A root `_comment` in `*.tfvars.json` is an undeclared variable assignment,
  not syntax trivia.
  OpenTofu completed the plan but emitted `Warning: Value for undeclared variable`.

The binary fallback is implemented in `cmd/sops/formats/formats.go:26-31`:

```go
func FormatFromString(formatString string) Format {
	format, found := stringToFormat[formatString]
	if !found {
		return Binary
	}
	return format
}
```

The `--input-type hcl` probe therefore succeeded only as whole-file binary encryption and emitted the same JSON
`data` envelope.

## Verified workarounds

### Use `*.tfvars.json` with structured SOPS encryption

Terraform and OpenTofu parse variable files ending in `.json` as JSON objects.
SOPS recognizes the same suffix as JSON,
so it preserves keys and encrypts leaf values separately.

```yaml
# .sops.yaml
creation_rules:
  - path_regex: \.tfvars\.json$
    age:
      - age1replace-with-project-recipient
```

```bash
sops encrypt --in-place secrets.tfvars.json
sops exec-file \
  --filename secrets.tfvars.json \
  secrets.tfvars.json \
  'terraform plan -var-file={}'
```

Pros:
structured encrypted diffs,
individual leaf encryption,
and native format support in both tools.

Cons:
JSON does not preserve native HCL comments or HCL-specific string forms such as heredocs.
The encrypted source still must not use an auto-loaded name in the Terraform working directory.

### Add fake JSON comments as data

A valid JSON property can carry prose:

```json
{
  "_comment": "Why these deployment values exist",
  "region": "eu-west-1"
}
```

SOPS encrypts `_comment` like any other value by default.
To keep comment properties readable in the encrypted file,
use an unencrypted-key rule:

```yaml
# .sops.yaml
creation_rules:
  - path_regex: \.tfvars\.json$
    age: age1replace-with-project-recipient
    unencrypted_regex: '^_comment$'
```

Pros:
prose stays beside the value and remains visible in encrypted diffs when the regex exemption is used.

Cons:
JSON and Terraform treat `_comment` as data.
At the root,
it produces an undeclared-variable warning unless the module declares a dummy `_comment` input.
Declaring that input pollutes the module interface.
Inside nested values,
type conversion determines whether the property is discarded or reaches providers and outputs.
A fixed-shape `object` discarded the probe property,
while `map(string)` retained it.

Use this only when the team accepts those data-model effects.
A sidecar Markdown file is cleaner for durable rationale,
while native binary `*.tfvars` is cleaner when real inline comments are required.

### Use native HCL as SOPS binary data

A creation rule can match `.tfvars` even though its content store is binary:

```yaml
# .sops.yaml
creation_rules:
  - path_regex: \.tfvars$
    age:
      - age1replace-with-project-recipient
```

```bash
sops encrypt --in-place secrets.tfvars
sops exec-file \
  --filename secrets.tfvars \
  secrets.tfvars \
  'terraform plan -var-file={}'
```

Pros:
retains complete HCL syntax and comments after decryption.
On Unix-like platforms,
the default FIFO keeps plaintext off persistent storage.

Cons:
On Windows or with `--no-fifo`,
SOPS writes plaintext to a temporary regular file and removes the temporary directory after the child exits.
the encrypted diff is opaque because the entire source is one encrypted value.
SOPS cannot select,
set,
or exempt individual HCL keys.

Ranking:
`*.tfvars.json` > binary `*.tfvars`,
because structured encryption and readable diffs usually outweigh HCL-only syntax for variable data.
Binary `*.tfvars` ranks second because it is the only built-in route that preserves native HCL exactly.

## What does not work

- Adding `path_regex: \.tfvars$` does not add an HCL parser.
  A creation rule chooses recipients and encryption settings;
  suffix detection still chooses the binary store.
- Setting `--input-type hcl` does not add HCL support.
  Unknown type names fall back to binary in SOPS 3.13.2.
- Treating HCL as Dotenv is unsafe beyond a limited scalar subset.
  Lists,
  objects,
  expressions,
  and HCL comments do not share Dotenv grammar.
- Passing encrypted files directly to Terraform does not decrypt them.
  Terraform sees the SOPS JSON envelope or encrypted JSON leaf strings.
- Keeping encrypted `terraform.tfvars`,
  `terraform.tfvars.json`,
  `*.auto.tfvars`,
  or `*.auto.tfvars.json` in the Terraform working directory causes automatic loading before the `exec-file`
  argument can help.
  Use a non-auto-loaded source name such as `secrets.tfvars` or keep encrypted sources outside that directory.

## Upstream filing artifact

### Upstream filing decision

`.out-of-scope/` was checked and has no SOPS or HCL-format exemption.
Open and closed issues and pull requests were searched for `tfvars`,
`HCL`,
`Terraform`,
and format support.
The matching thread is the open enhancement
[getsops/sops#292](https://github.com/getsops/sops/issues/292),
"Support HCL files".
Its body and all comments were read.
[getsops/sops#761](https://github.com/getsops/sops/pull/761) is the merged change that added `--filename` to
`exec-file`.

The filing constraints resolve as follows:

1.  **Is it really upstream's fault?**
    No bug was found.
    SOPS documents YAML,
    JSON,
    ENV,
    INI,
    and binary stores;
    native HCL is an open enhancement.
2.  **Can upstream fix it?**
    Yes.
    A complete HCL store could be added,
    but it would be a feature rather than a correction to binary fallback.
3.  **Are they supporting this use case?**
    Partly.
    Issue 292 is open with `enhancement` and `help wanted` labels,
    but SOPS does not currently advertise HCL as supported.
4.  **Would the repo welcome our contribution?**
    Yes.
    `CONTRIBUTING.md:3-28` says the project welcomes contributions,
    asks for tests and documentation,
    and contains no ban on AI-assisted work.
5.  **Will they likely fix it?**
    No current maintainer commitment supports that conclusion.
    A maintainer wrote in issue 292 that they were unlikely to implement HCL themselves but would review a
    community patch;
    the enhancement has remained open since 2018.
6.  **Have we prototyped a minimal compatible fix?**
    No.
    The auto-prototype gate does not trigger because constraints 1 and 5 fail.
    The verified consumer-side routes already solve the immediate Terraform use case without modifying SOPS.

No new issue should be filed because issue 292 is the exact duplicate.
The `exec-file --filename` workaround is absent from that thread,
so an additive comment draft is retained but is not authorized for posting and does not satisfy this project's filing
gate.

### Additive comment draft, do not post as-is

~~~md
SOPS 3.13.2 can handle native HCL `*.tfvars` through its binary store today,
then pass the decrypted bytes to Terraform without a persistent plaintext file:

```bash
sops encrypt --in-place secrets.tfvars
sops exec-file \
  --filename secrets.tfvars \
  secrets.tfvars \
  'terraform plan -var-file={}'
```

For structured encryption,
`secrets.tfvars.json` works better because both SOPS and Terraform recognize JSON:

```bash
sops exec-file \
  --filename secrets.tfvars.json \
  secrets.tfvars.json \
  'terraform plan -var-file={}'
```

The `--filename` suffix matters for JSON parsing.
Avoid encrypted auto-loaded names such as `terraform.tfvars` and `*.auto.tfvars` in the Terraform working directory,
because Terraform tries to parse those encrypted sources before the explicit `-var-file` is useful.

Verified with SOPS 3.13.2 and OpenTofu 1.12.5.
The HCL route round-tripped byte-identically and the default `exec-file` FIFO produced a successful plan.
~~~

## Sources

- [SOPS common operations](https://getsops.io/docs/usage/common-operations/)
- [SOPS advanced usage](https://getsops.io/docs/usage/advanced/)
- [SOPS references](https://getsops.io/docs/reference/)
- [Terraform input variables](https://developer.hashicorp.com/terraform/language/values/variables)
- [OpenTofu 1.12 input variables](https://opentofu.org/docs/v1.12/language/values/variables/)
- [SOPS HCL enhancement issue](https://github.com/getsops/sops/issues/292)
