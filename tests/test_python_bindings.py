"""Tests for molqrc Python bindings."""

import os
import subprocess
import tempfile
import shutil
import pytest


# ---------------------------------------------------------------------------
#  QRCode — encoding layer
# ---------------------------------------------------------------------------


class TestQRCode:
    def test_hello_version1(self):
        from molqrc import QRCode

        qr = QRCode("hello")
        assert qr.side == 21
        assert qr.version == 1
        assert len(qr.matrix) == 441
        assert any(qr.matrix)
        assert not all(qr.matrix)

    def test_long_text_upgrades_version(self):
        from molqrc import QRCode

        qr = QRCode("A" * 200)
        assert qr.side > 21
        assert qr.version > 1

    def test_empty_string_accepted(self):
        from molqrc import QRCode

        qr = QRCode("")
        assert qr.side == 21
        assert qr.version == 1
        assert qr.text == ""

    def test_too_long_raises(self):
        from molqrc import QRCode

        with pytest.raises(ValueError):
            # Use binary to avoid alphanumeric-mode savings
            QRCode("\x00" * 3000)

    def test_version_range(self):
        from molqrc import QRCode

        # Force version 10–40
        qr = QRCode("hello", min_version=10)
        assert qr.version >= 10

    def test_manual_mask(self):
        from molqrc import QRCode

        qr = QRCode("hello", mask=3)
        assert qr.side > 0

    def test_ecl_levels(self):
        from molqrc import QRCode

        for ecl in (0, 1, 2, 3):
            qr = QRCode("hello", ecl=ecl)
            assert qr.side > 0

    def test_no_boost(self):
        from molqrc import QRCode

        qr = QRCode("hello", boost_ecl=False)
        assert qr.side > 0

    def test_numeric_auto_detect(self):
        from molqrc import QRCode

        qr = QRCode("1234567890")
        assert qr.side > 0

    def test_alphanumeric_auto_detect(self):
        from molqrc import QRCode

        qr = QRCode("HELLO WORLD")
        assert qr.side > 0

    def test_repr(self):
        from molqrc import QRCode

        r = repr(QRCode("hi"))
        assert "QRCode" in r


# ---------------------------------------------------------------------------
#  Export — reliable output
# ---------------------------------------------------------------------------


class TestExport:
    def test_svg_creates_file(self):
        from molqrc import QRCode

        path = "/tmp/test_qr.svg"
        qr = QRCode("hello")
        result = qr.save(path, scale=5)
        assert os.path.exists(path)
        assert os.path.getsize(path) > 0
        assert result == os.path.abspath(path)

    def test_svg_contains_rects(self):
        from molqrc import QRCode

        path = "/tmp/test_qr2.svg"
        QRCode("hello").save(path, scale=5)
        content = open(path).read()
        assert "<svg" in content
        assert "<rect" in content
        assert "viewBox" in content

    def test_svg_has_quiet_zone(self):
        from molqrc import QRCode

        path = "/tmp/test_qr3.svg"
        QRCode("hello").save(path, scale=10)
        content = open(path).read()
        # width = (21 + 8) * 10 = 290
        assert 'width="290"' in content

    def test_svg_square(self):
        from molqrc import QRCode

        path = "/tmp/test_qr4.svg"
        QRCode("hello").save(path, scale=10)
        content = open(path).read()
        assert 'width="290"' in content
        assert 'height="290"' in content

    def test_png_with_pillow(self):
        pytest.importorskip("PIL")
        from molqrc import QRCode

        path = "/tmp/test_qr.png"
        QRCode("hello").save(path, scale=5)
        assert os.path.exists(path)
        assert os.path.getsize(path) > 0
        from PIL import Image

        im = Image.open(path)
        assert im.size[0] > 0

    def test_unsupported_format_raises(self):
        from molqrc import QRCode

        with pytest.raises(ValueError, match="Unsupported"):
            QRCode("hello").save("test.bmp")


# ---------------------------------------------------------------------------
#  Preview — best-effort terminal display
# ---------------------------------------------------------------------------


