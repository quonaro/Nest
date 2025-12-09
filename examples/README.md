# Nest Examples

This directory contains comprehensive examples demonstrating all features of Nest CLI.

## Files

- **nestfile** - Main example configuration file with all features
- **.env** - Example environment variables file
- **docker.nest** - Docker-related commands (included via @include)
- **database.nest** - Database-related commands (included via @include)
- **testing.nest** - Testing-related commands (included via @include)

## Quick Start

1. Navigate to this directory:
   ```bash
   cd examples
   ```

2. Run any command:
   ```bash
   nest build
   nest dev
   nest deploy v1.0.0
   ```

## Features Demonstrated

- ✅ Global variables and constants
- ✅ Functions with parameters
- ✅ Basic commands with parameters
- ✅ Command dependencies (with and without arguments)
- ✅ Environment variables (direct and from .env file)
- ✅ Conditional execution (if/elif/else)
- ✅ Nested commands (groups)
- ✅ Before/after/fallback scripts
- ✅ Parameter validation with regex
- ✅ Template variables ({{now}}, {{user}})
- ✅ Complex workflows
- ✅ **Include directives** - modular configuration with separate files
- ✅ Positional and named arguments

## Examples

### Build for different targets
```bash
nest build --target x86_64
nest build --target arm64 --release true
```

### Run development server
```bash
nest dev
nest dev --watch true
```

### Run tests with coverage
```bash
nest test --coverage true
nest dev test --coverage true
```

### Deploy with validation
```bash
nest deploy v1.0.0 --env production
```

### Docker operations
```bash
nest docker build --tag v1.0.0
nest docker run --port 3000
nest docker deploy
```

### Full release pipeline
```bash
nest full_release v1.0.0 --target x86_64 --dry-run true
```

### Using included commands

Commands from included files are available directly:

**Docker commands:**
```bash
nest docker build --tag v1.0.0
nest docker run --port 3000
nest docker deploy
nest docker stop
nest docker logs
```

**Database commands:**
```bash
nest database migrate
nest database seed
nest database backup
nest database restore backup.sql
```

**Testing commands:**
```bash
nest test unit --coverage true
nest test integration
nest test e2e
nest test watch
nest test all
```

These commands are defined in separate `.nest` files and included in the main `nestfile` using `@include` directives.

