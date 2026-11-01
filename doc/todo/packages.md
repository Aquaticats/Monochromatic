# Package-Specific Todo

## Package Development and Improvements

This file tracks improvements needed across different packages in the monorepo,
 organized by package category.

### Quick Links

- [**Module Library**](#module-library-packagemodulees):
   Functional programming utilities
- [**Configuration Packages**](#configuration-packages):
   Shareable tool configurations
- [**Style Packages**](#style-packages):
   CSS framework and design system
- [**Site Packages**](#site-packages):
   Applications and documentation
- [**Figma Plugins**](#figma-plugins):
   Design tool integrations
- [**Build Utilities**](#build-utilities):
   Build-time tools and scripts

---

## Module Library (package/module/es)

### High Priority

#### Expand Array Utilities

**Status**:
 Normal priority,
 core library expansion

- [ ] Add `array.shuffle()`:
       Random array shuffling with proper Fisher-Yates algorithm
- [ ] Add `array.rotate()`:
       Rotate array elements left/right by n positions
- [ ] Add `array.partition()`:
       Split array into chunks based on predicate
- [ ] Add `array.difference()`:
       Find elements in first array not in second
- [ ] Add `array.intersection()`:
       Find common elements between arrays
- [ ] Add `array.union()`:
       Combine arrays with unique elements
- [ ] Add `array.zip()`:
       Combine multiple arrays element-wise
- [ ] Add `array.unzip()`:
       Split array of tuples into separate arrays

#### Async Iterator Utilities

**Status**:
 High priority,
 missing critical functionality

- [ ] Add `asyncIterable.map()`:
       Transform async iterables
- [ ] Add `asyncIterable.filter()`:
       Filter async iterables
- [ ] Add `asyncIterable.reduce()`:
       Reduce async iterables
- [ ] Add `asyncIterable.take()`:
       Take first n items from async iterable
- [ ] Add `asyncIterable.skip()`:
       Skip first n items from async iterable
- [ ] Add `asyncIterable.batch()`:
       Batch async iterable into groups
- [ ] Add `asyncIterable.parallel()`:
       Process async iterable in parallel with concurrency limit

#### Object Utilities

**Status**:
 Normal priority,
 common operations

- [x] Add record `pick` helper in current `module-es` taxonomy
- [x] Add record `omit` helper in current `module-es` taxonomy
- [x] Add record `merge` helper in current `module-es` taxonomy
- [ ] Add root-friendly aliases or documentation for record pick,
       omit,
       and merge
- [ ] Add `object.flatten()`:
       Flatten nested objects with dot notation keys
- [ ] Add `object.unflatten()`:
       Convert dot notation keys back to nested objects
- [ ] Add `object.transform()`:
       Transform object keys and values
- [ ] Add `object.isEmpty()`:
       Check if object has no enumerable properties

#### Date and Time Utilities

**Status**:
 Normal priority,
 common need

- [ ] Add `date.format()`:
       Format dates with locale support
- [ ] Add `date.parse()`:
       Parse dates with multiple format support
- [ ] Add `date.add()`:
       Add time intervals to dates
- [ ] Add `date.subtract()`:
       Subtract time intervals from dates
- [ ] Add `date.diff()`:
       Calculate differences between dates
- [ ] Add `date.isValid()`:
       Validate date objects
- [ ] Add `date.timezone()`:
       Timezone conversion utilities

#### Validation and Type Guards

**Status**:
 High priority,
 type safety

- [ ] Add `validate.email()`:
       Email validation with proper RFC compliance
- [ ] Add `validate.url()`:
       URL validation with protocol options
- [ ] Add `validate.json()`:
       JSON validation with schema support
- [ ] Add `validate.range()`:
       Number range validation
- [ ] Add `validate.length()`:
       String/array length validation
- [ ] Add `is.plainObject()`:
       Check for plain objects vs class instances
- [ ] Add `is.emptyArray()`:
       Type-safe empty array check
- [ ] Add `is.nonEmptyArray()`:
       Type-safe non-empty array check

### Medium Priority

#### Math Utilities

- [ ] Add `math.clamp()`:
       Constrain number to range
- [ ] Add `math.lerp()`:
       Linear interpolation
- [ ] Add `math.round()`:
       Round to specified decimal places
- [ ] Add `math.random()`:
       Seeded random number generation
- [ ] Add `math.statistics()`:
       Mean,
       median,
       mode calculations

#### Path and URL Utilities

- [x] Decide whether `module-fs-path` should publicly export `normalize()`;
       path helpers moved out of `module-es`.
       Decided 2026-09-07:
       public,
       delegating to `node:path/posix` under the `node` condition
       (`package/fs-path/core/DECISIONS.md`)
- [ ] Add `url.build()`:
       Build URLs with query parameters
- [ ] Add `url.parse()`:
       Parse URLs into components

---

## Configuration Packages

### TypeScript Configuration (package/config/typescript)

#### High Priority

- [ ] Create strict configuration variant for new projects
- [ ] Add configuration for different build targets (Node 18/20,
       ES2022/ES2023)
- [ ] Document path mapping best practices
- [ ] Add configuration validation utilities

#### Medium Priority

- [ ] Create template tsconfig files for common scenarios
- [ ] Document performance optimization settings

> TypeScript project references are out of scope.
> See [`.out-of-scope/typescript-project-references.md`](../../.out-of-scope/typescript-project-references.md).

---

## Style Packages

### Monochromatic CSS Framework (package/style/monochromatic)

#### High Priority

- [ ] Complete color system documentation with usage examples
- [ ] Add component library (buttons,
       forms,
       navigation)
- [ ] Create responsive grid system
- [ ] Add dark mode support with CSS custom properties
- [ ] Document design tokens and usage patterns

#### Medium Priority

- [ ] Add animation and transition utilities
- [ ] Create print stylesheet support
- [ ] Add component testing with visual regression tests
- [ ] Create Figma design tokens integration
- [ ] Add CSS-in-JS export format

#### Low Priority

- [ ] Add right-to-left (RTL) language support
- [ ] Create CSS framework performance metrics
- [ ] Add browser compatibility testing

---

## Site Packages

### Documentation Site (package/site/astro-test)

#### High Priority

- [ ] Complete API documentation generation from TSDoc comments
- [ ] Add interactive code examples with live preview
- [ ] Implement search functionality across all documentation
- [ ] Add contribution guidelines and developer onboarding docs
- [ ] Create architecture decision record (ADR) templates

#### Medium Priority

- [ ] Add automated screenshots for visual components
- [ ] Implement feedback collection system
- [ ] Add performance monitoring for documentation site
- [ ] Create multi-language support for documentation

### RSS Reader (package/rss/app)

**Note**:
 RSS package has comprehensive TODO files already.
 See:

- [`package/rss/app/TODO.index.md`](../../package/rss/app/TODO.index.md) for overview
- Individual TODO files for testing,
   performance,
   caching,
   etc.

---

## Figma Plugins

### CSS Variables Plugin (package/figma-plugin/css-variables)

#### High Priority

- [ ] Add support for design tokens export in multiple formats (JSON,
       YAML,
       JS)
- [ ] Implement color palette synchronization with CSS framework
- [ ] Add validation for CSS variable naming conventions
- [ ] Create automated testing for Figma API integration
- [ ] Add error handling and user feedback improvements

#### Medium Priority

- [ ] Add support for typography scale export
- [ ] Create spacing scale integration
- [ ] Add component documentation generation
- [ ] Implement design system validation rules
- [ ] Add plugin performance monitoring

#### Low Priority

- [ ] Create additional Figma plugins for other design system needs
- [ ] Add integration with other design tools (Sketch,
       Adobe XD)
- [ ] Implement design token versioning and migration

---

## Build Utilities

### Backup Path Utility (package/build/backup-path)

#### Medium Priority

- [ ] Add configuration for backup retention policies
- [ ] Implement backup verification and integrity checks
- [ ] Add compression support for large backup files
- [ ] Create restore functionality with conflict resolution

### Dependency Checker (package/build/ensure-dependencies)

#### High Priority

- [ ] Add security vulnerability scanning
- [ ] Implement outdated dependency detection
- [ ] Add license compatibility checking
- [ ] Create dependency update automation with testing

### Time Utilities (package/build/time)

#### Low Priority

- [ ] Add build time tracking and reporting
- [ ] Create performance regression detection
- [ ] Add build time optimization recommendations

---

## Cross-Package Improvements

### Documentation Standards

- [ ] Ensure all packages have comprehensive README files
- [ ] Add consistent `package.json` metadata across all packages
- [ ] Create package development guidelines
- [ ] Add automated package documentation generation

### Testing Standards

- [ ] Establish minimum test coverage thresholds per package
- [ ] Add integration testing between packages
- [ ] Create performance benchmarking for critical packages
- [ ] Add visual regression testing for UI packages

### Build and Release

- [ ] Implement automated semantic versioning
- [ ] Add automated changelog generation
- [ ] Create package publishing automation
- [ ] Add dependency update automation with testing

---

## Success Criteria

- All packages have clear,
   focused improvements identified
- Each improvement has assigned priority and success criteria
- Package interdependencies are documented and managed
- All packages follow consistent development standards
- Regular package health monitoring is implemented
