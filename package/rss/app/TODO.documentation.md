# Documentation

**Priority**:
 Critical

## 1. Objectives

Create comprehensive documentation to improve maintainability,
 onboarding,
 and usage of the RSS reader application.

## 2. Tasks

### 2.1 Code Documentation

- [ ] 2.1.1 Add TSDoc comments to all public functions
- [ ] 2.1.2 Document function parameters and return types
- [ ] 2.1.3 Add usage examples in comments
- [ ] 2.1.4 Document complex algorithms and business logic
- [ ] 2.1.5 Add inline comments for non-obvious code

### 2.2 API Documentation

- [ ] 2.2.1 Create OpenAPI/Swagger specification
- [ ] 2.2.2 Document all endpoints:
  - 2.2.2.1 `GET /`:
     RSS reader interface
  - 2.2.2.2 `POST /api/ignore/new`:
     Append an ignored item to JSONL storage
- [ ] 2.2.3 Document request/response schemas
- [ ] 2.2.4 Document error responses and status codes
- [ ] 2.2.5 Add authentication requirements (if any)
- [ ] 2.2.6 Generate interactive API documentation

### 2.3 Architecture Documentation

- [ ] 2.3.1 Create system architecture diagram
- [ ] 2.3.2 Document data flow (OPML → Outline → Feed → Item → HTML)
- [ ] 2.3.3 Document Observable pattern usage
- [ ] 2.3.4 Document caching strategy
- [ ] 2.3.5 Document security measures (iframe sandboxing)
- [ ] 2.3.6 Document rate limiting implementation
- [ ] 2.3.7 Create sequence diagrams for key workflows

### 2.4 Setup and Deployment Guide

- [ ] 2.4.1 Document prerequisites
- [ ] 2.4.2 Create step-by-step installation guide
- [ ] 2.4.3 Document environment variables
- [ ] 2.4.4 Document OPML file configuration
- [ ] 2.4.5 Document Caddy configuration
- [ ] 2.4.6 Document Docker deployment
- [ ] 2.4.7 Document production best practices

### 2.5 Troubleshooting Guide

- [ ] 2.5.1 Common issues and solutions
- [ ] 2.5.2 Log file locations and interpretation
- [ ] 2.5.3 Debugging feed parsing issues
- [ ] 2.5.4 Handling rate limiting
- [ ] 2.5.5 Performance troubleshooting
- [ ] 2.5.6 Network connectivity issues

### 2.6 OPML Format Documentation

- [ ] 2.6.1 Document supported OPML structure
- [ ] 2.6.2 Document outline attributes
- [ ] 2.6.3 Document file:
      // protocol usage
- [ ] 2.6.4 Document http:
      // protocol usage
- [ ] 2.6.5 Provide example OPML files
- [ ] 2.6.6 Document limitations and constraints

### 2.7 User Guide

- [ ] 2.7.1 How to add/remove feeds
- [ ] 2.7.2 How to organize feeds in OPML
- [ ] 2.7.3 How to use the ignore feature
- [ ] 2.7.4 Understanding the UI elements
- [ ] 2.7.5 Keyboard shortcuts (if any)
- [ ] 2.7.6 Browser compatibility

### 2.8 Packaging and dependency conventions

- [ ] 2.8.1 Document app vs library metadata conventions
  - Apps are marked private and do not expose exports maps.
  - Libraries define an exports map with node/default targets (example:
     [exports map](../../logger/core/package.json:6)).
  - For apps,
     do not set a misleading "module" field.
     If present,
     it must point to a real file;
     otherwise remove it.

- [ ] 2.8.2 Document runtime vs devDependency classification
  - Any import used at runtime must be under "dependencies".
  - Example:
     h3 is used at runtime in `package/rss/app/src/index.ts` and belongs in dependencies,
     not devDependencies.

- [ ] 2.8.3 Document Node builtin import style
  - Prefer `"node:"` specifier for core modules.
  - Current RSS code already uses `node:path` in `package/rss/app/src/outline.ts`;
     keep that style for new imports.

- [x] 2.8.4 Document type import style for feedsmith
  - Current RSS code imports feedsmith types from the package root in `package/rss/app/src/feed.ts`.

- [ ] 2.8.5 Cross-link conventions to Code Organization
  - Reference actionable tasks in [Code Organization 2.6](TODO.code-organization.md) to keep docs and refactors aligned.

## 3. Documentation Standards

### 3.2 Diagram Tools

- 3.2.1 Use SVG for architecture diagrams
- 3.2.2 Use `graphviz` for complex system diagrams
- 3.2.3 Include diagrams in markdown documentation

### 3.3 Documentation Structure

```txt
doc/
├── api/                # API documentation
├── architecture/       # System architecture
├── guides/            # User and deployment guides
├── troubleshooting/   # Problem-solving guides
└── examples/          # Example configurations
```

## 4. Success Criteria

- 4.1 All public APIs are documented
- 4.2 New developers can onboard within 1 hour
- 4.3 Users can configure feeds without assistance
- 4.4 Troubleshooting guide covers 90% of issues
- 4.5 Documentation is searchable and well-organized
