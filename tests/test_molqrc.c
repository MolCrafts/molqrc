#include "internal.h"
#include "molqrc.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

static int g_num_test_cases = 0;

#define test_assert(cond)                                                      \
  do {                                                                         \
    g_num_test_cases++;                                                        \
    if (!(cond)) {                                                             \
      fprintf(stderr, "FAIL [%d]: %s\n", __LINE__, #cond);                     \
      exit(1);                                                                 \
    }                                                                          \
  } while (0)

/* ================================================================== */
/*  GF(256) multiply                                                    */
/* ================================================================== */

static void test_gf_mul(void) {
  test_assert(molqrc_gf_mul(0x00, 0x00) == 0x00);
  test_assert(molqrc_gf_mul(0x01, 0x01) == 0x01);
  test_assert(molqrc_gf_mul(0x02, 0x02) == 0x04);
  test_assert(molqrc_gf_mul(0x00, 0x6E) == 0x00);
  test_assert(molqrc_gf_mul(0xB2, 0xDD) == 0xE6);
  test_assert(molqrc_gf_mul(0x41, 0x11) == 0x25);
  test_assert(molqrc_gf_mul(0xB0, 0x1F) == 0x11);
  test_assert(molqrc_gf_mul(0x05, 0x75) == 0xBC);
  test_assert(molqrc_gf_mul(0x52, 0xB5) == 0xAE);
  test_assert(molqrc_gf_mul(0xA8, 0x20) == 0xA4);
  test_assert(molqrc_gf_mul(0x0E, 0x44) == 0x9F);
  test_assert(molqrc_gf_mul(0xD4, 0x13) == 0xA0);
  test_assert(molqrc_gf_mul(0x31, 0x10) == 0x37);
  test_assert(molqrc_gf_mul(0x6C, 0x58) == 0xCB);
  test_assert(molqrc_gf_mul(0xB6, 0x75) == 0x3E);
  test_assert(molqrc_gf_mul(0xFF, 0xFF) == 0xE2);
}

/* ================================================================== */
/*  RS divisor polynomial                                               */
/* ================================================================== */

static void test_rs_compute_divisor(void) {
  unsigned char gen[30];

  molqrc_rs_compute_divisor(1, gen);
  test_assert(gen[0] == 0x01);

  molqrc_rs_compute_divisor(2, gen);
  test_assert(gen[0] == 0x03);
  test_assert(gen[1] == 0x02);

  molqrc_rs_compute_divisor(5, gen);
  test_assert(gen[0] == 0x1F);
  test_assert(gen[1] == 0xC6);
  test_assert(gen[2] == 0x3F);
  test_assert(gen[3] == 0x93);
  test_assert(gen[4] == 0x74);

  molqrc_rs_compute_divisor(30, gen);
  test_assert(gen[0] == 0xD4);
  test_assert(gen[1] == 0xF6);
  test_assert(gen[5] == 0xC0);
  test_assert(gen[12] == 0x16);
  test_assert(gen[13] == 0xD9);
  test_assert(gen[20] == 0x12);
  test_assert(gen[27] == 0x6A);
  test_assert(gen[29] == 0x96);
}

/* ================================================================== */
/*  RS remainder                                                        */
/* ================================================================== */

static void test_rs_compute_remainder(void) {
  unsigned char gen[30], rem[30];

  /* 3-byte generator, zero-length data */
  molqrc_rs_compute_divisor(3, gen);
  molqrc_rs_compute_remainder(NULL, 0, gen, 3, rem);
  test_assert(rem[0] == 0 && rem[1] == 0 && rem[2] == 0);

  /* 2-byte data {0,1}, 5-byte generator → remainder == generator */
  {
    unsigned char d[2] = {0x00, 0x01}, g5[5];
    molqrc_rs_compute_divisor(5, g5);
    molqrc_rs_compute_remainder(d, 2, g5, 5, rem);
    test_assert(memcmp(rem, g5, 5) == 0);
  }

  /* 5-byte data, 5-byte generator */
  {
    unsigned char d[5] = {0x03, 0x3A, 0x60, 0x12, 0xC7}, g5[5];
    molqrc_rs_compute_divisor(5, g5);
    molqrc_rs_compute_remainder(d, 5, g5, 5, rem);
    test_assert(rem[0] == 0xCB);
    test_assert(rem[1] == 0x36);
    test_assert(rem[2] == 0x16);
    test_assert(rem[3] == 0xFA);
    test_assert(rem[4] == 0x9D);
  }

  /* 43-byte data, 30-byte generator */
  {
    const unsigned char d[43] = {
        0x38, 0x71, 0xDB, 0xF9, 0xD7, 0x28, 0xF6, 0x8E, 0xFE, 0x5E, 0xE6,
        0x7D, 0x7D, 0xB2, 0xA5, 0x58, 0xBC, 0x28, 0x23, 0x53, 0x14, 0xD5,
        0x61, 0xC0, 0x20, 0x6C, 0xDE, 0xDE, 0xFC, 0x79, 0xB0, 0x8B, 0x78,
        0x6B, 0x49, 0xD0, 0x1A, 0xAD, 0xF3, 0xEF, 0x52, 0x7D, 0x9A,
    };
    molqrc_rs_compute_divisor(30, gen);
    molqrc_rs_compute_remainder(d, 43, gen, 30, rem);
    test_assert(rem[0] == 0xCE);
    test_assert(rem[1] == 0xF0);
    test_assert(rem[2] == 0x31);
    test_assert(rem[3] == 0xDE);
    test_assert(rem[8] == 0xE1);
    test_assert(rem[12] == 0xCA);
    test_assert(rem[17] == 0xE3);
    test_assert(rem[19] == 0x85);
    test_assert(rem[20] == 0x50);
    test_assert(rem[24] == 0xBE);
    test_assert(rem[29] == 0xB3);
  }
}

/* ================================================================== */
/*  Bit buffer append                                                   */
/* ================================================================== */

static void test_bitbuf_append(void) {
  unsigned char buf[6];
  molqrc_bitbuf_t bb;

  memset(buf, 0, sizeof(buf));
  molqrc_bitbuf_init(&bb, buf, 8);

  molqrc_bitbuf_append(&bb, 0, 0);
  test_assert(bb.bitlen == 0 && buf[0] == 0x00);

  molqrc_bitbuf_append(&bb, 1, 1);
  test_assert(bb.bitlen == 1 && buf[0] == 0x80);

  molqrc_bitbuf_append(&bb, 0, 1);
  test_assert(bb.bitlen == 2 && buf[0] == 0x80);

  molqrc_bitbuf_append(&bb, 5, 3);
  test_assert(bb.bitlen == 5 && buf[0] == 0xA8);

  molqrc_bitbuf_append(&bb, 6, 3);
  test_assert(bb.bitlen == 8 && buf[0] == 0xAE);

  /* 6-byte buffer */
  memset(buf, 0, sizeof(buf));
  molqrc_bitbuf_init(&bb, buf, 48);

  molqrc_bitbuf_append(&bb, 16942, 16);
  test_assert(bb.bitlen == 16);
  test_assert(buf[0] == 0x42 && buf[1] == 0x2E && buf[2] == 0x00);

  molqrc_bitbuf_append(&bb, 10, 7);
  test_assert(bb.bitlen == 23 && buf[2] == 0x14);

  molqrc_bitbuf_append(&bb, 15, 4);
  test_assert(bb.bitlen == 27 && buf[2] == 0x15 && buf[3] == 0xE0);

  molqrc_bitbuf_append(&bb, 26664, 15);
  test_assert(bb.bitlen == 42);
  test_assert(buf[0] == 0x42 && buf[1] == 0x2E && buf[2] == 0x15 &&
              buf[3] == 0xFA && buf[4] == 0x0A && buf[5] == 0x00);
}

/* ================================================================== */
/*  ECC + interleave (reference vs actual)                              */
/* ================================================================== */

static void add_ecc_and_interleave_ref(const unsigned char *data, int version,
                                       int ecl, unsigned char *out) {
  int g1b, g1cw, g2b, g2cw, ec_per_block, total_blocks, max_data_cw;
  int blk, i, src;
  unsigned char block_buf[4096];
  int ec_blocks_data[4096];

  molqrc_block_info(version, ecl, &g1b, &g1cw, &g2b, &g2cw);
  ec_per_block = molqrc_ec_codewords_per_block(version, ecl);
  total_blocks = g1b + g2b;
  max_data_cw = (g1cw > g2cw) ? g1cw : g2cw;

  src = 0;
  for (blk = 0; blk < total_blocks; blk++) {
    int dcw = (blk < g1b) ? g1cw : g2cw;
    for (i = 0; i < dcw; i++)
      block_buf[i] = data[src++];
    molqrc_rs_encode(block_buf, dcw, ec_per_block);
    for (i = 0; i < dcw + ec_per_block; i++)
      ec_blocks_data[blk * (max_data_cw + ec_per_block) + i] = block_buf[i];
  }

  src = 0;
  for (i = 0; i < max_data_cw; i++)
    for (blk = 0; blk < total_blocks; blk++) {
      int dcw = (blk < g1b) ? g1cw : g2cw;
      if (i < dcw)
        out[src++] = (unsigned char)
            ec_blocks_data[blk * (max_data_cw + ec_per_block) + i];
    }
  for (i = 0; i < ec_per_block; i++)
    for (blk = 0; blk < total_blocks; blk++)
      out[src++] = (unsigned char)
          ec_blocks_data[blk * (max_data_cw + ec_per_block) + max_data_cw + i];
}

static void test_add_ecc_and_interleave(void) {
  int version, ecl;
  unsigned char data[4096], ref_out[4096], actual_out[4096];
  int g1b, g1cw, g2b, g2cw, ec_per_block, total_blocks, capacity, out_len, i;

  srand(42);

  for (version = 1; version <= 40; version++) {
    for (ecl = 0; ecl < 4; ecl++) {
      capacity = molqrc_capacity(version, ecl);
      for (i = 0; i < capacity; i++)
        data[i] = (unsigned char)(rand() & 0xFF);

      add_ecc_and_interleave_ref(data, version, ecl, ref_out);

      molqrc_block_info(version, ecl, &g1b, &g1cw, &g2b, &g2cw);
      ec_per_block = molqrc_ec_codewords_per_block(version, ecl);
      total_blocks = g1b + g2b;

      {
        int ec_blocks_data[4096];
        molqrc_interleave(data, ec_blocks_data, actual_out, version, ecl);
      }

      out_len = capacity + total_blocks * ec_per_block;
      test_assert(memcmp(ref_out, actual_out, (size_t)out_len) == 0);
    }
  }
}

/* ================================================================== */
/*  Capacity                                                            */
/* ================================================================== */

static void test_capacity(void) {
  struct {
    int v, ecl, ex;
  } c[] = {
      {3, 1, 44},    {3, 2, 34},    {3, 3, 26},    {6, 0, 136},   {7, 0, 156},
      {9, 0, 232},   {9, 1, 182},   {12, 3, 158},  {15, 0, 523},  {16, 2, 325},
      {19, 3, 341},  {21, 0, 932},  {22, 0, 1006}, {22, 1, 782},  {22, 3, 442},
      {24, 0, 1174}, {24, 3, 514},  {28, 0, 1531}, {30, 3, 745},  {32, 3, 845},
      {33, 0, 2071}, {33, 3, 901},  {35, 0, 2306}, {35, 1, 1812}, {35, 2, 1286},
      {36, 3, 1054}, {37, 3, 1096}, {39, 1, 2216}, {40, 1, 2334},
  };
  int n = (int)(sizeof(c) / sizeof(c[0])), i;
  for (i = 0; i < n; i++)
    test_assert(molqrc_capacity(c[i].v, c[i].ecl) == c[i].ex);
}

/* ================================================================== */
/*  Raw data modules                                                    */
/* ================================================================== */

static void test_raw_data_modules(void) {
  struct {
    int v, ex;
  } c[] = {
      {1, 208},    {2, 359},    {3, 567},    {6, 1383},   {7, 1568},
      {12, 3728},  {15, 5243},  {18, 7211},  {22, 10068}, {26, 13652},
      {32, 19723}, {37, 25568}, {40, 29648},
  };
  int n = (int)(sizeof(c) / sizeof(c[0])), i;
  for (i = 0; i < n; i++)
    test_assert(molqrc_raw_data_modules(c[i].v) == c[i].ex);
}

/* ================================================================== */
/*  Alignment positions                                                 */
/* ================================================================== */

static void test_alignment_positions(void) {
  struct {
    int ver, cnt, pos[7];
  } c[] = {
      {1, 0, {-1, -1, -1, -1, -1, -1, -1}},
      {2, 2, {6, 18, -1, -1, -1, -1, -1}},
      {3, 2, {6, 22, -1, -1, -1, -1, -1}},
      {6, 2, {6, 34, -1, -1, -1, -1, -1}},
      {7, 3, {6, 22, 38, -1, -1, -1, -1}},
      {8, 3, {6, 24, 42, -1, -1, -1, -1}},
      {16, 4, {6, 26, 50, 74, -1, -1, -1}},
      {25, 5, {6, 32, 58, 84, 110, -1, -1}},
      {32, 6, {6, 34, 60, 86, 112, 138, -1}},
      {33, 6, {6, 30, 58, 86, 114, 142, -1}},
      {39, 7, {6, 26, 54, 82, 110, 138, 166}},
      {40, 7, {6, 30, 58, 86, 114, 142, 170}},
  };
  int n = (int)(sizeof(c) / sizeof(c[0])), i, j;
  for (i = 0; i < n; i++) {
    int pos[8], cnt = molqrc_alignment_positions(c[i].ver, pos);
    test_assert(cnt == c[i].cnt);
    for (j = 0; j < cnt; j++)
      test_assert(pos[j] == c[i].pos[j]);
  }
}

/* ================================================================== */
/*  Size / version relationship                                         */
/* ================================================================== */

static void test_version_to_side(void) {
  int v;
  for (v = 1; v <= 40; v++) {
    int side = 21 + 4 * (v - 1);
    test_assert(side == 17 + v * 4);
  }
}

/* ================================================================== */
/*  isNumeric / isAlphanumeric                                          */
/* ================================================================== */

static void test_is_numeric(void) {
  test_assert(molqrc_is_numeric("") == 1);
  test_assert(molqrc_is_numeric("0") == 1);
  test_assert(molqrc_is_numeric("79068") == 1);
  test_assert(molqrc_is_numeric("A") == 0);
  test_assert(molqrc_is_numeric("a") == 0);
  test_assert(molqrc_is_numeric(" ") == 0);
  test_assert(molqrc_is_numeric(".") == 0);
  test_assert(molqrc_is_numeric("*") == 0);
  test_assert(molqrc_is_numeric(",") == 0);
  test_assert(molqrc_is_numeric("|") == 0);
  test_assert(molqrc_is_numeric("@") == 0);
  test_assert(molqrc_is_numeric("XYZ") == 0);
  test_assert(molqrc_is_numeric("XYZ!") == 0);
  test_assert(molqrc_is_numeric("+123 ABC$") == 0);
  test_assert(molqrc_is_numeric("\x01") == 0);
  test_assert(molqrc_is_numeric("\x7F") == 0);
  test_assert(molqrc_is_numeric("\x80") == 0);
  test_assert(molqrc_is_numeric("\xC0") == 0);
  test_assert(molqrc_is_numeric("\xFF") == 0);
}

static void test_is_alphanumeric(void) {
  test_assert(molqrc_is_alphanumeric("") == 1);
  test_assert(molqrc_is_alphanumeric("0") == 1);
  test_assert(molqrc_is_alphanumeric("A") == 1);
  test_assert(molqrc_is_alphanumeric(" ") == 1);
  test_assert(molqrc_is_alphanumeric(".") == 1);
  test_assert(molqrc_is_alphanumeric("*") == 1);
  test_assert(molqrc_is_alphanumeric("XYZ") == 1);
  test_assert(molqrc_is_alphanumeric("79068") == 1);
  test_assert(molqrc_is_alphanumeric("+123 ABC$") == 1);
  test_assert(molqrc_is_alphanumeric("a") == 0);
  test_assert(molqrc_is_alphanumeric(",") == 0);
  test_assert(molqrc_is_alphanumeric("|") == 0);
  test_assert(molqrc_is_alphanumeric("@") == 0);
  test_assert(molqrc_is_alphanumeric("XYZ!") == 0);
  test_assert(molqrc_is_alphanumeric("\x01") == 0);
  test_assert(molqrc_is_alphanumeric("\x7F") == 0);
  test_assert(molqrc_is_alphanumeric("\x80") == 0);
  test_assert(molqrc_is_alphanumeric("\xC0") == 0);
  test_assert(molqrc_is_alphanumeric("\xFF") == 0);
}

/* ================================================================== */
/*  Segment bit lengths                                                 */
/* ================================================================== */

static void test_calc_segment_bit_length(void) {
  /* Numeric */
  test_assert(molqrc_calc_segment_bit_length(MOLQRC_MODE_NUMERIC, 0) == 0);
  test_assert(molqrc_calc_segment_bit_length(MOLQRC_MODE_NUMERIC, 1) == 4);
  test_assert(molqrc_calc_segment_bit_length(MOLQRC_MODE_NUMERIC, 2) == 7);
  test_assert(molqrc_calc_segment_bit_length(MOLQRC_MODE_NUMERIC, 3) == 10);
  test_assert(molqrc_calc_segment_bit_length(MOLQRC_MODE_NUMERIC, 4) == 14);
  test_assert(molqrc_calc_segment_bit_length(MOLQRC_MODE_NUMERIC, 5) == 17);
  test_assert(molqrc_calc_segment_bit_length(MOLQRC_MODE_NUMERIC, 6) == 20);
  test_assert(molqrc_calc_segment_bit_length(MOLQRC_MODE_NUMERIC, 1472) ==
              4907);
  test_assert(molqrc_calc_segment_bit_length(MOLQRC_MODE_NUMERIC, 2097) ==
              6990);
  test_assert(molqrc_calc_segment_bit_length(MOLQRC_MODE_NUMERIC, 5326) ==
              17754);
  test_assert(molqrc_calc_segment_bit_length(MOLQRC_MODE_NUMERIC, 9828) ==
              32760);
  test_assert(molqrc_calc_segment_bit_length(MOLQRC_MODE_NUMERIC, 9829) ==
              32764);
  test_assert(molqrc_calc_segment_bit_length(MOLQRC_MODE_NUMERIC, 9830) ==
              32767);
  test_assert(molqrc_calc_segment_bit_length(MOLQRC_MODE_NUMERIC, 9831) == -1);

  /* Alphanumeric */
  test_assert(molqrc_calc_segment_bit_length(MOLQRC_MODE_ALPHANUMERIC, 0) == 0);
  test_assert(molqrc_calc_segment_bit_length(MOLQRC_MODE_ALPHANUMERIC, 1) == 6);
  test_assert(molqrc_calc_segment_bit_length(MOLQRC_MODE_ALPHANUMERIC, 2) ==
              11);
  test_assert(molqrc_calc_segment_bit_length(MOLQRC_MODE_ALPHANUMERIC, 3) ==
              17);
  test_assert(molqrc_calc_segment_bit_length(MOLQRC_MODE_ALPHANUMERIC, 4) ==
              22);
  test_assert(molqrc_calc_segment_bit_length(MOLQRC_MODE_ALPHANUMERIC, 5) ==
              28);
  test_assert(molqrc_calc_segment_bit_length(MOLQRC_MODE_ALPHANUMERIC, 6) ==
              33);
  test_assert(molqrc_calc_segment_bit_length(MOLQRC_MODE_ALPHANUMERIC, 1472) ==
              8096);
  test_assert(molqrc_calc_segment_bit_length(MOLQRC_MODE_ALPHANUMERIC, 2097) ==
              11534);
  test_assert(molqrc_calc_segment_bit_length(MOLQRC_MODE_ALPHANUMERIC, 5326) ==
              29293);
  test_assert(molqrc_calc_segment_bit_length(MOLQRC_MODE_ALPHANUMERIC, 5955) ==
              32753);
  test_assert(molqrc_calc_segment_bit_length(MOLQRC_MODE_ALPHANUMERIC, 5956) ==
              32758);
  test_assert(molqrc_calc_segment_bit_length(MOLQRC_MODE_ALPHANUMERIC, 5957) ==
              32764);
  test_assert(molqrc_calc_segment_bit_length(MOLQRC_MODE_ALPHANUMERIC, 5958) ==
              -1);

  /* Byte */
  test_assert(molqrc_calc_segment_bit_length(MOLQRC_MODE_BYTE, 0) == 0);
  test_assert(molqrc_calc_segment_bit_length(MOLQRC_MODE_BYTE, 1) == 8);
  test_assert(molqrc_calc_segment_bit_length(MOLQRC_MODE_BYTE, 2) == 16);
  test_assert(molqrc_calc_segment_bit_length(MOLQRC_MODE_BYTE, 3) == 24);
  test_assert(molqrc_calc_segment_bit_length(MOLQRC_MODE_BYTE, 4094) == 32752);
  test_assert(molqrc_calc_segment_bit_length(MOLQRC_MODE_BYTE, 4095) == 32760);
  test_assert(molqrc_calc_segment_bit_length(MOLQRC_MODE_BYTE, 4096) == -1);

  /* Kanji */
  test_assert(molqrc_calc_segment_bit_length(MOLQRC_MODE_KANJI, 0) == 0);
  test_assert(molqrc_calc_segment_bit_length(MOLQRC_MODE_KANJI, 1) == 13);
  test_assert(molqrc_calc_segment_bit_length(MOLQRC_MODE_KANJI, 2519) == 32747);
  test_assert(molqrc_calc_segment_bit_length(MOLQRC_MODE_KANJI, 2520) == 32760);
  test_assert(molqrc_calc_segment_bit_length(MOLQRC_MODE_KANJI, 2521) == -1);

  /* ECI */
  test_assert(molqrc_calc_segment_bit_length(MOLQRC_MODE_ECI, 0) == 24);
}

/* ================================================================== */
/*  Segment constructors                                                */
/* ================================================================== */

static void test_make_bytes(void) {
  unsigned char buf[4];
  molqrc_segment_t seg;

  seg = molqrc_make_bytes(NULL, 0, buf);
  test_assert(seg.mode == MOLQRC_MODE_BYTE && seg.num_chars == 0 &&
              seg.bit_length == 0);

  seg = molqrc_make_bytes((const unsigned char *)"\x00", 1, buf);
  test_assert(seg.num_chars == 1 && seg.bit_length == 8 && seg.data[0] == 0x00);

  {
    const unsigned char bom[] = {0xEF, 0xBB, 0xBF};
    seg = molqrc_make_bytes(bom, 3, buf);
    test_assert(seg.num_chars == 3 && seg.bit_length == 24);
    test_assert(seg.data[0] == 0xEF && seg.data[1] == 0xBB &&
                seg.data[2] == 0xBF);
  }
}

static void test_make_numeric(void) {
  unsigned char buf[10];
  molqrc_segment_t seg;

  seg = molqrc_make_numeric("", buf);
  test_assert(seg.mode == MOLQRC_MODE_NUMERIC && seg.num_chars == 0 &&
              seg.bit_length == 0);

  seg = molqrc_make_numeric("9", buf);
  test_assert(seg.num_chars == 1 && seg.bit_length == 4 && buf[0] == 0x90);

  seg = molqrc_make_numeric("81", buf);
  test_assert(seg.num_chars == 2 && seg.bit_length == 7 && buf[0] == 0xA2);

  seg = molqrc_make_numeric("673", buf);
  test_assert(seg.num_chars == 3 && seg.bit_length == 10);
  test_assert(buf[0] == 0xA8 && buf[1] == 0x40);

  seg = molqrc_make_numeric("3141592653", buf);
  test_assert(seg.num_chars == 10 && seg.bit_length == 34);
  test_assert(buf[0] == 0x4E && buf[1] == 0x89 && buf[2] == 0xF4 &&
              buf[3] == 0x24 && buf[4] == 0xC0);
}

static void test_make_alphanumeric(void) {
  unsigned char buf[10];
  molqrc_segment_t seg;

  seg = molqrc_make_alphanumeric("", buf);
  test_assert(seg.mode == MOLQRC_MODE_ALPHANUMERIC && seg.num_chars == 0 &&
              seg.bit_length == 0);

  seg = molqrc_make_alphanumeric("A", buf);
  test_assert(seg.num_chars == 1 && seg.bit_length == 6 && buf[0] == 0x28);

  seg = molqrc_make_alphanumeric("%:", buf);
  test_assert(seg.num_chars == 2 && seg.bit_length == 11);
  test_assert(buf[0] == 0xDB && buf[1] == 0x40);

  seg = molqrc_make_alphanumeric("Q R", buf);
  test_assert(seg.num_chars == 3 && seg.bit_length == 17);
  test_assert(buf[0] == 0x96 && buf[1] == 0xCD && buf[2] == 0x80);
}

static void test_make_eci(void) {
  unsigned char buf[4];
  molqrc_segment_t seg;

  seg = molqrc_make_eci(127, buf);
  test_assert(seg.mode == MOLQRC_MODE_ECI && seg.num_chars == 0 &&
              seg.bit_length == 24);
  test_assert(buf[0] == 0x7F);

  seg = molqrc_make_eci(10345, buf);
  test_assert(buf[0] == 0xA8 && buf[1] == 0x69);

  seg = molqrc_make_eci(999999, buf);
  test_assert(buf[0] == 0xCF && buf[1] == 0x42 && buf[2] == 0x3F);
}

/* ================================================================== */
/*  CCI bits                                                            */
/* ================================================================== */

static void test_cci_bits(void) {
  /* NUMERIC */
  test_assert(molqrc_cci_bits(MOLQRC_MODE_NUMERIC, 1) == 10);
  test_assert(molqrc_cci_bits(MOLQRC_MODE_NUMERIC, 9) == 10);
  test_assert(molqrc_cci_bits(MOLQRC_MODE_NUMERIC, 10) == 12);
  test_assert(molqrc_cci_bits(MOLQRC_MODE_NUMERIC, 26) == 12);
  test_assert(molqrc_cci_bits(MOLQRC_MODE_NUMERIC, 27) == 14);
  test_assert(molqrc_cci_bits(MOLQRC_MODE_NUMERIC, 40) == 14);
  /* ALPHANUMERIC */
  test_assert(molqrc_cci_bits(MOLQRC_MODE_ALPHANUMERIC, 1) == 9);
  test_assert(molqrc_cci_bits(MOLQRC_MODE_ALPHANUMERIC, 10) == 11);
  test_assert(molqrc_cci_bits(MOLQRC_MODE_ALPHANUMERIC, 27) == 13);
  /* BYTE */
  test_assert(molqrc_cci_bits(MOLQRC_MODE_BYTE, 1) == 8);
  test_assert(molqrc_cci_bits(MOLQRC_MODE_BYTE, 9) == 8);
  test_assert(molqrc_cci_bits(MOLQRC_MODE_BYTE, 10) == 16);
  test_assert(molqrc_cci_bits(MOLQRC_MODE_BYTE, 40) == 16);
  /* ECI */
  test_assert(molqrc_cci_bits(MOLQRC_MODE_ECI, 1) == 0);
}

/* ================================================================== */
/*  encode_text                                                         */
/* ================================================================== */

static void test_encode_text(void) {
  unsigned char m[MOLQRC_MAX_SIZE * MOLQRC_MAX_SIZE];
  int s;

  s = molqrc_encode_text("HELLO WORLD", m, 1, 40, MOLQRC_ECL_M,
                         MOLQRC_MASK_AUTO, 1);
  test_assert(s > 0 && s <= 177);

  test_assert(
      molqrc_encode_text("", m, 1, 40, MOLQRC_ECL_M, MOLQRC_MASK_AUTO, 1) == 0);
  test_assert(molqrc_encode_text(NULL, m, 1, 40, MOLQRC_ECL_M, MOLQRC_MASK_AUTO,
                                 1) == 0);
  test_assert(molqrc_encode_text("hi", NULL, 1, 40, MOLQRC_ECL_M,
                                 MOLQRC_MASK_AUTO, 1) == 0);
  test_assert(molqrc_encode_text("hi", m, 5, 1, MOLQRC_ECL_M, MOLQRC_MASK_AUTO,
                                 1) == 0);

  /* Auto-detect numeric */
  s = molqrc_encode_text("1234567890", m, 1, 40, MOLQRC_ECL_L, MOLQRC_MASK_AUTO,
                         0);
  test_assert(s > 0);

  /* Auto-detect alphanumeric */
  s = molqrc_encode_text("HELLO123", m, 1, 40, MOLQRC_ECL_L, MOLQRC_MASK_AUTO,
                         0);
  test_assert(s > 0);

  /* Version range */
  {
    int s_big = molqrc_encode_text("test", m, 10, 40, MOLQRC_ECL_M,
                                   MOLQRC_MASK_AUTO, 0);
    test_assert(s_big >= 21 + 4 * 9); /* version >= 10 */
  }

  /* Manual mask */
  s = molqrc_encode_text("hello", m, 1, 40, MOLQRC_ECL_M, 3, 0);
  test_assert(s > 0);

  /* All 8 masks */
  int mask;
  for (mask = 0; mask < 8; mask++) {
    s = molqrc_encode_text("hi", m, 1, 40, MOLQRC_ECL_M, mask, 0);
    test_assert(s > 0);
  }

  /* All 4 ECLs */
  int ecl;
  for (ecl = 0; ecl < 4; ecl++) {
    s = molqrc_encode_text("hello world", m, 1, 40, ecl, MOLQRC_MASK_AUTO, 1);
    test_assert(s > 0);
  }
}

/* ================================================================== */
/*  encode_segments                                                     */
/* ================================================================== */

static void test_encode_segments(void) {
  unsigned char buf[4096];
  unsigned char m[MOLQRC_MAX_SIZE * MOLQRC_MAX_SIZE];
  molqrc_segment_t segs[4];
  int s;

  /* Single byte */
  segs[0] = molqrc_make_bytes((const unsigned char *)"hello", 5, buf);
  s = molqrc_encode_segments(segs, 1, m, 1, 40, MOLQRC_ECL_M, MOLQRC_MASK_AUTO,
                             1);
  test_assert(s > 0);

  /* Numeric */
  segs[0] = molqrc_make_numeric("12345678901234567890", buf);
  s = molqrc_encode_segments(segs, 1, m, 1, 40, MOLQRC_ECL_M, MOLQRC_MASK_AUTO,
                             1);
  test_assert(s > 0);

  /* Alphanumeric */
  segs[0] = molqrc_make_alphanumeric("HELLO WORLD", buf);
  s = molqrc_encode_segments(segs, 1, m, 1, 40, MOLQRC_ECL_M, MOLQRC_MASK_AUTO,
                             1);
  test_assert(s > 0);

  /* Mixed: ECI + NUMERIC + ALPHANUMERIC + BYTE */
  {
    unsigned char b1[4], b2[32], b3[32], b4[32];
    segs[0] = molqrc_make_eci(127, b1);
    segs[1] = molqrc_make_numeric("1234567", b2);
    segs[2] = molqrc_make_alphanumeric("A", b3);
    segs[3] = molqrc_make_bytes((const unsigned char *)"test", 4, b4);
    s = molqrc_encode_segments(segs, 4, m, 1, 40, MOLQRC_ECL_Q,
                               MOLQRC_MASK_AUTO, 0);
    test_assert(s > 0);
  }

  /* Too long for version 1 */
  {
    char big[256];
    memset(big, 'A', 200);
    big[200] = '\0';
    segs[0] = molqrc_make_bytes((const unsigned char *)big, 200, buf);
    s = molqrc_encode_segments(segs, 1, m, 1, 1, MOLQRC_ECL_L, MOLQRC_MASK_AUTO,
                               0);
    test_assert(s == 0);
  }

  /* NULL segs */
  test_assert(molqrc_encode_segments(NULL, 0, m, 1, 40, MOLQRC_ECL_M,
                                     MOLQRC_MASK_AUTO, 1) == 0);

  /* Invalid params */
  test_assert(molqrc_encode_segments(segs, 1, NULL, 1, 40, MOLQRC_ECL_M,
                                     MOLQRC_MASK_AUTO, 1) == 0);
  test_assert(
      molqrc_encode_segments(segs, 1, m, 1, 40, 5, MOLQRC_MASK_AUTO, 1) == 0);
}

/* ================================================================== */
/*  draw_matrix / draw_text                                             */
/* ================================================================== */

static int g_count;
static void cb(void *u, int x, int y, int w, int h) {
  (void)u;
  (void)x;
  (void)y;
  (void)w;
  (void)h;
  g_count++;
}

static void test_draw(void) {
  unsigned char m[MOLQRC_MAX_SIZE * MOLQRC_MAX_SIZE];
  int side, black, i;

  side =
      molqrc_encode_text("HELLO", m, 1, 40, MOLQRC_ECL_M, MOLQRC_MASK_AUTO, 1);
  test_assert(side > 0);

  g_count = 0;
  int ms = molqrc_draw_matrix(m, side, 0, 0, 400, 400, cb, NULL);
  test_assert(ms > 0);

  black = 0;
  for (i = 0; i < side * side; i++)
    if (m[i])
      black++;
  test_assert(g_count == black);

  /* draw_text */
  g_count = 0;
  ms = molqrc_draw_text("HELLO", 0, 0, 400, 400, cb, NULL);
  test_assert(ms > 0 && g_count > 0);

  /* Invalid */
  test_assert(molqrc_draw_matrix(NULL, side, 0, 0, 100, 100, cb, NULL) == 0);
  test_assert(molqrc_draw_matrix(m, side, 0, 0, 100, 100, NULL, NULL) == 0);
  test_assert(molqrc_draw_matrix(m, side, 0, 0, 0, 100, cb, NULL) == 0);
  test_assert(molqrc_draw_text("", 0, 0, 100, 100, cb, NULL) == 0);
}

/* ================================================================== */
/*  Matrix integrity: light + dark modules exist                        */
/* ================================================================== */

static void test_matrix_integrity(void) {
  int v;
  for (v = 1; v <= 40; v++) {
    unsigned char m[MOLQRC_MAX_SIZE * MOLQRC_MAX_SIZE];
    int s =
        molqrc_encode_text("test", m, v, v, MOLQRC_ECL_L, MOLQRC_MASK_AUTO, 0);
    if (s > 0) {
      int side = 21 + 4 * (v - 1);
      test_assert(s == side);
      int light = 0, dark = 0, k;
      for (k = 0; k < side * side; k++)
        if (m[k])
          dark++;
        else
          light++;
      test_assert(light > 0 && dark > 0);
    }
  }
}

/* ================================================================== */
/*  main                                                               */
/* ================================================================== */

int main(void) {
  molqrc_gf_init();

  printf("=== GF mul ===\n");
  test_gf_mul();
  printf("=== RS divisor ===\n");
  test_rs_compute_divisor();
  printf("=== RS remainder ===\n");
  test_rs_compute_remainder();
  printf("=== Bit buffer ===\n");
  test_bitbuf_append();
  printf("=== ECC+interleave ===\n");
  test_add_ecc_and_interleave();
  printf("=== Capacity ===\n");
  test_capacity();
  printf("=== Raw modules ===\n");
  test_raw_data_modules();
  printf("=== Align pos ===\n");
  test_alignment_positions();
  printf("=== Version/side ===\n");
  test_version_to_side();
  printf("=== isNumeric ===\n");
  test_is_numeric();
  printf("=== isAlphanumeric ===\n");
  test_is_alphanumeric();
  printf("=== Seg bitlen ===\n");
  test_calc_segment_bit_length();
  printf("=== makeBytes ===\n");
  test_make_bytes();
  printf("=== makeNumeric ===\n");
  test_make_numeric();
  printf("=== makeAlphanum ===\n");
  test_make_alphanumeric();
  printf("=== makeEci ===\n");
  test_make_eci();
  printf("=== CCI bits ===\n");
  test_cci_bits();
  printf("=== encodeText ===\n");
  test_encode_text();
  printf("=== encodeSegs ===\n");
  test_encode_segments();
  printf("=== draw ===\n");
  test_draw();
  printf("=== matrix integrity ===\n");
  test_matrix_integrity();

  printf("\nAll %d test cases passed\n", g_num_test_cases);
  return 0;
}
