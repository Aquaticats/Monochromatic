# What:  the source snapshot a mutation campaign layers cargo-mutants over.
# Why:   the campaign runs in a mount-free container, so the crate, its lockfile, its tests and its cargo-mutants
#        configuration are baked into an image together with the fetched dependencies; the campaign itself runs offline.
#        The toolchain, cargo-nextest and cargo-mutants come from the base image built by mutants.Containerfile.
FROM localhost/forbidden-regex-mutants
WORKDIR /work
COPY . /work
RUN ["cargo", "fetch", "--locked"]
ENV CARGO_NET_OFFLINE=true
