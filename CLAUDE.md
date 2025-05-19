# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Implementation Notes

1. Always check existing utility functions before implementing new ones. For example, use `core/src/rename.rs` for case conversion rather than creating custom functions.

## Project Overview

Typeshare is a tool that converts Rust types into equivalent types in other languages (Swift, Go, Python, Kotlin, Scala, and TypeScript). It simplifies managing types across language boundaries by generating code that matches the Rust type definitions.

The project is organized into multiple crates:
- `typeshare` - Main workspace
- `typeshare-core` - Core library containing type definitions, parsing, and code generation
- `typeshare-annotation` - Provides the `#[typeshare]` proc macro attribute
- `typeshare-cli` - CLI tool for generating code
- `typeshare-lib` - Common utilities

## Development Commands

### Building the Project

```bash
# Build the entire project
cargo build

# Build a specific crate
cargo build -p typeshare-cli
cargo build -p typeshare-core
cargo build -p typeshare-annotation
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p typeshare-core

# Run a specific test
cargo test -p typeshare-core --test snapshot_tests -- can_handle_serde_rename_all::swift

# Update snapshot test expectations
env UPDATE_EXPECT=1 cargo test -p typeshare-core

# Update a specific test's expectations
env UPDATE_EXPECT=1 cargo test -p typeshare-core --test snapshot_tests -- can_handle_serde_rename_all::swift

# Run determinism tests
cd tests && just
```

### Running the CLI

```bash
# Build and run the CLI with cargo
cargo run -p typeshare-cli -- --lang=typescript input.rs

# With installed binary
typeshare --lang=typescript input.rs
typeshare --lang=swift input.rs
typeshare --lang=kotlin --java-package=com.some.package.name input.rs
typeshare --lang=scala --scala-package=com.some.package.name input.rs
```

## Core Architecture

### Workflow

1. **Annotation**: Rust types are annotated with `#[typeshare]` to mark them for code generation
2. **Parsing**: The CLI parses Rust source files to extract annotated types
3. **Reconciliation**: Type references are reconciled across the entire codebase
4. **Code Generation**: Type definitions are generated for the target language
5. **Formatting**: Generated code is formatted using the appropriate tool for the target language

### Key Components

- **Proc Macro Attribute**: The `#[typeshare]` attribute marks Rust types for conversion
- **Parser**: Extracts annotated types from Rust source code
- **Language Implementations**: Separate modules for each target language (Swift, TypeScript, etc.)
- **Configuration System**: Supports customization through CLI args or config files
- **Snapshot Testing**: Data-driven testing methodology for validating output

### Language Support

The core supports generating code for:
- TypeScript
- Swift
- Kotlin
- Scala
- Go (experimental/feature-gated)
- Python (experimental/feature-gated)

Each language has its own implementation in `core/src/language/` with specific formatting and type mapping logic.