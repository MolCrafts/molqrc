#include "internal.h"
#include <string.h>

/* ------------------------------------------------------------------ */
/*  Format information tables                                          */
/* ------------------------------------------------------------------ */

/* Format info = 5 data bits (2 ECL + 3 mask) → BCH(15,5) → XOR 0x5412 */
static const int g_format_info[4 * 8] = {
    /* L */ 0x5412, 0x5125, 0x5E7C, 0x5B4B,
    0x45F9,         0x40CE, 0x4F97, 0x4AA0,
    /* M */ 0x77C4, 0x72F3, 0x7DAA, 0x789D,
    0x662F,         0x6318, 0x6C41, 0x6976,
    /* Q */ 0x1689, 0x13BE, 0x1CE7, 0x19D0,
    0x0762,         0x0255, 0x0D0C, 0x083B,
    /* H */ 0x355F, 0x3068, 0x3F31, 0x3A06,
    0x24B4,         0x2183, 0x2EDA, 0x2BED,
};

/* ------------------------------------------------------------------ */
/*  Version information (BCH(18,6)) for versions 7-40                  */
/* ------------------------------------------------------------------ */

static const int g_version_info[34] = {
    0x07C94, 0x085BC, 0x09A99, 0x0A4D3, 0x0BBF6, 0x0C762, 0x0D847,
    0x0E60D, 0x0F928, 0x10B78, 0x1145D, 0x12A17, 0x13532, 0x149A6,
    0x15683, 0x168C9, 0x177EC, 0x18EC4, 0x191E1, 0x1AFAB, 0x1B08E,
    0x1CC1A, 0x1D33F, 0x1ED75, 0x1F250, 0x209D5, 0x216F0, 0x228BA,
    0x2379F, 0x24B0B, 0x2542E, 0x26A64, 0x27541, 0x28C69,
};

/* ------------------------------------------------------------------ */
/*  Helper: set/clear a module                                         */
/* ------------------------------------------------------------------ */

static inline void set_module(unsigned char *m, int side, int r, int c,
                              int val) {
  if (side <= 0)
    return;
  if (r < 0 || c < 0 || r >= side || c >= side)
    return;

  size_t idx = (size_t)r * (size_t)side + (size_t)c;
  m[idx] = (unsigned char)(val ? 1 : 0);
}

/* ------------------------------------------------------------------ */
/*  Finder patterns (3 corners)                                        */
/* ------------------------------------------------------------------ */

static void place_finders(unsigned char *m, int side) {
  int r, c, dr, dc;
  int corners[3][2] = {{0, 0}, {0, side - 7}, {side - 7, 0}};
  int fi;

  for (fi = 0; fi < 3; fi++) {
    int sr = corners[fi][0];
    int sc = corners[fi][1];
    for (dr = 0; dr < 7; dr++) {
      for (dc = 0; dc < 7; dc++) {
        int val = (dr == 0 || dr == 6 || dc == 0 || dc == 6 ||
                   (dr >= 2 && dr <= 4 && dc >= 2 && dc <= 4));
        set_module(m, side, sr + dr, sc + dc, val);
      }
    }
  }

  /* Separators (white border around finders) */
  for (fi = 0; fi < 3; fi++) {
    int sr = corners[fi][0];
    int sc = corners[fi][1];
    /* horizontal separator below/above finder */
    if (sr == 0) {
      for (c = 0; c < 8; c++) {
        set_module(m, side, 7, sc + c, 0);
        if (sc + c < side)
          set_module(m, side, sr + 7, sc + c, 0);
      }
    } else {
      for (c = 0; c < 8; c++) {
        set_module(m, side, sr - 1, sc + c, 0);
      }
    }
    /* vertical separator to right/left of finder */
    if (sc == 0) {
      for (r = 0; r < 8; r++) {
        set_module(m, side, sr + r, 7, 0);
      }
    } else {
      for (r = 0; r < 8; r++) {
        set_module(m, side, sr + r, sc - 1, 0);
      }
    }
  }
}

/* ------------------------------------------------------------------ */
/*  Alignment patterns                                                 */
/* ------------------------------------------------------------------ */

static void place_alignments(unsigned char *m, int side, int version) {
  int pos[8], count, i, j, dr, dc;

  if (version < 2)
    return;

  count = molqrc_alignment_positions(version, pos);

  for (i = 0; i < count; i++) {
    for (j = 0; j < count; j++) {
      int cr = pos[i];
      int cc = pos[j];

      /* Skip if overlaps with finder patterns */
      if ((cr <= 8 && cc <= 8) || (cr <= 8 && cc >= side - 9) ||
          (cr >= side - 9 && cc <= 8))
        continue;

      /* Draw 5x5 alignment */
      for (dr = -2; dr <= 2; dr++) {
        for (dc = -2; dc <= 2; dc++) {
          int val = (dr == -2 || dr == 2 || dc == -2 || dc == 2 ||
                     (dr == 0 && dc == 0));
          set_module(m, side, cr + dr, cc + dc, val);
        }
      }
    }
  }
}

