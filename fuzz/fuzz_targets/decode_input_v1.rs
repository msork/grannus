#![no_main]

use grannus_protocol::{decode_input_v1, encode_input_v1};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(decoded) = decode_input_v1(data) {
        let encoded = encode_input_v1(&decoded);
        assert_eq!(decode_input_v1(&encoded), Ok(decoded));
    }
});
