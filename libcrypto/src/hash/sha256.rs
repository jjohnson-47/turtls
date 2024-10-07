//! A software implementation of SHA-256.

use super::{BlockHasher, Hasher};

/// The first 32 bits of the fractional parts of
/// the cube roots of the first 64 prime numbers
// TODO: name this something useful
const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

/// A one-time helper function used by `update_hash()`
// TODO: name this something useful
#[inline]
const fn ch(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ (!x & z)
}

/// A one-time helper function used by `update_hash()`
// TODO: name this something useful
#[inline]
const fn maj(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ (x & z) ^ (y & z)
}

/// A one-time helper function used by `update_hash()`
// TODO: name this something useful
#[inline]
const fn sigma_0(x: u32) -> u32 {
    x.rotate_right(2) ^ x.rotate_right(13) ^ x.rotate_right(22)
}

/// A one-time helper function used by `update_hash()`
// TODO: name this something useful
#[inline]
const fn sigma_1(x: u32) -> u32 {
    x.rotate_right(6) ^ x.rotate_right(11) ^ x.rotate_right(25)
}

/// A one-time helper function used by `update_hash()`
// TODO: name this something useful
#[inline]
const fn little_sigma_0(x: u32) -> u32 {
    x.rotate_right(7) ^ x.rotate_right(18) ^ x >> 3
}

/// A one-time helper function used by `update_hash()`
// TODO: name this something useful
#[inline]
const fn little_sigma_1(x: u32) -> u32 {
    x.rotate_right(17) ^ x.rotate_right(19) ^ x >> 10
}

pub struct Sha256 {
    state: [u32; Self::HASH_SIZE / size_of::<u32>()],
    len: u64,
}

impl Sha256 {
    pub const HASH_SIZE: usize = 32;
    pub const BLOCK_SIZE: usize = 64;
    fn update_countless(&mut self, block: &[u8; Self::BLOCK_SIZE]) {
        let block = {
            // TODO: consider using uninitialized array
            let mut as_u32 = [0u32; Sha256::BLOCK_SIZE / 4];
            // TODO: use `array_chunks` once stabilized
            for (int, chunk) in as_u32.iter_mut().zip(block.chunks_exact(4)) {
                *int = u32::from_be_bytes(chunk.try_into().unwrap());
            }
            as_u32
        };

        let mut message_schedule = [0; 64];
        message_schedule[..block.len()].copy_from_slice(&block);

        for i in 16..message_schedule.len() {
            message_schedule[i] = little_sigma_1(message_schedule[i - 2])
                .wrapping_add(message_schedule[i - 7])
                .wrapping_add(little_sigma_0(message_schedule[i - 15]))
                .wrapping_add(message_schedule[i - 16]);
        }

        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = self.state;

        for i in 0..64 {
            let temp1 = h
                .wrapping_add(sigma_1(e))
                .wrapping_add(ch(e, f, g))
                .wrapping_add(K[i])
                .wrapping_add(message_schedule[i]);
            let temp2 = sigma_0(a).wrapping_add(maj(a, b, c));
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }
        self.state[0] = self.state[0].wrapping_add(a);
        self.state[1] = self.state[1].wrapping_add(b);
        self.state[2] = self.state[2].wrapping_add(c);
        self.state[3] = self.state[3].wrapping_add(d);
        self.state[4] = self.state[4].wrapping_add(e);
        self.state[5] = self.state[5].wrapping_add(f);
        self.state[6] = self.state[6].wrapping_add(g);
        self.state[7] = self.state[7].wrapping_add(h);
    }
}

impl Hasher<{ Sha256::HASH_SIZE }> for Sha256 {
    fn new() -> Self {
        Self {
            state: [
                0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
                0x5be0cd19,
            ],
            len: 0,
        }
    }
    fn finish_with(mut self, msg: &[u8]) -> [u8; Self::HASH_SIZE] {
        // TODO: use `array_chunks` once stabilized
        let blocks = msg.chunks_exact(Self::BLOCK_SIZE);
        let remainder = blocks.remainder();

        for block in blocks {
            self.update_countless(block.try_into().unwrap());
        }

        let mut last_block = [0; Self::BLOCK_SIZE];
        // we can safely write here because the excess must be less than `BLOCK_SIZE`
        last_block[..remainder.len()].copy_from_slice(remainder);

        last_block[remainder.len()] = 0x80;

        // does the length info fit without adding an extra block?
        if remainder.len() < Self::BLOCK_SIZE - size_of::<u64>() {
            last_block[Self::BLOCK_SIZE - size_of::<u64>()..]
                .copy_from_slice(&((msg.len() as u64 + self.len) * 8).to_be_bytes());
        } else {
            self.update_countless(&last_block);
            last_block = [0; Self::BLOCK_SIZE];
            last_block[Self::BLOCK_SIZE - size_of::<u64>()..]
                .copy_from_slice(&((msg.len() as u64 + self.len) * 8).to_be_bytes());
        }

        self.update(&last_block);
        u32_array_to_bytes(&self.state)
    }

