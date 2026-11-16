# Open Questions

- **Kanji mode.** `QrSegmentMode::Kanji` exists in the vendored enum, but
  upstream's Rust port ships no `make_kanji()` constructor (it is Java-only), so
  `make_segments()` never emits Kanji and CJK text falls back to Byte mode via
  UTF-8. Adding it would mean writing code that upstream does not have, which
  forks `src/lib.rs` — it would have to live in a separate module instead.
  README previously claimed Kanji support; corrected in 0.2.0.

- **npm package name.** Publishes as `@molcrafts/molqrc-wasm` (derived from the
  crate name), whereas molrs publishes as `@molcrafts/molrs`. Aligning would
  require renaming the wasm crate to `molqrc`, which collides with the core
  crate inside the same workspace.

- Windows CI? (currently ubuntu-only)
