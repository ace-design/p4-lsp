// This P4 file contains only a preamble, it is not meant to be used on its own.

#ifndef _COMMON_CONFIGP4
#define _COMMON_CONFIGP4

#undef TARGET_PSA
#undef TARGET_V1

#include <core.p4>

#ifdef TARGET_TOFINO
 #if TARGET_TOFINO == 2
  #include <t2na.p4>
 #else
  #include <tna.p4>
 #endif
#else // x86: might be PSA or v1 model, select here
// #define USE_PSA 1
 #ifdef USE_PSA
  #include <psa.p4>
  #define TARGET_PSA 1
 #else
  #include <v1model.p4>
  #define TARGET_V1 1
 #endif
#endif

#include "common/headers.p4"
#include "common/util.p4"

#ifdef TARGET_V1
struct mac_learn_digest {
    bit<48>  src_addr;
    PortId_t ingress_port;
}

struct arp_digest {
    bit<32> ip;                // destination (or nexthop)
    bit<48> mac;        // own MAC address to be used as SHA in ARP request
}
#else
struct mac_learn_digest_data {
    bit<48>  src_addr;
    PortId_t ingress_port;
}

struct arp_digest_data {
    bit<32> ip;                // destination (or nexthop)
    bit<48> mac;        // own MAC address to be used as SHA in ARP request
}

#endif

struct ppv_digest_t {
    bit<32> vql4s;
    bit<32> vqcl;
    bit<48> ts;
}

#endif // COMMON_CONFIG