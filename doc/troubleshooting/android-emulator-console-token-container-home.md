# Android Emulator 37.1.11 console rejects host adb 37.0.1 when homes differ across a container

## Symptom

A disposable `Fold_No_Hardware_Probe` emulator was launched inside a
`podman run --network=host` container and appeared as `emulator-5580` to
the host Android Debug Bridge.
At shutdown,
the host command

```text
adb -s emulator-5580 emu kill
```

printed these exact console responses instead of closing the emulator:

```text
KO: authentication token does not match ~/.emulator_console_auth_token
KO: unknown command, try 'help'
```

The command tool reported exit status zero,
so a successful shell status alone would have hidden the rejection.
This is a console-authentication failure,
not evidence that the AVD stopped or that Android guest shutdown is
unsupported.
The original user AVD was not involved.

## Root cause

The ADB client source at
`mirror/platform_packages_modules_adb` commit
`1cf2f017d312f73b3dc53bda85ef2610e35a80e9`,
`client/console.cpp:32-52`,
constructs the console authentication message from **the invoking
client's home directory**:

```cpp
static const char auth_token_filename[] = ".emulator_console_auth_token";
std::string auth_token_path = adb_get_homedir_path();
auth_token_path += OS_PATH_SEPARATOR;
auth_token_path += auth_token_filename;
// ...
std::string command = "auth ";
command += token;
command += '\n';
```

At `client/console.cpp:104-126`,
`adb_send_emulator_command` opens the emulator's loopback console port,
prepends that authentication message,
then appends the requested command:

```cpp
std::string commands = adb_construct_auth_command();
for (int i = 1; i < argc; i++) {
    commands.append(argv[i]);
    commands.push_back(i == argc - 1 ? '\n' : ' ');
}
commands.append("quit\n");
```

The [Android Emulator console guide][console-guide] says the token passed
to `auth` must match `.emulator_console_auth_token` in the home directory
of the running emulator's console environment.
The project container ran its emulator as root without a mount of the
host's console-token file.
The host client therefore read a different home-token context from the
container client.
The failed host invocation and successful in-container invocation are
consistent with that documented mechanism;
no token contents were printed or copied.
This is expected authentication isolation,
not an upstream emulator bug established by the probe.

## Verification

