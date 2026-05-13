#include "internal.h"
#include <limits.h>
#include <string.h>

/* ==================================================================
 *  QR Code capacity tables (ISO/IEC 18004:2015)
 *  For each version (1-40) and ECL (L=0,M=1,Q=2,H=3):
 *    total_data_codewords
 *    ec_codewords_per_block
 *    block division: g1_blocks, g1_codewords, g2_blocks, g2_codewords
 * ================================================================== */

typedef struct {
  int total_cw;
  int ec_per_block;
  int g1_blocks;
  int g1_cw;
  int g2_blocks;
  int g2_cw;
} version_ecl_t;

static const version_ecl_t g_version_table[40 * 4] = {
    /* V 1 */ {19, 7, 1, 19, 0, 0},        {16, 10, 1, 16, 0, 0},
    /* V 1 */ {13, 13, 1, 13, 0, 0},       {9, 17, 1, 9, 0, 0},
    /* V 2 */ {34, 10, 1, 34, 0, 0},       {28, 16, 1, 28, 0, 0},
    /* V 2 */ {22, 22, 1, 22, 0, 0},       {16, 28, 1, 16, 0, 0},
    /* V 3 */ {55, 15, 1, 55, 0, 0},       {44, 26, 1, 44, 0, 0},
    /* V 3 */ {34, 18, 2, 17, 0, 0},       {26, 22, 2, 13, 0, 0},
    /* V 4 */ {80, 20, 1, 80, 0, 0},       {64, 18, 2, 32, 0, 0},
    /* V 4 */ {48, 26, 2, 24, 0, 0},       {36, 16, 4, 9, 0, 0},
    /* V 5 */ {108, 26, 1, 108, 0, 0},     {86, 24, 2, 43, 0, 0},
    /* V 5 */ {62, 18, 2, 15, 2, 16},      {46, 22, 2, 11, 2, 12},
    /* V 6 */ {136, 18, 2, 68, 0, 0},      {108, 16, 4, 27, 0, 0},
    /* V 6 */ {76, 24, 4, 19, 0, 0},       {60, 28, 4, 15, 0, 0},
    /* V 7 */ {156, 20, 2, 78, 0, 0},      {124, 18, 4, 31, 0, 0},
    /* V 7 */ {88, 18, 2, 14, 4, 15},      {66, 26, 4, 13, 1, 14},
    /* V 8 */ {194, 24, 2, 97, 0, 0},      {154, 22, 2, 38, 2, 39},
    /* V 8 */ {110, 22, 4, 18, 2, 19},     {86, 26, 4, 14, 2, 15},
    /* V 9 */ {232, 30, 2, 116, 0, 0},     {182, 22, 3, 36, 2, 37},
    /* V 9 */ {132, 20, 4, 16, 4, 17},     {100, 24, 4, 12, 4, 13},
    /* V10 */ {274, 18, 2, 68, 2, 69},     {216, 26, 4, 43, 1, 44},
    /* V10 */ {154, 24, 6, 19, 2, 20},     {122, 28, 6, 15, 2, 16},
    /* V11 */ {324, 20, 4, 81, 0, 0},      {254, 30, 1, 50, 4, 51},
    /* V11 */ {180, 28, 4, 22, 4, 23},     {140, 24, 3, 12, 8, 13},
    /* V12 */ {370, 24, 2, 92, 2, 93},     {290, 22, 6, 36, 2, 37},
    /* V12 */ {206, 26, 4, 20, 6, 21},     {158, 28, 7, 14, 4, 15},
    /* V13 */ {428, 26, 4, 107, 0, 0},     {334, 22, 8, 37, 1, 38},
    /* V13 */ {244, 24, 8, 20, 4, 21},     {180, 22, 12, 11, 4, 12},
    /* V14 */ {461, 30, 3, 115, 1, 116},   {365, 24, 4, 40, 5, 41},
    /* V14 */ {261, 20, 11, 16, 5, 17},    {197, 24, 11, 12, 5, 13},
    /* V15 */ {523, 22, 5, 87, 1, 88},     {415, 24, 5, 41, 5, 42},
    /* V15 */ {295, 30, 5, 24, 7, 25},     {223, 24, 11, 12, 7, 13},
    /* V16 */ {589, 24, 5, 98, 1, 99},     {453, 28, 7, 45, 3, 46},
    /* V16 */ {325, 24, 15, 19, 2, 20},    {253, 30, 3, 15, 13, 16},
    /* V17 */ {647, 28, 1, 107, 5, 108},   {507, 28, 10, 46, 1, 47},
    /* V17 */ {367, 28, 1, 22, 15, 23},    {283, 28, 2, 14, 17, 15},
    /* V18 */ {721, 30, 5, 120, 1, 121},   {563, 26, 9, 43, 4, 44},
    /* V18 */ {397, 28, 17, 22, 1, 23},    {313, 28, 2, 14, 19, 15},
    /* V19 */ {795, 28, 3, 113, 4, 114},   {627, 26, 3, 44, 11, 45},
    /* V19 */ {445, 26, 17, 21, 4, 22},    {341, 26, 9, 13, 16, 14},
    /* V20 */ {861, 28, 3, 107, 5, 108},   {669, 26, 3, 41, 13, 42},
    /* V20 */ {485, 30, 15, 24, 5, 25},    {385, 28, 15, 15, 10, 16},
    /* V21 */ {932, 28, 4, 116, 4, 117},   {714, 26, 17, 42, 0, 0},
    /* V21 */ {512, 28, 17, 22, 6, 23},    {406, 30, 19, 16, 6, 17},
    /* V22 */ {1006, 28, 2, 111, 7, 112},  {782, 28, 17, 46, 0, 0},
    /* V22 */ {568, 30, 7, 24, 16, 25},    {442, 24, 34, 13, 0, 0},
    /* V23 */ {1094, 30, 4, 121, 5, 122},  {860, 28, 4, 47, 14, 48},
    /* V23 */ {614, 30, 11, 24, 14, 25},   {464, 30, 16, 15, 14, 16},
    /* V24 */ {1174, 30, 6, 117, 4, 118},  {914, 28, 6, 45, 14, 46},
    /* V24 */ {664, 30, 11, 24, 16, 25},   {514, 30, 30, 16, 2, 17},
    /* V25 */ {1276, 26, 8, 106, 4, 107},  {1000, 28, 8, 47, 13, 48},
    /* V25 */ {718, 30, 7, 24, 22, 25},    {538, 30, 22, 15, 13, 16},
    /* V26 */ {1370, 28, 10, 114, 2, 115}, {1062, 28, 19, 46, 4, 47},
    /* V26 */ {754, 28, 28, 22, 6, 23},    {596, 30, 33, 16, 4, 17},
    /* V27 */ {1468, 30, 8, 122, 4, 123},  {1128, 28, 22, 45, 3, 46},
    /* V27 */ {808, 30, 8, 23, 26, 24},    {628, 30, 12, 15, 28, 16},
    /* V28 */ {1531, 30, 3, 117, 10, 118}, {1193, 28, 3, 45, 23, 46},
    /* V28 */ {871, 30, 4, 24, 31, 25},    {661, 30, 11, 15, 31, 16},
    /* V29 */ {1631, 30, 7, 116, 7, 117},  {1267, 28, 21, 45, 7, 46},
    /* V29 */ {911, 30, 1, 23, 37, 24},    {701, 30, 19, 15, 26, 16},
    /* V30 */ {1735, 30, 5, 115, 10, 116}, {1373, 28, 19, 47, 10, 48},
    /* V30 */ {985, 30, 15, 24, 25, 25},   {745, 30, 23, 15, 25, 16},
    /* V31 */ {1843, 30, 13, 115, 3, 116}, {1455, 28, 2, 46, 29, 47},
    /* V31 */ {1033, 30, 42, 24, 1, 25},   {793, 30, 23, 15, 28, 16},
    /* V32 */ {1955, 30, 17, 115, 0, 0},   {1541, 28, 10, 46, 23, 47},
    /* V32 */ {1115, 30, 10, 24, 35, 25},  {845, 30, 19, 15, 35, 16},
    /* V33 */ {2071, 30, 17, 115, 1, 116}, {1631, 28, 14, 46, 21, 47},
    /* V33 */ {1171, 30, 29, 24, 19, 25},  {901, 30, 11, 15, 46, 16},
    /* V34 */ {2191, 30, 13, 115, 6, 116}, {1725, 28, 14, 46, 23, 47},
    /* V34 */ {1231, 30, 44, 24, 7, 25},   {961, 30, 59, 16, 1, 17},
    /* V35 */ {2306, 30, 12, 121, 7, 122}, {1812, 28, 12, 47, 26, 48},
    /* V35 */ {1286, 30, 39, 24, 14, 25},  {986, 30, 22, 15, 41, 16},
    /* V36 */ {2434, 30, 6, 121, 14, 122}, {1914, 28, 6, 47, 34, 48},
    /* V36 */ {1354, 30, 46, 24, 10, 25},  {1054, 30, 2, 15, 64, 16},
    /* V37 */ {2566, 30, 17, 122, 4, 123}, {1992, 28, 29, 46, 14, 47},
    /* V37 */ {1426, 30, 49, 24, 10, 25},  {1096, 30, 24, 15, 46, 16},
    /* V38 */ {2702, 30, 4, 122, 18, 123}, {2102, 28, 13, 46, 32, 47},
    /* V38 */ {1502, 30, 48, 24, 14, 25},  {1142, 30, 42, 15, 32, 16},
    /* V39 */ {2812, 30, 20, 117, 4, 118}, {2216, 28, 40, 47, 7, 48},
    /* V39 */ {1582, 30, 43, 24, 22, 25},  {1222, 30, 10, 15, 67, 16},
    /* V40 */ {2956, 30, 19, 118, 6, 119}, {2334, 28, 18, 47, 31, 48},
    /* V40 */ {1666, 30, 34, 24, 34, 25},  {1276, 30, 20, 15, 61, 16},
};

