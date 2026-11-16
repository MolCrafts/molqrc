// Derived from Project Nayuki's qrcodegen (MIT License); see LICENSE-MIT.
//
// The penalty score that picks the automatic mask (ISO/IEC 18004 section 7.8.3),
// with Nayuki's exact finder-like pattern detection: the light border around
// the symbol counts toward the light runs on either side of a 1:1:3:1:1 run.

use crate::canvas::Canvas;

const PENALTY_N1: u32 = 3;
const PENALTY_N2: u32 = 3;
const PENALTY_N3: u32 = 40;
const PENALTY_N4: u32 = 10;

/// Penalty score of the canvas's current modules; lower is better.
pub(crate) fn penalty_score(canvas: &Canvas) -> u32 {
    let size = canvas.size;
    let rows = (0..size).map(|y| line_penalty((0..size).map(|x| canvas.get(x, y)), size));
    let cols = (0..size).map(|x| line_penalty((0..size).map(|y| canvas.get(x, y)), size));
    let mut result: u32 = rows.chain(cols).sum();

    // 2x2 blocks of one colour.
    for y in 0..size - 1 {
        for x in 0..size - 1 {
            let c = canvas.get(x, y);
            if c == canvas.get(x + 1, y)
                && c == canvas.get(x, y + 1)
                && c == canvas.get(x + 1, y + 1)
            {
                result += PENALTY_N2;
            }
        }
    }

    // Balance of dark and light: the smallest k >= 0 such that the dark
    // share lies in [45 - 5k, 55 + 5k] percent. size is odd, so never exactly half.
    let dark = canvas.modules.iter().filter(|&&m| m).count();
    let total = size * size;
    let k = (dark * 20).abs_diff(total * 10).div_ceil(total) - 1;
    result + k as u32 * PENALTY_N4
}

/// Runs of five or more same-coloured modules, plus finder-like patterns, in
/// one row or column.
fn line_penalty(line: impl Iterator<Item = bool>, size: usize) -> u32 {
    let mut result = 0;
    let mut history = RunHistory::new(size);
    let mut run_color = false;
    let mut run_len = 0;
    for color in line {
        if color == run_color {
            run_len += 1;
            if run_len == 5 {
                result += PENALTY_N1;
            } else if run_len > 5 {
                result += 1;
            }
        } else {
            history.push(run_len);
            if !run_color {
                result += history.count_patterns() * PENALTY_N3;
            }
            run_color = color;
            run_len = 1;
        }
    }
    result + history.terminate_and_count(run_color, run_len) * PENALTY_N3
}

/// The last seven run lengths of a line, newest first.
struct RunHistory {
    size: usize,
    runs: [usize; 7],
}

impl RunHistory {
    fn new(size: usize) -> Self {
        Self { size, runs: [0; 7] }
    }

    /// Pushes a run to the front, dropping the oldest. The first run of a line
    /// is light and gets the light border in front of the symbol added.
    fn push(&mut self, mut run_len: usize) {
        if self.runs[0] == 0 {
            run_len += self.size;
        }
        self.runs.rotate_right(1);
        self.runs[0] = run_len;
    }

    /// Finder-like patterns ending at the newest run, which must be light:
    /// 0, 1 or 2 (a light run of 4 or more on either side of a dark
    /// 1:1:3:1:1 core).
    fn count_patterns(&self) -> u32 {
        let r = &self.runs;
        let n = r[1];
        let core = n > 0 && r[2] == n && r[3] == n * 3 && r[4] == n && r[5] == n;
        u32::from(core && r[0] >= n * 4 && r[6] >= n)
            + u32::from(core && r[6] >= n * 4 && r[0] >= n)
    }

    /// Closes the line, adding the light border after the symbol.
    fn terminate_and_count(mut self, run_color: bool, mut run_len: usize) -> u32 {
        if run_color {
            self.push(run_len);
            run_len = 0;
        }
        self.push(run_len + self.size);
        self.count_patterns()
    }
}
