use anyhow::{Context, Result};
use bech32::{decode, encode, Hrp};
use sha3::{Digest, Sha3_256};

pub const DOC_PREFIX: &str = "gbdoc";
pub const COL_PREFIX: &str = "gbcol";
pub const IDX_PREFIX: &str = "gbidx";

/// 计算 SHA256 哈希
fn sha256_hash(input: &str) -> Vec<u8> {
    let mut hasher = Sha3_256::new();
    hasher.update(input);
    hasher.finalize().to_vec()
}

/// 计算 Blake3 哈希
fn blake3_hash(input: &str) -> Vec<u8> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(input.as_bytes());
    hasher.finalize().as_bytes()[..10].to_vec() // 取前10字节，避免过长
}

/// 生成 Bech32 ID
fn generate_bech32_id(hrp: &str, data: &[u8]) -> Result<String> {
    let hrp = Hrp::parse(hrp).context("valid hrp")?;
    let encoded = encode::<bech32::Bech32m>(hrp, data).context("Failed to encode Bech32")?;
    Ok(encoded)
}

/// 生成文档 ID
pub fn generate_document_id(content: &str, timestamp: u64) -> Result<String> {
    let hash = sha256_hash(&format!("{}{}", content, timestamp));
    generate_bech32_id(DOC_PREFIX, &hash[..10]) // 取前10字节
}

/// 生成集合 ID
pub fn generate_collection_id(collection_name: &str) -> Result<String> {
    let hash = blake3_hash(collection_name);
    generate_bech32_id(COL_PREFIX, &hash)
}

/// 生成索引 ID
pub fn generate_index_id(index_name: &str, collection_name: &str) -> Result<String> {
    let hash = sha256_hash(&format!("{}{}", index_name, collection_name));
    generate_bech32_id(IDX_PREFIX, &hash[..10])
}

/// 解码 Bech32 ID
pub fn decode_bech32_id(encoded: &str) -> Result<(String, Vec<u8>)> {
    let (hrp, data) = decode(encoded).context("Failed to decode Bech32")?;
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
