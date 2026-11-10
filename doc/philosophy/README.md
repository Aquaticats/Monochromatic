# Philosophy

This document outlines the core philosophy and decision-making principles that guide the Monochromatic project.

## Organization

The philosophy is organized into focused documents:

- [Portability and core principles](portability.md):
   Foundational principles around portability,
   interoperability,
   and detachable solutions
- [AGENTS.md philosophy](agents.md):
   What belongs in `AGENTS.md`,
   non-obvious actionable guidance neither agent nor human can infer from context
- [Documentation organization](documentation.md):
   Why doc families nest under doc/<family>/,
   how they are named,
   and the lifecycle and reference rules behind the layout
- [Build and execution](build-execution.md):
   Technical decisions about build systems and script execution
- [Tool choices](tool-choices.md):
   Rationale for HTTP framework,
   editor,
   linting,
   testing,
   bundler,
   and AI SDK selections
- [Secrets management](secrets.md):
   Why local secrets use mise-native sops/age over fnox,
   why the encrypted store is gitignored rather than committed,
   and the tool-behavior traps found by testing
- [Browser support](browser-support.md):
   Future considerations for browser feature adoption
- [CSS](css.md):
   h-css hyperscript pattern,
   Shadow DOM style injection,
   and why alternatives don't fit
- [Portable VM dev environment](vm-dev-environment.md):
   Reasoning behind the portable VM dev environment architecture

Each document provides the reasoning behind specific technical decisions and architectural choices.
