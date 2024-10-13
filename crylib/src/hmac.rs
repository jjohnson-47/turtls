use crate::hash::{BlockHasher, BufHasher};

pub struct Hmac<const H_LEN: usize, const B_LEN: usize, H>
where
    H: BlockHasher<H_LEN, B_LEN>,
{
    state: H,
    opad: [u8; B_LEN],
}

impl<const H_LEN: usize, const B_LEN: usize, H> Hmac<H_LEN, B_LEN, H>
where
    H: BlockHasher<H_LEN, B_LEN>,
{
    pub fn new(key: &[u8]) -> Self {
        let mut ipad = [0x36; B_LEN];
        let mut opad = [0x5c; B_LEN];
        for ((ipad_byte, opad_byte), key_byte) in ipad.iter_mut().zip(opad.iter_mut()).zip(key) {
            *ipad_byte ^= key_byte;
            *opad_byte ^= key_byte;
        }

        let mut state = H::new();
        state.update(&ipad);
        Self { state, opad }
    }

    pub fn update(&mut self, block: &[u8; B_LEN]) {
        self.state.update(block);
    }

    pub fn finish_with(self, msg: &[u8]) -> [u8; H_LEN] {
        let inner_hash = self.state.finish_with(msg);
        outer_finish::<H_LEN, B_LEN, H>(&self.opad, &inner_hash)
    }

    pub fn finish(self) -> [u8; H_LEN] {
        let inner_hash = self.state.finish();
        outer_finish::<H_LEN, B_LEN, H>(&self.opad, &inner_hash)
    }

    pub fn auth(key: &[u8], msg: &[u8]) -> [u8; H_LEN] {
        let state = Self::new(key);
        state.finish_with(msg)
    }
}

impl<const H_LEN: usize, const B_LEN: usize, H> Hmac<H_LEN, B_LEN, BufHasher<H_LEN, B_LEN, H>>
where
    H: BlockHasher<H_LEN, B_LEN>,
{
    pub fn update_with(&mut self, msg: &[u8]) {
        self.state.update_with(msg)
    }
}

