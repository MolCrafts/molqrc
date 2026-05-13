"""molqrc — Python bindings for the molqrc QR Code C library."""

__version__ = "0.1.0"

import ctypes
import os
import sys
from ctypes import c_int, c_char_p, c_void_p, c_long, POINTER, CFUNCTYPE

# ---------------------------------------------------------------------------
#  Load the shared library
# ---------------------------------------------------------------------------

_lib_dir = os.path.dirname(os.path.abspath(__file__))

_lib_name = {
    "darwin": "libmolqrc.dylib",
    "linux": "libmolqrc.so",
    "win32": "molqrc.dll",
}.get(sys.platform, "libmolqrc.so")

# scikit-build-core installs the shared lib into the package directory
_lib_path = os.path.join(_lib_dir, _lib_name)
if not os.path.exists(_lib_path):
    # Fallback: local build directory
    _build_dir = os.path.normpath(os.path.join(_lib_dir, "..", "build"))
    _lib_path = os.path.join(_build_dir, _lib_name)

_lib = ctypes.CDLL(_lib_path)

# ---------------------------------------------------------------------------
#  Constants
# ---------------------------------------------------------------------------

ECL_L, ECL_M, ECL_Q, ECL_H = 0, 1, 2, 3
MODE_NUMERIC, MODE_ALPHANUMERIC, MODE_BYTE, MODE_KANJI, MODE_ECI = (
    0x1,
    0x2,
    0x4,
    0x8,
    0x7,
)
MASK_AUTO = -1
VERSION_MIN, VERSION_MAX = 1, 40
MAX_SIZE = 177

# ---------------------------------------------------------------------------
#  C function signatures
# ---------------------------------------------------------------------------

# int molqrc_encode_text(const char *, unsigned char *, int, int, int, int, int)
_lib.molqrc_encode_text.argtypes = [
    c_char_p,
    POINTER(ctypes.c_ubyte),
    c_int,
    c_int,
    c_int,
    c_int,
    c_int,
]
_lib.molqrc_encode_text.restype = c_int


# Segment type
class MolqrcSegment(ctypes.Structure):
    _fields_ = [
        ("mode", c_int),
        ("num_chars", c_int),
        ("data", POINTER(ctypes.c_ubyte)),
        ("bit_length", c_int),
    ]


# int molqrc_encode_segments(const MolqrcSegment *, int, unsigned char *, int, int, int, int, int)
_lib.molqrc_encode_segments.argtypes = [
    POINTER(MolqrcSegment),
    c_int,
    POINTER(ctypes.c_ubyte),
    c_int,
    c_int,
    c_int,
    c_int,
    c_int,
]
_lib.molqrc_encode_segments.restype = c_int

# Segment constructors
_lib.molqrc_is_numeric.argtypes = [c_char_p]
_lib.molqrc_is_numeric.restype = c_int

_lib.molqrc_is_alphanumeric.argtypes = [c_char_p]
_lib.molqrc_is_alphanumeric.restype = c_int

_lib.molqrc_calc_segment_bit_length.argtypes = [c_int, c_int]
_lib.molqrc_calc_segment_bit_length.restype = c_int

_lib.molqrc_make_bytes.argtypes = [
    POINTER(ctypes.c_ubyte),
    c_int,
    POINTER(ctypes.c_ubyte),
]
_lib.molqrc_make_bytes.restype = MolqrcSegment

_lib.molqrc_make_numeric.argtypes = [c_char_p, POINTER(ctypes.c_ubyte)]
_lib.molqrc_make_numeric.restype = MolqrcSegment

_lib.molqrc_make_alphanumeric.argtypes = [c_char_p, POINTER(ctypes.c_ubyte)]
_lib.molqrc_make_alphanumeric.restype = MolqrcSegment

_lib.molqrc_make_eci.argtypes = [c_long, POINTER(ctypes.c_ubyte)]
_lib.molqrc_make_eci.restype = MolqrcSegment

# Render
DrawRectFn = CFUNCTYPE(None, c_void_p, c_int, c_int, c_int, c_int)

_lib.molqrc_draw_text.argtypes = [
    c_char_p,
    c_int,
    c_int,
    c_int,
    c_int,
    DrawRectFn,
    c_void_p,
]
_lib.molqrc_draw_text.restype = c_int

_lib.molqrc_draw_matrix.argtypes = [
    POINTER(ctypes.c_ubyte),
    c_int,
    c_int,
    c_int,
    c_int,
    c_int,
    DrawRectFn,
    c_void_p,
]
_lib.molqrc_draw_matrix.restype = c_int


# ---------------------------------------------------------------------------
#  QRCode class
# ---------------------------------------------------------------------------