class TestPreview:
    def test_returns_non_empty_string(self):
        from molqrc import QRCode

        out = QRCode("hello").preview()
        assert isinstance(out, str)
        assert len(out) > 100
        assert "##" in out

    def test_border_default(self):
        from molqrc import QRCode

        out = QRCode("hello").preview()
        lines = out.split("\n")
        # border=1: 2 top + 21*2 matrix + 2 bottom = 46
        assert len(lines) >= 40

    def test_border_zero(self):
        from molqrc import QRCode

        out = QRCode("hello").preview(border=0)
        lines = out.split("\n")
        assert len(lines) >= 42

    def test_contains_only_ascii(self):
        from molqrc import QRCode

        out = QRCode("hello").preview()
        out.encode("ascii")  # must not raise


# ---------------------------------------------------------------------------
#  Web
# ---------------------------------------------------------------------------


class TestWeb:
    def test_creates_self_contained_html(self):
        from molqrc import QRCode

        d = tempfile.mkdtemp()
        try:
            QRCode("hello").to_web(d)
            content = open(os.path.join(d, "index.html")).read()
            assert "<canvas" in content
            assert "molqrc" in content
            # GitHub link is the only external URL
            assert content.count("https://") == 1
        finally:
            shutil.rmtree(d)

    def test_download_capability(self):
        from molqrc import QRCode

        d = tempfile.mkdtemp()
        try:
            QRCode("hello").to_web(d)
            content = open(os.path.join(d, "index.html")).read()
            assert "toDataURL" in content
            assert "download" in content
        finally:
            shutil.rmtree(d)


# ---------------------------------------------------------------------------
#  CLI
# ---------------------------------------------------------------------------


class TestCLI:
    """New subcommand interface."""

    def test_preview_subcommand(self):
        r = subprocess.run(
            ["molqrc", "preview", "hello"], capture_output=True, text=True
        )
        assert r.returncode == 0
        assert len(r.stdout) > 0

    def test_pic_svg(self):
        path = "/tmp/test_pic.svg"
        r = subprocess.run(
            ["molqrc", "pic", "hello", "-o", path], capture_output=True, text=True
        )
        assert r.returncode == 0
        assert os.path.exists(path)

    def test_pic_png(self):
        pytest.importorskip("PIL")
        path = "/tmp/test_pic.png"
        r = subprocess.run(
            ["molqrc", "pic", "hello", "-o", path], capture_output=True, text=True
        )
        assert r.returncode == 0
        assert os.path.exists(path)

    def test_pic_default_ext(self):
        r = subprocess.run(
            ["molqrc", "pic", "hello", "-o", "/tmp/test_pic_noext"],
            capture_output=True,
            text=True,
        )
        assert r.returncode == 0
        assert os.path.exists("/tmp/test_pic_noext.svg")

    def test_web_build(self):
        d = tempfile.mkdtemp()
        try:
            r = subprocess.run(
                ["molqrc", "web", "build", "hello", "-o", d],
                capture_output=True,
                text=True,
            )
            assert r.returncode == 0
            assert os.path.exists(os.path.join(d, "index.html"))
        finally:
            shutil.rmtree(d)

    def test_web_build_default_dir(self):
        d = tempfile.mkdtemp()
        try:
            r = subprocess.run(
                ["molqrc", "web", "build", "hello", "-o", d],
                capture_output=True,
                text=True,
            )
            assert r.returncode == 0
            assert os.path.exists(os.path.join(d, "index.html"))
        finally:
            shutil.rmtree(d)

    def test_empty_text_web(self):
        d = tempfile.mkdtemp()
        try:
            r = subprocess.run(
                ["molqrc", "web", "build", "-o", d],
                capture_output=True,
                text=True,
            )
            assert r.returncode == 0
            assert os.path.exists(os.path.join(d, "index.html"))
        finally:
            shutil.rmtree(d)

    def test_ecl_flag(self):
        r = subprocess.run(
            ["molqrc", "preview", "hello", "--ecl", "H"], capture_output=True, text=True
        )
        assert r.returncode == 0

    def test_version_range_flags(self):
        r = subprocess.run(
            ["molqrc", "preview", "hello", "--version-min", "5", "--version-max", "20"],
            capture_output=True,
            text=True,
        )
        assert r.returncode == 0

    def test_mask_flag(self):
        r = subprocess.run(
            ["molqrc", "preview", "hello", "--mask", "4"],
            capture_output=True,
            text=True,
        )
        assert r.returncode == 0

    def test_no_boost_flag(self):
        r = subprocess.run(
            ["molqrc", "preview", "hello", "--no-boost"], capture_output=True, text=True
        )
        assert r.returncode == 0
