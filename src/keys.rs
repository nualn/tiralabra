use std::fs::File;
use std::io::{self, prelude::*, Error};

use crate::algorithms::{self, modular_pow};
use num_bigint::{BigInt, RandomBits};
use rand::Rng;
use std::str::FromStr;

const KEY_SIZE: u64 = 1024;
const MR_ITERATIONS: isize = 4;
const DEFAULT_EXP: i32 = 65_537;

#[derive(Debug)]
pub struct Key {
    exp: BigInt,
    modulus: BigInt,
}

impl Key {
    fn write_to_file(&self, path: &str) -> std::io::Result<()> {
        let key_string = self.modulus.to_string() + "\n" + &self.exp.to_string();

        let mut file = File::create(path)?;
        file.write_all(&key_string.as_bytes())?;

        Ok(())
    }

    pub fn from_file(path: &str) -> Result<Key, Error> {
        let file = File::open(path)?;
        let mut file_buf = io::BufReader::new(file).lines();

        let modulus = match file_buf.next() {
            Some(string) => string?,
            None => panic!("Invalid key file: {}", path),
        };
        let exp = match file_buf.next() {
            Some(string) => string?,
            None => panic!("Invalid key file: {}", path),
        };

        Ok(Key {
            modulus: match BigInt::from_str(&modulus) {
                Ok(num) => num,
                Err(_) => panic!("Invalid key file: {}", path),
            },
            exp: match BigInt::from_str(&exp) {
                Ok(num) => num,
                Err(_) => panic!("Invalid key file: {}", path),
            },
        })
    }

    pub fn encrypt(&self, input: &mut dyn Read, output: &mut dyn Write) -> std::io::Result<()> {
        let in_bytes: usize = (self.modulus.bits() / 8).try_into().unwrap();
        let out_bytes: usize = ((self.modulus.bits() + 7) / 8).try_into().unwrap();

        let mut current_in_bytes: Vec<u8> = vec![0u8; in_bytes];

        let mut amount_of_bytes_read = in_bytes;

        while amount_of_bytes_read > 0 {
            current_in_bytes.fill(0);
            amount_of_bytes_read = input.read(&mut current_in_bytes)?;
            if amount_of_bytes_read == 0 {
                break;
            }

            let mut dencrypted_bytes = modular_pow(
                &BigInt::from_bytes_le(num_bigint::Sign::Plus, &current_in_bytes),
                &self.exp,
                &self.modulus,
            )
            .to_bytes_le()
            .1;

            // Fill in missing bytes
            let mut i = 0;
            while i < out_bytes - dencrypted_bytes.len() {
                dencrypted_bytes.push(0u8);
                i += 1;
            }

            output.write(&dencrypted_bytes)?;
        }

        Ok(())
    }

    pub fn decrypt(&self, input: &mut dyn Read, output: &mut dyn Write) -> std::io::Result<()> {
        let in_bytes: usize = ((self.modulus.bits() + 7) / 8).try_into().unwrap();

        let mut current_in_bytes: Vec<u8> = vec![0u8; in_bytes];

        let mut amount_of_bytes_read = in_bytes;

        while amount_of_bytes_read > 0 {
            current_in_bytes.fill(0);
            amount_of_bytes_read = input.read(&mut current_in_bytes)?;
            if amount_of_bytes_read == 0 {
                break;
            }

            let dencrypted_bytes = modular_pow(
                &BigInt::from_bytes_le(num_bigint::Sign::Plus, &current_in_bytes),
                &self.exp,
                &self.modulus,
            )
            .to_bytes_le()
            .1;

            output.write(&dencrypted_bytes)?;
        }

        Ok(())
    }
}
pub struct KeyPair {
    public: Key,
    private: Key,
}

impl KeyPair {
    pub fn generate() -> KeyPair {
        loop {
            let p = generate_probable_prime();
            let q = generate_probable_prime();

            match generate_from_primes(&p, &q) {
                Ok(key_pair) => return key_pair,
                Err(_) => continue,
            }
        }
    }

    pub fn write_to_file(&self) -> std::io::Result<()> {
        self.public.write_to_file("key.public")?;
        self.private.write_to_file("key.private")?;
        Ok(())
    }
}

