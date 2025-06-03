use alloy_primitives::Address;
use k256::ecdsa::{signature::Signer, SigningKey};
use rand::thread_rng;
use sha3::{Digest, Keccak256};

pub fn gen_random_address() -> Address {
    let signing_key = SigningKey::random(&mut thread_rng());
    let verifying_key = signing_key.verifying_key();

    let pubkey = verifying_key.to_encoded_point(false);
    let pubkey_bytes = pubkey.as_bytes();
    let pubkey = &pubkey_bytes[1..];

    let hash = Keccak256::digest(pubkey);

    let address = Address::from_slice(&hash[12..]);

    address
}

#[cfg(test)]
mod test {
    use super::gen_random_address;
    #[test]
    pub fn test_addr() {
        gen_random_address();
    }
}
