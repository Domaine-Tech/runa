use std::fs::File;
use std::io::Read;
use std::path::Path;

use shared_types::RunaError;

/// Hash blake3 hexadécimal (intégrité modèle / contenu).
pub fn blake3_hex(data: &[u8]) -> String {
    blake3::hash(data).to_hex().to_string()
}

/// Hash blake3 d'un fichier (lecture par chunks).
pub fn blake3_file(path: &Path) -> Result<String, RunaError> {
    let mut file = File::open(path).map_err(|e| RunaError::Internal {
        message: format!("ouverture {}: {e}", path.display()),
    })?;
    let mut hasher = blake3::Hasher::new();
    let mut buf = [0_u8; 64 * 1024];
    loop {
        let n = file.read(&mut buf).map_err(|e| RunaError::Internal {
            message: format!("lecture {}: {e}", path.display()),
        })?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hasher.finalize().to_hex().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn stable_hash() {
        let a = blake3_hex(b"runa");
        let b = blake3_hex(b"runa");
        assert_eq!(a, b);
        assert_ne!(a, blake3_hex(b"other"));
    }

    #[test]
    fn file_hash_matches_buffer() {
        let dir = std::env::temp_dir().join("runa_blake3_test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("sample.txt");
        {
            let mut f = File::create(&path).expect("create");
            f.write_all(b"runa-model").expect("write");
        }
        let from_file = blake3_file(&path).expect("hash file");
        let from_buf = blake3_hex(b"runa-model");
        assert_eq!(from_file, from_buf);
        let _ = std::fs::remove_file(&path);
    }
}
