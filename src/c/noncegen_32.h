#pragma once

#include <stdint.h>
#include <stdlib.h>

void init_shabal();
void noncegen(char *cache, const size_t cache_size, const size_t chunk_offset,
                   const uint64_t numeric_id, const uint64_t local_startnonce,
                   const uint64_t local_nonces);

void find_best_deadline_sph(char* scoops, uint64_t nonce_count, char* gensig,
                             uint64_t* best_deadline, uint64_t* best_offset);
