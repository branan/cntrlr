# Cntrlr-macros

This crate is part of [Cntrlr](https://crates.io/crates/cntrlr).

Cntrlr-macros provides two sets of procedural macros: Those used to
implement Cntlr, and those used to implement applications based on
Cntrlr.

The application-support macros are re-exported by Cntrlr under the
module `cntrlr::macros`. Applications should prefer that location to
accessing this crate directly.