/* Remainder bits per version */
static const int g_remainder_bits[40] = {
    0, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 3, 3, 3, 3, 3, 3, 3,
    4, 4, 4, 4, 4, 4, 4, 3, 3, 3, 3, 3, 3, 3, 0, 0, 0, 0, 0, 0,
};

/* Alignment pattern positions per version (zero-terminated) */
static const int g_align_positions[41][8] = {
    {0},        /* V  0 (unused) */
    {0},        /* V  1 */
    {6, 18, 0}, /* V  2 */
    {6, 22, 0}, /* V  3 */
    {6, 26, 0}, /* V  4 */
    {6, 30, 0}, /* V  5 */
    {6, 34, 0}, /* V  6 */
    /* V 7 */ {6, 22, 38, 0},
    /* V 8 */ {6, 24, 42, 0},
    /* V 9 */ {6, 26, 46, 0},
    /* V10 */ {6, 28, 50, 0},
    /* V11 */ {6, 30, 54, 0},
    /* V12 */ {6, 32, 58, 0},
    /* V13 */ {6, 34, 62, 0},
    /* V14 */ {6, 26, 46, 66, 0},
    /* V15 */ {6, 26, 48, 70, 0},
    /* V16 */ {6, 26, 50, 74, 0},
    /* V17 */ {6, 30, 54, 78, 0},
    /* V18 */ {6, 30, 56, 82, 0},
    /* V19 */ {6, 30, 58, 86, 0},
    /* V20 */ {6, 34, 62, 90, 0},
    /* V21 */ {6, 28, 50, 72, 94, 0},
    /* V22 */ {6, 26, 50, 74, 98, 0},
    /* V23 */ {6, 30, 54, 78, 102, 0},
    /* V24 */ {6, 28, 54, 80, 106, 0},
    /* V25 */ {6, 32, 58, 84, 110, 0},
    /* V26 */ {6, 30, 58, 86, 114, 0},
    /* V27 */ {6, 34, 62, 90, 118, 0},
    /* V28 */ {6, 26, 50, 74, 98, 122, 0},
    /* V29 */ {6, 30, 54, 78, 102, 126, 0},
    /* V30 */ {6, 26, 52, 78, 104, 130, 0},
    /* V31 */ {6, 30, 56, 82, 108, 134, 0},
    /* V32 */ {6, 34, 60, 86, 112, 138, 0},
    /* V33 */ {6, 30, 58, 86, 114, 142, 0},
    /* V34 */ {6, 34, 62, 90, 118, 146, 0},
    /* V35 */ {6, 30, 54, 78, 102, 126, 150, 0},
    /* V36 */ {6, 24, 50, 76, 102, 128, 154, 0},
    /* V37 */ {6, 28, 54, 80, 106, 132, 158, 0},
    /* V38 */ {6, 32, 58, 84, 110, 136, 162, 0},
    /* V39 */ {6, 26, 54, 82, 110, 138, 166, 0},
    /* V40 */ {6, 30, 58, 86, 114, 142, 170, 0},
};

