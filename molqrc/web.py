"""Interactive web page generator for QR Codes.

Generates a single self-contained index.html with inline CSS and
a pure-JS QR encoder — fully interactive text input, style
customisation, and download.  No external dependencies.
"""

import json
import os


def generate(matrix, side, dir_path, *, title="QR Code", text=""):
    """Generate a self-contained interactive web page.

    Args:
        matrix: bytearray of side*side bytes (0=white, 1=black).
        side: QR Code side length in modules.
        dir_path: Directory to write index.html into (created if needed).
        title: HTML page title.
        text: The original encoded text (pre-fills the input).
    """
    os.makedirs(dir_path, exist_ok=True)

    matrix_json = json.dumps(list(matrix))
    text_json = json.dumps(text)

    html = f"""<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>{_escape(title)} | molqrc</title>
<link rel="icon" href="data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 36 36'><rect x='2' y='2' width='8' height='8' rx='1.5' fill='white'/><rect x='12' y='2' width='6' height='8' rx='1.5' fill='white'/><rect x='20' y='2' width='8' height='8' rx='1.5' fill='white'/><rect x='2' y='12' width='8' height='6' rx='1.5' fill='white'/><rect x='12' y='12' width='8' height='8' rx='1.5' fill='%2303a3d7'/><rect x='22' y='12' width='6' height='6' rx='1.5' fill='white'/><rect x='2' y='20' width='8' height='8' rx='1.5' fill='white'/><rect x='12' y='22' width='8' height='6' rx='1.5' fill='white'/><rect x='22' y='20' width='6' height='6' rx='1.5' fill='white'/></svg>">
<style>
/* ===================================================================
   Reset & Base
   =================================================================== */
*, *::before, *::after {{ box-sizing: border-box; margin: 0; padding: 0; }}

:root {{
  --bg: #0a0f14;
  --surface: rgba(15, 25, 35, 0.85);
  --surface-border: rgba(255, 255, 255, 0.08);
  --text: #e8e8f0;
  --text-muted: #8899aa;
  --accent: #10b981;
  --accent2: #03a3d7;
  --glow: rgba(16, 185, 129, 0.3);
  --radius: 16px;
  --transition: 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}}

body {{
  font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  background: var(--bg);
  color: var(--text);
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  align-items: center;
  overflow-x: hidden;
}}

/* ===================================================================
   Animated Background
   =================================================================== */
.bg-mesh {{
  position: fixed;
  inset: 0;
  z-index: 0;
  overflow: hidden;
  pointer-events: none;
}}

.bg-orb {{
  position: absolute;
  border-radius: 50%;
  filter: blur(120px);
  opacity: 0.4;
  animation: orbFloat 20s ease-in-out infinite;
}}

.bg-orb:nth-child(1) {{
  width: 600px; height: 600px;
  background: radial-gradient(circle, var(--accent), transparent);
  top: -200px; left: -150px;
  animation-delay: 0s;
}}

.bg-orb:nth-child(2) {{
  width: 500px; height: 500px;
  background: radial-gradient(circle, var(--accent2), transparent);
  bottom: -150px; right: -150px;
  animation-delay: -7s;
}}

.bg-orb:nth-child(3) {{
  width: 400px; height: 400px;
  background: radial-gradient(circle, #fd79a8, transparent);
  top: 50%; left: 50%;
  transform: translate(-50%, -50%);
  animation-delay: -13s;
}}

@keyframes orbFloat {{
  0%, 100% {{ transform: translate(0, 0) scale(1); }}
  25% {{ transform: translate(60px, -40px) scale(1.1); }}
  50% {{ transform: translate(-30px, 50px) scale(0.9); }}
  75% {{ transform: translate(-50px, -30px) scale(1.05); }}
}}

/* ===================================================================
   Layout
   =================================================================== */
.app {{
  position: relative;
  z-index: 1;
  width: 100%;
  max-width: 1100px;
  padding: 24px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 24px;
}}

/* ===================================================================
   Header
   =================================================================== */
.header {{
  text-align: center;
  padding-top: 12px;
}}

.header-logo {{
  display: inline-flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 6px;
}}

.header-logo svg {{
  width: 36px;
  height: 36px;
}}

.header h1 {{
  font-size: 2.2rem;
  font-weight: 700;
  background: linear-gradient(135deg, var(--accent2), var(--accent));
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  letter-spacing: -0.5px;
}}

.header .tagline {{
  font-size: 0.9rem;
  color: var(--text-muted);
  margin-top: 2px;
}}

/* ===================================================================
   Main Layout
   =================================================================== */
.main {{
  display: flex;
  gap: 24px;
  width: 100%;
  flex-wrap: wrap;
  justify-content: center;
  align-items: flex-start;
}}

/* ===================================================================
   Glass Card
   =================================================================== */
.glass {{
  background: var(--surface);
  border: 1px solid var(--surface-border);
  border-radius: var(--radius);
  backdrop-filter: blur(24px);
  -webkit-backdrop-filter: blur(24px);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  padding: 24px;
}}

/* ===================================================================
   Controls Panel
   =================================================================== */
.controls {{
  flex: 1 1 320px;
  max-width: 420px;
  display: flex;
  flex-direction: column;
  gap: 18px;
}}

.control-group {{ display: flex; flex-direction: column; gap: 6px; }}

.control-group label {{
  font-size: 0.8rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--text-muted);
}}

.control-group input[type="text"] {{
  width: 100%;
  padding: 10px 14px;
  border: 1px solid rgba(255,255,255,0.12);
  border-radius: 10px;
  background: rgba(255,255,255,0.04);
  color: var(--text);
  font-size: 0.95rem;
  transition: var(--transition);
  outline: none;
  font-family: inherit;
}}

.control-group input[type="text"]:focus {{
  border-color: var(--accent);
  box-shadow: 0 0 0 3px var(--glow);
}}

/* ===================================================================
   Theme Presets
   =================================================================== */
.presets {{
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}}

.preset-dot {{
  width: 34px;
  height: 34px;
  border-radius: 50%;
  cursor: pointer;
  border: 2px solid transparent;
  transition: var(--transition);
  position: relative;
}}

.preset-dot:hover {{ transform: scale(1.12); }}

.preset-dot.active {{
  border-color: white;
  box-shadow: 0 0 12px currentColor;
}}

.preset-dot.classic {{
  background: #111;
  border: 2px solid #444;
}}

.preset-dot.neon {{
  background: linear-gradient(135deg, #00ff88, #00d2ff);
}}

.preset-dot.sunset {{
  background: linear-gradient(135deg, #ff6b6b, #feca57);
}}

.preset-dot.royal {{
  background: linear-gradient(135deg, #a855f7, #6366f1);
}}

.preset-dot.forest {{
  background: linear-gradient(135deg, #22c55e, #059669);
}}

.preset-dot.mono {{
  background: linear-gradient(135deg, #f8fafc, #94a3b8);
}}

/* ===================================================================
   Dot Style Toggles
   =================================================================== */
.toggles {{
  display: flex;
  gap: 8px;
}}

.toggle-btn {{
  flex: 1;
  padding: 8px 12px;
  border: 1px solid rgba(255,255,255,0.12);
  border-radius: 8px;
  background: rgba(255,255,255,0.03);
  color: var(--text-muted);
  font-size: 0.82rem;
  cursor: pointer;
  transition: var(--transition);
  text-align: center;
  font-family: inherit;
}}

.toggle-btn:hover {{ background: rgba(255,255,255,0.06); }}

.toggle-btn.active {{
  background: var(--accent);
  border-color: var(--accent);
  color: white;
}}

/* ===================================================================
   Color Pickers
   =================================================================== */
.color-row {{
  display: flex;
  gap: 12px;
  align-items: center;
}}

.color-row input[type="color"] {{
  width: 36px;
  height: 36px;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  background: transparent;
  padding: 0;
}}

.color-row input[type="color"]::-webkit-color-swatch-wrapper {{ padding: 0; }}

.color-row input[type="color"]::-webkit-color-swatch {{
  border-radius: 8px;
  border: 1px solid rgba(255,255,255,0.15);
}}

/* ===================================================================
   QR Panel
   =================================================================== */
.qr-panel {{
  flex: 0 0 auto;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  min-width: 340px;
}}

.qr-wrapper {{
  position: relative;
  border-radius: var(--radius);
  overflow: hidden;
}}

.qr-glow {{
  position: absolute;
  inset: -4px;
  border-radius: calc(var(--radius) + 4px);
  filter: blur(20px);
  opacity: 0.6;
  pointer-events: none;
  transition: background 0.5s;
}}

canvas {{
  display: block;
  border-radius: var(--radius);
  position: relative;
  z-index: 1;
  max-width: 340px;
  max-height: 340px;
}}

.qr-title-display {{
  font-size: 1.1rem;
  font-weight: 600;
  color: var(--text);
  text-align: center;
  min-height: 1.4em;
}}

.qr-subtitle-display {{
  font-size: 0.85rem;
  color: var(--text-muted);
  text-align: center;
  min-height: 1.2em;
}}

/* ===================================================================
   Download Button + Dropdown
   =================================================================== */
.btn-group {{
  position: relative;
  display: inline-flex;
}}

.btn {{
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 10px 22px;
  border: none;
  border-radius: 10px;
  font-size: 0.9rem;
  font-weight: 600;
  cursor: pointer;
  transition: var(--transition);
  font-family: inherit;
  white-space: nowrap;
}}

.btn-primary {{
  background: linear-gradient(135deg, var(--accent), #0284a7);
  color: white;
  border-radius: 10px 0 0 10px;
}}

.btn-primary:hover {{
  transform: translateY(-1px);
  box-shadow: 0 4px 16px var(--glow);
}}

.btn-dropdown {{
  background: linear-gradient(135deg, var(--accent), #0284a7);
  color: white;
  border-radius: 0 10px 10px 0;
  padding: 10px 12px;
  border-left: 1px solid rgba(255,255,255,0.25);
}}

.btn-dropdown:hover {{
  transform: translateY(-1px);
  box-shadow: 0 4px 16px var(--glow);
}}

.dropdown-menu {{
  position: absolute;
  top: 100%;
  right: 0;
  margin-top: 6px;
  background: rgba(30, 30, 55, 0.96);
  border: 1px solid rgba(255,255,255,0.1);
  border-radius: 10px;
  padding: 6px;
  min-width: 160px;
  display: none;
  z-index: 10;
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  box-shadow: 0 8px 24px rgba(0,0,0,0.5);
}}

.dropdown-menu.open {{ display: block; }}

.dropdown-item {{
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 10px 14px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--text);
  font-size: 0.85rem;
  cursor: pointer;
  transition: background 0.15s;
  font-family: inherit;
  text-align: left;
}}

.dropdown-item:hover {{ background: rgba(255,255,255,0.08); }}

.dropdown-item svg {{ flex-shrink: 0; }}

/* ===================================================================
   Footer / GitHub
   =================================================================== */
.footer {{
  position: relative;
  z-index: 1;
  padding: 20px;
  text-align: center;
}}

.github-link {{
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 10px 20px;
  border-radius: 12px;
  background: rgba(255,255,255,0.04);
  border: 1px solid rgba(255,255,255,0.08);
  color: var(--text-muted);
  text-decoration: none;
  font-size: 0.88rem;
  font-weight: 500;
  transition: var(--transition);
}}

.github-link:hover {{
  background: rgba(255,255,255,0.08);
  color: var(--text);
  border-color: rgba(255,255,255,0.15);
  transform: translateY(-1px);
}}

.github-link svg {{
  width: 20px;
  height: 20px;
  opacity: 0.8;
}}

.footer-tag {{
  margin-top: 10px;
  font-size: 0.72rem;
  color: rgba(255,255,255,0.2);
}}

/* ===================================================================
   Responsive
   =================================================================== */
@media (max-width: 800px) {{
  .main {{
    flex-direction: column;
    align-items: center;
  }}
  .controls {{
    max-width: 100%;
    flex: 1 1 auto;
  }}
  .qr-panel {{
    min-width: 280px;
  }}
  canvas {{
    max-width: 280px;
    max-height: 280px;
  }}
  .header h1 {{
    font-size: 1.6rem;
  }}
}}
</style>
</head>
<body>

<div class="bg-mesh">
  <div class="bg-orb"></div>
  <div class="bg-orb"></div>
  <div class="bg-orb"></div>
</div>

<div class="app">

  <header class="header">
    <div class="header-logo">
      <svg viewBox="0 0 36 36" fill="none">
        <rect x="2" y="2" width="8" height="8" rx="1.5" fill="white"/>
        <rect x="12" y="2" width="6" height="8" rx="1.5" fill="white"/>
        <rect x="20" y="2" width="8" height="8" rx="1.5" fill="white"/>
        <rect x="2" y="12" width="8" height="6" rx="1.5" fill="white"/>
        <rect x="12" y="12" width="8" height="8" rx="1.5" fill="#03a3d7"/>
        <rect x="22" y="12" width="6" height="6" rx="1.5" fill="white"/>
        <rect x="2" y="20" width="8" height="8" rx="1.5" fill="white"/>
        <rect x="12" y="22" width="8" height="6" rx="1.5" fill="white"/>
        <rect x="22" y="20" width="6" height="6" rx="1.5" fill="white"/>
      </svg>
      <h1>molqrc</h1>
    </div>
    <p class="tagline">Make your QR code stand out</p>
  </header>

  <div class="main">

    <!-- Controls -->
    <div class="controls glass">
      <div class="control-group">
        <label for="text-input">Text / URL</label>
        <input type="text" id="text-input" placeholder="Type something…" autofocus>
      </div>

      <div class="control-group">
        <label for="title-input">Title</label>
        <input type="text" id="title-input" placeholder="Optional title">
      </div>

      <div class="control-group">
        <label for="subtitle-input">Subtitle</label>
        <input type="text" id="subtitle-input" placeholder="Optional subtitle">
      </div>

      <div class="control-group">
        <label>Theme</label>
        <div class="presets" id="theme-presets">
          <button class="preset-dot classic active" data-theme="classic" title="Classic"></button>
          <button class="preset-dot neon" data-theme="neon" title="Neon"></button>
          <button class="preset-dot sunset" data-theme="sunset" title="Sunset"></button>
          <button class="preset-dot royal" data-theme="royal" title="Royal"></button>
          <button class="preset-dot forest" data-theme="forest" title="Forest"></button>
          <button class="preset-dot mono" data-theme="mono" title="Mono Light"></button>
        </div>
      </div>

      <div class="control-group">
        <label>Module Style</label>
        <div class="toggles" id="dot-toggles">
          <button class="toggle-btn active" data-dot="square">Squares</button>
          <button class="toggle-btn" data-dot="rounded">Rounded</button>
          <button class="toggle-btn" data-dot="circle">Circles</button>
        </div>
      </div>

      <div class="control-group">
        <label>Custom Colours</label>
        <div class="color-row">
          <span style="font-size:0.82rem;color:var(--text-muted)">FG</span>
          <input type="color" id="fg-color" value="#000000">
          <span style="font-size:0.82rem;color:var(--text-muted);margin-left:8px">BG</span>
          <input type="color" id="bg-color" value="#ffffff">
        </div>
      </div>
    </div>

    <!-- QR Display -->
    <div class="qr-panel glass">
      <div class="qr-title-display" id="qr-title"></div>
      <div class="qr-wrapper" id="qr-wrapper">
        <div class="qr-glow" id="qr-glow"></div>
        <canvas id="qr-canvas"></canvas>
      </div>
      <div class="qr-subtitle-display" id="qr-subtitle"></div>
      <div class="btn-group">
        <button class="btn btn-primary" id="btn-download">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
          Download
        </button>
        <button class="btn btn-dropdown" id="btn-dropdown-toggle" aria-label="Format selector">
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round"><polyline points="6 9 12 15 18 9"/></svg>
        </button>
        <div class="dropdown-menu" id="dropdown-menu">
          <button class="dropdown-item" data-fmt="png">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><rect x="3" y="3" width="18" height="18" rx="2"/><circle cx="8.5" cy="8.5" r="1.5"/><polyline points="21 15 16 10 5 21"/></svg>
            PNG Image
          </button>
          <button class="dropdown-item" data-fmt="svg">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M3 9h18"/><path d="M9 21V9"/></svg>
            SVG Vector
          </button>
        </div>
      </div>
    </div>

  </div>

  <footer class="footer">
    <a class="github-link" href="https://github.com/molcrafts/molqrc" target="_blank" rel="noopener">
      <svg viewBox="0 0 24 24" fill="currentColor">
        <path d="M12 0C5.37 0 0 5.37 0 12c0 5.3 3.438 9.8 8.205 11.387.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61-.546-1.387-1.333-1.756-1.333-1.756-1.09-.745.083-.73.083-.73 1.205.085 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.418-1.305.762-1.604-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.605-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 21.795 24 17.295 24 12c0-6.63-5.37-12-12-12z"/>
      </svg>
      molcrafts/molqrc
    </a>
    <div class="footer-tag">molqrc — QR codes, simply</div>
  </footer>

</div>

<script>
// =====================================================================
//  QR Code Encoder — pure JS
// =====================================================================

const QR = (() => {{

  // -- GF(256) --------------------------------------------------------
  const EXP = new Int32Array(512);
  const LOG = new Int32Array(256);
  (() => {{
    let x = 1;
    for (let i = 0; i < 255; i++) {{
      EXP[i] = x; EXP[i + 255] = x;
      LOG[x] = i;
      x <<= 1; if (x & 256) x ^= 0x11D;
    }}
    LOG[1] = 0;
  }})();

  const gfMul = (a, b) => (a && b) ? EXP[LOG[a] + LOG[b]] : 0;
  const gfPow = (a, n) => n === 0 ? 1 : a === 0 ? 0 : EXP[(LOG[a] * n) % 255];

  const polyMul = (p1, p2) => {{
    const out = new Int32Array(p1.length + p2.length - 1);
    for (let i = 0; i < p1.length; i++)
      for (let j = 0; j < p2.length; j++)
        out[i + j] ^= gfMul(p1[i], p2[j]);
    return out;
  }};

  const RS_GEN = [];
  const getRSGen = (nSym) => {{
    if (!RS_GEN[nSym]) {{
      let g = new Int32Array([1]);
      for (let i = 0; i < nSym; i++)
        g = polyMul(g, new Int32Array([1, gfPow(2, i)]));
      RS_GEN[nSym] = g;
    }}
    return RS_GEN[nSym];
  }};

  const rsEncode = (data, nSym) => {{
    const gen = getRSGen(nSym);
    const res = new Int32Array(data.length + nSym);
    res.set(data);
    for (let i = 0; i < data.length; i++) {{
      const coef = res[i];
      if (coef) for (let j = 0; j < gen.length; j++)
        res[i + j] ^= gfMul(gen[j], coef);
    }}
    return res.slice(data.length);
  }};

  // -- Capacity tables (EC level M, 15%) — versions 1–40 -------------
  const CAP = [
    null,
    [26, 10, 1, 16, 0, 0],
    [44, 16, 1, 28, 0, 0],
    [70, 26, 1, 44, 0, 0],
    [100, 18, 2, 32, 0, 0],
    [134, 24, 2, 43, 0, 0],
    [172, 16, 4, 27, 0, 0],
    [196, 18, 4, 31, 0, 0],
    [242, 22, 2, 36, 2, 37],
    [292, 22, 3, 40, 2, 41],
    [346, 26, 4, 42, 1, 43],
    [404, 30, 1, 50, 4, 51],
    [466, 32, 6, 52, 2, 53],
    [532, 34, 8, 55, 1, 56],
    [581, 26, 4, 37, 5, 38],
    [655, 30, 5, 42, 6, 43],
    [733, 34, 7, 47, 7, 48],
    [815, 38, 10, 52, 6, 53],
    [901, 42, 13, 58, 6, 59],
    [991, 36, 17, 42, 0, 0],
    [1085, 40, 16, 48, 3, 49],
    [1156, 44, 17, 53, 1, 54],
    [1258, 34, 7, 40, 16, 41],
    [1364, 38, 4, 47, 18, 48],
    [1474, 42, 6, 54, 16, 55],
    [1588, 46, 8, 61, 20, 62],
    [1706, 38, 19, 44, 4, 45],
    [1828, 42, 22, 49, 3, 50],
    [1921, 46, 3, 55, 23, 56],
    [2051, 38, 21, 38, 7, 39],
    [2185, 42, 19, 43, 10, 44],
    [2323, 46, 2, 49, 27, 50],
    [2465, 50, 10, 56, 23, 57],
    [2611, 54, 14, 63, 21, 64],
    [2761, 48, 14, 47, 23, 48],
    [2876, 50, 12, 49, 26, 50],
    [3034, 54, 6, 58, 32, 59],
    [3196, 58, 29, 67, 14, 68],
    [3362, 52, 13, 54, 33, 55],
    [3532, 56, 40, 68, 7, 69],
    [3706, 60, 18, 73, 29, 74],
  ];

  const ALIGN_POS = [
    null, null,
    [6, 18], [6, 22], [6, 26], [6, 30], [6, 34],
    [6, 22, 38], [6, 24, 42], [6, 26, 46], [6, 28, 50],
    [6, 30, 54], [6, 32, 58], [6, 34, 62], [6, 26, 46, 66],
    [6, 26, 48, 70], [6, 26, 50, 74], [6, 30, 54, 78],
    [6, 30, 56, 82], [6, 30, 58, 86], [6, 34, 62, 90],
    [6, 28, 50, 72, 94], [6, 26, 50, 74, 98],
    [6, 30, 54, 78, 102], [6, 28, 54, 80, 106],
    [6, 32, 58, 84, 110], [6, 30, 58, 86, 114],
    [6, 34, 62, 90, 118], [6, 26, 50, 74, 98, 122],
    [6, 30, 54, 78, 102, 126], [6, 26, 52, 78, 104, 130],
    [6, 30, 56, 82, 108, 134], [6, 34, 60, 86, 112, 138],
    [6, 30, 58, 86, 114, 142], [6, 34, 62, 90, 118, 146],
    [6, 30, 54, 78, 102, 126, 150], [6, 24, 50, 76, 102, 128, 154],
    [6, 28, 54, 80, 106, 132, 158], [6, 32, 58, 84, 110, 136, 162],
    [6, 26, 54, 82, 110, 138, 166], [6, 30, 58, 86, 114, 142, 170],
  ];

  const REM = [null,0,7,7,7,7,7,0,0,0,0,0,0,0,3,3,3,3,3,3,3,4,4,4,4,4,4,4,3,3,3,3,3,3,3,0,0,0,0,0,0];

  // -- Bit Buffer ----------------------------------------------------
  class BitBuf {{
    constructor() {{
      this.buf = [];
      this.len = 0;
    }}
    put(n, bits) {{
      for (let i = bits - 1; i >= 0; i--) {{
        const bi = this.len >> 3;
        while (this.buf.length <= bi) this.buf.push(0);
        if ((n >>> i) & 1) this.buf[bi] |= 0x80 >>> (this.len & 7);
        this.len++;
      }}
    }}
    padToByte() {{
      const rem = this.len & 7;
      if (rem) this.put(0, 8 - rem);
    }}
    byteLen() {{ return this.buf.length; }}
  }}

  // -- Data encoding (byte mode) -------------------------------------
  const encodeData = (text, version) => {{
    const cap = CAP[version];
    if (!cap) return null;
    const totalData = cap[1];
    const ecPerBlock = cap[2];
    const b1 = cap[3], c1 = cap[4], b2 = cap[5], c2 = cap[6];
    const dataCap = totalData - ecPerBlock * (b1 + b2);

    const bytes = new TextEncoder().encode(text);
    if (bytes.length > dataCap) return null;

    const bb = new BitBuf();
    bb.put(0x4, 4);  // byte mode
    bb.put(bytes.length, version <= 9 ? 8 : 16);
    for (const b of bytes) bb.put(b, 8);

    // Terminator
    const termLen = Math.min(4, dataCap * 8 - bb.len);
    if (termLen > 0) bb.put(0, termLen);
    bb.padToByte();

    // Pad bytes 0xEC, 0x11
    while (bb.byteLen() < dataCap) {{
      bb.put(0xEC, 8);
      if (bb.byteLen() < dataCap) bb.put(0x11, 8);
    }}

    // Split into blocks and compute EC
    const blocks = [];
    let offset = 0;
    for (let i = 0; i < b1; i++) {{ blocks.push(bb.buf.slice(offset, offset + c1)); offset += c1; }}
    for (let i = 0; i < b2; i++) {{ blocks.push(bb.buf.slice(offset, offset + c2)); offset += c2; }}

    const ecBlocks = blocks.map(b => rsEncode(b, ecPerBlock));

    // Interleave data
    const interleaved = [];
    let maxLen = 0;
    for (const b of blocks) maxLen = Math.max(maxLen, b.length);
    for (let i = 0; i < maxLen; i++)
      for (const b of blocks) if (i < b.length) interleaved.push(b[i]);

    // Interleave EC
    for (let i = 0; i < ecPerBlock; i++)
      for (const ec of ecBlocks) interleaved.push(ec[i]);

    return {{ data: interleaved, remBits: REM[version] }};
  }};

  // -- Matrix construction -------------------------------------------
  const buildMatrix = (version) => {{
    const size = 17 + version * 4;
    const m = new Int32Array(size * size);
    m.fill(-1);

    const placeFinder = (r, c) => {{
      for (let dr = -1; dr <= 7; dr++) {{
        for (let dc = -1; dc <= 7; dc++) {{
          const rr = r + dr, cc = c + dc;
          if (rr < 0 || rr >= size || cc < 0 || cc >= size) continue;
          m[rr * size + cc] = (dr >= 0 && dr <= 6 && dc >= 0 && dc <= 6 &&
            (dr === 0 || dr === 6 || dc === 0 || dc === 6 ||
             (dr >= 2 && dr <= 4 && dc >= 2 && dc <= 4))) ? 1 : 0;
        }}
      }}
    }};
    placeFinder(0, 0);
    placeFinder(0, size - 7);
    placeFinder(size - 7, 0);

    // Timing
    for (let i = 8; i < size - 8; i++)
      m[6 * size + i] = m[i * size + 6] = (i & 1) ? 0 : 1;

    // Alignment patterns
    const ap = ALIGN_POS[version];
    if (ap) {{
      for (const ar of ap) {{
        for (const ac of ap) {{
          if ((ar === 6 && ac === 6) || (ar === 6 && ac === size - 7) || (ar === size - 7 && ac === 6)) continue;
          for (let dr = -2; dr <= 2; dr++) {{
            for (let dc = -2; dc <= 2; dc++) {{
              const rr = ar + dr, cc = ac + dc;
              m[rr * size + cc] = (dr === -2 || dr === 2 || dc === -2 || dc === 2 || (dr === 0 && dc === 0)) ? 1 : 0;
            }}
          }}
        }}
      }}
    }}

    // Dark module
    m[(size - 8) * size + 8] = 1;
    return m;
  }};

  // -- Place data bits ------------------------------------------------
  const placeData = (matrix, data) => {{
    const size = Math.floor(Math.sqrt(matrix.length));
    let c = size - 1, r = size - 1;
    let dir = -1;
    let bitIdx = 0;

    while (c > 0) {{
      if (c === 6) c = 5;
      for (let i = 0; i < size; i++) {{
        const rr = dir < 0 ? size - 1 - i : i;
        for (const cc of [c, c - 1]) {{
          if (bitIdx < data.length * 8) {{
            const byte = data[bitIdx >> 3];
            const bit = (byte >>> (7 - (bitIdx & 7))) & 1;
            if (rr >= 0 && rr < size && cc >= 0 && cc < size && matrix[rr * size + cc] === -1)
              matrix[rr * size + cc] = bit;
            bitIdx++;
          }}
        }}
      }}
      c -= 2;
      dir = -dir;
    }}

    for (let i = 0; i < matrix.length; i++)
      if (matrix[i] === -1) matrix[i] = 0;
  }};

  // -- Format info (EC level M) --------------------------------------
  const fmtInfo = (mask) => {{
    const bits = [
      [1,0,1,0,1,0,0,0,1,0,0,1,1,1,0],
      [1,0,1,1,1,1,0,1,1,1,1,0,0,0,1],
      [1,0,0,0,1,1,1,1,0,0,1,1,0,1,0],
      [1,0,1,0,0,1,1,0,1,1,1,1,1,0,0],
      [1,1,0,0,0,0,0,1,0,0,1,0,0,1,1],
      [1,1,0,1,1,0,1,0,0,1,0,1,1,0,0],
      [1,1,1,0,0,0,1,1,0,0,0,1,0,0,1],
      [1,1,1,1,0,1,0,0,1,0,0,0,0,1,0],
    ];
    return bits[mask];
  }};

  const isFunctional = (r, c, size) => {{
    if (r <= 8 && c <= 8) return true;
    if (r <= 8 && c >= size - 8) return true;
    if (r >= size - 8 && c <= 8) return true;
    if (r === 6 || c === 6) return true;
    if (r <= 8 && c === 8) return true;
    if (r === 8 && c <= 8) return true;
    if (r === 8 && c >= size - 8) return true;
    if (r >= size - 8 && c === 8) return true;
    return false;
  }};

  const applyFormat = (matrix, mask) => {{
    const size = Math.floor(Math.sqrt(matrix.length));
    const bits = fmtInfo(mask);
    for (let i = 0; i <= 5; i++) matrix[i * size + 8] = bits[i];
    matrix[7 * size + 8] = bits[6];
    matrix[8 * size + 8] = bits[7];
    matrix[8 * size + 7] = bits[8];
    for (let i = 0; i <= 5; i++) matrix[8 * size + (5 - i)] = bits[9 + i];
    for (let i = 0; i <= 7; i++) matrix[8 * size + (size - 1 - i)] = bits[i];
    for (let i = 0; i <= 7; i++) matrix[(size - 1 - i) * size + 8] = bits[14 - i];
    matrix[size * 8] = 1;
  }};

  // -- Version info (v7+) --------------------------------------------
  const applyVersion = (matrix, version) => {{
    if (version < 7) return;
    const size = Math.floor(Math.sqrt(matrix.length));
    const V = [
      null,null,null,null,null,null,null,
      0x07C94, 0x085BC, 0x09A99, 0x0A4D3, 0x0BBF6,
      0x0C762, 0x0D847, 0x0E60D, 0x0F928, 0x10B78,
      0x1145D, 0x12A17, 0x13532, 0x149A6, 0x15683,
      0x168C9, 0x177EC, 0x18EC4, 0x191E1, 0x1AFAB,
      0x1B08E, 0x1CC1A, 0x1D33F, 0x1ED75, 0x1F250,
      0x209D5, 0x216F0, 0x228BA, 0x2379F, 0x24B0B,
      0x2542E, 0x26A64, 0x27541, 0x28C69,
    ];
    const bits = V[version];
    for (let i = 0; i < 18; i++) {{
      const bit = (bits >>> i) & 1;
      const rr = Math.floor(i / 3);
      const cc = (size - 11) + (i % 3);
      matrix[rr * size + cc] = bit;
      matrix[cc * size + rr] = bit;
    }}
  }};

  // -- Mask patterns -------------------------------------------------
  const MASK_FNS = [
    (r, c) => ((r + c) & 1) === 0,
    (r, c) => (r & 1) === 0,
    (r, c) => (c % 3) === 0,
    (r, c) => ((r + c) % 3) === 0,
    (r, c) => ((Math.floor(r / 2) + Math.floor(c / 3)) & 1) === 0,
    (r, c) => (((r * c) & 1) + ((r * c) % 3)) === 0,
    (r, c) => ((((r * c) & 1) + ((r * c) % 3)) & 1) === 0,
    (r, c) => ((((r + c) & 1) + ((r * c) % 3)) & 1) === 0,
  ];

  // -- Penalty scoring -----------------------------------------------
  const scoreMatrix = (matrix) => {{
    const size = Math.floor(Math.sqrt(matrix.length));
    let score = 0;

    // Rule 1: consecutive runs
    for (let r = 0; r < size; r++) {{
      let run = 1;
      for (let c = 1; c < size; c++) {{
        if (matrix[r * size + c] === matrix[r * size + c - 1]) run++;
        else {{ if (run >= 5) score += run - 2; run = 1; }}
      }}
      if (run >= 5) score += run - 2;
    }}
    for (let c = 0; c < size; c++) {{
      let run = 1;
      for (let r = 1; r < size; r++) {{
        if (matrix[r * size + c] === matrix[(r - 1) * size + c]) run++;
        else {{ if (run >= 5) score += run - 2; run = 1; }}
      }}
      if (run >= 5) score += run - 2;
    }}

    // Rule 2: 2x2 blocks
    for (let r = 0; r < size - 1; r++)
      for (let c = 0; c < size - 1; c++)
        if (matrix[r * size + c] === matrix[(r+1) * size + c] &&
            matrix[r * size + c] === matrix[r * size + c + 1] &&
            matrix[r * size + c] === matrix[(r+1) * size + c + 1])
          score += 3;

    // Rule 3: finder-like patterns
    const p1 = [1,0,1,1,1,0,1,0,0,0,0];
    const p2 = [0,0,0,0,1,0,1,1,1,0,1];
    for (let r = 0; r < size; r++) {{
      for (let c = 0; c < size - 10; c++) {{
        let m1 = true, m2 = true;
        for (let k = 0; k < 11; k++) {{
          if (matrix[r * size + c + k] !== p1[k]) m1 = false;
          if (matrix[r * size + c + k] !== p2[k]) m2 = false;
        }}
        if (m1 || m2) score += 40;
      }}
    }}
    for (let c = 0; c < size; c++) {{
      for (let r = 0; r < size - 10; r++) {{
        let m1 = true, m2 = true;
        for (let k = 0; k < 11; k++) {{
          if (matrix[(r + k) * size + c] !== p1[k]) m1 = false;
          if (matrix[(r + k) * size + c] !== p2[k]) m2 = false;
        }}
        if (m1 || m2) score += 40;
      }}
    }}

    // Rule 4: dark/light ratio
    let dark = 0;
    for (const v of matrix) if (v) dark++;
    const pct = Math.floor((dark * 100) / matrix.length);
    const prev = pct - (pct % 5);
    score += Math.min(Math.abs(prev - 50) / 5, Math.abs(prev + 5 - 50) / 5) * 10;

    return score;
  }};

  // -- Choose + apply mask -------------------------------------------
  const chooseMask = (matrix, version) => {{
    const size = Math.floor(Math.sqrt(matrix.length));
    let bestMask = 0, bestScore = Infinity;

    for (let m = 0; m < 8; m++) {{
      const test = new Int32Array(matrix);
      const fn = MASK_FNS[m];
      for (let r = 0; r < size; r++)
        for (let c = 0; c < size; c++)
          if (!isFunctional(r, c, size) && test[r * size + c] >= 0)
            test[r * size + c] ^= fn(r, c) ? 1 : 0;
      const s = scoreMatrix(test);
      if (s < bestScore) {{ bestScore = s; bestMask = m; }}
    }}

    // Apply best mask
    const fn = MASK_FNS[bestMask];
    for (let r = 0; r < size; r++)
      for (let c = 0; c < size; c++)
        if (!isFunctional(r, c, size) && matrix[r * size + c] >= 0)
          matrix[r * size + c] ^= fn(r, c) ? 1 : 0;

    applyFormat(matrix, bestMask);
    applyVersion(matrix, version);
  }};

  // -- Version selection ----------------------------------------------
  const chooseVersion = (text) => {{
    const len = new TextEncoder().encode(text).length;
    for (let v = 1; v <= 40; v++) {{
      const cap = CAP[v];
      if (!cap) continue;
      const dataCap = cap[1] - cap[2] * (cap[3] + cap[5]);
      if (len <= dataCap) return v;
    }}
    return null;
  }};

  // -- Public API -----------------------------------------------------
  const encode = (text) => {{
    if (!text) return null;
    const version = chooseVersion(text);
    if (!version) return null;

    const size = 17 + version * 4;
    const matrix = buildMatrix(version);
    const encoded = encodeData(text, version);
    if (!encoded) return null;

    placeData(matrix, encoded.data);
    chooseMask(matrix, version);

    return {{ matrix: Array.from(matrix), side: size, version }};
  }};

  return {{ encode }};
}})();

// =====================================================================
//  State — plain object, no magic
// =====================================================================

const state = {{
  theme: 'classic',
  dotStyle: 'square',
  customFg: '#000000',
  customBg: '#ffffff',
  data: null,  // current QR data (or null)
}};

// Initial server-provided QR data
const INITIAL_TEXT = {text_json};
const INITIAL_DATA = {{ matrix: {matrix_json}, side: {side}, version: (({side} - 21) / 4 + 1) }};
state.data = INITIAL_DATA;

// =====================================================================
//  Theme helpers
// =====================================================================

const THEMES = {{
  classic:  {{ fg: null,   bg: null,    glow: 'transparent' }}        , // uses customFg/customBg
  neon:     {{ fg: '#00ff88', bg: '#0d1117', glow: 'rgba(0,255,136,0.4)' }},
  sunset:   {{ fg: '#ff6b6b', bg: '#1a1a2e', glow: 'rgba(255,107,107,0.35)' }},
  royal:    {{ fg: '#a855f7', bg: '#0f0a1a', glow: 'rgba(168,85,247,0.4)' }},
  forest:   {{ fg: '#22c55e', bg: '#0a1a0f', glow: 'rgba(34,197,94,0.35)' }},
  mono:     {{ fg: '#1e293b', bg: '#f8fafc', glow: 'rgba(148,163,184,0.3)' }},
}};

function getFg() {{
  const t = THEMES[state.theme];
  return t.fg !== null ? t.fg : state.customFg;
}}

function getBg() {{
  const t = THEMES[state.theme];
  return t.bg !== null ? t.bg : state.customBg;
}}

function getGlow() {{
  return THEMES[state.theme].glow;
}}

// =====================================================================
//  Render
// =====================================================================

const canvas = document.getElementById('qr-canvas');
const ctx = canvas.getContext('2d');
const glowEl = document.getElementById('qr-glow');

function draw() {{
  const data = state.data;
  const fg = getFg();
  const bg = getBg();

  glowEl.style.background = getGlow();

  if (!data) {{
    const s = 300;
    canvas.width = s; canvas.height = s;
    ctx.fillStyle = bg;
    ctx.fillRect(0, 0, s, s);
    ctx.fillStyle = 'rgba(255,255,255,0.05)';
    ctx.font = '14px Inter, sans-serif';
    ctx.textAlign = 'center';
    ctx.fillText('Type something to generate', s / 2, s / 2);
    return;
  }}

  const quiet = 4;
  const total = data.side + 2 * quiet;
  const scale = Math.min(12, Math.floor(320 / total));
  const size = total * scale;

  canvas.width = size; canvas.height = size;

  ctx.fillStyle = bg;
  ctx.fillRect(0, 0, size, size);

  ctx.fillStyle = fg;
  const ds = state.dotStyle;

  for (let r = 0; r < data.side; r++) {{
    for (let c = 0; c < data.side; c++) {{
      if (!data.matrix[r * data.side + c]) continue;
      const x = (quiet + c) * scale;
      const y = (quiet + r) * scale;

      if (ds === 'square') {{
        ctx.fillRect(x, y, scale, scale);
      }} else if (ds === 'circle') {{
        ctx.beginPath();
        ctx.arc(x + scale / 2, y + scale / 2, scale * 0.48, 0, Math.PI * 2);
        ctx.fill();
      }} else {{
        // rounded
        const rr = scale * 0.3;
        ctx.beginPath();
        ctx.moveTo(x + rr, y);
        ctx.lineTo(x + scale - rr, y);
        ctx.quadraticCurveTo(x + scale, y, x + scale, y + rr);
        ctx.lineTo(x + scale, y + scale - rr);
        ctx.quadraticCurveTo(x + scale, y + scale, x + scale - rr, y + scale);
        ctx.lineTo(x + rr, y + scale);
        ctx.quadraticCurveTo(x, y + scale, x, y + scale - rr);
        ctx.lineTo(x, y + rr);
        ctx.quadraticCurveTo(x, y, x + rr, y);
        ctx.closePath();
        ctx.fill();
      }}
    }}
  }}
}}

// =====================================================================
//  App logic
// =====================================================================

const textInput = document.getElementById('text-input');
const titleInput = document.getElementById('title-input');
const subtitleInput = document.getElementById('subtitle-input');
const qrTitleEl = document.getElementById('qr-title');
const qrSubtitleEl = document.getElementById('qr-subtitle');
const fgColor = document.getElementById('fg-color');
const bgColor = document.getElementById('bg-color');

textInput.value = INITIAL_TEXT;

function refresh() {{
  const text = textInput.value.trim();
  if (text) {{
    const result = QR.encode(text);
    if (result) {{
      state.data = result;
    }} else {{
      // Encoding failed (e.g. too long) — keep previous data
    }}
  }} else {{
    state.data = INITIAL_DATA;
  }}
  draw();

  qrTitleEl.textContent = titleInput.value.trim();
  qrSubtitleEl.textContent = subtitleInput.value.trim();
}}

// Debounced text input
let debounce;
textInput.addEventListener('input', () => {{
  clearTimeout(debounce);
  debounce = setTimeout(refresh, 120);
}});
titleInput.addEventListener('input', refresh);
subtitleInput.addEventListener('input', refresh);

// Theme presets
document.querySelectorAll('#theme-presets .preset-dot').forEach(btn => {{
  btn.addEventListener('click', () => {{
    document.querySelectorAll('#theme-presets .preset-dot').forEach(b => b.classList.remove('active'));
    btn.classList.add('active');
    state.theme = btn.dataset.theme;
    draw();
  }});
}});

// Dot style toggles
document.querySelectorAll('#dot-toggles .toggle-btn').forEach(btn => {{
  btn.addEventListener('click', () => {{
    document.querySelectorAll('#dot-toggles .toggle-btn').forEach(b => b.classList.remove('active'));
    btn.classList.add('active');
    state.dotStyle = btn.dataset.dot;
    draw();
  }});
}});

// Custom colours — switching a colour auto-selects classic theme
fgColor.addEventListener('input', () => {{
  state.customFg = fgColor.value;
  state.theme = 'classic';
  document.querySelectorAll('#theme-presets .preset-dot').forEach(b => b.classList.remove('active'));
  document.querySelector('#theme-presets .preset-dot.classic').classList.add('active');
  draw();
}});
bgColor.addEventListener('input', () => {{
  state.customBg = bgColor.value;
  state.theme = 'classic';
  document.querySelectorAll('#theme-presets .preset-dot').forEach(b => b.classList.remove('active'));
  document.querySelector('#theme-presets .preset-dot.classic').classList.add('active');
  draw();
}});

// =====================================================================
//  Download (single button + dropdown)
// =====================================================================

const btnDownload = document.getElementById('btn-download');
const btnDropdown = document.getElementById('btn-dropdown-toggle');
const dropdownMenu = document.getElementById('dropdown-menu');

btnDropdown.addEventListener('click', (e) => {{
  e.stopPropagation();
  dropdownMenu.classList.toggle('open');
}});

document.addEventListener('click', () => {{
  dropdownMenu.classList.remove('open');
}});

document.querySelectorAll('.dropdown-item').forEach(item => {{
  item.addEventListener('click', (e) => {{
    e.stopPropagation();
    dropdownMenu.classList.remove('open');
    const fmt = item.dataset.fmt;
    if (fmt === 'png') downloadPNG();
    else downloadSVG();
  }});
}});

// Main download button defaults to PNG
btnDownload.addEventListener('click', () => downloadPNG());

function downloadPNG() {{
  const link = document.createElement('a');
  link.download = 'molqrc.png';
  link.href = canvas.toDataURL('image/png');
  link.click();
}}

function downloadSVG() {{
  if (!state.data) return;
  const quiet = 4;
  const total = state.data.side + 2 * quiet;
  const scale = 10;
  const size = total * scale;
  const fg = getFg();
  const bg = getBg();
  const ds = state.dotStyle;
  const m = state.data.matrix;
  const side = state.data.side;

  let svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${{size}}" height="${{size}}" viewBox="0 0 ${{size}} ${{size}}">`;
  svg += `<rect width="${{size}}" height="${{size}}" fill="${{bg}}"/>`;

  for (let r = 0; r < side; r++) {{
    for (let c = 0; c < side; c++) {{
      if (!m[r * side + c]) continue;
      const x = (quiet + c) * scale;
      const y = (quiet + r) * scale;
      if (ds === 'circle') {{
        svg += `<circle cx="${{x + scale / 2}}" cy="${{y + scale / 2}}" r="${{scale * 0.48}}" fill="${{fg}}"/>`;
      }} else if (ds === 'rounded') {{
        svg += `<rect x="${{x}}" y="${{y}}" width="${{scale}}" height="${{scale}}" rx="${{scale * 0.3}}" fill="${{fg}}"/>`;
      }} else {{
        svg += `<rect x="${{x}}" y="${{y}}" width="${{scale}}" height="${{scale}}" fill="${{fg}}"/>`;
      }}
    }}
  }}
  svg += '</svg>';

  const blob = new Blob([svg], {{ type: 'image/svg+xml' }});
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.download = 'molqrc.svg';
  link.href = url;
  link.click();
  URL.revokeObjectURL(url);
}}

// Initial draw
draw();
</script>
</body>
</html>"""

    filepath = os.path.join(dir_path, "index.html")
    with open(filepath, "w", encoding="utf-8") as f:
        f.write(html)


def _escape(s):
    return (
        str(s)
        .replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace('"', "&quot;")
    )
