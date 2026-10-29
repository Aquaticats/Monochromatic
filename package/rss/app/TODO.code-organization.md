# Code Organization

**Priority**:
 High

## 1. Objectives

Improve maintainability and structure by reorganizing code into logical modules with clear separation of concerns.

## 2. Tasks

### 2.1 Directory Structure

Create organized directory structure:

```text
src/
├── types/          # TypeScript interfaces and types
├── utils/          # Shared utility functions
├── services/       # Business logic services
├── api/            # API route handlers
├── config/         # Configuration management
├── client/         # Client-side code
└── server/         # Server-side code
```

#### 2.1.1 Types Directory (`/types`)

- [ ] 2.1.1.1 Create `feed.types.ts`:
       Feed-related interfaces
- [ ] 2.1.1.2 Create `item.types.ts`:
       Item interfaces
- [ ] 2.1.1.3 Create `outline.types.ts`:
       OPML/Outline types
- [ ] 2.1.1.4 Create `api.types.ts`:
       API request/response types
- [ ] 2.1.1.5 Create `config.types.ts`:
       Configuration interfaces
- [ ] 2.1.1.6 Move all interfaces from existing files

#### 2.1.2 Utils Directory (`/utils`)

- [ ] 2.1.2.1 Create `date.utils.ts`:
       Date parsing and formatting
- [ ] 2.1.2.2 Create `observable.utils.ts`:
       Observable helpers
- [ ] 2.1.2.3 Create `file.utils.ts`:
       File system operations
- [ ] 2.1.2.4 Create `validation.utils.ts`:
       Input validation
- [ ] 2.1.2.5 Create `constants.ts`:
       Shared constants
- [ ] 2.1.2.6 Extract utility functions from existing modules

#### 2.1.3 Services Directory (`/services`)

- [ ] 2.1.3.1 Move `feed.ts` → `services/feed.service.ts`
- [ ] 2.1.3.2 Move `item.ts` → `services/item.service.ts`
- [ ] 2.1.3.3 Move `outline.ts` → `services/outline.service.ts`
- [ ] 2.1.3.4 Move `ignore.ts` → `services/ignore.service.ts`
- [ ] 2.1.3.5 Move `asset.ts` → `services/asset.service.ts`
- [ ] 2.1.3.6 Move `opmls.ts` → `services/opml.service.ts`
- [ ] 2.1.3.7 Create `cache.service.ts` for caching logic

#### 2.1.4 API Directory (`/api`)

- [ ] 2.1.4.1 Extract API routes from `index.ts`
- [ ] 2.1.4.2 Create `api/feed.routes.ts`
- [ ] 2.1.4.3 Create `api/ignore.routes.ts`
- [ ] 2.1.4.4 Create `api/asset.routes.ts`
- [ ] 2.1.4.5 Create `api/middleware.ts` for rate limiting
- [ ] 2.1.4.6 Create `api/validation.ts` for request validation

#### 2.1.5 Config Directory (`/config`)

- [ ] 2.1.5.1 Create `config/index.ts`:
       Main configuration
- [ ] 2.1.5.2 Create `config/paths.ts`:
       Path configurations
- [ ] 2.1.5.3 Create `config/server.ts`:
       Server settings
- [ ] 2.1.5.4 Create `config/feed.ts`:
       Feed-specific settings
- [ ] 2.1.5.5 Move constants from existing files

### 2.2 Extract Constants

- [ ] 2.2.1 Move `FETCH_INTERVAL_MS` to config
- [ ] 2.2.2 Move display limit (100 items) to config
- [ ] 2.2.3 Move poll interval (1000ms) to config
- [ ] 2.2.4 Move file paths to config
- [ ] 2.2.5 Create environment-based configuration

### 2.3 Dependency Injection

- [ ] 2.3.1 Create service container/registry
- [ ] 2.3.2 Implement dependency injection for services
- [ ] 2.3.3 Remove direct imports between services
- [ ] 2.3.4 Use interfaces for service contracts
- [ ] 2.3.5 Enable easy mocking for tests

### 2.4 Error Classes

Create proper error hierarchy:

- [ ] 2.4.1 Create `errors/base.error.ts`
- [ ] 2.4.2 Create `errors/feed.error.ts`
- [ ] 2.4.3 Create `errors/parse.error.ts`
- [ ] 2.4.4 Create `errors/network.error.ts`
- [ ] 2.4.5 Create `errors/validation.error.ts`
- [ ] 2.4.6 Implement proper error handling

### 2.5 Factory Pattern

- [ ] 2.5.1 Create `factories/feed.factory.ts`
- [ ] 2.5.2 Create `factories/item.factory.ts`
- [ ] 2.5.3 Create `factories/observable.factory.ts`
- [ ] 2.5.4 Use factories for complex object creation
- [ ] 2.5.5 Simplify object initialization

### 2.6 Packaging and dependency hygiene

#### 2.6.1 Package metadata

- [x] 2.6.1.1 Package is private in `package/rss/app/package.json`.
- [x] 2.6.1.2 No misleading application `"module"` field is present.
- [ ] 2.6.1.3 Do not add an exports map unless this package is intended to be consumed as a library.
       See library example in [logger package exports](../../logger/core/package.json:6).

#### 2.6.2 Dependency classification

- [x] 2.6.2.1 Runtime server dependency is h3,
       not Elysia,
       and lives under `dependencies`.
- [ ] 2.6.2.2 Audit remaining runtime imports and ensure they live
        under `dependencies` in `package/rss/app/package.json`.

#### 2.6.3 Import style consistency

- [x] 2.6.3.1 Feedsmith types are imported from the package root,
       not deep `node_modules` paths.
- [x] 2.6.3.2 Node builtin path imports use the `node:` scheme in current RSS source.

#### 2.6.4 Monorepo conventions alignment

- [ ] 2.6.4.1 For applications,
       avoid library-oriented metadata (exports map,
       module field).
       Use the library package as reference for when an exports map is appropriate:
       [logger exports](../../logger/core/package.json:6).
- [ ] 2.6.4.2 Ensure mise tasks reflect app vs library roles (no change required now;
       just confirm tasks are app-focused for apps,
       and dual-build for libraries).

## 3. Migration Strategy

### 3.1 Phase 1: Create Structure

1. 3.1.1 Create new directory structure
2. 3.1.2 Copy files to new locations
3. 3.1.3 Update imports gradually

### 3.2 Phase 2: Refactor

1. 3.2.1 Extract interfaces to types directory
2. 3.2.2 Extract utilities to utils directory
3. 3.2.3 Extract constants to config

### 3.3 Phase 3: Enhance

1. 3.3.1 Implement dependency injection
2. 3.3.2 Add error classes
3. 3.3.3 Add factory patterns

### 3.4 Phase 4: Cleanup

1. 3.4.1 Remove old files
2. 3.4.2 Update documentation
3. 3.4.3 Update tests

## 4. Benefits

- 4.1 **Improved Maintainability**:
   Clear separation of concerns
- 4.2 **Better Testing**:
   Easier to mock dependencies
- 4.3 **Enhanced Readability**:
   Logical organization
- 4.4 **Easier Onboarding**:
   Clear structure for new developers
- 4.5 **Reduced Coupling**:
   Services communicate through interfaces
- 4.6 **Better Scalability**:
   Easy to add new features

## 5. Success Criteria

- 5.1 All code organized into appropriate directories
- 5.2 No circular dependencies
- 5.3 All constants externalized
- 5.4 Dependency injection implemented
- 5.5 Error handling improved
- 5.6 Tests updated to match new structure
