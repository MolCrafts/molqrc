#include "molqrc.h"
#include "internal.h"
#include <string.h>

static int g_initialized = 0;

static void ensure_init(void) {
  if (!g_initialized) {
    molqrc_gf_init();
    g_initialized = 1;
  }
}

/* ------------------------------------------------------------------ */
/*  Version selection helper                                           */
/* ------------------------------------------------------------------ */

static int find_version(const molqrc_segment_t *segs, int num_segs, int min_ver,
                        int max_ver, int *ecl_inout, int boost_ecl) {
  int version, ecl, total_bits, i;

  for (version = min_ver; version <= max_ver; version++) {
    total_bits = 0;
    for (i = 0; i < num_segs; i++) {
      const molqrc_segment_t *seg = &segs[i];
      int cci = molqrc_cci_bits(seg->mode, version);

      if (seg->mode != MOLQRC_MODE_ECI) {
        long max_count = (1L << cci) - 1;
        if (seg->num_chars > max_count)
          goto next_version;
      }
      total_bits += 4;
      if (seg->mode != MOLQRC_MODE_ECI)
        total_bits += cci;
      total_bits += seg->bit_length;
    }

    for (ecl = 3; ecl >= 0; ecl--) {
      int capacity = molqrc_capacity(version, ecl);
      if (total_bits <= capacity * 8) {
        if (boost_ecl) {
          /* Try to boost ECL within the same version */
          while (ecl < 3) {
            int next_cap = molqrc_capacity(version, ecl + 1);
            if (total_bits <= next_cap * 8)
              ecl++;
            else
              break;
          }
        }
        *ecl_inout = ecl;
        return version;
      }
    }
  next_version:;
  }
  return 0;
}

/* ------------------------------------------------------------------ */
/*  encode_segments                                                    */
/* ------------------------------------------------------------------ */

int molqrc_encode_segments(const molqrc_segment_t segs[], int num_segs,
                           unsigned char *out_matrix, int min_version,
                           int max_version, int ecl, int mask, int boost_ecl) {
  int version, capacity, total_cw, ec_per_block, total_blocks;
  int g1b, g1cw, g2b, g2cw, max_data_cw, side;
  int blk, i, src;
  unsigned char data_cw[4096];
  int ec_blocks_data[4096];
  unsigned char interleaved[4096];

  if (!segs || num_segs <= 0 || !out_matrix)
    return 0;
  if (min_version < 1 || max_version > 40 || min_version > max_version)
    return 0;
  if (ecl < 0 || ecl > 3)
    return 0;
  if (mask < -1 || mask > 7)
    return 0;

  ensure_init();

  version =
      find_version(segs, num_segs, min_version, max_version, &ecl, boost_ecl);
  if (version == 0)
    return 0;

  /* Encode segments → data codewords */
  capacity = molqrc_encode_segments_to_codewords(
      segs, num_segs, data_cw, (int)sizeof(data_cw), version, ecl);
  if (capacity == 0)
    return 0;
  total_cw = capacity;

  /* RS encoding + interleaving */
  molqrc_block_info(version, ecl, &g1b, &g1cw, &g2b, &g2cw);
  ec_per_block = molqrc_ec_codewords_per_block(version, ecl);
  total_blocks = g1b + g2b;
  max_data_cw = (g1cw > g2cw) ? g1cw : g2cw;

  src = 0;
  {
    unsigned char block_buf[4096];
    for (blk = 0; blk < total_blocks; blk++) {
      int dcw = (blk < g1b) ? g1cw : g2cw;
      for (i = 0; i < dcw; i++)
        block_buf[i] = data_cw[src++];
      molqrc_rs_encode(block_buf, dcw, ec_per_block);
      for (i = 0; i < dcw + ec_per_block; i++)
        ec_blocks_data[blk * (max_data_cw + ec_per_block) + i] = block_buf[i];
    }
  }

  /* Interleave */
  {
    int out_pos = 0;
    for (i = 0; i < max_data_cw; i++) {
      for (blk = 0; blk < total_blocks; blk++) {
        int dcw = (blk < g1b) ? g1cw : g2cw;
        if (i < dcw)
          interleaved[out_pos++] = (unsigned char)
              ec_blocks_data[blk * (max_data_cw + ec_per_block) + i];
      }
    }
    for (i = 0; i < ec_per_block; i++) {
      for (blk = 0; blk < total_blocks; blk++) {
        interleaved[out_pos++] =
            (unsigned char)ec_blocks_data[blk * (max_data_cw + ec_per_block) +
                                          max_data_cw + i];
      }
    }
  }

  /* Build matrix */
  side = 21 + 4 * (version - 1);
  {
    int total_codewords = total_cw + total_blocks * ec_per_block;
    molqrc_build_matrix(out_matrix, side, interleaved, total_codewords, version,
                        ecl, mask);
  }

  return side;
}

/* ------------------------------------------------------------------ */
/*  encode_text (convenience)                                          */
/* ------------------------------------------------------------------ */

int molqrc_encode_text(const char *text, unsigned char *out_matrix,
                       int min_version, int max_version, int ecl, int mask,
                       int boost_ecl) {
  unsigned char buf[4096];
  molqrc_segment_t seg;

  if (!text || !*text || !out_matrix)
    return 0;

  if (molqrc_is_numeric(text)) {
    seg = molqrc_make_numeric(text, buf);
  } else if (molqrc_is_alphanumeric(text)) {
    seg = molqrc_make_alphanumeric(text, buf);
  } else {
    seg =
        molqrc_make_bytes((const unsigned char *)text, (int)strlen(text), buf);
  }

  return molqrc_encode_segments(&seg, 1, out_matrix, min_version, max_version,
                                ecl, mask, boost_ecl);
}

/* ------------------------------------------------------------------ */
/*  draw_text                                                          */
/* ------------------------------------------------------------------ */

int molqrc_draw_text(const char *text, int left, int top, int width, int height,
                     molqrc_draw_rect_fn draw_rect, void *user) {
  unsigned char matrix[MOLQRC_MAX_SIZE * MOLQRC_MAX_SIZE];
  int side;

  if (!text || !*text || !draw_rect)
    return 0;
  if (width <= 0 || height <= 0)
    return 0;

  side = molqrc_encode_text(text, matrix, 1, 40, MOLQRC_ECL_M, MOLQRC_MASK_AUTO,
                            1);
  if (side == 0)
    return 0;

  return molqrc_draw_matrix(matrix, side, left, top, width, height, draw_rect,
                            user);
}

/* ------------------------------------------------------------------ */
/*  draw_matrix                                                        */
/* ------------------------------------------------------------------ */

int molqrc_draw_matrix(const unsigned char *matrix, int side, int left, int top,
                       int width, int height, molqrc_draw_rect_fn draw_rect,
                       void *user) {
  int module_size, total_size, offset_x, offset_y, r, c;

  if (!matrix || !draw_rect)
    return 0;
  if (width <= 0 || height <= 0)
    return 0;

  {
    int dim = (width < height) ? width : height;
    module_size = dim / (side + 8);
    if (module_size < 1)
      module_size = 1;
    total_size = module_size * (side + 8);
    offset_x = left + (width - total_size) / 2;
    offset_y = top + (height - total_size) / 2;
  }

  for (r = 0; r < side; r++) {
    for (c = 0; c < side; c++) {
      if (matrix[r * side + c]) {
        int x = offset_x + (c + 4) * module_size;
        int y = offset_y + (r + 4) * module_size;
        draw_rect(user, x, y, module_size, module_size);
      }
    }
  }

  return module_size;
}
