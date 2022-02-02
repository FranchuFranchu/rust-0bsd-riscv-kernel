use pbkdf2::pbkdf2;
use sha2::Digest;
use sodiumoxide::crypto::stream::xsalsa20;

fn base32(input: &str) -> Option<Vec<u8>> {
    let alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

    let mut out = vec![];
    let mut skip = 0u8;
    let mut byte = 0u8;
    for c in input.chars() {
        let c = match c {
            '0' => 'O',
            '1' => 'I',
            c => c.to_uppercase().next().unwrap(),
        };
        let mut val = alphabet.find(c)? as u8;
        val <<= 3;
        byte |= val >> skip;
        skip += 5;
        if skip >= 8 {
            out.push(byte);
            skip -= 8;
            if skip > 0 {
                byte = val << (5 - skip);
            } else {
                byte = 0;
            }
        }
    }
    Some(out)
}

pub fn decrypt_identity(identity: &str, password: &str) -> Option<(String, Vec<u8>)> {
    let identity = identity.replace("-", "");
    let identity = base32(&identity)?;
    let (salt, identity) = identity.split_at(8);

    let mut key = [0u8; 32];
    pbkdf2::<hmac::Hmac<sha2::Sha256>>(password.as_bytes(), salt, 100000, &mut key);

    let plain = xsalsa20::stream_xor(
        identity,
        &xsalsa20::Nonce::from_slice(&[0u8; xsalsa20::NONCEBYTES])?,
        &xsalsa20::Key::from_slice(&key)?,
    );

    let (identity, plain) = plain.split_at(8);
    let (private_key, expected_hash) = plain.split_at(32);

    let mut md = sha2::Sha256::new();
    md.update(identity);
    md.update(private_key);
    let hash = md.finalize();
    let hash = hash.as_slice();

    if expected_hash[0] != hash[0] || expected_hash[1] != hash[1] {
        None
    } else {
        Some((
            String::from_utf8(identity.to_vec()).ok()?,
            private_key.to_vec(),
        ))
    }
}
