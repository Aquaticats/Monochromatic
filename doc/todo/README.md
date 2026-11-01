# Todo

## Quick Links to Specialized Todo Lists

- [**Build System & Package Management**](build-system.md):
   mise,
   TypeScript,
   dependencies
- [**CLI Tools & Utilities**](cli-tools.md):
   Custom tools and automation scripts
- [**Documentation & UI/UX**](documentation.md):
   Content,
   design,
   and user experience
- [**Development Environment**](development.md):
   Tooling,
   setup
- [**Code Quality & Patterns**](code-quality.md):
   Linting,
   testing,
   best practices
- [**Package-Specific Improvements**](packages.md):
   Module library,
   config packages,
   style framework
- [**Security & Infrastructure**](security.md):
   Application security,
   deployment hardening
- [**Performance & Optimization**](performance.md):
   Build performance,
   runtime optimization
- [**Automation & DevOps**](automation.md):
   CI/CD,
   development automation,
   release management
- [**VM Dev Environment**](vm-dev-environment.md):
   Portable immutable VM image ([rationale](../philosophy/vm-dev-environment.md))

## Priority Overview

### Critical priority

Immediate action required:
 blocking development or production.

1. **Build System Reliability** → [Build System Todo](build-system.md#critical-issues)
   - Fresh clone setup problems blocking new developers
   - Package build order issues preventing successful builds
   - Mise task dependency optimization needed

2. **Security Fundamentals** → [Security Todo](security.md#application-security)
   - Dependency vulnerability scanning and management
   - Input validation for all user-facing applications
   - Secrets management audit and implementation

3. **Core Library Completion** → [Packages Todo](packages.md#module-library-packagemodulees)
   - Missing async iterator utilities for complete functional programming
   - Essential array and object utilities for developer productivity
   - Type guard and validation utilities for type safety

### High priority

Important improvements that significantly impact developer experience.

1. **Performance Optimization** → [Performance Todo](performance.md#build-performance)
   - Build system performance improvements
   - TypeScript compilation optimization
   - Bundle size and runtime performance optimization

2. **Development Automation** → [Automation Todo](automation.md#development-automation)
   - Enhanced pre-commit hooks and code quality automation
   - Development environment automation and consistency
   - Code generation and templating for productivity

### Normal priority

Valuable improvements that enhance the project.

1. **Documentation Enhancement** → [Documentation Todo](documentation.md#documentation-system-improvements)
   - PlantUML integration for architecture diagrams
   - Comprehensive API documentation generation
   - User experience improvements and content optimization

2. **Package Ecosystem** → [Packages Todo](packages.md#configuration-packages)
   - Configuration package improvements and templates
   - Style framework component library development
   - Figma plugin enhancements and additional tools

3. **Infrastructure Improvements** → [Security Todo](security.md#infrastructure-security)
   - Container security and deployment hardening
   - Monitoring and incident response procedures
   - Compliance framework implementation

### Low priority

Nice-to-have features and experimental improvements.

1. **Advanced Features** → [Packages Todo](packages.md#cross-package-improvements)
   - Advanced utilities and specialized functionality
   - Experimental patterns and libraries
   - Integration with external tools and services

2. **Optimization Projects** → [Performance Todo](performance.md#phase-3-advanced-weeks-5-8)
   - Advanced caching strategies
   - Infrastructure performance tuning
   - Predictive analytics and monitoring

## Current Sprint Focus

### Active development (March 2026)

- **MCP packages**:
   mcp-stdio implements MCP revision 2026-07-28 and backs package/mcp/mvm,
   verified against a live client;
   remaining work is concurrency
   ([#433](https://github.com/Aquaticat/Monochromatic/issues/433))
   and argument validation
   ([#434](https://github.com/Aquaticat/Monochromatic/issues/434))
- **Build System**:
   Fixing ordering issues for fresh clones → [Build System Todo](build-system.md#fresh-clone-setup-problems)
- **Security**:
   Establishing security fundamentals → [Security Todo](security.md#dependency-security)

### Next Sprint Candidates

- **Package Development**:
   Expanding module library functionality → [Packages Todo](packages.md#module-library-packagemodulees)
- **Performance**:
   Build system optimization → [Performance Todo](performance.md#build-performance)
- **Documentation**:
   API documentation automation → [Documentation Todo](documentation.md#api-documentation-automation)
- **Automation**:
   CI/CD pipeline enhancement → [Automation Todo](automation.md#cicd-pipeline)