fn generate_from_primes(p: &BigInt, q: &BigInt) -> Result<KeyPair, &'static str> {
    let n = p * q;

    let lambda_n = lcm(&(p - BigInt::from(1)), &(q - BigInt::from(1)));
    let e = BigInt::from(DEFAULT_EXP);

    let (_, d, _) = algorithms::extended_eucledian(&e, &lambda_n);

    if d < BigInt::from(0) {
        Err("Failed to generate private key")
    } else {
        Ok(KeyPair {
            public: Key {
                exp: e,
                modulus: n.clone(),
            },
            private: Key { exp: d, modulus: n },
        })
    }
}

fn generate_probable_prime() -> BigInt {
    let mut num: BigInt = rand::thread_rng().sample(RandomBits::new(KEY_SIZE));
    while !algorithms::miller_rabin(&num, MR_ITERATIONS) {
        num = rand::thread_rng().sample(RandomBits::new(KEY_SIZE));
    }
    return num;
}

fn lcm(a: &BigInt, b: &BigInt) -> BigInt {
    let (qcd, _, _) = algorithms::extended_eucledian(a, b);
    a * b / qcd
}

#[cfg(test)]
mod tests {
    use super::{Key, KeyPair};
    use num_bigint::BigInt;
    use std::str::FromStr;

    fn get_test_keys() -> KeyPair {
        KeyPair {
            private: Key {
                modulus: BigInt::from_str("1036094667116699957794031654006081978994519669637716761721879892060921789104339276119982642913634892651733197723792916672490510973174371735308852113790826056473350952392537787124370663975479266036459517990539460120339327077229962893991754754588995075350011727457840136185573281158165376037935679447822863292727314069042603139807056816418241224303148746879694236180240345992665098156479345946045666246915319417310506472587982909698492734403006522827558508404716161793199143147983405663796210020223235604312596277251183247192863971627633753966391027463195544174516160377912482286386280685703288417180144207312345469879").unwrap(),
                exp: BigInt::from_str("83823589842337345716418534590881196875435896898405184197878463072091858738260690885059858855840890997781375963671981878180622207276287410204364232662535538028357299053623155931732212167813402837119710102265467966716905494371924280804633753980549664902039873368265936932500324990678840725836417520570031050977020391623788866928987762824073297097323130060361852489990090764313238485589932494406080968498265640453817169552540095733083773775638207721011670181249752626263778607463361772158444127287048534847623537777283779305764113256091027874343061999145002176744068249207148202460458229711423683286138201987329205533").unwrap(),
            },
            public: Key {
                modulus: BigInt::from_str("1036094667116699957794031654006081978994519669637716761721879892060921789104339276119982642913634892651733197723792916672490510973174371735308852113790826056473350952392537787124370663975479266036459517990539460120339327077229962893991754754588995075350011727457840136185573281158165376037935679447822863292727314069042603139807056816418241224303148746879694236180240345992665098156479345946045666246915319417310506472587982909698492734403006522827558508404716161793199143147983405663796210020223235604312596277251183247192863971627633753966391027463195544174516160377912482286386280685703288417180144207312345469879").unwrap(),
                exp: BigInt::from_str("65537").unwrap(),

            }
        }
    }

    #[test]
    fn encrypt_returns_different_string() {
        let keys = get_test_keys();

        let original = Vec::from("FooBarBaz".as_bytes());
        let mut encrypted = Vec::new();

        keys.private
            .encrypt(&mut &original[..], &mut encrypted)
            .unwrap();

        assert_ne!(original, encrypted)
    }

    #[test]
    fn decrypt_returns_original_string() {
        let keys = get_test_keys();

        let original = Vec::from("FooBarBaz".as_bytes());
        let mut encrypted = Vec::new();

        keys.private
            .encrypt(&mut &original[..], &mut encrypted)
            .unwrap();

        let mut decrypted = Vec::new();

        keys.public
            .decrypt(&mut &encrypted[..], &mut decrypted)
            .unwrap();

        assert_eq!(original, decrypted)
    }
}