class QRCode:
    """A QR Code generated from text.

    Parameters
    ----------
    text : str
        Text to encode. Mode auto-detected: numeric → alphanumeric → byte.
    min_version : int
        Minimum QR version (1–40). Default 1.
    max_version : int
        Maximum QR version (1–40). Default 40.
    ecl : int
        Error correction level: ECL_L=0, ECL_M=1, ECL_Q=2, ECL_H=3.
        Default ECL_M.
    mask : int
        Mask pattern 0–7, or MASK_AUTO (-1). Default MASK_AUTO.
    boost_ecl : bool
        Boost ECL if it doesn't increase version. Default True.
    """

    def __init__(
        self,
        text,
        *,
        min_version=1,
        max_version=40,
        ecl=ECL_M,
        mask=MASK_AUTO,
        boost_ecl=True,
    ):
        if not text:
            raise ValueError("text must not be empty")
        buf = (ctypes.c_ubyte * (MAX_SIZE * MAX_SIZE))()
        side = _lib.molqrc_encode_text(
            text.encode("utf-8"),
            buf,
            min_version,
            max_version,
            ecl,
            mask,
            1 if boost_ecl else 0,
        )
        if side == 0:
            raise ValueError("encoding failed — text may be too long")
        self._side = side
        self._matrix = bytearray(buf[: side * side])
        self._text = text
        self._ecl = ecl

    @property
    def side(self):
        """Side length in modules."""
        return self._side

    @property
    def matrix(self):
        """Row-major module matrix: 0 = white, 1 = black."""
        return self._matrix

    @property
    def version(self):
        """QR Code version (1–40)."""
        return (self._side - 21) // 4 + 1

    @property
    def text(self):
        """The original encoded text."""
        return self._text

    def save(self, path, *, scale=10):
        """Save to SVG or PNG file."""
        ext = os.path.splitext(path)[1].lower()
        if ext == ".svg":
            from . import export

            return export.to_svg(self._matrix, self._side, path, scale=scale)
        if ext == ".png":
            from . import _png

            return _png.write(self._matrix, self._side, path, scale=scale)
        raise ValueError(f"Unsupported format '{ext}'. Use .svg or .png.")

    def preview(self, *, border=1):
        """Terminal preview (best-effort, not scannable)."""
        from . import preview

        return preview.to_text(self._matrix, self._side, border=border)

    def to_web(self, dir_path, *, title="QR Code"):
        """Generate a self-contained interactive HTML page."""
        from . import web

        web.generate(self._matrix, self._side, dir_path, title=title, text=self._text)

    def __repr__(self):
        return f"QRCode(version={self.version}, side={self.side}, text={self.text!r})"


# ---------------------------------------------------------------------------
#  Low-level segment API
# ---------------------------------------------------------------------------


def is_numeric(text):
    """Check if text can be encoded in numeric mode."""
    return bool(_lib.molqrc_is_numeric(text.encode("utf-8")))


def is_alphanumeric(text):
    """Check if text can be encoded in alphanumeric mode."""
    return bool(_lib.molqrc_is_alphanumeric(text.encode("utf-8")))


def make_segment_bytes(data, buf=None):
    """Create a byte-mode segment from raw bytes."""
    if buf is None:
        buf = (ctypes.c_ubyte * max(len(data), 1))()
    seg = _lib.molqrc_make_bytes(
        (ctypes.c_ubyte * len(data))(*data),
        len(data),
        buf,
    )
    return seg


def make_segment_numeric(digits):
    """Create a numeric-mode segment."""
    buf = (ctypes.c_ubyte * ((len(digits) * 10 + 7) // 8 + 1))()
    return _lib.molqrc_make_numeric(digits.encode("utf-8"), buf)


def make_segment_alphanumeric(text):
    """Create an alphanumeric-mode segment."""
    buf = (ctypes.c_ubyte * ((len(text) * 11 + 7) // 8 + 1))()
    return _lib.molqrc_make_alphanumeric(text.encode("utf-8"), buf)


def make_segment_eci(assign_val):
    """Create an ECI segment."""
    buf = (ctypes.c_ubyte * 3)()
    return _lib.molqrc_make_eci(assign_val, buf)


def encode_segments(
    segments,
    *,
    min_version=1,
    max_version=40,
    ecl=ECL_M,
    mask=MASK_AUTO,
    boost_ecl=True,
):
    """Encode a list of segments into a QR matrix.

    Returns (side, matrix) tuple.
    """
    n = len(segments)
    if n == 0:
        raise ValueError("at least one segment required")
    seg_array = (MolqrcSegment * n)(*segments)
    buf = (ctypes.c_ubyte * (MAX_SIZE * MAX_SIZE))()
    side = _lib.molqrc_encode_segments(
        seg_array,
        n,
        buf,
        min_version,
        max_version,
        ecl,
        mask,
        1 if boost_ecl else 0,
    )
    if side == 0:
        raise ValueError("encoding failed")
    return side, bytearray(buf[: side * side])
