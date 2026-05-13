"""CLI entry point for molqrc — QR Code generator.

Usage:
    molqrc preview "hello"                 Terminal preview
    molqrc pic "hello" -o qr.svg           Save as SVG (zero-dependency)
    molqrc pic "hello" -o qr.png           Save as PNG (needs Pillow)
    molqrc web build "hello" -o dir/       Static web page
    molqrc web serve "hello"               Build + serve locally
"""

import argparse
import os
import sys
import tempfile
import webbrowser


def main():
    parser = argparse.ArgumentParser(
        prog="molqrc", description="Generate QR Codes from text"
    )
    sub = parser.add_subparsers(dest="command", required=True)

    # Shared options for encode control
    def add_encode_args(p):
        p.add_argument("--version-min", type=int, default=1, help="Min version (1-40)")
        p.add_argument("--version-max", type=int, default=40, help="Max version (1-40)")
        p.add_argument(
            "--ecl",
            choices=["L", "M", "Q", "H"],
            default="M",
            help="Error correction level",
        )
        p.add_argument(
            "--mask", type=int, default=-1, help="Mask pattern 0-7, -1 for auto"
        )
        p.add_argument("--no-boost", action="store_true", help="Disable ECL boost")

    # -- preview ---------------------------------------------------------
    pv = sub.add_parser("preview", help="Terminal preview")
    pv.add_argument("text", help="Text to encode")
    pv.add_argument("--border", type=int, default=1, help="White-module border")
    add_encode_args(pv)

    # -- pic -------------------------------------------------------------
    pic = sub.add_parser("pic", help="Save QR as SVG or PNG image")
    pic.add_argument("text", help="Text to encode")
    pic.add_argument("-o", "--output", required=True, help="Output path (.svg or .png)")
    pic.add_argument("--scale", type=int, default=10, help="Module scale in pixels")
    pic.add_argument("--fg", default="#000", help="Foreground colour")
    pic.add_argument("--bg", default="#fff", help="Background colour")
    add_encode_args(pic)

    # -- web -------------------------------------------------------------
    web = sub.add_parser("web", help="Web page operations")
    web_sub = web.add_subparsers(dest="web_command", required=True)

    wb = web_sub.add_parser("build", help="Build static web page")
    wb.add_argument("text", help="Text to encode")
    wb.add_argument("-o", "--output", default="molqrc_web", help="Output directory")
    wb.add_argument("--title", default="QR Code", help="Page title")
    add_encode_args(wb)

    ws = web_sub.add_parser("serve", help="Build + serve locally")
    ws.add_argument("text", help="Text to encode")
    ws.add_argument("-o", "--output", help="Output directory (temp dir if omitted)")
    ws.add_argument("--port", type=int, default=8080, help="Port")
    ws.add_argument(
        "--open", action="store_true", dest="open_browser", help="Auto-open browser"
    )
    ws.add_argument("--title", default="QR Code", help="Page title")
    add_encode_args(ws)

    args = parser.parse_args()

    ecl_map = {"L": 0, "M": 1, "Q": 2, "H": 3}

    kw = dict(
        ecl=ecl_map.get(args.ecl, 1),
        mask=args.mask,
        boost_ecl=not args.no_boost,
    )
    if hasattr(args, "version_min"):
        kw["min_version"] = args.version_min
        kw["max_version"] = args.version_max

    if args.command == "preview":
        _preview(args.text, border=args.border, **kw)
    elif args.command == "pic":
        _pic(args.text, args.output, scale=args.scale, fg=args.fg, bg=args.bg, **kw)
    elif args.command == "web":
        if args.web_command == "build":
            _web_build(args.text, args.output, title=args.title, **kw)
        elif args.web_command == "serve":
            _web_serve(
                args.text,
                args.output,
                port=args.port,
                open_browser=args.open_browser,
                title=args.title,
                **kw,
            )


# -------------------------------------------------------------------
#  Commands
# -------------------------------------------------------------------


def _preview(text, *, border=1, **kw):
    qr = _make_qr(text, **kw)
    print(qr.preview(border=border))
    print(f"\n  {qr.text}")


def _pic(text, output, *, scale=10, fg="#000", bg="#fff", **kw):
    qr = _make_qr(text, **kw)
    _save(qr, output, scale=scale, fg=fg, bg=bg)


def _web_build(text, output, *, title="QR Code", **kw):
    qr = _make_qr(text, **kw)
    qr.to_web(output, title=title)
    print(f"Web page saved to {os.path.abspath(output)}/index.html")


def _web_serve(text, output, *, port=8080, open_browser=False, title="QR Code", **kw):
    import http.server
    import socketserver
    import time

    qr = _make_qr(text, **kw)
    dir_path = output or tempfile.mkdtemp(prefix="molqrc_web_")
    qr.to_web(dir_path, title=title)

    class Handler(http.server.SimpleHTTPRequestHandler):
        def __init__(self, *a, **kw2):
            super().__init__(*a, directory=dir_path, **kw2)

        def do_GET(self):
            path = self.translate_path(self.path)
            if not os.path.exists(path):
                self.path = "/index.html"
            super().do_GET()

    max_tries = 10
    for offset in range(max_tries):
        try_port = port + offset
        try:
            httpd = socketserver.TCPServer(("", try_port), Handler)
            break
        except OSError:
            if offset == max_tries - 1:
                print(
                    f"molqrc: cannot serve on port {port}–{port + max_tries - 1} — all occupied",
                    file=sys.stderr,
                )
                sys.exit(1)
            print(f"Port {try_port} in use, trying {try_port + 1}…", file=sys.stderr)
            time.sleep(0.1)

    url = f"http://localhost:{try_port}"
    print(f"Serving at {url}")
    print(f"Directory : {os.path.abspath(dir_path)}")
    print("Press Ctrl+C to stop.")

    if open_browser:
        webbrowser.open(url)

    try:
        with httpd:
            httpd.serve_forever()
    except KeyboardInterrupt:
        print("\nStopped.")


# -------------------------------------------------------------------
#  Helpers
# -------------------------------------------------------------------


def _make_qr(text, **kw):
    try:
        from molqrc import QRCode
    except ImportError as e:
        print(f"molqrc: failed to import — {e}", file=sys.stderr)
        print(
            "Build the C library first: cmake -B build && cmake --build build",
            file=sys.stderr,
        )
        sys.exit(1)

    try:
        return QRCode(text, **kw)
    except ValueError as e:
        print(f"molqrc: {e}", file=sys.stderr)
        sys.exit(1)


def _save(qr, path, *, scale=10, fg="#000", bg="#fff"):
    ext = os.path.splitext(path)[1].lower()
    if not ext:
        path += ".svg"
        ext = ".svg"

    if ext == ".svg":
        from . import export

        export.to_svg(qr.matrix, qr.side, path, scale=scale, dark=fg, light=bg)
        print(f"QR Code saved to {path}")
    elif ext == ".png":
        try:
            from . import _png

            _png.write(qr.matrix, qr.side, path, scale=scale)
            print(f"QR Code saved to {path}")
        except ImportError as e:
            print(f"molqrc: {e}", file=sys.stderr)
            sys.exit(1)
    else:
        print(f"molqrc: unsupported format '{ext}'. Use .svg or .png.", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
