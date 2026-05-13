"""Best-effort terminal preview of QR Codes.

.. warning::

   Terminal preview is **not** a reliable QR Code scan target.  Font
   metrics, line spacing, character width, colour support, and terminal
   emulation vary across environments.  The output may look visually
   plausible but should never be assumed scannable.

   For reliable output use :func:`molqrc.export.to_svg` or
   ``molqrc 'text' -o qr.svg``.

This module is intentionally kept simple.  Alternative preview backends
(e.g. Kitty-protocol images, sixel) can be added later without touching
the encoder.
"""

import shutil


def to_text(matrix, side, *, border=1, chars=("##", "  ")):
    """Render a QR Code as a plain-text string for terminal display.

    Each module is represented by two characters repeated on two
    consecutive lines — a cheap way to approximate square cells.

    Args:
        matrix: ``bytearray`` of ``side * side`` bytes (0=white, 1=black).
        side: Side length in modules.
        border: White-module margin (≥ 1 recommended).
        chars: ``(dark, light)`` character pairs.  Default ``("##", "  ")``.

    Returns:
        A multi-line string suitable for ``print()``.
    """
    try:
        tw = shutil.get_terminal_size().columns
    except (ValueError, OSError):
        tw = 80

    # 2 columns per module for roughly-square cells; drop to 1 if too wide
    wide = 2 if (side + 2 * border) * 2 <= tw else 1
    dark = chars[0] if wide == 2 else chars[0][:1] * 2
    light = chars[1] if wide == 2 else chars[1][:1] * 2

    lines = []

    def _row(text_row):
        lines.append(text_row)
        lines.append(text_row)

    # top border
    for _ in range(border):
        _row(light * (side + 2 * border))

    # matrix
    for r in range(side):
        parts = [light * border]
        for c in range(side):
            parts.append(dark if matrix[r * side + c] else light)
        parts.append(light * border)
        _row("".join(parts))

    # bottom border
    for _ in range(border):
        _row(light * (side + 2 * border))

    return "\n".join(lines)
