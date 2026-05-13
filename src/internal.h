#ifndef MOLQRC_INTERNAL_H
#define MOLQRC_INTERNAL_H

#include "molqrc.h"
#include <stddef.h>

/* ------------------------------------------------------------------ */
/*  QR Code constants (mirrors public header)                          */
/* ------------------------------------------------------------------ */

#define MOLQRC_MAX_SIZE ((MOLQRC_VERSION_MAX - 1) * 4 + 21) /* 177 */

#define MOLQRC_ECL_L 0
#define MOLQRC_ECL_M 1
#define MOLQRC_ECL_Q 2
#define MOLQRC_ECL_H 3

/* MOLQRC_MODE_* and molqrc_segment_t are defined in molqrc.h */

/* ------------------------------------------------------------------ */
/*  GF(256) tables                                                     */
/* ------------------------------------------------------------------ */

extern unsigned char molqrc_gf_exp[512];
extern unsigned char molqrc_gf_log[256];

void molqrc_gf_init(void);

static inline unsigned char molqrc_gf_mul(unsigned char a, unsigned char b) {
  if (a == 0 || b == 0)
    return 0;
  return molqrc_gf_exp[molqrc_gf_log[a] + molqrc_gf_log[b]];
}

/* ------------------------------------------------------------------ */
/*  Version capacity tables                                            */
/* ------------------------------------------------------------------ */

int molqrc_capacity(int version, int ecl);
int molqrc_ec_codewords_per_block(int version, int ecl);
void molqrc_block_info(int version, int ecl, int *g1_blocks, int *g1_codewords,
                       int *g2_blocks, int *g2_codewords);
int molqrc_remainder_bits(int version);
int molqrc_alignment_positions(int version, int *positions);
int molqrc_raw_data_modules(int version);

/* ------------------------------------------------------------------ */
/*  Reed-Solomon                                                       */
/* ------------------------------------------------------------------ */

void molqrc_rs_encode(unsigned char *data_in_out, int ndata, int nec);
void molqrc_rs_compute_divisor(int degree, unsigned char *out);
void molqrc_rs_compute_remainder(const unsigned char *data, int data_len,
                                 const unsigned char *generator, int gen_degree,
                                 unsigned char *out);

/* ------------------------------------------------------------------ */
/*  Bit buffer                                                         */
/* ------------------------------------------------------------------ */

typedef struct {
  unsigned char *buf;
  int bitlen;
  int capacity;
} molqrc_bitbuf_t;

void molqrc_bitbuf_init(molqrc_bitbuf_t *bb, unsigned char *buf, int capacity);
void molqrc_bitbuf_append(molqrc_bitbuf_t *bb, int value, int nbits);

/* ------------------------------------------------------------------ */
/*  Segment construction                                               */
/* ------------------------------------------------------------------ */

int molqrc_is_numeric(const char *text);
int molqrc_is_alphanumeric(const char *text);
int molqrc_calc_segment_bit_length(int mode, int num_chars);

molqrc_segment_t molqrc_make_bytes(const unsigned char *data, int len,
                                   unsigned char *buf);
molqrc_segment_t molqrc_make_numeric(const char *digits, unsigned char *buf);
molqrc_segment_t molqrc_make_alphanumeric(const char *text, unsigned char *buf);
molqrc_segment_t molqrc_make_eci(long assign_val, unsigned char *buf);

/* ------------------------------------------------------------------ */
/*  CCI bits per (mode, version)                                       */
/* ------------------------------------------------------------------ */

int molqrc_cci_bits(int mode, int version);

/* ------------------------------------------------------------------ */
/*  Encoding: segments → codewords                                     */
/* ------------------------------------------------------------------ */

int molqrc_encode_segments_to_codewords(const molqrc_segment_t *segs,
                                        int num_segs, unsigned char *codewords,
                                        int max_cw, int version, int ecl);

/* ------------------------------------------------------------------ */
/*  Matrix builder                                                     */
/* ------------------------------------------------------------------ */

int molqrc_build_matrix(unsigned char *matrix, int side,
                        const unsigned char *interleaved, int total_cw,
                        int version, int ecl, int mask);

/* ------------------------------------------------------------------ */
/*  Interleaving                                                       */
/* ------------------------------------------------------------------ */

void molqrc_interleave(const unsigned char *data, int *ec_blocks,
                       unsigned char *out, int version, int ecl);

#endif
