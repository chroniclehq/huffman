use anyhow::{anyhow, Result};

pub mod http;

pub fn get_path_without_ext(path: &str) -> &str {
    match path.rsplit_once('.') {
        Some((new_path, _)) => new_path,
        None => path,
    }
}

pub fn get_ext_from_path(path: &str) -> Option<&str> {
    match path.rsplit_once('.') {
        Some((_, ext)) => Some(ext),
        None => None,
    }
}

pub fn is_allowed_type(ext: &str) -> Result<()> {
    let allowed_ext = ["jpeg", "avif", "jpg", "jpeg", "png", "webp"];
    if allowed_ext.contains(&ext) {
        return Ok(());
    } else {
        return Err(anyhow!("Unsupported file format"));
    }
}
