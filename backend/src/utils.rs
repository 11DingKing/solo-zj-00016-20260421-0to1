use rand::Rng;
use uuid::Uuid;

const BASE62_CHARS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

pub fn generate_short_code(length: usize) -> String {
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..BASE62_CHARS.len());
            BASE62_CHARS[idx] as char
        })
        .collect()
}

pub fn generate_user_cookie() -> String {
    Uuid::new_v4().to_string().replace("-", "")
}

pub fn validate_url(url: &str) -> bool {
    if url.is_empty() {
        return false;
    }
    
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return false;
    }
    
    url.len() <= 2048
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_short_code() {
        let code = generate_short_code(6);
        assert_eq!(code.len(), 6);
        for c in code.chars() {
            assert!(BASE62_CHARS.contains(&(c as u8)));
        }
    }

    #[test]
    fn test_validate_url() {
        assert!(validate_url("https://example.com"));
        assert!(validate_url("http://test.org/path?query=1"));
        assert!(!validate_url("ftp://example.com"));
        assert!(!validate_url("example.com"));
        assert!(!validate_url(""));
    }

    #[test]
    fn test_generate_user_cookie() {
        let cookie = generate_user_cookie();
        assert_eq!(cookie.len(), 32);
    }
}
