# Pinned Rust 1.97 image plus its verified rust-src component supports actual standard-library resolution.
# The original compiler base is 62ba2f7ce22ba9bc501110d3452c7ae814fba367c46f7eea3629a79286353884.
FROM 84f24e75017a7d8afa1c69e51c52f37e7d8d644cd2597a01f4732ad0b386dc20
COPY vendor /work/vendor
COPY cargo-config /work/.cargo
COPY clippy.toml /work/clippy.toml
# The registry guard test reads the repository configuration beside the work root.
COPY monochromatic-lint.config.jsonc /work/monochromatic-lint.config.jsonc
COPY jsonc-edit /work/package/jsonc-edit/core-rust
COPY monochromatic-lint /work/package/linter/monochromatic-lint
WORKDIR /work/package/linter/monochromatic-lint
# Semantic-backend consumer controls used symbol-free development builds under the same memory cap.
COPY cargo-nextest /usr/local/cargo/bin/cargo-nextest
RUN ["cargo", "nextest", "--version"]
ENV CARGO_BUILD_JOBS=2 CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_NET_OFFLINE=true
CMD ["cargo", "nextest", "run", "--offline", "--locked", "--all-targets"]
