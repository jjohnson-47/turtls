use crate::aes;

const R: u128 = 0xe1 << 120;
pub struct GcmCipher {
    round_keys: [[u8; aes::BLOCK_SIZE]; aes::NUM_ROUNDS + 1],
    h: u128,
}
pub const IV_SIZE: usize = 12;

pub struct InvalidData;

impl GcmCipher {
    pub fn new(key: [u8; 32]) -> GcmCipher {
        let mut h = [0u8; aes::BLOCK_SIZE];
        let round_keys = aes::expand_key(key);
        aes::encrypt_inline(&mut h, &round_keys);
        GcmCipher {
            round_keys,
            h: u128::from_be_bytes(h),
        }
    }

    pub fn encrypt_inline(
        &self,
        data: &mut [u8],
        add_data: &[u8],
        init_vector: &[u8; IV_SIZE],
    ) -> [u8; aes::BLOCK_SIZE] {
        let counter = {
            let mut counter = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];
            counter.copy_from_slice(init_vector);
            counter
        };
        self.xor_bit_stream(data, &counter);

        self.g_hash(data, add_data, &counter)
    }

    pub fn decrypt_inline(
        &self,
        data: &mut [u8],
        add_data: &[u8],
        counter: [u8; aes::BLOCK_SIZE],
        tag: [u8; aes::BLOCK_SIZE],
    ) -> Result<(), InvalidData> {
        if !self.packet_is_valid(data, add_data, &counter, tag) {
            return Err(InvalidData);
        }
        Ok(())
    }

    pub fn generate_iv() -> [u8; IV_SIZE] {
        todo!();
    }

    /// produces a tag for given data
    pub fn g_hash(
        &self,
        data: &[u8],
        add_data: &[u8],
        counter: &[u8; aes::BLOCK_SIZE],
    ) -> [u8; aes::BLOCK_SIZE] {
        let mut tag = 0u128;

        for block in add_data.chunks_exact(aes::BLOCK_SIZE) {
            add_block(&mut tag, block.try_into().unwrap(), self.h);
        }

        let last_block = {
            let end = add_data.len() % aes::BLOCK_SIZE;
            let mut last_block = [0u8; aes::BLOCK_SIZE];
            last_block[..end]
                .copy_from_slice(&add_data[add_data.len() - end..]);
            last_block
        };

        add_block(&mut tag, last_block, self.h);

        for block in data.chunks_exact(aes::BLOCK_SIZE) {
            add_block(&mut tag, block.try_into().unwrap(), self.h);
        }

        let last_block = {
            let end = data.len() % aes::BLOCK_SIZE;
            let mut last_block = [0u8; aes::BLOCK_SIZE];
            last_block[..end].copy_from_slice(&data[data.len() - end..]);
            last_block
        };

        add_block(&mut tag, last_block, self.h);

        tag ^= ((add_data.len() as u128 * 8) << 64) + data.len() as u128 * 8;
        tag = gf_2to128_mult(tag, self.h);

        let encrypted_iv =
            u128::from_be_bytes(aes::encrypt(counter, &self.round_keys));

        tag ^= encrypted_iv;
        tag.to_be_bytes()
    }

    /// verifies that a give packet has not been tampered with
    pub fn packet_is_valid(
        &self,
        data: &[u8],
        add_data: &[u8],
        counter: &[u8; aes::BLOCK_SIZE],
        tag: [u8; aes::BLOCK_SIZE],
    ) -> bool {
        self.g_hash(data, add_data, counter) == tag
    }

    /// encrypts/decrypts the data
    // use multi-threading in the future
    fn xor_bit_stream(&self, data: &mut [u8], counter: &[u8; aes::BLOCK_SIZE]) {
        let iv_as_int = u128::from_be_bytes(*counter);

        for (counter, block) in data.chunks_mut(aes::BLOCK_SIZE).enumerate() {
            let mut stream = (iv_as_int + 1 + counter as u128).to_be_bytes();
            aes::encrypt_inline(&mut stream, &self.round_keys);

            for (data_byte, stream_byte) in block.iter_mut().zip(stream) {
                *data_byte ^= stream_byte;
            }
        }
    }
}

fn gf_2to128_mult(a: u128, b: u128) -> u128 {
    let mut product = 0;
    let mut temp = a;
    for i in (0..128).rev() {
        if b & (1 << i) == 1 << i {
            product ^= temp;
        }
        if temp & 1 == 0 {
            temp >>= 1;
        } else {
            temp = (temp >> 1) ^ R;
        }
    }
    product
}

fn add_block(tag: &mut u128, block: [u8; aes::BLOCK_SIZE], h: u128) {
    *tag ^= u128::from_be_bytes(block);
    *tag = gf_2to128_mult(*tag, h);
}

#[cfg(test)]
mod tests {

