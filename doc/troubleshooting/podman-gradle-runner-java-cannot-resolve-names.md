# Podman Gradle runner: Java cannot resolve or reach hosts

## Symptom

A Gradle run in the capped podman container (host `/usr` mounted read-only over the Fedora image, `--network=host`)
fails to download a dependency with `dl.google.com: Temporary failure in name resolution`,
or, once names resolve, hangs in `NioSocketImpl.timedFinishConnect` for as long as the container lives.
`getent hosts` and `curl` inside the same container work.
Runs whose dependencies are all cached never notice.

## Observed

The host `nsswitch.conf` lists `resolve [!UNAVAIL=return]`, and the container has no `/run/systemd/resolve`.
A `Probe.java` run with the mise JDK inside the container failed every lookup while `getent hosts` succeeded,
so the missing socket is the suspect for the Java failure, not an established cause.
Mounting `/run/systemd/resolve:/run/systemd/resolve:ro` made the Java lookups succeed.
Connections from the container's Java then still time out (IPv4) or abort (IPv6) to Maven Central and Google Maven,
while host Java and in-container `curl` connect; the cause was not found.

## Workaround

Fetch new dependencies with a host Gradle run into the same `GRADLE_USER_HOME`
(`gradlew :app:testDebugUnitTest --no-daemon`, JDK and `ANDROID_HOME` set as the runner sets them),
then run the container with `--network=none` and Gradle `--offline`, which proves the run needs no network.
Keep the test-time downloads inside Gradle: the Robolectric framework jar is a Gradle configuration
(`robolectricRuntime` in `package/music-player/android-app/app/build.gradle.kts`), not Robolectric's own `~/.m2` fetch.
