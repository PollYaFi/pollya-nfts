use base64::{engine::general_purpose, Engine};

pub fn from_base64(val: &String) -> Vec<u8> {
    let mut buffer = Vec::<u8>::new();
    let decoded_size = general_purpose::URL_SAFE_NO_PAD.decode_vec(val, &mut buffer).unwrap();
    buffer
}

pub fn to_base64(val: Vec<u8>) -> String {
    let mut buffer = Vec::<u8>::new();
    buffer.resize( val.len() * 4 / 3 + 4, 0);
    let encoded_size = general_purpose::URL_SAFE_NO_PAD.encode_slice(val.as_slice(), &mut buffer).unwrap();
    String::from_utf8(buffer.as_slice()[0..encoded_size].to_vec()).unwrap()
}