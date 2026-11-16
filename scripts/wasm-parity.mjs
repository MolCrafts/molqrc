// Compares the compiled WebAssembly build against native encoder output,
// module for module. Driven by scripts/verify-wasm-parity.sh.
//
//   node scripts/wasm-parity.mjs <path-to-nodejs-pkg/molqrc.js> <native_ref.json>

import { readFileSync } from 'node:fs';
import { createRequire } from 'node:module';

const require = createRequire(import.meta.url);
const wasm = require(process.argv[2]);
const cases = JSON.parse(readFileSync(process.argv[3], 'utf8'));

let ok = 0;
let fail = 0;

function bad(label, c, msg) {
  fail++;
  console.log(`MISMATCH [${label}] text=${JSON.stringify(c.text).slice(0, 40)} ecl=${c.ecl}: ${msg}`);
}

for (const c of cases) {
  // qr_modules / qr_side mirror QrCode::encode_text, which boosts the ECC level
  // when that is free.
  const mods = Array.from(wasm.qr_modules(c.text, c.ecl)).join('');
  const side = wasm.qr_side(c.text, c.ecl);
  if (mods === c.boost.bits && side === c.boost.side) ok++;
  else bad('qr_modules', c, `side ${side} vs ${c.boost.side}, bits differ=${mods !== c.boost.bits}`);

  // qr_encode mirrors encode_segments_advanced(.., boostecl = false), so it must
  // reproduce the requested ECC level, version and mask exactly.
  const r = wasm.qr_encode(c.text, c.ecl, 1, -1);
  const rb = Array.from(r.modules).join('');
  if (rb === c.noboost.bits && r.side === c.noboost.side
      && r.version === c.noboost.version && r.mask === c.noboost.mask) ok++;
  else bad('qr_encode', c, `v${r.version}/m${r.mask}/s${r.side} vs v${c.noboost.version}/m${c.noboost.mask}/s${c.noboost.side}`);

  // The SVG path must carry exactly one "M" subpath per dark module.
  const svg = wasm.qr_svg(c.text, c.ecl, 4);
  const dark = [...c.boost.bits].filter((b) => b === '1').length;
  const subpaths = (svg.match(/M/g) || []).length;
  if (subpaths === dark) ok++;
  else bad('qr_svg', c, `${subpaths} subpaths vs ${dark} dark modules`);
}

console.log(`\nwasm/native parity: ${ok} checks identical, ${fail} mismatched`);
process.exit(fail ? 1 : 0);
