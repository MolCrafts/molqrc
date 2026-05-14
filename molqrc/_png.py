"""Optional PNG export via Pillow.

Internal module — use ``QRCode.save("file.png")`` instead.
"""


def write(matrix, side, path, *, scale=10):
    """Write a QR Code matrix as a PNG file.

    Requires Pillow (``pip install Pillow``).
    """
    try:
        from PIL import Image
    except ImportError:
        raise ImportError(
            "Pillow is needed for PNG.  "
            "Use .svg for SVG output, or: pip install Pillow"
        )

    quiet = 4
    total = side + 2 * quiet
    size = total * scale

    img = Image.new("RGB", (size, size), (255, 255, 255))
    pixels = img.load()

    for r in range(side):
        for c in range(side):
            if matrix[r * side + c]:
                y0 = (quiet + r) * scale
                x0 = (quiet + c) * scale
                for dy in range(scale):
                    for dx in range(scale):
                        pixels[x0 + dx, y0 + dy] = (0, 0, 0)

    img.save(path, "PNG")
    return path
