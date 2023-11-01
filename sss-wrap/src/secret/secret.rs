//! Purpose: Defines the Secret struct.

use std::fmt::{self, Debug, Formatter};

use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

use crate::polynomial::galois::{Coeff, GaloisPolynomial};

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub struct Metadata {
    pub shares_required: u8,
    pub shares_to_create: u8,
    pub sec_len: usize,
}
impl Metadata {
    pub fn new(shares_required: u8, shares_to_create: u8, sec_len: usize) -> Metadata {
        Metadata {
            shares_required,
            shares_to_create,
            sec_len,
        }
    }
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub struct ShareMeta {
    pub share: Share,
    pub meta: Metadata,
}

impl ShareMeta {
    pub fn new(share: Share, meta: Metadata) -> ShareMeta {
        ShareMeta { share, meta }
    }
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct Share {
    x: u8,
    #[serde(with = "hex::serde")]
    ys: Vec<u8>,
}

impl Debug for Share {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Share {{ x: {}, ys: {:?} }}", self.x, self.ys)
    }
}

pub struct RenewableShare {
    poly: GaloisPolynomial,
}

impl Debug for RenewableShare {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "RenewableShare {{ poly: **** }}")
    }
}

impl From<Vec<(u8, u8)>> for Share {
    fn from(shares: Vec<(u8, u8)>) -> Self {
        let x = shares[0].0;
        let ys = shares.into_iter().map(|(_, y)| y).collect::<Vec<_>>();
        Self::new(x, ys)
    }
}

impl From<Share> for Vec<(u8, u8)> {
    fn from(share: Share) -> Self {
        share
            .ys
            .into_iter()
            .map(|y| (share.x, y))
            .collect::<Vec<_>>()
    }
}

impl From<Share> for Vec<u8> {
    fn from(share: Share) -> Self {
        let mut v = vec![share.x];
        v.extend(share.ys);
        v
    }
}

impl Share {
    pub fn new(x: u8, ys: Vec<u8>) -> Self {
        Self { x, ys }
    }

    pub fn renew_poly(&self, metadata: &Metadata) -> RenewableShare {
        RenewableShare::new(self, metadata)
    }

    pub fn id(&self) -> u8 {
        self.x
    }

    pub fn ys_len(&self) -> usize {
        self.ys.len()
    }
}

impl RenewableShare {
    pub fn new(share: &Share, metadata: &Metadata) -> Self {
        let mut rng = thread_rng();

        let mut coeffs: Vec<u8> =
            Vec::with_capacity(metadata.sec_len * metadata.shares_to_create as usize);
        unsafe { coeffs.set_len(metadata.sec_len * metadata.shares_to_create as usize) };
        rng.fill(coeffs.as_mut_slice());

        let mut share_poly = GaloisPolynomial::new();
        share_poly.set_coeff(Coeff(0), 0);
        for i in 1..(metadata.shares_required as usize) {
            let curr_co = coeffs[(share.x as usize * i) + i];
            share_poly.set_coeff(Coeff(curr_co), i);
        }
        Self { poly: share_poly }
    }

    pub fn renew(&self, share: &mut Share) {
        share
            .ys
            .iter_mut()
            .for_each(|y| *y = *(Coeff(*y) + Coeff(self.poly.get_y_value(share.x))));
    }

    pub fn get_share(&self, x: u8, ys_len: usize) -> Share {
        let ys = (0..ys_len)
            .map(|_| self.poly.get_y_value(x))
            .collect::<Vec<_>>();
        Share::new(x, ys)
    }

    pub fn renew_with_share(new_share: &Share, share: &Share) -> Share {
        let new_ys = share
            .ys
            .iter()
            .zip(new_share.ys.iter())
            .map(|(y, new_y)| *(Coeff(*y) + Coeff(*new_y)))
            .collect::<Vec<_>>();
        Share::new(share.x, new_ys)
    }
}

#[cfg(test)]
mod tests {
    use sss_rs::basic_sharing::{from_secrets, reconstruct_secrets};

    use crate::secret::secret::{Metadata, Share};

    #[test]
    fn test_renewable_share() {
        let mut share = Share::new(1, vec![1, 2, 3]);
        let mut share_2 = Share::new(1, vec![1, 2, 3]);
        let renewable_share = share.renew_poly(&Metadata::new(2, 3, 3));
        renewable_share.renew(&mut share_2);
        renewable_share.renew(&mut share);
        assert_eq!(share, share_2);
    }

    #[test]
    fn test_renewable_share_with_recovery() {
        let shares_required = 2;
        let shares_to_create = 3;
        let secret: Vec<u8> = vec![5, 4, 9, 1, 2, 128, 43];
        let shares = from_secrets(&secret, shares_required, shares_to_create, None).unwrap();

        let mut shares_vec: Vec<Share> = shares.into_iter().map(|s| s.into()).collect::<Vec<_>>();
        let share = shares_vec.first().unwrap();
        let stable_poly = share.renew_poly(&Metadata::new(
            shares_required,
            shares_to_create,
            secret.len(),
        ));

        shares_vec.iter_mut().for_each(|i| stable_poly.renew(i));

        shares_vec.remove(2);

        let shares: Vec<Vec<(u8, u8)>> =
            shares_vec.into_iter().map(|s| s.into()).collect::<Vec<_>>();

        let recon = reconstruct_secrets(shares).unwrap();
        assert_eq!(secret, recon);
    }
}