/* ------------------------------------------------------------------ */
/*  Timing patterns                                                    */
/* ------------------------------------------------------------------ */

static void place_timing(unsigned char *m, int side) {
  int i;
  for (i = 8; i < side - 8; i++) {
    /* Horizontal timing (row 6) */
    set_module(m, side, 6, i, (i % 2 == 0));
    /* Vertical timing (col 6) */
    set_module(m, side, i, 6, (i % 2 == 0));
  }
}

/* ------------------------------------------------------------------ */
/*  Dark module                                                        */
/* ------------------------------------------------------------------ */

static void place_dark_module(unsigned char *m, int side) {
  set_module(m, side, side - 8, 8, 1);
}

/* ------------------------------------------------------------------ */
/*  Reserve areas for format + version info                             */
/* ------------------------------------------------------------------ */

/* Mark areas that must be skipped during data placement.
   Returns number of reserved modules. */

static int is_function_area(int r, int c, int side) {
  /* Finder patterns + separators */
  if (r <= 8 && c <= 8)
    return 1; /* top-left finder */
  if (r <= 8 && c >= side - 8)
    return 1; /* top-right finder */
  if (r >= side - 8 && c <= 8)
    return 1; /* bottom-left finder */

  /* Timing patterns */
  if (r == 6 || c == 6)
    return 1;

  /* Dark module area (version info sits near it for v>=7) */
  /* These will be filled later */

  return 0;
}

/* Alignment area check (dynamic, based on version) */
static int is_alignment_area(int r, int c, int version) {
  int pos[8], count, i, j;
  int side = 21 + 4 * (version - 1);

  if (version < 2)
    return 0;

  count = molqrc_alignment_positions(version, pos);

  for (i = 0; i < count; i++) {
    for (j = 0; j < count; j++) {
      int ar = pos[i];
      int ac = pos[j];
      if ((ar <= 8 && ac <= 8) || (ar <= 8 && ac >= side - 9) ||
          (ar >= side - 9 && ac <= 8))
        continue;
      if (r >= ar - 2 && r <= ar + 2 && c >= ac - 2 && c <= ac + 2)
        return 1;
    }
  }
  return 0;
}

/* ------------------------------------------------------------------ */
/*  Data bit placement (ISO/IEC 18004 §7.7.3)                          */
/* ------------------------------------------------------------------ */

typedef struct {
  unsigned char *m;
  int side;
  int version;
  int total_cw;
} place_ctx_t;

static int is_reserved(const place_ctx_t *ctx, int r, int c) {
  if (is_function_area(r, c, ctx->side))
    return 1;
  if (is_alignment_area(r, c, ctx->version))
    return 1;
  return 0;
}

static void place_data_bits(place_ctx_t *ctx, const unsigned char *data,
                            int total_bits) {
  int side = ctx->side;
  int bit = 0;
  int c = side - 1;
  int dir = -1; /* upward */

  while (c > 0) {
    /* Skip vertical timing pattern column */
    if (c == 6) {
      c--;
      continue;
    }

    int r;
    if (dir == -1) {
      for (r = side - 1; r >= 0; r--) {
        int c0 = c, c1 = c - 1;
        /* Right module of pair */
        if (!is_reserved(ctx, r, c0) && bit < total_bits) {
          ctx->m[r * side + c0] =
              (unsigned char)((data[bit / 8] >> (7 - (bit % 8))) & 1);
          bit++;
        }
        /* Left module of pair */
        if (!is_reserved(ctx, r, c1) && bit < total_bits) {
          ctx->m[r * side + c1] =
              (unsigned char)((data[bit / 8] >> (7 - (bit % 8))) & 1);
          bit++;
        }
      }
    } else {
      for (r = 0; r < side; r++) {
        int c0 = c, c1 = c - 1;
        if (!is_reserved(ctx, r, c0) && bit < total_bits) {
          ctx->m[r * side + c0] =
              (unsigned char)((data[bit / 8] >> (7 - (bit % 8))) & 1);
          bit++;
        }
        if (!is_reserved(ctx, r, c1) && bit < total_bits) {
          ctx->m[r * side + c1] =
              (unsigned char)((data[bit / 8] >> (7 - (bit % 8))) & 1);
          bit++;
        }
      }
    }
    dir = -dir;
    c -= 2;
  }
}

