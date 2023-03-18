# Hyperborea

# Protocol specification

## Nodes

| Standard | Cryptographic algorithm | Features |
| - | - | - |
| v1 | [secp256k1](https://crates.io/crates/k256) | IPv4 / IPv6, UDP |

Same applies to `OwnedNode`

### Addresses

Addresses standards are related to nodes' standards: v1 node uses v1 address, and so on

Each node address represents encoded public key of asymmetric cryptography algorithm

| Standard | Cryptographic algorithm | Encoding algorithm | Example |
| - | - | - | - |
| v1 | [secp256k1](https://crates.io/crates/k256) | [DNSSEC base32](https://tools.ietf.org/html/rfc5155)* | `v1:08qn0a0vddm8e1rngljl5cmhsrdnoo8ej3la6m3m3k4v5j5euoh6i` |

> *alphabet: `0123456789abcdefghijklmnopqrstuv`

## Packet

### v1

| Name | Fields | Description |
| - | - | - |
| AuthRequest | u64 | - |
| AuthResponse | u64 | - |
