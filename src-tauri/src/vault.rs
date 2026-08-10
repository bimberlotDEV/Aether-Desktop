use crate::db::repositories::vault::VaultItem;
use std::fs;
use std::path::{Component, Path, PathBuf};
use uuid::Uuid;

pub struct PreparedVaultFile {
    pub original_name: String,
    pub stored_path: String,
    pub media_type: String,
    pub size_bytes: i64,
    pub managed_directory: Option<PathBuf>,
}

fn path_text(path: &Path, field: &str) -> Result<String, String> {
    path.to_str()
        .map(str::to_string)
        .ok_or_else(|| format!("{} contains unsupported Unicode", field))
}

fn canonical_vault_root(root: &Path) -> Result<PathBuf, String> {
    fs::create_dir_all(root)
        .map_err(|error| format!("Failed to create Vault storage: {}", error))?;
    root.canonicalize()
        .map_err(|error| format!("Failed to resolve Vault storage: {}", error))
}

fn controlled_child_directory(parent: &Path, name: &str) -> Result<PathBuf, String> {
    let directory = parent.join(name);
    fs::create_dir_all(&directory)
        .map_err(|error| format!("Failed to create Vault {} directory: {}", name, error))?;
    let canonical = directory
        .canonicalize()
        .map_err(|error| format!("Failed to resolve Vault {} directory: {}", name, error))?;
    if canonical.parent() != Some(parent) {
        return Err(format!("Vault {} directory escaped Aether storage", name));
    }
    Ok(canonical)
}

fn media_type_for(name: &str) -> &'static str {
    match Path::new(name)
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("pdf") => "application/pdf",
        Some("txt") => "text/plain",
        Some("md" | "markdown") => "text/markdown",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("bmp") => "image/bmp",
        _ => "application/octet-stream",
    }
}

pub fn prepare_import(
    vault_root: &Path,
    id: &str,
    source: &Path,
    storage_mode: &str,
) -> Result<PreparedVaultFile, String> {
    let root = canonical_vault_root(vault_root)?;
    let source = source
        .canonicalize()
        .map_err(|error| format!("Selected file is unavailable: {}", error))?;
    if source.starts_with(&root) {
        return Err("A file already owned by Aether Vault cannot be imported again".to_string());
    }
    let metadata = source
        .metadata()
        .map_err(|error| format!("Could not inspect selected file: {}", error))?;
    if !metadata.is_file() {
        return Err("Vault imports require a regular file".to_string());
    }
    let original_name = source
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "Selected file has no supported filename".to_string())?
        .to_string();
    let size_bytes = i64::try_from(metadata.len())
        .map_err(|_| "Selected file is too large to index".to_string())?;
    let media_type = media_type_for(&original_name).to_string();

    match storage_mode {
        "linked" => Ok(PreparedVaultFile {
            original_name,
            stored_path: path_text(&source, "Selected file path")?,
            media_type,
            size_bytes,
            managed_directory: None,
        }),
        "managed" => {
            let items_root = controlled_child_directory(&root, "items")?;
            let item_directory = items_root.join(id);
            if item_directory.exists() {
                return Err("Vault storage already exists for this item".to_string());
            }
            fs::create_dir_all(&item_directory)
                .map_err(|error| format!("Failed to create managed Vault directory: {}", error))?;
            let item_directory = item_directory
                .canonicalize()
                .map_err(|error| format!("Failed to resolve managed Vault directory: {}", error))?;
            if item_directory.parent() != Some(items_root.as_path()) {
                let _ = fs::remove_dir_all(&item_directory);
                return Err("Managed Vault directory escaped Aether storage".to_string());
            }
            let partial = item_directory.join(".import.partial");
            let target = item_directory.join(&original_name);
            let copy_result = fs::copy(&source, &partial)
                .and_then(|_| fs::rename(&partial, &target))
                .map_err(|error| format!("Failed to copy file into Aether Vault: {}", error));
            if let Err(error) = copy_result {
                let _ = fs::remove_dir_all(&item_directory);
                return Err(error);
            }
            let relative = Path::new("items").join(id).join(&original_name);
            Ok(PreparedVaultFile {
                original_name,
                stored_path: path_text(&relative, "Managed Vault path")?,
                media_type,
                size_bytes,
                managed_directory: Some(item_directory),
            })
        }
        other => Err(format!("Invalid Vault storage mode: {}", other)),
    }
}

