[Unreleased]
============

The version currently under development.

Breaking changes:

- `set()` now accepts only `&T` instead of `Borrow<T>`, avoiding implicit
  temporary copies of types implementing `Copy`.

Version 0.1.0 — 2019-03-12
==========================

Initial release of fluid-let.

- `fluid_let!` macro for defining global dynamically-scoped variables.
- `get()` and `set()` methods with closure-based interface for querying
  and modifying the dynamic environment.
