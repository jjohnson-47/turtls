//! Diffie-Hellman key exchange.

use crate::extensions::{KeyShare, SupGroups};
use crate::Alert;
use crylib::big_int::{InputTooLargeError, UBigInt};
use crylib::ec::{AffinePoint, EllipticCurve, Secp256r1};
use crylib::finite_field::FieldElement;
use getrandom::getrandom;

#[repr(u16)]
pub(crate) enum NamedGroup {
    Secp256r1 = 0x17,
    #[expect(unused, reason = "Secp384r1 is not yet supported")]
    Secp384r1 = 0x18,
    #[expect(unused, reason = "Secp512r1 is not yet supported")]
    Secp521r1 = 0x19,

    #[expect(unused, reason = "X25519 is not yet supported")]
    X25519 = 0x1d,
    #[expect(unused, reason = "X448 is not yet supported")]
    X448 = 0x1e,

    #[expect(unused, reason = "FFDH is not supported")]
    Ffdhe2048 = 0x100,
    #[expect(unused, reason = "FFDH is not supported")]
    Ffdhe3072 = 0x101,
    #[expect(unused, reason = "FFDH is not supported")]
    Ffdhe4096 = 0x102,
    #[expect(unused, reason = "FFDH is not supported")]
    Ffdhe6144 = 0x103,
    #[expect(unused, reason = "FFDH is not supported")]
    Ffdhe8192 = 0x104,
}

impl NamedGroup {
    pub(crate) const fn as_int(self) -> u16 {
        self as u16
    }

    pub(crate) const fn to_be_bytes(self) -> [u8; 2] {
        self.as_int().to_be_bytes()
    }
}

pub(crate) struct GroupKeys {
    pub(crate) secp256r1: FieldElement<4, <Secp256r1 as EllipticCurve>::Order>,
}

impl GroupKeys {
    pub(crate) fn generate(groups: SupGroups) -> Result<Self, KeyGenError> {
        if groups.groups == 0 {
            return Err(KeyGenError::NoGroups);
        }
        let mut buf = [0; 32];
        getrandom(&mut buf)?;

        if buf == [0; 32] {
            return Err(KeyGenError::PrivKeyIsZero);
        }

        // SAFETY: `[u64; 4]` and `[u8; 32]` have the same memory layout.
        let as_u64s: [u64; 4] = unsafe { std::mem::transmute(buf) };
        return Ok(Self {
            secp256r1: FieldElement::<4, _>::new(UBigInt(as_u64s)),
        });
    }
}

pub(crate) enum KeyGenError {
    RngError,
    PrivKeyIsZero,
    NoGroups,
}

impl From<getrandom::Error> for KeyGenError {
    fn from(_: getrandom::Error) -> Self {
        Self::RngError
    }
}

/// Parse the KeyShare extensions and calculate the shared secret.
pub(crate) fn shared_secret(
    key_share: &[u8],
    sup_groups: SupGroups,
    group_keys: &GroupKeys,

) -> Result<Box<[u8]>, Alert> {
    if key_share.len() < KeyShare::MIN_SER_LEN {
        return Err(Alert::DecodeError);
    }
    if sup_groups.groups & SupGroups::SECP256R1 > 0
        && key_share[0..2] == NamedGroup::Secp256r1.to_be_bytes()
    {
        let raw_x = UBigInt::<4>::from_be_bytes(key_share[3..][..32].try_into().unwrap());
        let x: FieldElement<4, Secp256r1> = FieldElement::try_from(raw_x).map_err(|_| Alert::IllegalParam)?;

        let raw_y = UBigInt::<4>::from_be_bytes(key_share[35..][..32].try_into().unwrap());
        let y: FieldElement<4, Secp256r1> = FieldElement::try_from(raw_y).map_err(|_| Alert::IllegalParam)?;

        let mut point = AffinePoint::new(x, y).ok_or(Alert::IllegalParam)?.as_projective();
        point.mul_scalar_assign(&group_keys.secp256r1);
        let as_affine = point.as_affine().expect("private key isn't zero and point is on curve");

        return Ok(Box::new(as_affine.x().to_be_bytes()));
    }
    todo!()
}