    #[test]
    fn ctr_mode() {
        let key = [
            0xfe, 0xff, 0xe9, 0x92, 0x86, 0x65, 0x73, 0x1c, 0x6d, 0x6a, 0x8f,
            0x94, 0x67, 0x30, 0x83, 0x08, 0xfe, 0xff, 0xe9, 0x92, 0x86, 0x65,
            0x73, 0x1c, 0x6d, 0x6a, 0x8f, 0x94, 0x67, 0x30, 0x83, 0x08,
        ];
        let mut plain_text = [
            0xd9, 0x31, 0x32, 0x25, 0xf8, 0x84, 0x06, 0xe5, 0xa5, 0x59, 0x09,
            0xc5, 0xaf, 0xf5, 0x26, 0x9a, 0x86, 0xa7, 0xa9, 0x53, 0x15, 0x34,
            0xf7, 0xda, 0x2e, 0x4c, 0x30, 0x3d, 0x8a, 0x31, 0x8a, 0x72, 0x1c,
            0x3c, 0x0c, 0x95, 0x95, 0x68, 0x09, 0x53, 0x2f, 0xcf, 0x0e, 0x24,
            0x49, 0xa6, 0xb5, 0x25, 0xb1, 0x6a, 0xed, 0xf5, 0xaa, 0x0d, 0xe6,
            0x57, 0xba, 0x63, 0x7b, 0x39, 0x1a, 0xaf, 0xd2, 0x55,
        ];
        let counter = [
            0xca, 0xfe, 0xba, 0xbe, 0xfa, 0xce, 0xdb, 0xad, 0xde, 0xca, 0xf8,
            0x88, 0x00, 0x00, 0x00, 0x01,
        ];
        let cipher_text = [
            0x52, 0x2d, 0xc1, 0xf0, 0x99, 0x56, 0x7d, 0x07, 0xf4, 0x7f, 0x37,
            0xa3, 0x2a, 0x84, 0x42, 0x7d, 0x64, 0x3a, 0x8c, 0xdc, 0xbf, 0xe5,
            0xc0, 0xc9, 0x75, 0x98, 0xa2, 0xbd, 0x25, 0x55, 0xd1, 0xaa, 0x8c,
            0xb0, 0x8e, 0x48, 0x59, 0x0d, 0xbb, 0x3d, 0xa7, 0xb0, 0x8b, 0x10,
            0x56, 0x82, 0x88, 0x38, 0xc5, 0xf6, 0x1e, 0x63, 0x93, 0xba, 0x7a,
            0x0a, 0xbc, 0xc9, 0xf6, 0x62, 0x89, 0x80, 0x15, 0xad,
        ];
        let cipher = super::GcmCipher::new(key);
        cipher.xor_bit_stream(&mut plain_text, &counter);
        assert_eq!(plain_text, cipher_text);
    }

    #[test]
    fn g_hash() {
        let key = [
            0xfe, 0xff, 0xe9, 0x92, 0x86, 0x65, 0x73, 0x1c, 0x6d, 0x6a, 0x8f,
            0x94, 0x67, 0x30, 0x83, 0x08, 0xfe, 0xff, 0xe9, 0x92, 0x86, 0x65,
            0x73, 0x1c, 0x6d, 0x6a, 0x8f, 0x94, 0x67, 0x30, 0x83, 0x08,
        ];
        let cipher = super::GcmCipher::new(key);

        let counter = [
            0xca, 0xfe, 0xba, 0xbe, 0xfa, 0xce, 0xdb, 0xad, 0xde, 0xca, 0xf8,
            0x88, 0x00, 0x00, 0x00, 0x01,
        ];

        let cipher_text = [
            0x52, 0x2d, 0xc1, 0xf0, 0x99, 0x56, 0x7d, 0x07, 0xf4, 0x7f, 0x37,
            0xa3, 0x2a, 0x84, 0x42, 0x7d, 0x64, 0x3a, 0x8c, 0xdc, 0xbf, 0xe5,
            0xc0, 0xc9, 0x75, 0x98, 0xa2, 0xbd, 0x25, 0x55, 0xd1, 0xaa, 0x8c,
            0xb0, 0x8e, 0x48, 0x59, 0x0d, 0xbb, 0x3d, 0xa7, 0xb0, 0x8b, 0x10,
            0x56, 0x82, 0x88, 0x38, 0xc5, 0xf6, 0x1e, 0x63, 0x93, 0xba, 0x7a,
            0x0a, 0xbc, 0xc9, 0xf6, 0x62,
        ];
        let tag = [
            0x76, 0xfc, 0x6e, 0xce, 0x0f, 0x4e, 0x17, 0x68, 0xcd, 0xdf, 0x88,
            0x53, 0xbb, 0x2d, 0x55, 0x1b,
        ];

        let add_data = [
            0xfe, 0xed, 0xfa, 0xce, 0xde, 0xad, 0xbe, 0xef, 0xfe, 0xed, 0xfa,
            0xce, 0xde, 0xad, 0xbe, 0xef, 0xab, 0xad, 0xda, 0xd2,
        ];

        let h = 0xacbef20579b4b8ebce889bac8732dad7;
        assert_eq!(cipher.h, h);

        assert_eq!(tag, cipher.g_hash(&cipher_text, &add_data, &counter));
    }

    #[test]
    fn mult() {
        let a = 0x66e94bd4ef8a2c3b884cfa59ca342b2e;
        let b = 0x0388dace60b6a392f328c2b971b2fe78;
        let product = 0x5e2ec746917062882c85b0685353deb7;
        assert_eq!(super::gf_2to128_mult(a, b), product);
    }
}
