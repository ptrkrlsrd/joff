use percent_encoding::{utf8_percent_encode, percent_decode, AsciiSet, CONTROLS};

const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');

pub fn encode(url: &String) -> String {
    let encoded_url_iter = utf8_percent_encode(&url, FRAGMENT);
    let encoded_url: String = encoded_url_iter.collect();

    return encoded_url;
}

pub fn decode(url: &str) -> Result<String, std::str::Utf8Error> {
    let decoded_iter = percent_decode(url.as_bytes());
    let decoded = decoded_iter.decode_utf8();

    return match decoded {
        Ok(url) => Ok(url.to_string()),
        Err(error) => Err(error),
    };
}