[Unreleased]
============

The version currently under development.

New features:

- Convenience getters `copied()` and `cloned()` for copyable types.
- Convenience setter `fluid_set!` for scoped assignment.
- `fluid_let!` now allows `'static` initializers.

Version 0.1.0 — 2019-03-12
==========================

Initial release of fluid-let.

- `fluid_let!` macro for defining global dynamically-scoped variables.
- `get()` and `set()` methods with closure-based interface for querying
  and modifying the dynamic environment.