fn validate_managed_relative(item: &VaultItem) -> Result<PathBuf, String> {
    let relative = PathBuf::from(&item.stored_path);
    if relative.is_absolute()
        || relative.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err("Managed Vault path is outside Aether storage".to_string());
    }
    let mut components = relative.components();
    let valid_prefix = matches!(components.next(), Some(Component::Normal(value)) if value == "items")
        && matches!(components.next(), Some(Component::Normal(value)) if value == item.id.as_str());
    if !valid_prefix || components.next().is_none() || components.next().is_some() {
        return Err("Managed Vault path does not match its item ownership".to_string());
    }
    Ok(relative)
}

pub fn resolve_item_path(vault_root: &Path, item: &VaultItem) -> Result<PathBuf, String> {
    let candidate = if item.storage_mode == "managed" {
        canonical_vault_root(vault_root)?.join(validate_managed_relative(item)?)
    } else if item.storage_mode == "linked" {
        PathBuf::from(&item.stored_path)
    } else {
        return Err("Vault item has an invalid storage mode".to_string());
    };
    let resolved = candidate
        .canonicalize()
        .map_err(|_| "Vault file is no longer available at its stored location".to_string())?;
    if !resolved.is_file() {
        return Err("Vault item does not resolve to a regular file".to_string());
    }
    if item.storage_mode == "managed" {
        let root = canonical_vault_root(vault_root)?;
        let items_root = controlled_child_directory(&root, "items")?;
        let owned_directory = items_root
            .join(&item.id)
            .canonicalize()
            .map_err(|_| "Managed Vault directory is unavailable".to_string())?;
        if owned_directory.parent() != Some(items_root.as_path()) {
            return Err("Managed Vault directory escaped Aether storage".to_string());
        }
        if !resolved.starts_with(&owned_directory) {
            return Err("Managed Vault path escaped its owned directory".to_string());
        }
    }
    Ok(resolved)
}

pub fn discard_import(directory: Option<&Path>) {
    if let Some(directory) = directory {
        let _ = fs::remove_dir_all(directory);
    }
}

pub fn quarantine_managed(
    vault_root: &Path,
    item: &VaultItem,
) -> Result<Option<(PathBuf, PathBuf)>, String> {
    if item.storage_mode != "managed" {
        return Ok(None);
    }
    validate_managed_relative(item)?;
    let root = canonical_vault_root(vault_root)?;
    let items_root = controlled_child_directory(&root, "items")?;
    let item_directory = items_root.join(&item.id);
    if !item_directory.exists() {
        return Ok(None);
    }
    let canonical_item = item_directory
        .canonicalize()
        .map_err(|error| format!("Failed to resolve managed Vault item: {}", error))?;
    if canonical_item.parent() != Some(items_root.as_path()) {
        return Err("Managed Vault directory escaped Aether storage".to_string());
    }
    let trash = controlled_child_directory(&root, ".trash")?;
    let quarantined = trash.join(format!("{}-{}", item.id, Uuid::now_v7()));
    fs::rename(&canonical_item, &quarantined)
        .map_err(|error| format!("Failed to quarantine managed Vault file: {}", error))?;
    Ok(Some((quarantined, item_directory)))
}

pub fn restore_quarantine(quarantined: &Path, original: &Path) -> Result<(), String> {
    fs::rename(quarantined, original)
        .map_err(|error| format!("Failed to restore quarantined Vault file: {}", error))
}