/* ==================================================================
 *  Public table accessors
 * ================================================================== */

int molqrc_capacity(int version, int ecl) {
  return g_version_table[(version - 1) * 4 + ecl].total_cw;
}

int molqrc_ec_codewords_per_block(int version, int ecl) {
  return g_version_table[(version - 1) * 4 + ecl].ec_per_block;
}

void molqrc_block_info(int version, int ecl, int *g1_blocks, int *g1_cw,
                       int *g2_blocks, int *g2_cw) {
  const version_ecl_t *v = &g_version_table[(version - 1) * 4 + ecl];
  *g1_blocks = v->g1_blocks;
  *g1_cw = v->g1_cw;
  *g2_blocks = v->g2_blocks;
  *g2_cw = v->g2_cw;
}

int molqrc_remainder_bits(int version) { return g_remainder_bits[version - 1]; }

int molqrc_alignment_positions(int version, int *positions) {
  int i;
  const int *src = g_align_positions[version];
  for (i = 0; src[i] != 0 && i < 8; i++)
    positions[i] = src[i];
  return i;
}

int molqrc_raw_data_modules(int version) {
  /* Total codewords = data + EC is constant per version (independent of ECL).
     Use ECL L (0) for convenience. */
  int data_cw = molqrc_capacity(version, 0);
  int ec_per_block = molqrc_ec_codewords_per_block(version, 0);
  int g1b, g1cw, g2b, g2cw;
  molqrc_block_info(version, 0, &g1b, &g1cw, &g2b, &g2cw);
  int total_blocks = g1b + g2b;
  int total_cw = data_cw + total_blocks * ec_per_block;
  int rem = molqrc_remainder_bits(version);
  return total_cw * 8 + rem;
}