/* ------------------------------------------------------------------ */
/*  Masks (0-7)                                                        */
/* ------------------------------------------------------------------ */

static int mask_condition(int mask, int r, int c) {
  switch (mask) {
  case 0:
    return (r + c) % 2 == 0;
  case 1:
    return (r) % 2 == 0;
  case 2:
    return (c) % 3 == 0;
  case 3:
    return (r + c) % 3 == 0;
  case 4:
    return ((r / 2) + (c / 3)) % 2 == 0;
  case 5:
    return ((r * c) % 2) + ((r * c) % 3) == 0;
  case 6:
    return (((r * c) % 2) + ((r * c) % 3)) % 2 == 0;
  case 7:
    return (((r + c) % 2) + ((r * c) % 3)) % 2 == 0;
  }
  return 0;
}

static void apply_mask(unsigned char *m, int side, int version, int mask) {
  int r, c;
  for (r = 0; r < side; r++) {
    for (c = 0; c < side; c++) {
      if (is_function_area(r, c, side) || is_alignment_area(r, c, version))
        continue;
      /* Skip timing pattern overlap */
      if (r == 6 || c == 6)
        continue;
      if (mask_condition(mask, r, c))
        m[r * side + c] ^= 1;
    }
  }
}

/* ------------------------------------------------------------------ */
/*  Penalty scoring (4 rules)                                          */
/* ------------------------------------------------------------------ */

static int penalty_rule1(const unsigned char *m, int side) {
  /* 5+ consecutive same-color modules in a row or column.
     Score = N1 + 3 for each run of 5, +1 per extra. */
  int score = 0;
  int r, c;

  /* Horizontal */
  for (r = 0; r < side; r++) {
    int run = 0;
    int color = -1;
    for (c = 0; c < side; c++) {
      int v = m[r * side + c];
      if (v == color) {
        run++;
      } else {
        if (run >= 5)
          score += run - 2;
        color = v;
        run = 1;
      }
    }
    if (run >= 5)
      score += run - 2;
  }

  /* Vertical */
  for (c = 0; c < side; c++) {
    int run = 0;
    int color = -1;
    for (r = 0; r < side; r++) {
      int v = m[r * side + c];
      if (v == color) {
        run++;
      } else {
        if (run >= 5)
          score += run - 2;
        color = v;
        run = 1;
      }
    }
    if (run >= 5)
      score += run - 2;
  }

  return score;
}

static int penalty_rule2(const unsigned char *m, int side) {
  /* 2x2 blocks of same color. Score = 3 per block. */
  int score = 0;
  int r, c;
  for (r = 0; r < side - 1; r++) {
    for (c = 0; c < side - 1; c++) {
      int v = m[r * side + c];
      if (m[r * side + c + 1] == v && m[(r + 1) * side + c] == v &&
          m[(r + 1) * side + c + 1] == v)
        score += 3;
    }
  }
  return score;
}

static int penalty_rule3(const unsigned char *m, int side) {
  /* 1:1:3:1:1 ratio pattern (dark-light-dark-dark-dark-light-dark)
     preceded by 4 white modules.
     Horizontal and vertical. Score = 40 per occurrence. */
  int score = 0;
  int r, c, i;
  int finder_like[11] = {0, 0, 0, 0, 1, 0, 1, 1, 1, 0, 1};

  /* Horizontal */
  for (r = 0; r < side; r++) {
    for (c = 0; c <= side - 11; c++) {
      int match = 1;
      for (i = 0; i < 11; i++) {
        if (m[r * side + c + i] != finder_like[i]) {
          match = 0;
          break;
        }
      }
      if (match)
        score += 40;
      /* Also check reverse */
      match = 1;
      for (i = 0; i < 11; i++) {
        if (m[r * side + c + i] != finder_like[10 - i]) {
          match = 0;
          break;
        }
      }
      if (match)
        score += 40;
    }
  }

  /* Vertical */
  for (c = 0; c < side; c++) {
    for (r = 0; r <= side - 11; r++) {
      int match = 1;
      for (i = 0; i < 11; i++) {
        if (m[(r + i) * side + c] != finder_like[i]) {
          match = 0;
          break;
        }
      }
      if (match)
        score += 40;
      match = 1;
      for (i = 0; i < 11; i++) {
        if (m[(r + i) * side + c] != finder_like[10 - i]) {
          match = 0;
          break;
        }
      }
      if (match)
        score += 40;
    }
  }

  return score;
}