pub fn finalize_quarantine(vault_root: &Path, quarantined: &Path) -> Result<(), String> {
    let root = canonical_vault_root(vault_root)?;
    let canonical_trash = controlled_child_directory(&root, ".trash")?;
    let canonical_item = quarantined
        .canonicalize()
        .map_err(|error| format!("Failed to resolve quarantined Vault item: {}", error))?;
    if canonical_item.parent() != Some(canonical_trash.as_path()) {
        return Err("Refusing to delete outside the Vault quarantine".to_string());
    }
    fs::remove_dir_all(canonical_item)
        .map_err(|error| format!("Failed to remove quarantined Vault file: {}", error))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn item(id: &str, mode: &str, stored_path: &str) -> VaultItem {
        VaultItem {
            id: id.to_string(),
            space_id: None,
            storage_mode: mode.to_string(),
            display_title: "Document".to_string(),
            original_name: "document.md".to_string(),
            stored_path: stored_path.to_string(),
            media_type: "text/markdown".to_string(),
            size_bytes: 7,
            tags: vec![],
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    #[test]
    fn linked_import_never_owns_source() {
        let temp = TempDir::new().unwrap();
        let vault_root = temp.path().join("vault");
        let source = temp.path().join("outside.md");
        fs::write(&source, "outside").unwrap();
        let prepared = prepare_import(&vault_root, "linked-1", &source, "linked").unwrap();
        assert!(prepared.managed_directory.is_none());
        assert_eq!(
            PathBuf::from(prepared.stored_path),
            source.canonicalize().unwrap()
        );
        let linked_item = item(
            "linked-1",
            "linked",
            source.canonicalize().unwrap().to_str().unwrap(),
        );
        assert!(quarantine_managed(&vault_root, &linked_item)
            .unwrap()
            .is_none());
        assert_eq!(fs::read_to_string(&source).unwrap(), "outside");
    }

    #[test]
    fn managed_import_copies_and_resolves_inside_owned_directory() {
        let temp = TempDir::new().unwrap();
        let vault_root = temp.path().join("vault");
        let source = temp.path().join("source.md");
        fs::write(&source, "managed").unwrap();
        let prepared = prepare_import(&vault_root, "managed-1", &source, "managed").unwrap();
        let vault_item = item("managed-1", "managed", &prepared.stored_path);
        let resolved = resolve_item_path(&vault_root, &vault_item).unwrap();
        assert!(resolved.starts_with(
            vault_root
                .join("items")
                .join("managed-1")
                .canonicalize()
                .unwrap()
        ));
        assert_eq!(fs::read_to_string(resolved).unwrap(), "managed");
        assert_eq!(fs::read_to_string(source).unwrap(), "managed");
    }

    #[test]
    fn rejects_reimport_and_tampered_managed_paths() {
        let temp = TempDir::new().unwrap();
        let vault_root = temp.path().join("vault");
        fs::create_dir_all(&vault_root).unwrap();
        let inside = vault_root.join("inside.txt");
        fs::write(&inside, "inside").unwrap();
        assert!(prepare_import(&vault_root, "bad", &inside, "linked").is_err());
        let tampered = item("bad", "managed", "../outside.txt");
        assert!(resolve_item_path(&vault_root, &tampered).is_err());
        assert!(quarantine_managed(&vault_root, &tampered).is_err());
    }

    #[test]
    fn quarantine_can_restore_or_finalize_only_owned_directory() {
        let temp = TempDir::new().unwrap();
        let vault_root = temp.path().join("vault");
        let source = temp.path().join("source.md");
        fs::write(&source, "managed").unwrap();
        let prepared = prepare_import(&vault_root, "managed-1", &source, "managed").unwrap();
        let vault_item = item("managed-1", "managed", &prepared.stored_path);

        let (quarantined, original) = quarantine_managed(&vault_root, &vault_item)
            .unwrap()
            .unwrap();
        assert!(!original.exists());
        restore_quarantine(&quarantined, &original).unwrap();
        assert!(original.exists());

        let (quarantined, _) = quarantine_managed(&vault_root, &vault_item)
            .unwrap()
            .unwrap();
        finalize_quarantine(&vault_root, &quarantined).unwrap();
        assert!(!quarantined.exists());
        assert!(source.exists());
    }
}
