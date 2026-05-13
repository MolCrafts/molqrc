#include "internal.h"
#include <string.h>

#define ALPHANUMERIC_CHARSET "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ $%*+-./:"

/* Helper: write `nbits` of `value` to `buf` at current bit position */
static void write_bits(unsigned char *buf, int *bit_pos, unsigned int value, int nbits)
{
    int b;
    for (b = nbits - 1; b >= 0; b--) {
        int bytepos = *bit_pos / 8;
        int bitpos  = 7 - (*bit_pos % 8);
        if (value & (1u << b))
            buf[bytepos] |= (unsigned char)(1 << bitpos);
        (*bit_pos)++;
    }
}

/* ------------------------------------------------------------------ */
/*  Validation                                                         */
/* ------------------------------------------------------------------ */

int molqrc_is_numeric(const char *text)
{
    if (!text) return 0;
    for (; *text; text++) {
        if (*text < '0' || *text > '9') return 0;
    }
    return 1;
}

int molqrc_is_alphanumeric(const char *text)
{
    if (!text) return 0;
    for (; *text; text++) {
        if (!strchr(ALPHANUMERIC_CHARSET, *text)) return 0;
    }
    return 1;
}

/* ------------------------------------------------------------------ */
/*  Bit-length calculation                                             */
/* ------------------------------------------------------------------ */

int molqrc_calc_segment_bit_length(int mode, int num_chars)
{
    long result;
    long nc = num_chars;

    if (nc < 0 || nc > 32767) return -1;

    switch (mode) {
    case MOLQRC_MODE_NUMERIC:
        result = (nc * 10 + 2) / 3;   /* ceil(nc * 10 / 3) */
        break;
    case MOLQRC_MODE_ALPHANUMERIC:
        result = (nc * 11 + 1) / 2;   /* ceil(nc * 11 / 2) */
        break;
    case MOLQRC_MODE_BYTE:
        result = nc * 8;
        break;
    case MOLQRC_MODE_KANJI:
        result = nc * 13;
        break;
    case MOLQRC_MODE_ECI:
        result = 3 * 8;
        break;
    default:
        return -1;
    }

    if (result > 32767) return -1;
    return (int)result;
}

/* ------------------------------------------------------------------ */
/*  Constructors                                                       */
/* ------------------------------------------------------------------ */

molqrc_segment_t molqrc_make_bytes(const unsigned char *data, int len,
                                    unsigned char *buf)
{
    molqrc_segment_t seg;
    int bit_len;

    seg.mode      = MOLQRC_MODE_BYTE;
    seg.num_chars = len;
    bit_len       = (len == 0) ? 0 : len * 8;
    seg.bit_length = bit_len;

    if (bit_len > 0)
        memcpy(buf, data, (size_t)len);
    seg.data = buf;
    return seg;
}

molqrc_segment_t molqrc_make_numeric(const char *digits, unsigned char *buf)
{
    molqrc_segment_t seg;
    int len, bit_len, i;
    unsigned int accum;
    int accum_count;
    int bit_pos;

    len = (int)strlen(digits);
    seg.mode      = MOLQRC_MODE_NUMERIC;
    seg.num_chars = len;
    bit_len       = molqrc_calc_segment_bit_length(MOLQRC_MODE_NUMERIC, len);
    seg.bit_length = bit_len;

    if (bit_len == 0) {
        seg.data = buf;
        return seg;
    }

    memset(buf, 0, (size_t)((bit_len + 7) / 8));
    accum = 0;
    accum_count = 0;
    bit_pos = 0;

    for (i = 0; i < len; i++) {
        accum = accum * 10 + (unsigned int)(digits[i] - '0');
        accum_count++;
        if (accum_count == 3) {
            write_bits(buf, &bit_pos, accum, 10);
            accum = 0;
            accum_count = 0;
        }
    }

    if (accum_count > 0) {
        int nbits = accum_count * 3 + 1;  /* 1 digit→4, 2 digits→7 */
        write_bits(buf, &bit_pos, accum, nbits);
    }

    seg.data = buf;
    return seg;
}

molqrc_segment_t molqrc_make_alphanumeric(const char *text, unsigned char *buf)
{
    molqrc_segment_t seg;
    int len, bit_len, i;
    unsigned int accum;
    int accum_count;
    int bit_pos;

    len = (int)strlen(text);
    seg.mode      = MOLQRC_MODE_ALPHANUMERIC;
    seg.num_chars = len;
    bit_len       = molqrc_calc_segment_bit_length(MOLQRC_MODE_ALPHANUMERIC, len);
    seg.bit_length = bit_len;

    if (bit_len == 0) {
        seg.data = buf;
        return seg;
    }

    memset(buf, 0, (size_t)((bit_len + 7) / 8));
    accum = 0;
    accum_count = 0;
    bit_pos = 0;

    for (i = 0; i < len; i++) {
        const char *p = strchr(ALPHANUMERIC_CHARSET, text[i]);
        accum = accum * 45 + (unsigned int)(p - ALPHANUMERIC_CHARSET);
        accum_count++;
        if (accum_count == 2) {
            write_bits(buf, &bit_pos, accum, 11);
            accum = 0;
            accum_count = 0;
        }
    }

    if (accum_count == 1) {
        write_bits(buf, &bit_pos, accum, 6);
    }

    seg.data = buf;
    return seg;
}

molqrc_segment_t molqrc_make_eci(long assign_val, unsigned char *buf)
{
    molqrc_segment_t seg;
    int bit_pos = 0;

    seg.mode      = MOLQRC_MODE_ECI;
    seg.num_chars = 0;
    seg.bit_length = 3 * 8;
    memset(buf, 0, 3);

    if (assign_val < (1L << 7)) {
        /* 8 bits: value as-is */
        write_bits(buf, &bit_pos, (unsigned int)assign_val, 8);
    } else if (assign_val < (1L << 14)) {
        /* 16 bits: 2-bit header (10) + 14-bit value */
        write_bits(buf, &bit_pos, 2, 2);
        write_bits(buf, &bit_pos, (unsigned int)assign_val, 14);
    } else {
        /* 24 bits: 3-bit header (110) + 21-bit value */
        write_bits(buf, &bit_pos, 6, 3);
        write_bits(buf, &bit_pos, (unsigned int)assign_val, 21);
    }

    seg.data = buf;
    return seg;
}
