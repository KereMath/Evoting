#ifndef SHA512_H
#define SHA512_H

#include <stddef.h>

#define SHA512_DIGEST_LENGTH 64

void SHA512(const unsigned char *data, size_t len, unsigned char *hash);

#endif