static int penalty_rule4(const unsigned char *m, int side) {
  /* Dark/light ratio. Score = |dark% - 50| / 5 * 10 */
  int total = side * side;
  int dark = 0;
  int i;
  for (i = 0; i < total; i++)
    if (m[i])
      dark++;
  int pct = dark * 100 / total;
  int diff = (pct > 50) ? (pct - 50) : (50 - pct);
  return (diff / 5) * 10;
}

static int compute_penalty(const unsigned char *m, int side) {
  return penalty_rule1(m, side) + penalty_rule2(m, side) +
         penalty_rule3(m, side) + penalty_rule4(m, side);
}

/* ------------------------------------------------------------------ */
/*  Format information placement                                       */
/* ------------------------------------------------------------------ */

static void place_format_info(unsigned char *m, int side, int ecl, int mask) {
  int fi = g_format_info[ecl * 8 + mask];
  int i;

  /* Around top-left finder */
  for (i = 0; i <= 5; i++)
    set_module(m, side, i, 8, (fi >> i) & 1);
  set_module(m, side, 7, 8, (fi >> 6) & 1);
  set_module(m, side, 8, 8, (fi >> 7) & 1);
  set_module(m, side, 8, 7, (fi >> 8) & 1);
  for (i = 9; i <= 14; i++)
    set_module(m, side, 8, 14 - i, (fi >> i) & 1);

  /* Top-right finder */
  for (i = 0; i <= 7; i++)
    set_module(m, side, 8, side - 1 - i, (fi >> i) & 1);

  /* Bottom-left finder */
  for (i = 0; i <= 7; i++)
    set_module(m, side, side - 1 - i, 8, (fi >> i) & 1);

  /* Dark module (always black) */
  set_module(m, side, side - 8, 8, 1);
}

/* ------------------------------------------------------------------ */
/*  Version information placement (v >= 7)                             */
/* ------------------------------------------------------------------ */

static void place_version_info(unsigned char *m, int side, int version) {
  if (version < 7)
    return;

  int vi = g_version_info[version - 7];
  int i, j;

  /* Bottom-left: 3 columns x 6 rows */
  for (i = 0; i < 6; i++) {
    for (j = 0; j < 3; j++) {
      int bit = (vi >> (i * 3 + j)) & 1;
      set_module(m, side, side - 11 + i, j, bit);
    }
  }

  /* Top-right: 6 columns x 3 rows */
  for (i = 0; i < 6; i++) {
    for (j = 0; j < 3; j++) {
      int bit = (vi >> (i * 3 + j)) & 1;
      set_module(m, side, j, side - 11 + i, bit);
    }
  }
}

/* ------------------------------------------------------------------ */
/*  Main: build matrix                                                 */
/* ------------------------------------------------------------------ */

int molqrc_build_matrix(unsigned char *matrix, int side,
                        const unsigned char *interleaved, int total_cw,
                        int version, int ecl, int mask) {
  unsigned char work[MOLQRC_MAX_SIZE * MOLQRC_MAX_SIZE];
  int total_bits = total_cw * 8;
  int selected_mask;

  /* Build base matrix (unmasked) */
  memset(work, 0, sizeof(work));

  place_finders(work, side);
  place_alignments(work, side, version);
  place_timing(work, side);
  place_dark_module(work, side);

  {
    place_ctx_t ctx;
    ctx.m = work;
    ctx.side = side;
    ctx.version = version;
    ctx.total_cw = total_cw;
    place_data_bits(&ctx, interleaved, total_bits);
  }

  if (mask == MOLQRC_MASK_AUTO) {
    unsigned char best_matrix[MOLQRC_MAX_SIZE * MOLQRC_MAX_SIZE];
    int best_penalty = 0x7FFFFFFF;
    int m;

    selected_mask = 0;
    for (m = 0; m < 8; m++) {
      unsigned char masked[MOLQRC_MAX_SIZE * MOLQRC_MAX_SIZE];
      int penalty;

      memcpy(masked, work, side * side);
      apply_mask(masked, side, version, m);
      place_format_info(masked, side, ecl, m);
      penalty = compute_penalty(masked, side);

      if (penalty < best_penalty) {
        best_penalty = penalty;
        selected_mask = m;
        memcpy(best_matrix, masked, side * side);
      }
    }
    memcpy(matrix, best_matrix, side * side);
  } else {
    selected_mask = mask;
    memcpy(matrix, work, side * side);
    apply_mask(matrix, side, version, selected_mask);
  }

  place_format_info(matrix, side, ecl, selected_mask);
  place_version_info(matrix, side, version);

  return selected_mask;
}