    fn hash(msg: &[u8]) -> [u8; Sha256::HASH_SIZE] {
        let hasher = Self::new();
        hasher.finish_with(msg)
    }

    fn finish(mut self) -> [u8; Self::HASH_SIZE] {
        let mut padding = [0; Self::BLOCK_SIZE];
        padding[0] = 0x80;
        padding[Sha256::BLOCK_SIZE - size_of::<u64>()..]
            .copy_from_slice(&(self.len * 8).to_be_bytes());
        self.update_countless(&padding);
        u32_array_to_bytes(&self.state)
    }
}

impl BlockHasher<{ Self::HASH_SIZE }, { Self::BLOCK_SIZE }> for Sha256 {
    fn update(&mut self, block: &[u8; Self::BLOCK_SIZE]) {
        self.update_countless(block);
        self.len += Self::BLOCK_SIZE as u64;
    }
}

fn u32_array_to_bytes(
    array: &[u32; Sha256::HASH_SIZE / size_of::<u32>()],
) -> [u8; Sha256::HASH_SIZE] {
    // TODO: consider using uninitialized array
    let mut as_bytes = [0u8; Sha256::HASH_SIZE];
    // TODO: use `array_chunks` once stabilized
    for (chunk, int) in as_bytes.chunks_exact_mut(size_of::<u32>()).zip(array) {
        chunk.copy_from_slice(&int.to_be_bytes())
    }
    as_bytes
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hash() {
        let msg = b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";
        let digest: [u8; Sha256::HASH_SIZE] = [
            0x24, 0x8D, 0x6A, 0x61, 0xD2, 0x06, 0x38, 0xB8, 0xE5, 0xC0, 0x26, 0x93, 0x0C, 0x3E,
            0x60, 0x39, 0xA3, 0x3C, 0xE4, 0x59, 0x64, 0xFF, 0x21, 0x67, 0xF6, 0xEC, 0xED, 0xD4,
            0x19, 0xDB, 0x06, 0xC1,
        ];
        assert_eq!(Sha256::hash(msg), digest);
        let msg = [
            0x45, 0x11, 0x01, 0x25, 0x0e, 0xc6, 0xf2, 0x66, 0x52, 0x24, 0x9d, 0x59, 0xdc, 0x97,
            0x4b, 0x73, 0x61, 0xd5, 0x71, 0xa8, 0x10, 0x1c, 0xdf, 0xd3, 0x6a, 0xba, 0x3b, 0x58,
            0x54, 0xd3, 0xae, 0x08, 0x6b, 0x5f, 0xdd, 0x45, 0x97, 0x72, 0x1b, 0x66, 0xe3, 0xc0,
            0xdc, 0x5d, 0x8c, 0x60, 0x6d, 0x96, 0x57, 0xd0, 0xe3, 0x23, 0x28, 0x3a, 0x52, 0x17,
            0xd1, 0xf5, 0x3f, 0x2f, 0x28, 0x4f, 0x57, 0xb8, 0x5c, 0x8a, 0x61, 0xac, 0x89, 0x24,
            0x71, 0x1f, 0x89, 0x5c, 0x5e, 0xd9, 0x0e, 0xf1, 0x77, 0x45, 0xed, 0x2d, 0x72, 0x8a,
            0xbd, 0x22, 0xa5, 0xf7, 0xa1, 0x34, 0x79, 0xa4, 0x62, 0xd7, 0x1b, 0x56, 0xc1, 0x9a,
            0x74, 0xa4, 0x0b, 0x65, 0x5c, 0x58, 0xed, 0xfe, 0x0a, 0x18, 0x8a, 0xd2, 0xcf, 0x46,
            0xcb, 0xf3, 0x05, 0x24, 0xf6, 0x5d, 0x42, 0x3c, 0x83, 0x7d, 0xd1, 0xff, 0x2b, 0xf4,
            0x62, 0xac, 0x41, 0x98, 0x00, 0x73, 0x45, 0xbb, 0x44, 0xdb, 0xb7, 0xb1, 0xc8, 0x61,
            0x29, 0x8c, 0xdf, 0x61, 0x98, 0x2a, 0x83, 0x3a, 0xfc, 0x72, 0x8f, 0xae, 0x1e, 0xda,
            0x2f, 0x87, 0xaa, 0x2c, 0x94, 0x80, 0x85, 0x8b, 0xec,
        ];
        let digest = [
            0x3c, 0x59, 0x3a, 0xa5, 0x39, 0xfd, 0xcd, 0xae, 0x51, 0x6c, 0xdf, 0x2f, 0x15, 0x00,
            0x0f, 0x66, 0x34, 0x18, 0x5c, 0x88, 0xf5, 0x05, 0xb3, 0x97, 0x75, 0xfb, 0x9a, 0xb1,
            0x37, 0xa1, 0x0a, 0xa2,
        ];
        assert_eq!(Sha256::hash(&msg), digest);
        let msg = [
            0x6b, 0x91, 0x8f, 0xb1, 0xa5, 0xad, 0x1f, 0x9c, 0x5e, 0x5d, 0xbd, 0xf1, 0x0a, 0x93,
            0xa9, 0xc8, 0xf6, 0xbc, 0xa8, 0x9f, 0x37, 0xe7, 0x9c, 0x9f, 0xe1, 0x2a, 0x57, 0x22,
            0x79, 0x41, 0xb1, 0x73, 0xac, 0x79, 0xd8, 0xd4, 0x40, 0xcd, 0xe8, 0xc6, 0x4c, 0x4e,
            0xbc, 0x84, 0xa4, 0xc8, 0x03, 0xd1, 0x98, 0xa2, 0x96, 0xf3, 0xde, 0x06, 0x09, 0x00,
            0xcc, 0x42, 0x7f, 0x58, 0xca, 0x6e, 0xc3, 0x73, 0x08, 0x4f, 0x95, 0xdd, 0x6c, 0x7c,
            0x42, 0x7e, 0xcf, 0xbf, 0x78, 0x1f, 0x68, 0xbe, 0x57, 0x2a, 0x88, 0xdb, 0xcb, 0xb1,
            0x88, 0x58, 0x1a, 0xb2, 0x00, 0xbf, 0xb9, 0x9a, 0x3a, 0x81, 0x64, 0x07, 0xe7, 0xdd,
            0x6d, 0xd2, 0x10, 0x03, 0x55, 0x4d, 0x4f, 0x7a, 0x99, 0xc9, 0x3e, 0xbf, 0xce, 0x5c,
            0x30, 0x2f, 0xf0, 0xe1, 0x1f, 0x26, 0xf8, 0x3f, 0xe6, 0x69, 0xac, 0xef, 0xb0, 0xc1,
            0xbb, 0xb8, 0xb1, 0xe9, 0x09, 0xbd, 0x14, 0xaa, 0x48, 0xba, 0x34, 0x45, 0xc8, 0x8b,
            0x0e, 0x11, 0x90, 0xee, 0xf7, 0x65, 0xad, 0x89, 0x8a, 0xb8, 0xca, 0x2f, 0xe5, 0x07,
            0x01, 0x5f, 0x15, 0x78, 0xf1, 0x0d, 0xce, 0x3c, 0x11, 0xa5, 0x5f, 0xb9, 0x43, 0x4e,
            0xe6, 0xe9, 0xad, 0x6c, 0xc0, 0xfd, 0xc4, 0x68, 0x44, 0x47, 0xa9, 0xb3, 0xb1, 0x56,
            0xb9, 0x08, 0x64, 0x63, 0x60, 0xf2, 0x4f, 0xec, 0x2d, 0x8f, 0xa6, 0x9e, 0x2c, 0x93,
            0xdb, 0x78, 0x70, 0x8f, 0xcd, 0x2e, 0xef, 0x74, 0x3d, 0xcb, 0x93, 0x53, 0x81, 0x9b,
            0x8d, 0x66, 0x7c, 0x48, 0xed, 0x54, 0xcd, 0x43, 0x6f, 0xb1, 0x47, 0x65, 0x98, 0xc4,
            0xa1, 0xd7, 0x02, 0x8e, 0x6f, 0x2f, 0xf5, 0x07, 0x51, 0xdb, 0x36, 0xab, 0x6b, 0xc3,
            0x24, 0x35, 0x15, 0x2a, 0x00, 0xab, 0xd3, 0xd5, 0x8d, 0x9a, 0x87, 0x70, 0xd9, 0xa3,
            0xe5, 0x2d, 0x5a, 0x36, 0x28, 0xae, 0x3c, 0x9e, 0x03, 0x25,
        ];
        let digest = [
            0x46, 0x50, 0x0b, 0x6a, 0xe1, 0xab, 0x40, 0xbd, 0xe0, 0x97, 0xef, 0x16, 0x8b, 0x0f,
            0x31, 0x99, 0x04, 0x9b, 0x55, 0x54, 0x5a, 0x15, 0x88, 0x79, 0x2d, 0x39, 0xd5, 0x94,
            0xf4, 0x93, 0xdc, 0xa7,
        ];
        assert_eq!(Sha256::hash(&msg), digest);
    }

    #[test]
    fn update() {
        let msg = [
            0x6b, 0x91, 0x8f, 0xb1, 0xa5, 0xad, 0x1f, 0x9c, 0x5e, 0x5d, 0xbd, 0xf1, 0x0a, 0x93,
            0xa9, 0xc8, 0xf6, 0xbc, 0xa8, 0x9f, 0x37, 0xe7, 0x9c, 0x9f, 0xe1, 0x2a, 0x57, 0x22,
            0x79, 0x41, 0xb1, 0x73, 0xac, 0x79, 0xd8, 0xd4, 0x40, 0xcd, 0xe8, 0xc6, 0x4c, 0x4e,
            0xbc, 0x84, 0xa4, 0xc8, 0x03, 0xd1, 0x98, 0xa2, 0x96, 0xf3, 0xde, 0x06, 0x09, 0x00,
            0xcc, 0x42, 0x7f, 0x58, 0xca, 0x6e, 0xc3, 0x73, 0x08, 0x4f, 0x95, 0xdd, 0x6c, 0x7c,
            0x42, 0x7e, 0xcf, 0xbf, 0x78, 0x1f, 0x68, 0xbe, 0x57, 0x2a, 0x88, 0xdb, 0xcb, 0xb1,
            0x88, 0x58, 0x1a, 0xb2, 0x00, 0xbf, 0xb9, 0x9a, 0x3a, 0x81, 0x64, 0x07, 0xe7, 0xdd,
            0x6d, 0xd2, 0x10, 0x03, 0x55, 0x4d, 0x4f, 0x7a, 0x99, 0xc9, 0x3e, 0xbf, 0xce, 0x5c,
            0x30, 0x2f, 0xf0, 0xe1, 0x1f, 0x26, 0xf8, 0x3f, 0xe6, 0x69, 0xac, 0xef, 0xb0, 0xc1,
            0xbb, 0xb8, 0xb1, 0xe9, 0x09, 0xbd, 0x14, 0xaa, 0x48, 0xba, 0x34, 0x45, 0xc8, 0x8b,
            0x0e, 0x11, 0x90, 0xee, 0xf7, 0x65, 0xad, 0x89, 0x8a, 0xb8, 0xca, 0x2f, 0xe5, 0x07,
            0x01, 0x5f, 0x15, 0x78, 0xf1, 0x0d, 0xce, 0x3c, 0x11, 0xa5, 0x5f, 0xb9, 0x43, 0x4e,
            0xe6, 0xe9, 0xad, 0x6c, 0xc0, 0xfd, 0xc4, 0x68, 0x44, 0x47, 0xa9, 0xb3, 0xb1, 0x56,
            0xb9, 0x08, 0x64, 0x63, 0x60, 0xf2, 0x4f, 0xec, 0x2d, 0x8f, 0xa6, 0x9e, 0x2c, 0x93,
            0xdb, 0x78, 0x70, 0x8f, 0xcd, 0x2e, 0xef, 0x74, 0x3d, 0xcb, 0x93, 0x53, 0x81, 0x9b,
            0x8d, 0x66, 0x7c, 0x48, 0xed, 0x54, 0xcd, 0x43, 0x6f, 0xb1, 0x47, 0x65, 0x98, 0xc4,
            0xa1, 0xd7, 0x02, 0x8e, 0x6f, 0x2f, 0xf5, 0x07, 0x51, 0xdb, 0x36, 0xab, 0x6b, 0xc3,
            0x24, 0x35, 0x15, 0x2a, 0x00, 0xab, 0xd3, 0xd5, 0x8d, 0x9a, 0x87, 0x70, 0xd9, 0xa3,
            0xe5, 0x2d, 0x5a, 0x36, 0x28, 0xae, 0x3c, 0x9e, 0x03, 0x25,
        ];
        let digest = [
            0x46, 0x50, 0x0b, 0x6a, 0xe1, 0xab, 0x40, 0xbd, 0xe0, 0x97, 0xef, 0x16, 0x8b, 0x0f,
            0x31, 0x99, 0x04, 0x9b, 0x55, 0x54, 0x5a, 0x15, 0x88, 0x79, 0x2d, 0x39, 0xd5, 0x94,
            0xf4, 0x93, 0xdc, 0xa7,
        ];
        let mut hasher = Sha256::new();

        let blocks = msg.chunks_exact(Sha256::BLOCK_SIZE);
        let remainder = blocks.remainder();

        for block in blocks {
            hasher.update(block.try_into().unwrap());
        }
        assert_eq!(hasher.finish_with(remainder), digest);
    }
}