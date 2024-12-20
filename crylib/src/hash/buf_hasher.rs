use super::{BlockHasher, Hasher};

#[derive(Clone)]
pub struct BufHasher<const H_LEN: usize, const B_LEN: usize, H>
where
    H: BlockHasher<H_LEN, B_LEN>,
{
    hasher: H,
    buf: [u8; B_LEN],
    len: usize,
}

impl<const H_LEN: usize, const B_LEN: usize, H> BufHasher<H_LEN, B_LEN, H>
where
    H: BlockHasher<H_LEN, B_LEN>,
{
    pub fn update_with(&mut self, msg: &[u8]) {
        if msg.len() <= B_LEN - self.len {
            self.buf[self.len..][..msg.len()].copy_from_slice(msg);
            self.len += msg.len();
            return;
        }

        self.buf[self.len..].copy_from_slice(&msg[..B_LEN - self.len]);
        self.hasher.update(&self.buf);

        // TODO: use array_chunks once stabilized
        let blocks = msg[B_LEN - self.len..].chunks_exact(B_LEN);
        let remainder = blocks.remainder();

        for block in blocks {
            self.hasher.update(block.try_into().unwrap());
        }
        self.buf[..remainder.len()].copy_from_slice(remainder);
        self.len = remainder.len();
    }
}

impl<const H_LEN: usize, const B_LEN: usize, H> Hasher<H_LEN> for BufHasher<H_LEN, B_LEN, H>
where
    H: BlockHasher<H_LEN, B_LEN>,
{
    fn new() -> Self {
        Self {
            hasher: H::new(),
            buf: [0; B_LEN],
            len: 0,
        }
    }

    fn finish(self) -> [u8; H_LEN] {
        self.hasher.finish_with(&self.buf[..self.len])
    }

    fn hash(msg: &[u8]) -> [u8; H_LEN] {
        <H as Hasher<H_LEN>>::hash(msg)
    }

    fn finish_with(mut self, msg: &[u8]) -> [u8; H_LEN] {
        self.update_with(msg);
        self.finish()
    }
}

impl<const H_LEN: usize, const B_LEN: usize, H> BlockHasher<H_LEN, B_LEN>
    for BufHasher<H_LEN, B_LEN, H>
where
    H: BlockHasher<H_LEN, B_LEN>,
{
    fn update(&mut self, block: &[u8; B_LEN]) {
        self.buf[self.len..].copy_from_slice(&block[..B_LEN - self.len]);
        self.hasher.update(&self.buf);

        self.buf[..self.len].copy_from_slice(&block[B_LEN - self.len..]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hash::Sha256;

    #[test]
    fn hash() {
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
    fn update_with() {
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

        let mut hasher = BufHasher::<{ Sha256::HASH_SIZE }, { Sha256::BLOCK_SIZE }, Sha256>::new();
        for block in msg.chunks(11) {
            hasher.update_with(block);
        }
        assert_eq!(hasher.finish(), digest);
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
        let mut hasher = BufHasher::<{ Sha256::HASH_SIZE }, { Sha256::BLOCK_SIZE }, Sha256>::new();
        hasher.update_with(&msg[..6]);
        for block in msg[6..].chunks_exact(Sha256::BLOCK_SIZE) {
            hasher.update(block.try_into().unwrap());
        }
        assert_eq!(hasher.finish(), digest);
    }
}
