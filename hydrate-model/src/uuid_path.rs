use std::path::{Path, PathBuf};
use uuid::Uuid;

pub fn uuid_to_path(
    root: &Path,
    uuid: Uuid,
    extension: &str,
) -> PathBuf {
    // Convert UUID to a 32-character hex string (no hyphens)
    // example: 8cf25195abd839981ea3c93c8fd2843f
    let mut buffer = [0; 32];
    let encoded = uuid.to_simple().encode_lower(&mut buffer).to_string();
    // Produce path like [root]/8/cf/25195abd839981ea3c93c8fd2843f
    root.join(&encoded[0..1]).join(&encoded[1..3]).join(format!(
        "{}.{}",
        &encoded[3..32],
        extension
    ))
}

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

    // Produce path like [root]/8/cf/25195abd839981ea3c93c8fd2843f
    root.join(&uuid_encoded[0..1])
        .join(&uuid_encoded[1..3])
        .join(format!("{}-{:x}.{}", &uuid_encoded[3..32], hash, extension))
}

pub fn path_to_uuid(
    root: &Path,
    file_path: &Path,
) -> Option<Uuid> {
    // Remove root from the path
    let relative_path_from_root = file_path.strip_prefix(root).ok()?;

    // We append the path into this string
    let mut path_and_name = String::with_capacity(32);

    // Split the path by directory paths
    let components: Vec<_> = relative_path_from_root.components().collect();

    // Iterate all segments of the path except the last one
    if components.len() > 1 {
        for component in components[0..components.len() - 1].iter() {
            path_and_name.push_str(&component.as_os_str().to_str().unwrap());
        }
    }

    // Append the last segment, removing the extension if there is one
    if let Some(last_component) = components.last() {
        let mut last_str = last_component.as_os_str().to_str()?;

        // Remove the extension
        if let Some(extension_begin) = last_str.find('.') {
            last_str = last_str.strip_suffix(&last_str[extension_begin..]).unwrap();
        }

        // Add zero padding between dirs (which should be highest order bits) and filename
        //TODO: Maybe just assert all the component lengths are as expected
        let str_len = path_and_name.len() + last_str.len();
        if str_len < 32 {
            path_and_name.push_str(&"0".repeat(32 - str_len));
        }

        path_and_name.push_str(last_str);
    }

    u128::from_str_radix(&path_and_name, 16)
        .ok()
        .map(|x| Uuid::from_u128(x))
}

pub fn path_to_uuid_and_hash(
    root: &Path,
    file_path: &Path,
) -> Option<(Uuid, u64)> {
    // Remove root from the path
    let relative_path_from_root = file_path.strip_prefix(root).ok()?;

    // We append the path into this string
    let mut path_and_name = String::with_capacity(32);

    // Split the path by directory paths
    let components: Vec<_> = relative_path_from_root.components().collect();

    // Iterate all segments of the path except the last one
    if components.len() > 1 {
        for component in components[0..components.len() - 1].iter() {
            path_and_name.push_str(&component.as_os_str().to_str().unwrap());
        }
    }

    let mut hash = 0;

    // Append the last segment, removing the extension if there is one
    if let Some(last_component) = components.last() {
        let mut last_str = last_component.as_os_str().to_str()?;

        // Remove the extension
        if let Some(extension_begin) = last_str.find('.') {
            last_str = last_str.strip_suffix(&last_str[extension_begin..]).unwrap();
        }

        if let Some(hash_begin) = last_str.find('-') {
            hash = u64::from_str_radix(&last_str[hash_begin..], 16).ok()?;
            last_str = last_str.strip_suffix(&last_str[hash_begin..]).unwrap();
        }

        // Add zero padding between dirs (which should be highest order bits) and filename
        //TODO: Maybe just assert all the component lengths are as expected
        let str_len = path_and_name.len() + last_str.len();
        if str_len < 32 {
            path_and_name.push_str(&"0".repeat(32 - str_len));
        }

        path_and_name.push_str(last_str);
    }

    let uuid = u128::from_str_radix(&path_and_name, 16)
        .ok()
        .map(|x| Uuid::from_u128(x))?;

    Some((uuid, hash))
}
