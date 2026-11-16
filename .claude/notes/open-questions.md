# Open Questions

- **Kanji mode.** `QrSegmentMode::Kanji` exists in the enum, but
  upstream's Rust library ships no `make_kanji()` constructor (it is Java-only), so
  `make_segments()` never emits Kanji and CJK text falls back to Byte mode via
  UTF-8. Adding one needs a reference to conform to (Java's QrSegmentAdvanced).
  README previously claimed Kanji support; corrected in 0.2.0.

- **npm package name.** Publishes as `@molcrafts/molqrc-wasm` (derived from the
  crate name), whereas molrs publishes as `@molcrafts/molrs`. Aligning would
  require renaming the wasm crate to `molqrc`, which collides with the core
  crate inside the same workspace.

- Windows CI? (currently ubuntu-only)