/* ==================================================================
 *  CCI bits table
 * ================================================================== */

int molqrc_cci_bits(int mode, int version) {
  /* Version groups: 1-9, 10-26, 27-40 */
  int group = (version <= 9) ? 0 : (version <= 26) ? 1 : 2;

  switch (mode) {
  case MOLQRC_MODE_NUMERIC:
    return (group == 0) ? 10 : (group == 1) ? 12 : 14;
  case MOLQRC_MODE_ALPHANUMERIC:
    return (group == 0) ? 9 : (group == 1) ? 11 : 13;
  case MOLQRC_MODE_BYTE:
    return (group == 0) ? 8 : 16;
  case MOLQRC_MODE_KANJI:
    return (group == 0) ? 8 : (group == 1) ? 10 : 12;
  case MOLQRC_MODE_ECI:
    return 0;
  default:
    return 0;
  }
}

/* ==================================================================
 *  Bit buffer
 * ================================================================== */

void molqrc_bitbuf_init(molqrc_bitbuf_t *bb, unsigned char *buf, int capacity) {
  bb->buf = buf;
  bb->bitlen = 0;
  bb->capacity = capacity;
  memset(buf, 0, (size_t)((capacity + 7) / 8));
}

void molqrc_bitbuf_append(molqrc_bitbuf_t *bb, int value, int nbits) {
  int i;
  for (i = nbits - 1; i >= 0; i--) {
    int bytepos = bb->bitlen / 8;
    int bitpos = 7 - (bb->bitlen % 8);
    if (bytepos < bb->capacity / 8) {
      if (value & (1 << i))
        bb->buf[bytepos] |= (unsigned char)(1 << bitpos);
    }
    bb->bitlen++;
  }
}

/* ==================================================================
 *  Encode segments → data codewords
 * ================================================================== */

