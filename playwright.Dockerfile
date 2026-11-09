# Named playwright.Dockerfile (not Dockerfile.playwright) because nvim-web-devicons
# matches icons by file extension: "Dockerfile" is a recognized extension.
#
# Playwright test runner for immutable OSes (Bazzite, Fedora Atomic, etc.)
# that lack apt-get for installing browser dependencies.
#
# Only browsers and system libraries live in this container.
# @playwright/test stays in the host catalog for type checking and IDE support
# (test files import { test, expect } from it). Do not remove it from package.json.
#
# The version tag MUST match the @playwright/test resolution in pnpm-lock.yaml.
# Find available tags: https://mcr.microsoft.com/v2/playwright/tags/list
FROM mcr.microsoft.com/playwright:v1.65.0-noble@sha256:6e3fb709361521eb27161606e6da6d2674e5e89fa16516cb37d6fdd91372273d

RUN apt-get update && apt-get install -y unzip && rm -rf /var/lib/apt/lists/*

ENV BUN_INSTALL="/usr/local"
RUN curl -fsSL https://bun.sh/install | bash

WORKDIR /work
