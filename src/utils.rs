use secp256k1::rand;
use secp256k1::SecretKey;

pub fn rng_secret_key() -> SecretKey {
    SecretKey::new(&mut rand::thread_rng())
}
