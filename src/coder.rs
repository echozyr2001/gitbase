use bech32::{decode, encode, Hrp};
use error_stack::ResultExt;
use sha3::{Digest, Sha3_256};

use crate::error::{CoderError, CoderResult};

pub const DOC_PREFIX: &str = "gbdoc";
pub const COL_PREFIX: &str = "gbcol";
pub const IDX_PREFIX: &str = "gbidx";

/// Compute SHA256 hash
fn sha256_hash(input: &str) -> Vec<u8> {
    let mut hasher = Sha3_256::new();
    hasher.update(input);
    hasher.finalize().to_vec()
}

/// Compute Blake3 hash
fn blake3_hash(input: &str) -> Vec<u8> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(input.as_bytes());
    hasher.finalize().as_bytes()[..10].to_vec() // Take the first 10 bytes to avoid excessive length
}

/// Generate Bech32 ID
fn generate_bech32_id(hrp: &str, data: &[u8]) -> CoderResult<String> {
    let hrp = Hrp::parse(hrp)
        .change_context(CoderError::InvalidHRP)
        .attach_printable("HRP parsing failed")?;

    let encoded = encode::<bech32::Bech32m>(hrp, data)
        .change_context_lazy(|| CoderError::EncodingError("encoding failed".to_string()))
        .attach_printable("Bech32 encoding failed")?;
    Ok(encoded)
}

/// Generate document ID
pub fn generate_document_id(content: &str, timestamp: u64) -> CoderResult<String> {
    let hash = sha256_hash(&format!("{}{}", content, timestamp));
    generate_bech32_id(DOC_PREFIX, &hash[..10]) // Take the first 10 bytes
}

/// Generate collection ID
pub fn generate_collection_id(collection_name: &str) -> CoderResult<String> {
    let hash = blake3_hash(collection_name);
    generate_bech32_id(COL_PREFIX, &hash)
}

/// Generate index ID
pub fn generate_index_id(index_name: &str, collection_name: &str) -> CoderResult<String> {
    let hash = sha256_hash(&format!("{}{}", index_name, collection_name));
    generate_bech32_id(IDX_PREFIX, &hash[..10])
}

/// Decode Bech32 ID
pub fn decode_bech32_id(encoded: &str) -> CoderResult<(String, Vec<u8>)> {
    let (hrp, data) = decode(encoded)
        .change_context_lazy(|| CoderError::DecodingError("decoding failed".to_string()))
        .attach_printable("Bech32 decoding failed")?;

    Ok((hrp.to_string(), data))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_document_id() {
        let content = r#"{"title": "Hello", "body": "GitBase"}"#;
        let timestamp = 1700000000;
        let doc_id = generate_document_id(content, timestamp).unwrap();

        assert!(doc_id.starts_with(DOC_PREFIX));

        let (hrp, data) = decode(&doc_id).unwrap();
        assert_eq!(hrp, Hrp::parse(DOC_PREFIX).unwrap());
        assert_eq!(data.len(), 10);
    }

    #[test]
    fn test_generate_collection_id() {
        let collection_name = "notes";
        let col_id = generate_collection_id(collection_name).unwrap();

        assert!(col_id.starts_with(COL_PREFIX));

        let (hrp, data) = decode(&col_id).unwrap();
        assert_eq!(hrp, Hrp::parse(COL_PREFIX).unwrap());
        assert_eq!(data.len(), 10);
    }

    #[test]
    fn test_generate_index_id() {
        let index_name = "by_date";
        let collection_name = "notes";
        let idx_id = generate_index_id(index_name, collection_name).unwrap();

        assert!(idx_id.starts_with(IDX_PREFIX));

        let (hrp, data) = decode(&idx_id).unwrap();
        assert_eq!(hrp, Hrp::parse(IDX_PREFIX).unwrap());
        assert_eq!(data.len(), 10);
    }

    #[test]
    fn test_decode_bech32_id() {
        let encoded = "gbdoc1p05pynsthd39yw2d6yn44l";
        let (hrp, data) = decode_bech32_id(encoded).unwrap();

        assert_eq!(hrp, DOC_PREFIX);
        assert_eq!(data.len(), 10);

        let reencoded = encode::<bech32::Bech32m>(Hrp::parse(DOC_PREFIX).unwrap(), &data).unwrap();
        assert_eq!(reencoded, encoded);
    }
}
