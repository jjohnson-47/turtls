//! AES-256

use std::mem::{transmute, MaybeUninit};

const NUM_ROUNDS: usize = 14;
const N_K: usize = 8;
const BLOCK_SIZE: usize = 16;

const S_BOX: [u8; 256] = [
    0x63, 0x7C, 0x77, 0x7B, 0xF2, 0x6B, 0x6F, 0xC5, 0x30, 0x01, 0x67, 0x2B,
    0xFE, 0xD7, 0xAB, 0x76, 0xCA, 0x82, 0xC9, 0x7D, 0xFA, 0x59, 0x47, 0xF0,
    0xAD, 0xD4, 0xA2, 0xAF, 0x9C, 0xA4, 0x72, 0xC0, 0xB7, 0xFD, 0x93, 0x26,
    0x36, 0x3F, 0xF7, 0xCC, 0x34, 0xA5, 0xE5, 0xF1, 0x71, 0xD8, 0x31, 0x15,
    0x04, 0xC7, 0x23, 0xC3, 0x18, 0x96, 0x05, 0x9A, 0x07, 0x12, 0x80, 0xE2,
    0xEB, 0x27, 0xB2, 0x75, 0x09, 0x83, 0x2C, 0x1A, 0x1B, 0x6E, 0x5A, 0xA0,
    0x52, 0x3B, 0xD6, 0xB3, 0x29, 0xE3, 0x2F, 0x84, 0x53, 0xD1, 0x00, 0xED,
    0x20, 0xFC, 0xB1, 0x5B, 0x6A, 0xCB, 0xBE, 0x39, 0x4A, 0x4C, 0x58, 0xCF,
    0xD0, 0xEF, 0xAA, 0xFB, 0x43, 0x4D, 0x33, 0x85, 0x45, 0xF9, 0x02, 0x7F,
    0x50, 0x3C, 0x9F, 0xA8, 0x51, 0xA3, 0x40, 0x8F, 0x92, 0x9D, 0x38, 0xF5,
    0xBC, 0xB6, 0xDA, 0x21, 0x10, 0xFF, 0xF3, 0xD2, 0xCD, 0x0C, 0x13, 0xEC,
    0x5F, 0x97, 0x44, 0x17, 0xC4, 0xA7, 0x7E, 0x3D, 0x64, 0x5D, 0x19, 0x73,
    0x60, 0x81, 0x4F, 0xDC, 0x22, 0x2A, 0x90, 0x88, 0x46, 0xEE, 0xB8, 0x14,
    0xDE, 0x5E, 0x0B, 0xDB, 0xE0, 0x32, 0x3A, 0x0A, 0x49, 0x06, 0x24, 0x5C,
    0xC2, 0xD3, 0xAC, 0x62, 0x91, 0x95, 0xE4, 0x79, 0xE7, 0xC8, 0x37, 0x6D,
    0x8D, 0xD5, 0x4E, 0xA9, 0x6C, 0x56, 0xF4, 0xEA, 0x65, 0x7A, 0xAE, 0x08,
    0xBA, 0x78, 0x25, 0x2E, 0x1C, 0xA6, 0xB4, 0xC6, 0xE8, 0xDD, 0x74, 0x1F,
    0x4B, 0xBD, 0x8B, 0x8A, 0x70, 0x3E, 0xB5, 0x66, 0x48, 0x03, 0xF6, 0x0E,
    0x61, 0x35, 0x57, 0xB9, 0x86, 0xC1, 0x1D, 0x9E, 0xE1, 0xF8, 0x98, 0x11,
    0x69, 0xD9, 0x8E, 0x94, 0x9B, 0x1E, 0x87, 0xE9, 0xCE, 0x55, 0x28, 0xDF,
    0x8C, 0xA1, 0x89, 0x0D, 0xBF, 0xE6, 0x42, 0x68, 0x41, 0x99, 0x2D, 0x0F,
    0xB0, 0x54, 0xBB, 0x16,
];

