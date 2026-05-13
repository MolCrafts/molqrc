#ifndef MOLQRC_H
#define MOLQRC_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stddef.h>

/* ------------------------------------------------------------------ */
/*  Constants                                                          */
/* ------------------------------------------------------------------ */

#define MOLQRC_VERSION_MIN 1
#define MOLQRC_VERSION_MAX 40
#define MOLQRC_MAX_SIZE ((MOLQRC_VERSION_MAX - 1) * 4 + 21) /* 177 */

/* ------------------------------------------------------------------ */
/*  Enums                                                              */
/* ------------------------------------------------------------------ */

enum molqrc_Ecc {
  MOLQRC_ECL_L = 0,
  MOLQRC_ECL_M = 1,
  MOLQRC_ECL_Q = 2,
  MOLQRC_ECL_H = 3,
};

enum molqrc_Mode {
  MOLQRC_MODE_NUMERIC = 0x1,
  MOLQRC_MODE_ALPHANUMERIC = 0x2,
  MOLQRC_MODE_BYTE = 0x4,
  MOLQRC_MODE_KANJI = 0x8,
  MOLQRC_MODE_ECI = 0x7,
};

#define MOLQRC_MASK_AUTO (-1)

/* ------------------------------------------------------------------ */
/*  Segment type                                                       */
/* ------------------------------------------------------------------ */

typedef struct {
  int mode;
  int num_chars;
  const unsigned char *data;
  int bit_length;
} molqrc_segment_t;

/* ------------------------------------------------------------------ */
/*  High-level encode                                                  */
/* ------------------------------------------------------------------ */

/* Encode text → QR matrix. Auto-selects version + ECL + mask. */
int molqrc_encode_text(const char *text, unsigned char *out_matrix,
                       int min_version, int max_version, int ecl, int mask,
                       int boost_ecl);

/* Encode segments → QR matrix with full control. */
int molqrc_encode_segments(const molqrc_segment_t segs[], int num_segs,
                           unsigned char *out_matrix, int min_version,
                           int max_version, int ecl, int mask, int boost_ecl);

/* ------------------------------------------------------------------ */
/*  Segment constructors                                               */
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
/*  Render via callback                                                */
/* ------------------------------------------------------------------ */

typedef void (*molqrc_draw_rect_fn)(void *user, int x, int y, int w, int h);

int molqrc_draw_text(const char *text, int left, int top, int width, int height,
                     molqrc_draw_rect_fn draw_rect, void *user);

int molqrc_draw_matrix(const unsigned char *matrix, int side, int left, int top,
                       int width, int height, molqrc_draw_rect_fn draw_rect,
                       void *user);

#ifdef __cplusplus
}
#endif

#endif