int molqrc_encode_segments_to_codewords(const molqrc_segment_t *segs,
                                        int num_segs, unsigned char *codewords,
                                        int max_cw, int version, int ecl) {
  molqrc_bitbuf_t bb;
  int capacity, total_bits, i;

  (void)max_cw;

  if (!segs || num_segs <= 0)
    return 0;

  capacity = molqrc_capacity(version, ecl);

  /* Compute total bits needed */
  total_bits = 0;
  for (i = 0; i < num_segs; i++) {
    const molqrc_segment_t *seg = &segs[i];
    int cci = molqrc_cci_bits(seg->mode, version);

    /* Check character count fits in CCI field */
    if (seg->mode != MOLQRC_MODE_ECI) {
      long max_count = (1L << cci) - 1;
      if (seg->num_chars > max_count)
        return 0;
    }

    total_bits += 4; /* mode indicator */
    if (seg->mode != MOLQRC_MODE_ECI)
      total_bits += cci;           /* character count indicator */
    total_bits += seg->bit_length; /* data */
  }

  if (total_bits > capacity * 8)
    return 0;

  molqrc_bitbuf_init(&bb, codewords, capacity * 8);

  /* Write segments */
  for (i = 0; i < num_segs; i++) {
    const molqrc_segment_t *seg = &segs[i];

    /* Mode indicator (4 bits) */
    molqrc_bitbuf_append(&bb, seg->mode, 4);

    /* Character count indicator */
    if (seg->mode != MOLQRC_MODE_ECI) {
      int cci = molqrc_cci_bits(seg->mode, version);
      molqrc_bitbuf_append(&bb, seg->num_chars, cci);
    }

    /* Data bits */
    {
      int b;
      for (b = 0; b < seg->bit_length; b++) {
        int bytepos = b / 8;
        int bitpos = 7 - (b % 8);
        int bit = (seg->data[bytepos] >> bitpos) & 1;
        molqrc_bitbuf_append(&bb, bit, 1);
      }
    }
  }

  /* Terminator (up to 4 bits of 0) */
  {
    int remaining = capacity * 8 - bb.bitlen;
    if (remaining > 4)
      remaining = 4;
    if (remaining > 0)
      molqrc_bitbuf_append(&bb, 0, remaining);
  }

  /* Pad to byte boundary */
  while (bb.bitlen % 8 != 0)
    molqrc_bitbuf_append(&bb, 0, 1);

  /* Pad codewords: alternate 0xEC and 0x11 */
  {
    int cw_pos = bb.bitlen / 8;
    int pad = 1;
    while (cw_pos < capacity) {
      codewords[cw_pos++] = (unsigned char)(pad ? 0xEC : 0x11);
      pad = !pad;
    }
  }

  return capacity;
}

/* ==================================================================
 *  Block interleaving
 * ================================================================== */

void molqrc_interleave(const unsigned char *data, int *ec_blocks,
                       unsigned char *out, int version, int ecl) {
  int g1b, g1cw, g2b, g2cw, ec_per_block;
  int total_blocks, max_data_cw;
  int blk, i;

  molqrc_block_info(version, ecl, &g1b, &g1cw, &g2b, &g2cw);
  ec_per_block = molqrc_ec_codewords_per_block(version, ecl);
  total_blocks = g1b + g2b;
  max_data_cw = (g1cw > g2cw) ? g1cw : g2cw;

  {
    int src = 0;
    unsigned char block_buf[4096];

    for (blk = 0; blk < total_blocks; blk++) {
      int dcw = (blk < g1b) ? g1cw : g2cw;
      for (i = 0; i < dcw; i++)
        block_buf[i] = data[src++];
      molqrc_rs_encode(block_buf, dcw, ec_per_block);
      for (i = 0; i < dcw + ec_per_block; i++)
        ec_blocks[blk * (max_data_cw + ec_per_block) + i] = block_buf[i];
    }
  }

  /* Interleave data codewords */
  {
    int out_pos = 0;
    for (i = 0; i < max_data_cw; i++) {
      for (blk = 0; blk < total_blocks; blk++) {
        int dcw = (blk < g1b) ? g1cw : g2cw;
        if (i < dcw)
          out[out_pos++] =
              (unsigned char)ec_blocks[blk * (max_data_cw + ec_per_block) + i];
      }
    }
    for (i = 0; i < ec_per_block; i++) {
      for (blk = 0; blk < total_blocks; blk++) {
        int off = max_data_cw + i;
        out[out_pos++] =
            (unsigned char)ec_blocks[blk * (max_data_cw + ec_per_block) + off];
      }
    }
  }
}