const R_CON: [u32; 256] = [
    0x8d, 0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80, 0x1b, 0x36, 0x6c,
    0xd8, 0xab, 0x4d, 0x9a, 0x2f, 0x5e, 0xbc, 0x63, 0xc6, 0x97, 0x35, 0x6a,
    0xd4, 0xb3, 0x7d, 0xfa, 0xef, 0xc5, 0x91, 0x39, 0x72, 0xe4, 0xd3, 0xbd,
    0x61, 0xc2, 0x9f, 0x25, 0x4a, 0x94, 0x33, 0x66, 0xcc, 0x83, 0x1d, 0x3a,
    0x74, 0xe8, 0xcb, 0x8d, 0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80,
    0x1b, 0x36, 0x6c, 0xd8, 0xab, 0x4d, 0x9a, 0x2f, 0x5e, 0xbc, 0x63, 0xc6,
    0x97, 0x35, 0x6a, 0xd4, 0xb3, 0x7d, 0xfa, 0xef, 0xc5, 0x91, 0x39, 0x72,
    0xe4, 0xd3, 0xbd, 0x61, 0xc2, 0x9f, 0x25, 0x4a, 0x94, 0x33, 0x66, 0xcc,
    0x83, 0x1d, 0x3a, 0x74, 0xe8, 0xcb, 0x8d, 0x01, 0x02, 0x04, 0x08, 0x10,
    0x20, 0x40, 0x80, 0x1b, 0x36, 0x6c, 0xd8, 0xab, 0x4d, 0x9a, 0x2f, 0x5e,
    0xbc, 0x63, 0xc6, 0x97, 0x35, 0x6a, 0xd4, 0xb3, 0x7d, 0xfa, 0xef, 0xc5,
    0x91, 0x39, 0x72, 0xe4, 0xd3, 0xbd, 0x61, 0xc2, 0x9f, 0x25, 0x4a, 0x94,
    0x33, 0x66, 0xcc, 0x83, 0x1d, 0x3a, 0x74, 0xe8, 0xcb, 0x8d, 0x01, 0x02,
    0x04, 0x08, 0x10, 0x20, 0x40, 0x80, 0x1b, 0x36, 0x6c, 0xd8, 0xab, 0x4d,
    0x9a, 0x2f, 0x5e, 0xbc, 0x63, 0xc6, 0x97, 0x35, 0x6a, 0xd4, 0xb3, 0x7d,
    0xfa, 0xef, 0xc5, 0x91, 0x39, 0x72, 0xe4, 0xd3, 0xbd, 0x61, 0xc2, 0x9f,
    0x25, 0x4a, 0x94, 0x33, 0x66, 0xcc, 0x83, 0x1d, 0x3a, 0x74, 0xe8, 0xcb,
    0x8d, 0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80, 0x1b, 0x36, 0x6c,
    0xd8, 0xab, 0x4d, 0x9a, 0x2f, 0x5e, 0xbc, 0x63, 0xc6, 0x97, 0x35, 0x6a,
    0xd4, 0xb3, 0x7d, 0xfa, 0xef, 0xc5, 0x91, 0x39, 0x72, 0xe4, 0xd3, 0xbd,
    0x61, 0xc2, 0x9f, 0x25, 0x4a, 0x94, 0x33, 0x66, 0xcc, 0x83, 0x1d, 0x3a,
    0x74, 0xe8, 0xcb, 0x8d,
];

pub fn encrypt(input: [u8; 16], key: [u8; 32]) -> [u8; 16] {
    // SAFETY: a 2d array is represented by the same memory as a 1d array
    let mut state: [[u8; 4]; 4] = unsafe { transmute(input) };

    let round_keys = key_expansion(key);

    state = add_round_key(state, round_keys[0]);
    for round in round_keys.iter().take(NUM_ROUNDS).skip(1) {
        state = sub_bytes(state);
        state = shift_rows(state);
        state = mix_columns(state);
        state = add_round_key(state, *round);
    }
    state = sub_bytes(state);
    state = shift_rows(state);
    state = add_round_key(state, round_keys[NUM_ROUNDS]);

    // use flatten() once stabilized
    unsafe { transmute(state) }
}

fn key_expansion(key: [u8; 32]) -> [[[u8; 4]; 4]; NUM_ROUNDS + 1] {
    // let key = key.
    let key: [MaybeUninit<u32>; 8] = unsafe { transmute(key) };
    let mut expanded_keys = [MaybeUninit::<u32>::uninit(); 4 * NUM_ROUNDS + 4];

    // key is guaranteed to be initialized
    expanded_keys[0..key.len()].clone_from_slice(&key);
    for i in N_K..expanded_keys.len() {
        // SAFETY: all indexes less than `i` are guaranteed to be initialized
        let mut temp = unsafe { expanded_keys[i - 1].assume_init() };
        temp = match i % N_K {
            0 => sub_word(rotate_word(temp)) ^ R_CON[i / N_K],
            4 => sub_word(temp),
            _ => temp,
        };
        expanded_keys[i]
            // SAFETY: all indexes less than `i` are guaranteed to be initialized
            .write(unsafe { expanded_keys[i - N_K].assume_init() } ^ temp);
    }
    unsafe { transmute(expanded_keys) }
}

#[inline]
fn add_round_key(
    mut state: [[u8; 4]; 4],
    round_key: [[u8; 4]; 4],
) -> [[u8; 4]; 4] {
    for col in 0..state.len() {
        for row in 0..state[0].len() {
            state[col][row] ^= round_key[col][row];
        }
    }
    state
}

#[inline]
fn sub_bytes(state: [[u8; 4]; 4]) -> [[u8; 4]; 4] {
    state.map(|col| col.map(s_box))
}

#[inline]
fn shift_rows(state: [[u8; 4]; 4]) -> [[u8; 4]; 4] {
    todo!();
}

#[inline]
fn mix_columns(state: [[u8; 4]; 4]) -> [[u8; 4]; 4] {
    state.map(|mut col| {
        let column = col;
        col[0] = (0x2 * column[0]) ^ (0x3 * column[1]) ^ column[2] ^ column[3];

        col[1] = column[0] ^ (0x2 * column[1]) ^ (0x3 * column[2]) ^ column[3];

        col[2] = column[0] ^ column[1] ^ (0x2 * column[2]) ^ (0x3 * column[3]);

        col[3] = (0x3 * column[0]) ^ column[1] ^ column[2] ^ (0x2 * column[3]);
        col
    })
}

#[inline]
fn s_box(byte: u8) -> u8 {
    S_BOX[byte as usize]
}

#[inline]
fn sub_word(word: u32) -> u32 {
    u32::from_ne_bytes(word.to_ne_bytes().map(s_box))
}

#[inline]
fn rotate_word(word: u32) -> u32 {
    word.rotate_left(8)
}

fn eq_inv_cipher() {
    todo!();
}

fn inv_cipher(input: [u8; 16]) {
    todo!();
}

fn inv_mix_columns() {
    todo!();
}

fn inv_s_box() {
    todo!();
}

fn inv_shift_rows() {
    todo!();
}

fn inv_sub_bytes() {
    todo!();
}

fn key_expansion_eic() {
    todo!()
}

fn x_times() {
    todo!();
}
