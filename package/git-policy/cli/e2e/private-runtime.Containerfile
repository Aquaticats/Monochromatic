# What:
#  An image with a Git, a D-Bus daemon, a counting stand-in for a service manager's program, and a wrapper under
#  test as `git`, for `bin/proof-private-runtime-directory.ts`.
# Why:
#  Issue 6293's reproduction runs in a container whose bus and manager are its own, so nothing in this image can
#  reach this machine's manager or session bus.
#  The stand-in is the program the wrapper looks for and its only manager interaction: it records every call, then
#  does what `systemd-run --user --scope` does, which is run the command as its child in the caller's environment.
#  The wrapper is copied in rather than compiled here, so the two arms (before and after the fix) are one build of
#  this branch's executable each and the image needs no Rust.
FROM debian:stable-slim
RUN apt-get update \
 && apt-get install -y --no-install-recommends git dbus procps \
 && rm -rf /var/lib/apt/lists/* \
 && mkdir -p /proof
COPY git-under-test /usr/local/bin/git
COPY stand-in-systemd-run /usr/local/bin/systemd-run
COPY repro-in-a-container.sh /proof/repro-in-a-container.sh
RUN chmod 0555 /usr/local/bin/git /usr/local/bin/systemd-run /proof/repro-in-a-container.sh
CMD ["/bin/sh", "/proof/repro-in-a-container.sh"]