The emulator binary reported version `37.1.11.0` (build `15917651`);
the host and container used the Android SDK's `adb` version `37.0.1-15733141`.
The cloned ADB client source was checked at the commit cited in
[Root cause](#root-cause).
The disposable emulator was already running inside the named container
`fold-search-ranking-avd`.

### Working pattern

```text
podman exec fold-search-ranking-avd /home/user/Android/Sdk/platform-tools/adb -s emulator-5580 emu kill
OK: killing emulator, bye bye
OK
```

The emulator's own log then reported a graceful shutdown wait.
`adb devices -l` no longer listed `emulator-5580`,
and `podman ps --filter name=fold-search-ranking-avd` listed no running
container.
A residual `multiinstance.lock` path had no holder according to `fuser`;
its existence alone did not establish a live emulator.

### Failing pattern

```text
/home/user/Android/Sdk/platform-tools/adb -s emulator-5580 emu kill
KO: authentication token does not match ~/.emulator_console_auth_token
KO: unknown command, try 'help'
```

The failure was reproduced once on the same live disposable emulator
before the working in-container command stopped it.
The work did not test another emulator image,
original AVD,
container user mapping or future SDK build.

## Verified workaround

Invoke `adb emu` from the container that owns the running emulator,
using its SDK-mounted ADB binary and the explicit disposable serial:

```text
podman exec fold-search-ranking-avd /home/user/Android/Sdk/platform-tools/adb -s emulator-5580 emu kill
```

This keeps the authentication-token lookup in the same container-home
context as the emulator and uses the console's documented `kill` command.
It requires the container to be alive and the caller to have permission
to run `podman exec`;
it closes the emulator and container,
so it is not suitable when a session must stay active.
No token copy,
console-auth disablement or host-home mutation is needed.

## Explicit existing-server boundary for a namespace client

A later ADB `37.0.1-15733141` control used a client entering only an owned
container's user/network namespaces.
The server starts inside the bounded container before the emulator.
The client keeps the generated runtime home for console authentication and
uses numeric loopback explicitly:

```text
# Inside the owned network namespace; connect to its existing server.
adb -H 127.0.0.1 -P 5037 -s emulator-5582 get-state
```

The same source revision's `socket_spec.cpp:155` to `158` treats only an
empty hostname or literal `localhost` as local for server-start decisions:

```cpp
// socket_spec.cpp, comparable checked source
static bool tcp_host_is_local(std::string_view hostname) {
    // FIXME
    return hostname.empty() || hostname == "localhost";
}
```

`client/adb_client.cpp:257` to `261` rejects a missing nonlocal server
rather than launching a replacement:

```cpp
// client/adb_client.cpp, comparable checked source
bool local = is_local_socket_spec(__adb_server_socket_spec);
if (fd == -2 && !local) {
    fprintf(stderr, "* cannot start server on remote host\n");
    return false;
}
```

An actual disposable 2 GiB/2 CPU control first started a default local
server on unused port `5049`,
observed `daemon started successfully`,
and killed it.
The explicit `-H 127.0.0.1 -P 5049 get-state` negative then returned:

```text
# Installed ADB diagnostic after the positive server was stopped.
* cannot start server on remote host
error: cannot connect to daemon at tcp:127.0.0.1:5049: failed to connect to '127.0.0.1:5049': Connection refused
```

It did not print a daemon-start success.
This verifies the missing-server boundary in that control,
not every ADB lifecycle condition.
The initial help guard inspected stdout alone and was corrected to include
stderr;
that rejected probe was a harness mistake,
not evidence that the flags were absent.
The numeric-loopback connection stays inside the entered namespace;
ADB's word `remote` describes its startup classification.
The tradeoff is intentional:
a missing server must be restarted by its bounded owner,
not silently spawned by an outside client.
No third-party source was changed and no upstream defect is claimed.

## The same split without a container: a private-HOME copy on the host (2026-10-15)

A release-workflow session ran a private reflink copy of the owned AVD on the host, as the systemd user unit
`rel-emu-music-player` from the launch form in `android-emulator-37-software-renderer-sigsegv.md`.
The unit set `HOME`, `ANDROID_USER_HOME` and `ANDROID_AVD_HOME` to directories of the copy and
`ANDROID_ADB_SERVER_PORT=5640`, with the emulator on ports `5612,5613`.
The calling shell kept the real `HOME` and exported only the port.
Emulator `37.1.11` and the SDK's `adb`, both from `/home/user/Android/Sdk`.

Two failures followed from the shell and the unit having different homes.

### Failure 1: the private adb server saw the device as unauthorized

The shell's `adb` calls ran under the real `HOME` against port `5640`.
Which call (or the emulator itself) started the server on that port was not recorded,
and the process list showed `adb -L tcp:5640 fork-server server`.
`adb devices` in the shell printed:

```text
List of devices attached
emulator-5612	unauthorized
```

The emulator's own journal, whose adb calls reached that same server, repeated:

```text
This adb server's $ADB_VENDOR_KEYS is not set
Try 'adb kill-server' if that seems wrong.
Otherwise check for a confirmation dialog on your device.'
```

Stopping the server and starting it again with the copy's home fixed it:

```text
ANDROID_ADB_SERVER_PORT=5640 /home/user/Android/Sdk/platform-tools/adb kill-server
HOME=/home/user/temp/agent/rel-emu-copy/home ANDROID_ADB_SERVER_PORT=5640 \
  /home/user/Android/Sdk/platform-tools/adb start-server
# adb devices -> emulator-5612	device
```

Which file decided the authorization is not established here.
The working hypothesis is that the copied guest trusts the key in the copy's `.android` directory (`adbkey.pub`),
and the server started under the real `HOME` presented a different key.
No `adb` client or server source was read for this, and no key content was printed.
The observation is only that the server's `HOME` decided `unauthorized` against `device`.

### Failure 2: `emu kill` from the real HOME failed the console token check

After the test, the graceful shutdown from the shell with the real `HOME` printed the same two lines as the
container case, and the unit stayed `active`:

```text
KO: authentication token does not match ~/.emulator_console_auth_token
KO: unknown command, try 'help'
active
```

This is the mechanism traced in [Root cause](#root-cause): the client builds the `auth` command from the token file in
its own home directory, and the emulator holds the token of the home it runs in.
Here the two homes differed on one machine, not across a container boundary.
Running the client with the emulator's home worked (the `adb: device still authorizing` line is from the
`shell sync` that ran while the new server was still starting):

```text
export ANDROID_ADB_SERVER_PORT=5640 HOME=/home/user/temp/agent/rel-emu-copy/home
adb start-server; adb -s emulator-5612 shell sync; sleep 8; adb -s emulator-5612 shell sync
adb -s emulator-5612 emu kill
# adb: device still authorizing
# OK: killing emulator, bye bye
# OK
# (systemctl --user is-active rel-emu-music-player -> inactive)
```

### Rule for a private-HOME copy

Run every `adb` client and the server of that emulator with the same `HOME`, `ANDROID_USER_HOME` and
`ANDROID_ADB_SERVER_PORT` as the emulator unit, including the first `adb devices` and the final `emu kill`.
The tradeoff is that the shell command needs those variables on each call, or an exported block, for the whole session.
Not tested: exporting only `HOME` without `ANDROID_USER_HOME` (the key lookup may differ), another emulator build,
and the same sequence with the real `HOME` and a copied token file (which this repository does not do).
No upstream defect is claimed; the behavior is consistent with the documented console authentication.

## What does not work

- Host `adb emu kill` with the container's console port visible through
  host networking fails authentication;
  visibility is not token parity.
- `adb shell cmd power help` listed sleep and wake controls,
  not a shutdown operation.
  This one command listing does not prove all Android guest-side shutdown
  paths impossible.
- Do not treat the host ADB command's zero shell status as a successful
  shutdown when the console printed `KO`.

## Upstream filing decision

`.out-of-scope/` was checked;
no emulator-console exemption applied.
A GitHub issue and PR search for `emulator_console_auth_token container`
and `emulator console auth token mismatch` returned no matches in the
queried index.
That is not evidence that no comparable report exists on Google's
separate issue tracker.

### Upstream fault

No upstream fault is established.
The source and official guide describe authentication tied to a home token;
the observed container and host used different home contexts.

### Fixability

An improved diagnostic could name the client's token location,
but there is no demonstrated emulator defect requiring a patch.

### Supported use case

`adb emu` and console authentication are documented;
token sharing between differing container homes is not a promised behavior
in the cited guide.

### Contribution policy

Not assessed because no upstream bug or documentation gap is established
for filing.

### Likely upstream response

Not assessed;
search silence is not a maintainer decision.

### Minimal upstream fix prototype

Not applicable because the consumer boundary workaround solved the problem
without changing upstream.

Nothing additive is ready to send upstream.
Do not file the following draft as-is:

~~~md
Title: Clarify emulator console token location for containerized adb clients

Host `adb -s emulator-5580 emu kill` returned
`KO: authentication token does not match ~/.emulator_console_auth_token`
for an emulator running under a different container home.
The Android Emulator console guide already explains home-directory token
matching;
`podman exec <running-emulator-container> adb -s emulator-5580 emu kill`
succeeded without copying a token.
No upstream bug or missing guidance has been demonstrated.
~~~

[console-guide]: https://developer.android.com/studio/run/emulator-console
