use percent_encoding::{percent_encode, AsciiSet, CONTROLS};

const PATH_SEGMENT_ENCODE_SET: &AsciiSet = &CONTROLS
    // "QUERY_ENCODE_SET" additions:
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'<')
    .add(b'>')
    // "DEFAULT_ENCODE_SET" additions:
    .add(b'`')
    .add(b'?')
    .add(b'{')
    .add(b'}')
    // "PATH_SEGMENT_ENCODE_SET" additions
    .add(b'%')
    .add(b'/')
    // The following were NOT in PATH_SEGMENT but are URI reserved characters not covered above.
    // ':' and '@' are explicitly permitted in paths, so we don't add them.
    .add(b'[')
    .add(b']');

pub fn url_encode_key(key: &str) -> String {
    percent_encode(key.as_bytes(), PATH_SEGMENT_ENCODE_SET).to_string()
}
