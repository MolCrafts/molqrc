# molqrc

Zero-dependency QR Code generator — C library + Python CLI.

Version 1–40, all four ECL levels (L/M/Q/H), Reed-Solomon error correction,
mask selection, automatic encoding mode detection (Numeric / Alphanumeric / Byte).

## Quick start

```bash
# Build the C library
cmake -B build && cmake --build build

# Install Python package
pip install -e .

# Terminal preview
molqrc preview "hello world"

# Save as SVG (zero-dependency)
molqrc pic "hello world" -o qr.svg

# Save as PNG (needs Pillow)
molqrc pic "hello world" -o qr.png

# Static web page → deploy to Cloudflare Pages
molqrc web build "hello world" -o dist/
```

## Deploy to Cloudflare Pages

Generate the static site, then deploy the output directory:

```bash
molqrc web build "Your Text" -o dist/

# Option A: via Wrangler CLI
npx wrangler pages deploy dist/

# Option B: connect GitHub repo → Cloudflare Pages dashboard
#   Build command: pip install -e . && cmake -B build && cmake --build build && molqrc web build "Text" -o dist/
#   Publish directory: dist/
```

The generated page is a single self-contained `index.html` — zero external dependencies,
deployable to any static host (Cloudflare Pages, Netlify, GitHub Pages, etc.).

## Python API

```python
from molqrc import QRCode

qr = QRCode("hello")

# Core
qr.version   # 2
qr.side      # 25 (modules)
qr.matrix    # bytearray, 0=white 1=black, row-major

# Advanced options
qr = QRCode("hello",
    min_version=1,        # 1–40
    max_version=40,
    ecl=ECL_M,            # ECL_L=0, ECL_M=1, ECL_Q=2, ECL_H=3
    mask=MASK_AUTO,       # 0–7 or -1 for auto
    boost_ecl=True,       # auto-boost ECL if version unchanged
)

# Export
qr.save("qr.svg")         # SVG (zero-dependency)
qr.save("qr.png")         # PNG (needs Pillow)

# Terminal preview (best-effort)
print(qr.preview())

# Static web page
qr.to_web("dist/", title="My QR")
```

## Segment API

```python
from molqrc import (
    make_segment_bytes, make_segment_numeric,
    make_segment_alphanumeric, make_segment_eci,
    encode_segments, ECL_M, MASK_AUTO,
)

# Mixed-mode: ECI + numeric + alphanumeric + binary
segs = [
    make_segment_eci(127),
    make_segment_numeric("1234567"),
    make_segment_alphanumeric("HELLO"),
    make_segment_bytes(b"binary data"),
]
side, matrix = encode_segments(segs, min_version=1, max_version=40,
                                ecl=ECL_M, mask=MASK_AUTO, boost_ecl=True)
```

## C API

```c
#include <molqrc.h>

// Simple encode
unsigned char matrix[177 * 177];
int side = molqrc_encode_text("hello", matrix, 1, 40,
                               MOLQRC_ECL_M, MOLQRC_MASK_AUTO, 1);

// Segment-based encode with full control
unsigned char buf[4096];
molqrc_segment_t segs[2];
segs[0] = molqrc_make_numeric("1234567890", buf);
segs[1] = molqrc_make_alphanumeric("HELLO", buf + 512);

int side = molqrc_encode_segments(segs, 2, matrix,
                                   1, 40, MOLQRC_ECL_Q,
                                   MOLQRC_MASK_AUTO, 0);

// Render matrix via callback
void draw(void *user, int x, int y, int w, int h) {
    // fill rect on your canvas
}
molqrc_draw_matrix(matrix, side, 0, 0, 400, 400, draw, NULL);
```

## Requirements

- **C**: C99 compiler, CMake 3.14+
- **Python**: 3.9+
- **Optional**: Pillow (for PNG output)

## Project structure

```
src/          C library (matrix, encoding, Reed-Solomon, segments)
include/      Public header (molqrc.h)
bindings/     Python ctypes bindings + CLI
tests/        C tests (ctest) + Python tests (pytest)
```

## License

BSD-3-Clause