fn outer_finish<const H_LEN: usize, const B_LEN: usize, H>(
    opad: &[u8; B_LEN],
    inner_hash: &[u8; H_LEN],
) -> [u8; H_LEN]
where
    H: BlockHasher<H_LEN, B_LEN>,
{
    let mut hasher = H::new();
    hasher.update(opad);
    hasher.finish_with(inner_hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hash::{Sha256, Sha512};
    // test vectors from https://datatracker.ietf.org/doc/html/rfc4231

    #[test]
    fn hmac() {
        let key = [
            0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b,
            0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b,
        ];
        let msg = [0x48, 0x69, 0x20, 0x54, 0x68, 0x65, 0x72, 0x65];
        let mac_sha256 = [
            0xb0, 0x34, 0x4c, 0x61, 0xd8, 0xdb, 0x38, 0x53, 0x5c, 0xa8, 0xaf, 0xce, 0xaf, 0x0b,
            0xf1, 0x2b, 0x88, 0x1d, 0xc2, 0x00, 0xc9, 0x83, 0x3d, 0xa7, 0x26, 0xe9, 0x37, 0x6c,
            0x2e, 0x32, 0xcf, 0xf7,
        ];
        assert_eq!(
            Hmac::<{ Sha256::HASH_SIZE }, { Sha256::BLOCK_SIZE }, Sha256>::auth(&key, &msg),
            mac_sha256
        );
        let mac_sha512 = [
            0x87, 0xaa, 0x7c, 0xde, 0xa5, 0xef, 0x61, 0x9d, 0x4f, 0xf0, 0xb4, 0x24, 0x1a, 0x1d,
            0x6c, 0xb0, 0x23, 0x79, 0xf4, 0xe2, 0xce, 0x4e, 0xc2, 0x78, 0x7a, 0xd0, 0xb3, 0x05,
            0x45, 0xe1, 0x7c, 0xde, 0xda, 0xa8, 0x33, 0xb7, 0xd6, 0xb8, 0xa7, 0x02, 0x03, 0x8b,
            0x27, 0x4e, 0xae, 0xa3, 0xf4, 0xe4, 0xbe, 0x9d, 0x91, 0x4e, 0xeb, 0x61, 0xf1, 0x70,
            0x2e, 0x69, 0x6c, 0x20, 0x3a, 0x12, 0x68, 0x54,
        ];
        assert_eq!(
            Hmac::<{ Sha512::HASH_SIZE }, { Sha512::BLOCK_SIZE }, Sha512>::auth(&key, &msg),
            mac_sha512
        );

        let msg = [
            0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd,
            0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd,
            0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd,
            0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd,
        ];
        let key = [
            0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
            0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
        ];
        let mac_sha256 = [
            0x77, 0x3e, 0xa9, 0x1e, 0x36, 0x80, 0x0e, 0x46, 0x85, 0x4d, 0xb8, 0xeb, 0xd0, 0x91,
            0x81, 0xa7, 0x29, 0x59, 0x09, 0x8b, 0x3e, 0xf8, 0xc1, 0x22, 0xd9, 0x63, 0x55, 0x14,
            0xce, 0xd5, 0x65, 0xfe,
        ];
        assert_eq!(
            Hmac::<{ Sha256::HASH_SIZE }, { Sha256::BLOCK_SIZE }, Sha256>::auth(&key, &msg),
            mac_sha256
        );
        let mac_sha512 = [
            0xfa, 0x73, 0xb0, 0x08, 0x9d, 0x56, 0xa2, 0x84, 0xef, 0xb0, 0xf0, 0x75, 0x6c, 0x89,
            0x0b, 0xe9, 0xb1, 0xb5, 0xdb, 0xdd, 0x8e, 0xe8, 0x1a, 0x36, 0x55, 0xf8, 0x3e, 0x33,
            0xb2, 0x27, 0x9d, 0x39, 0xbf, 0x3e, 0x84, 0x82, 0x79, 0xa7, 0x22, 0xc8, 0x06, 0xb4,
            0x85, 0xa4, 0x7e, 0x67, 0xc8, 0x07, 0xb9, 0x46, 0xa3, 0x37, 0xbe, 0xe8, 0x94, 0x26,
            0x74, 0x27, 0x88, 0x59, 0xe1, 0x32, 0x92, 0xfb,
        ];
        assert_eq!(
            Hmac::<{ Sha512::HASH_SIZE }, { Sha512::BLOCK_SIZE }, Sha512>::auth(&key, &msg),
            mac_sha512
        );
    }

    #[test]
    fn update_with() {
        let msg = [
            0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd,
            0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd,
            0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd,
            0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd,
        ];
        let key = [
            0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
            0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
        ];
        let mac_sha256 = [
            0x77, 0x3e, 0xa9, 0x1e, 0x36, 0x80, 0x0e, 0x46, 0x85, 0x4d, 0xb8, 0xeb, 0xd0, 0x91,
            0x81, 0xa7, 0x29, 0x59, 0x09, 0x8b, 0x3e, 0xf8, 0xc1, 0x22, 0xd9, 0x63, 0x55, 0x14,
            0xce, 0xd5, 0x65, 0xfe,
        ];
        let mut hmac = Hmac::<
            { Sha256::HASH_SIZE },
            { Sha256::BLOCK_SIZE },
            BufHasher<{ Sha256::HASH_SIZE }, { Sha256::BLOCK_SIZE }, Sha256>,
        >::new(&key);
        for block in msg.chunks(3) {
            hmac.update_with(block);
        }
        let mac = hmac.finish();
        assert_eq!(mac, mac_sha256);
        let mac_sha512 = [
            0xfa, 0x73, 0xb0, 0x08, 0x9d, 0x56, 0xa2, 0x84, 0xef, 0xb0, 0xf0, 0x75, 0x6c, 0x89,
            0x0b, 0xe9, 0xb1, 0xb5, 0xdb, 0xdd, 0x8e, 0xe8, 0x1a, 0x36, 0x55, 0xf8, 0x3e, 0x33,
            0xb2, 0x27, 0x9d, 0x39, 0xbf, 0x3e, 0x84, 0x82, 0x79, 0xa7, 0x22, 0xc8, 0x06, 0xb4,
            0x85, 0xa4, 0x7e, 0x67, 0xc8, 0x07, 0xb9, 0x46, 0xa3, 0x37, 0xbe, 0xe8, 0x94, 0x26,
            0x74, 0x27, 0x88, 0x59, 0xe1, 0x32, 0x92, 0xfb,
        ];
        let mut hmac = Hmac::<
            { Sha512::HASH_SIZE },
            { Sha512::BLOCK_SIZE },
            BufHasher<{ Sha512::HASH_SIZE }, { Sha512::BLOCK_SIZE }, Sha512>,
        >::new(&key);
        for block in msg.chunks(3) {
            hmac.update_with(block);
        }
        let mac = hmac.finish();
        assert_eq!(mac, mac_sha512);
        assert_eq!(mac, mac_sha512);
    }
}