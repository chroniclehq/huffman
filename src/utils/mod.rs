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
