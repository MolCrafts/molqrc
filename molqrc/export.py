"""QR Code export — file output.

Generates standard image formats from QR module matrices. Every
exporter preserves quiet zone, square aspect ratio, and the original
matrix content unchanged.

Currently supports SVG (vector format). Additional backends
can be added without touching the encoder.
"""

import os


def to_svg(matrix, side, path, *, scale=10, dark="#000", light="#fff"):
    """Write the QR Code as an SVG image file.

    Each module becomes a ``<rect>`` element.  The output is a clean,
    minimal SVG that scales perfectly and is decodable by standard QR
    scanners.

    Args:
        matrix: ``bytearray`` of ``side * side`` bytes (0=white, 1=black).
        side: QR Code side length in modules.
        path: Output ``.svg`` file path.
        scale: Pixels per module (default 10).
        dark: Fill colour for dark modules (default black).
        light: Fill colour for light modules (default white).

    Returns:
        The absolute path written.
    """
    quiet = 4
    size = (side + 2 * quiet) * scale

    lines = [
        '<?xml version="1.0" encoding="UTF-8"?>',
        f'<svg xmlns="http://www.w3.org/2000/svg"'
        f' width="{size}" height="{size}"'
        f' viewBox="0 0 {size} {size}">',
        f'<rect width="{size}" height="{size}" fill="{light}"/>',
    ]

    for r in range(side):
        y = (quiet + r) * scale
        for c in range(side):
            if matrix[r * side + c]:
                x = (quiet + c) * scale
                lines.append(
                    f'<rect x="{x}" y="{y}" width="{scale}" '
                    f'height="{scale}" fill="{dark}"/>'
                )

    lines.append("</svg>")

    with open(path, "w", encoding="utf-8") as f:
        f.write("\n".join(lines))

    return os.path.abspath(path)
