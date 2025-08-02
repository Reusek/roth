# Roth Forth Standard Library

This directory contains the standard library for the Roth Forth interpreter, organized into logical modules.

## Structure

- **`std.rt`** - Main include file that loads all standard library modules
- **`core.rt`** - Core constants, utilities, and basic operations
- **`stack.rt`** - Extended stack manipulation operations
- **`math.rt`** - Mathematical functions and operations
- **`io.rt`** - Input/output formatting and utilities
- **`control.rt`** - Extended control flow structures
- **`compiler.rt`** - Documentation of compiler-level words

## Usage

To use the standard library in your Forth programs:

```forth
INCLUDE std/std.rt
```

Or include individual modules:

```forth
INCLUDE std/core.rt
INCLUDE std/stack.rt
```

## Word Categories

### Compiler-Level Words (Cannot be redefined)
These are implemented directly in the compiler and provide the foundation:

**Stack Operations:** `DUP`, `DROP`, `SWAP`, `OVER`, `ROT`
**Arithmetic:** `+`, `-`, `*`, `/`, `MOD`, `NEGATE`
**Comparison:** `=`, `<>`, `<`, `>`, `<=`, `>=`
**Logical:** `AND`, `OR`, `NOT`
**I/O:** `.`, `.S`, `EMIT`, `KEY`, `CR`
**Control Flow:** `DO`, `?DO`, `LOOP`, `I`, `J`
**Definition:** `:`, `;`

### Standard Library Words (Implemented in Forth)
These are built using the compiler-level words:

**Core Extensions:** `TRUE`, `FALSE`, `1+`, `1-`, `0=`, `0<`, `0>`
**Stack Extensions:** `2DUP`, `2DROP`, `NIP`, `TUCK`, `2SWAP`
**Math Extensions:** `ABS`, `MIN`, `MAX`, `SQUARE`, `SIGN`
**I/O Extensions:** `SPACE`, `SPACES`, `TAB`
**Control Extensions:** `TIMES`, `FOR`, `NEXT`

## Implementation Notes

- All standard library words are implemented using only the compiler-level primitives
- The library follows traditional Forth conventions and naming
- Words are organized by functionality for easy maintenance
- Each module can be included independently if needed