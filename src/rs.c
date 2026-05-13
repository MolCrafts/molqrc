#include "internal.h"

/* ------------------------------------------------------------------ */
/*  GF(256) tables                                                     */
/* ------------------------------------------------------------------ */

unsigned char molqrc_gf_exp[512];
unsigned char molqrc_gf_log[256];

void molqrc_gf_init(void) {
  int i, x;

  x = 1;
  for (i = 0; i < 255; i++) {
    molqrc_gf_exp[i] = (unsigned char)x;
    molqrc_gf_log[x] = (unsigned char)i;
    x <<= 1;
    if (x & 0x100)
      x ^= 0x11d; /* primitive polynomial x^8 + x^4 + x^3 + x^2 + 1 */
  }
  /* Duplicate exp table for [255..510] to simplify multiplication */
  for (i = 255; i < 511; i++)
    molqrc_gf_exp[i] = molqrc_gf_exp[i - 255];

  /* log[0] is undefined; set to 0 */
  molqrc_gf_log[0] = 0;
}

/* ------------------------------------------------------------------ */
/*  RS generator polynomial                                             */
/* ------------------------------------------------------------------ */

static void rs_generator(int nec, unsigned char *gen) {
  int i, j;

  gen[0] = 1;
  for (i = 1; i <= nec; i++) {
    gen[i] = 0;
  }

  for (i = 0; i < nec; i++) {
    for (j = nec; j > 0; j--) {
      gen[j] = gen[j - 1] ^ molqrc_gf_mul(gen[j], molqrc_gf_exp[i]);
    }
    gen[0] = molqrc_gf_mul(gen[0], molqrc_gf_exp[i]);
  }
}

/* ------------------------------------------------------------------ */
/*  RS encode: data → data + EC bytes                                   */
/* ------------------------------------------------------------------ */

void molqrc_rs_encode(unsigned char *data, int ndata, int nec) {
  int i, j;
  unsigned char gen[256];
  unsigned char remainder[256];

  if (nec > 255)
    nec = 255;

  rs_generator(nec, gen);

  for (i = 0; i < nec; i++)
    remainder[i] = 0;

  for (i = 0; i < ndata; i++) {
    unsigned char feedback = data[i] ^ remainder[0];
    if (feedback != 0) {
      for (j = 0; j < nec - 1; j++)
        remainder[j] =
            remainder[j + 1] ^ molqrc_gf_mul(feedback, gen[nec - 1 - j]);
      remainder[nec - 1] = molqrc_gf_mul(feedback, gen[0]);
    } else {
      for (j = 0; j < nec - 1; j++)
        remainder[j] = remainder[j + 1];
      remainder[nec - 1] = 0;
    }
  }

  for (i = 0; i < nec; i++)
    data[ndata + i] = remainder[i];
}

/* ------------------------------------------------------------------ */
/*  RS divisor polynomial (Nayuki-compatible: degree coefficients,     */
/*  descending order, implicit x^degree = 1)                            */
/* ------------------------------------------------------------------ */

void molqrc_rs_compute_divisor(int degree, unsigned char *out) {
  int i, j;
  unsigned char full[256]; /* degree+1 coefficients, ascending */

  full[0] = 1;
  for (i = 1; i <= degree; i++)
    full[i] = 0;

  for (i = 0; i < degree; i++) {
    for (j = degree; j > 0; j--)
      full[j] = full[j - 1] ^ molqrc_gf_mul(full[j], molqrc_gf_exp[i]);
    full[0] = molqrc_gf_mul(full[0], molqrc_gf_exp[i]);
  }

  /* Return degree coefficients in descending order:
     out[0] = coef of x^(degree-1), ..., out[degree-1] = coef of x^0.
     The leading coefficient (x^degree = 1) is implicit. */
  for (i = 0; i < degree; i++)
    out[i] = full[degree - 1 - i];
}

/* ------------------------------------------------------------------ */
/*  RS remainder (Nayuki-compatible: generator is degree coefficients  */
/*  in descending order, implicit leading 1)                           */
/* ------------------------------------------------------------------ */

void molqrc_rs_compute_remainder(const unsigned char *data, int data_len,
                                 const unsigned char *generator, int gen_degree,
                                 unsigned char *out) {
  int i, j;

  for (i = 0; i < gen_degree; i++)
    out[i] = 0;

  for (i = 0; i < data_len; i++) {
    unsigned char factor = data[i] ^ out[0];

    /* Shift remainder left by one */
    for (j = 0; j < gen_degree - 1; j++)
      out[j] = out[j + 1];
    out[gen_degree - 1] = 0;

    /* XOR generator * factor */
    if (factor != 0) {
      for (j = 0; j < gen_degree; j++)
        out[j] ^= molqrc_gf_mul(generator[j], factor);
    }
  }
}
