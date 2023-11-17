use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Converts a UUID to a path starting at the given root and with the given extension
/// Example: /2/d/2d4154f72b3c422387677e8d1fa70447.af
pub fn uuid_to_path(
    root: &Path,
    uuid: Uuid,
    extension: &str,
) -> PathBuf {
    // Convert UUID to a 32-character hex string (no hyphens)
    // example: 8cf25195abd839981ea3c93c8fd2843f
    let mut buffer = [0; 32];
    let encoded = uuid.to_simple().encode_lower(&mut buffer).to_string();
    // Produce path like [root]/8/c/8cf25195abd839981ea3c93c8fd2843f
    root.join(&encoded[0..1]).join(&encoded[1..2]).join(format!(
        "{}.{}",
        &encoded[0..32],
        extension
    ))
}

/// Converts a UUID to a path starting at the given root and with the given extension
/// and appends a u64 hash of the contents
/// Example: /2/d/2d41f453d6224b2fab9bc8021a6c7dde-45647afbadf0c93d.bf
pub fn uuid_and_hash_to_path(
    root: &Path,
    uuid: Uuid,
    hash: u64,
    extension: &str,
) -> PathBuf {
    // Convert UUID to a 32-character hex string (no hyphens)
    // example: 8cf25195abd839981ea3c93c8fd2843f
    let mut buffer = [0; 32];
    let uuid_encoded = uuid.to_simple().encode_lower(&mut buffer).to_string();

    // Produce path like [root]/8/c/8cf25195abd839981ea3c93c8fd2843f
    root.join(&uuid_encoded[0..1])
        .join(&uuid_encoded[1..2])
        .join(format!("{}-{:x}.{}", &uuid_encoded[0..32], hash, extension))
}

/// Converts a path within a root to a UUID
pub fn path_to_uuid(
    root: &Path,
    file_path: &Path,
) -> Option<Uuid> {
    // Remove root from the path
    let relative_path_from_root = file_path.strip_prefix(root).ok()?;

    // Split the path by directory paths
    let components: Vec<_> = relative_path_from_root.components().collect();

    let mut filename = components[2].as_os_str().to_str().unwrap();

    if components.len() != 3 {
        return None;
    }

    if components[0].as_os_str().to_str().unwrap().as_bytes()[0] != filename.as_bytes()[0] {
        return None;
    }

    if components[1].as_os_str().to_str().unwrap().as_bytes()[0] != filename.as_bytes()[1] {
        return None;
    }

    // Remove the extension
    if let Some(extension_begin) = filename.find('.') {
        filename = filename.strip_suffix(&filename[extension_begin..]).unwrap();
    }

    u128::from_str_radix(&filename, 16)
        .ok()
        .map(|x| Uuid::from_u128(x))
}

/// Converts a path within a root to a UUID + u64 hash
pub fn path_to_uuid_and_hash(
    root: &Path,
    file_path: &Path,
) -> Option<(Uuid, u64)> {
    // Remove root from the path
    let relative_path_from_root = file_path.strip_prefix(root).ok()?;

    // Split the path by directory paths
    let components: Vec<_> = relative_path_from_root.components().collect();

    let mut filename = components[2].as_os_str().to_str().unwrap();

    if components.len() != 3 {
        return None;
    }

    if components[0].as_os_str().to_str().unwrap().as_bytes()[0] != filename.as_bytes()[0] {
        return None;
    }

    if components[1].as_os_str().to_str().unwrap().as_bytes()[0] != filename.as_bytes()[1] {
        return None;
    }

    // Remove the extension
    if let Some(extension_begin) = filename.find('.') {
        filename = filename.strip_suffix(&filename[extension_begin..]).unwrap();
    }

    if let Some(hash_begin) = filename.find('-') {
        let hash = u64::from_str_radix(&filename[(hash_begin + 1)..], 16).ok()?;

        let uuid = u128::from_str_radix(&filename[0..hash_begin], 16)
            .ok()
            .map(|x| Uuid::from_u128(x))?;

        Some((uuid, hash))
    } else {
        None
    }
}
