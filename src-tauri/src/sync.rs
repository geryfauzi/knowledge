use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::drive::{
    download_drive_file, ensure_drive_folder, list_drive_files, trash_drive_file,
    upload_drive_file, DriveItem,
};
use crate::oauth::get_valid_access_token;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LedgerFileEntry {
    pub drive_id: String,
    pub sha256: String,
    pub last_synced: i64,
    #[serde(default)]
    pub remote_md5: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SyncLedger {
    pub drive_root_id: Option<String>,
    pub files: HashMap<String, LedgerFileEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SyncSummary {
    pub uploaded: usize,
    pub downloaded: usize,
    pub conflicts: usize,
    pub deleted: usize,
    pub errors: Vec<String>,
}

fn get_ledger_path(vault_path: &str) -> PathBuf {
    Path::new(vault_path).join(".knowledge").join("sync_ledger.json")
}

fn load_ledger(vault_path: &str) -> SyncLedger {
    let path = get_ledger_path(vault_path);
    if !path.exists() {
        return SyncLedger::default();
    }
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|data| serde_json::from_str(&data).ok())
        .unwrap_or_default()
}

fn save_ledger(vault_path: &str, ledger: &SyncLedger) -> Result<(), String> {
    let path = get_ledger_path(vault_path);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let json = serde_json::to_string_pretty(ledger).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| format!("Failed to write ledger: {}", e))?;
    Ok(())
}

fn calculate_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

pub fn parse_iso8601_to_epoch(iso: &str) -> Option<i64> {
    if iso.len() < 19 {
        return None;
    }
    let year: i32 = iso[0..4].parse().ok()?;
    let month: u32 = iso[5..7].parse().ok()?;
    let day: u32 = iso[8..10].parse().ok()?;
    let hour: u32 = iso[11..13].parse().ok()?;
    let minute: u32 = iso[14..16].parse().ok()?;
    let second: u32 = iso[17..19].parse().ok()?;

    let days_before_month = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    if month < 1 || month > 12 || day < 1 || day > 31 {
        return None;
    }

    let mut days = (year - 1970) as i64 * 365;
    for y in 1970..year {
        if (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0) {
            days += 1;
        }
    }
    let is_leap_current = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
    let mut month_days = days_before_month[(month - 1) as usize];
    if month > 2 && is_leap_current {
        month_days += 1;
    }
    days += month_days as i64;
    days += (day - 1) as i64;

    let seconds = days * 86400 + (hour as i64) * 3600 + (minute as i64) * 60 + (second as i64);
    Some(seconds)
}

fn normalize_relative_path(path: &Path, base: &Path) -> String {
    path.strip_prefix(base)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn collect_local_files(dir: &Path, base: &Path, list: &mut Vec<(String, PathBuf)>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            // Skip hidden directories and non-markdown files
            if name.starts_with('.') || name == "node_modules" || name == "target" {
                continue;
            }

            if path.is_dir() {
                collect_local_files(&path, base, list);
            } else if name.ends_with(".md") {
                let rel = normalize_relative_path(&path, base);
                list.push((rel, path));
            }
        }
    }
}

async fn get_or_create_subfolder(
    token: &str,
    root_id: &str,
    folder_path: &str,
    cache: &mut HashMap<String, String>,
) -> Result<String, String> {
    if folder_path.is_empty() || folder_path == "." {
        return Ok(root_id.to_string());
    }

    if let Some(id) = cache.get(folder_path) {
        return Ok(id.clone());
    }

    let parts: Vec<&str> = folder_path.split('/').filter(|p| !p.is_empty()).collect();
    let mut current_parent = root_id.to_string();
    let mut built_path = String::new();

    for part in parts {
        if !built_path.is_empty() {
            built_path.push('/');
        }
        built_path.push_str(part);

        if let Some(cached_id) = cache.get(&built_path) {
            current_parent = cached_id.clone();
        } else {
            let folder_id = ensure_drive_folder(token, part, Some(&current_parent)).await?;
            cache.insert(built_path.clone(), folder_id.clone());
            current_parent = folder_id;
        }
    }

    Ok(current_parent)
}

#[tauri::command]
pub async fn google_sync_now(
    app: tauri::AppHandle,
    vault_path: String,
    folder_name: Option<String>,
) -> Result<SyncSummary, String> {
    let token = get_valid_access_token(&app).await?;
    let target_folder_name = folder_name.unwrap_or_else(|| "Knowledge-Vault".to_string());

    let mut ledger = load_ledger(&vault_path);
    let root_id = ensure_drive_folder(&token, &target_folder_name, None).await?;
    ledger.drive_root_id = Some(root_id.clone());

    let mut summary = SyncSummary::default();
    let mut folder_cache: HashMap<String, String> = HashMap::new();

    let vault_base = Path::new(&vault_path);
    let mut local_files = Vec::new();
    collect_local_files(vault_base, vault_base, &mut local_files);

    let local_file_map: HashMap<String, PathBuf> = local_files.into_iter().collect();

    // 1. Scan remote Drive files recursively
    let mut remote_items_by_rel_path: HashMap<String, DriveItem> = HashMap::new();
    collect_remote_files(&token, &root_id, &mut remote_items_by_rel_path, &mut folder_cache).await?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    // 2. Process local files: Upload new / modified
    for (rel_path, abs_path) in &local_file_map {
        let content = match std::fs::read(abs_path) {
            Ok(c) => c,
            Err(e) => {
                summary.errors.push(format!("Failed to read {}: {}", rel_path, e));
                continue;
            }
        };

        let local_hash = calculate_sha256(&content);
        let parent_folder_rel = Path::new(rel_path)
            .parent()
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_default();
        let filename = Path::new(rel_path)
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| rel_path.clone());

        let remote_item = remote_items_by_rel_path.get(rel_path);

        let local_mtime = std::fs::metadata(abs_path)
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let remote_mtime = remote_item
            .as_ref()
            .and_then(|r| r.modified_time.as_deref())
            .and_then(parse_iso8601_to_epoch)
            .unwrap_or(0);

        if let Some(entry) = ledger.files.get(rel_path) {
            let local_changed = entry.sha256 != local_hash;
            let remote_changed = if let Some(remote) = remote_item {
                if let Some(stored_md5) = &entry.remote_md5 {
                    remote.md5_checksum.as_ref() != Some(stored_md5)
                } else {
                    remote_mtime > entry.last_synced + 2
                }
            } else {
                false
            };

            if !local_changed && !remote_changed {
                // In sync! If remote_md5 was missing from ledger, populate it
                if entry.remote_md5.is_none() {
                    if let Some(remote) = remote_item {
                        if remote.md5_checksum.is_some() {
                            let mut updated_entry = entry.clone();
                            updated_entry.remote_md5 = remote.md5_checksum.clone();
                            ledger.files.insert(rel_path.clone(), updated_entry);
                        }
                    }
                }
                continue;
            } else if !local_changed && remote_changed {
                // Remote modified on Drive -> Download and update local file!
                if let Some(remote) = remote_item {
                    match download_drive_file(&token, &remote.id).await {
                        Ok(text) => {
                            if let Ok(()) = std::fs::write(abs_path, text.as_bytes()) {
                                let hash = calculate_sha256(text.as_bytes());
                                ledger.files.insert(
                                    rel_path.clone(),
                                    LedgerFileEntry {
                                        drive_id: remote.id.clone(),
                                        sha256: hash,
                                        last_synced: now,
                                        remote_md5: remote.md5_checksum.clone(),
                                    },
                                );
                                summary.downloaded += 1;
                            }
                        }
                        Err(e) => summary.errors.push(e),
                    }
                }
                continue;
            } else if local_changed && !remote_changed {
                // Local modified -> Upload to Google Drive!
                let drive_parent_id = match get_or_create_subfolder(
                    &token,
                    &root_id,
                    &parent_folder_rel,
                    &mut folder_cache,
                )
                .await
                {
                    Ok(id) => id,
                    Err(e) => {
                        summary.errors.push(e);
                        continue;
                    }
                };

                let target_drive_id = remote_item.map(|r| r.id.as_str()).unwrap_or(&entry.drive_id);
                match upload_drive_file(&token, &filename, &content, &drive_parent_id, Some(target_drive_id)).await {
                    Ok(drive_id) => {
                        ledger.files.insert(
                            rel_path.clone(),
                            LedgerFileEntry {
                                drive_id,
                                sha256: local_hash,
                                last_synced: now,
                                remote_md5: None,
                            },
                        );
                        summary.uploaded += 1;
                    }
                    Err(e) => summary.errors.push(e),
                }
                continue;
            } else {
                // Conflict: Both modified! Last-Modified-Wins with Conflict Backup
                summary.conflicts += 1;
                if let Some(remote) = remote_item {
                    let timestamp = chrono_lite_timestamp();
                    if local_mtime >= remote_mtime {
                        // Local is newer: backup remote, upload local
                        let conflict_rel = rel_path.replace(".md", &format!(" (Drive Conflict {}).md", timestamp));
                        let conflict_abs = vault_base.join(&conflict_rel);
                        if let Ok(remote_content) = download_drive_file(&token, &remote.id).await {
                            let _ = std::fs::write(&conflict_abs, remote_content);
                        }

                        let drive_parent_id = match get_or_create_subfolder(
                            &token,
                            &root_id,
                            &parent_folder_rel,
                            &mut folder_cache,
                        )
                        .await
                        {
                            Ok(id) => id,
                            Err(e) => {
                                summary.errors.push(e);
                                continue;
                            }
                        };

                        match upload_drive_file(&token, &filename, &content, &drive_parent_id, Some(&remote.id)).await {
                            Ok(drive_id) => {
                                ledger.files.insert(
                                    rel_path.clone(),
                                    LedgerFileEntry {
                                        drive_id,
                                        sha256: local_hash,
                                        last_synced: now,
                                        remote_md5: None,
                                    },
                                );
                                summary.uploaded += 1;
                            }
                            Err(e) => summary.errors.push(e),
                        }
                    } else {
                        // Remote is newer: backup local, download remote
                        let conflict_rel = rel_path.replace(".md", &format!(" (Local Conflict {}).md", timestamp));
                        let conflict_abs = vault_base.join(&conflict_rel);
                        let _ = std::fs::write(&conflict_abs, &content);

                        match download_drive_file(&token, &remote.id).await {
                            Ok(text) => {
                                if let Ok(()) = std::fs::write(abs_path, text.as_bytes()) {
                                    let hash = calculate_sha256(text.as_bytes());
                                    ledger.files.insert(
                                        rel_path.clone(),
                                        LedgerFileEntry {
                                            drive_id: remote.id.clone(),
                                            sha256: hash,
                                            last_synced: now,
                                            remote_md5: remote.md5_checksum.clone(),
                                        },
                                    );
                                    summary.downloaded += 1;
                                }
                            }
                            Err(e) => summary.errors.push(e),
                        }
                    }
                }
                continue;
            }
        } else {
            // New local file (not in ledger)
            let drive_parent_id = match get_or_create_subfolder(
                &token,
                &root_id,
                &parent_folder_rel,
                &mut folder_cache,
            )
            .await
            {
                Ok(id) => id,
                Err(e) => {
                    summary.errors.push(e);
                    continue;
                }
            };

            if let Some(remote) = remote_item {
                // Exists on both sides, not yet in ledger: resolve by timestamp
                if local_mtime >= remote_mtime {
                    match upload_drive_file(&token, &filename, &content, &drive_parent_id, Some(&remote.id)).await {
                        Ok(drive_id) => {
                            ledger.files.insert(
                                rel_path.clone(),
                                LedgerFileEntry {
                                    drive_id,
                                    sha256: local_hash,
                                    last_synced: now,
                                    remote_md5: None,
                                },
                            );
                            summary.uploaded += 1;
                        }
                        Err(e) => summary.errors.push(e),
                    }
                } else {
                    match download_drive_file(&token, &remote.id).await {
                        Ok(text) => {
                            if let Ok(()) = std::fs::write(abs_path, text.as_bytes()) {
                                let hash = calculate_sha256(text.as_bytes());
                                ledger.files.insert(
                                    rel_path.clone(),
                                    LedgerFileEntry {
                                        drive_id: remote.id.clone(),
                                        sha256: hash,
                                        last_synced: now,
                                        remote_md5: remote.md5_checksum.clone(),
                                    },
                                );
                                summary.downloaded += 1;
                            }
                        }
                        Err(e) => summary.errors.push(e),
                    }
                }
            } else {
                // Exists only locally: upload to Drive
                match upload_drive_file(&token, &filename, &content, &drive_parent_id, None).await {
                    Ok(drive_id) => {
                        ledger.files.insert(
                            rel_path.clone(),
                            LedgerFileEntry {
                                drive_id,
                                sha256: local_hash,
                                last_synced: now,
                                remote_md5: None,
                            },
                        );
                        summary.uploaded += 1;
                    }
                    Err(e) => summary.errors.push(e),
                }
            }
        }
    }

    // 3. Process remote files: Download new files from Drive
    for (rel_path, remote_item) in &remote_items_by_rel_path {
        if !local_file_map.contains_key(rel_path) {
            // File exists on remote but not locally
            if ledger.files.contains_key(rel_path) {
                // File was deleted locally! Trash on Drive.
                if let Err(e) = trash_drive_file(&token, &remote_item.id).await {
                    summary.errors.push(e);
                } else {
                    ledger.files.remove(rel_path);
                    summary.deleted += 1;
                }
            } else {
                // New remote file! Download to local.
                match download_drive_file(&token, &remote_item.id).await {
                    Ok(text) => {
                        let target_path = vault_base.join(rel_path);
                        if let Some(parent) = target_path.parent() {
                            let _ = std::fs::create_dir_all(parent);
                        }
                        if let Ok(()) = std::fs::write(&target_path, text.as_bytes()) {
                            let hash = calculate_sha256(text.as_bytes());
                            ledger.files.insert(
                                rel_path.clone(),
                                LedgerFileEntry {
                                    drive_id: remote_item.id.clone(),
                                    sha256: hash,
                                    last_synced: now,
                                    remote_md5: remote_item.md5_checksum.clone(),
                                },
                            );
                            summary.downloaded += 1;
                        }
                    }
                    Err(e) => summary.errors.push(e),
                }
            }
        }
    }

    // 4. Clean up remote folders that no longer exist locally and have no active files
    let mut remote_folders: Vec<(String, String)> = folder_cache.into_iter().collect();
    // Sort deepest first so child folders are processed before parents
    remote_folders.sort_by(|a, b| b.0.split('/').count().cmp(&a.0.split('/').count()));

    for (folder_rel, folder_id) in remote_folders {
        let local_dir = vault_base.join(&folder_rel);
        if !local_dir.exists() {
            let prefix = format!("{}/", folder_rel);
            let has_active_files = ledger.files.keys().any(|f| f.starts_with(&prefix));
            if !has_active_files {
                let _ = trash_drive_file(&token, &folder_id).await;
            }
        }
    }

    // 5. Purge stale files from ledger
    ledger.files.retain(|rel_path, _| {
        local_file_map.contains_key(rel_path) || remote_items_by_rel_path.contains_key(rel_path)
    });

    let _ = save_ledger(&vault_path, &ledger);
    Ok(summary)
}

#[tauri::command]
pub async fn google_sync_file(
    app: tauri::AppHandle,
    vault_path: String,
    file_path: String,
) -> Result<(), String> {
    let token = match get_valid_access_token(&app).await {
        Ok(t) => t,
        Err(_) => return Ok(()), // Silently skip if user is not logged in
    };

    let vault_base = Path::new(&vault_path);
    let target_file = Path::new(&file_path);

    if !target_file.exists() || !file_path.ends_with(".md") {
        return Ok(());
    }

    let rel_path = normalize_relative_path(target_file, vault_base);
    let content = std::fs::read(target_file).map_err(|e| e.to_string())?;
    let local_hash = calculate_sha256(&content);

    let mut ledger = load_ledger(&vault_path);
    let root_id = if let Some(id) = &ledger.drive_root_id {
        id.clone()
    } else {
        let id = ensure_drive_folder(&token, "Knowledge-Vault", None).await?;
        ledger.drive_root_id = Some(id.clone());
        id
    };

    let parent_folder_rel = Path::new(&rel_path)
        .parent()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_default();
    let filename = Path::new(&rel_path)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| rel_path.clone());

    let mut folder_cache = HashMap::new();
    let parent_id = get_or_create_subfolder(&token, &root_id, &parent_folder_rel, &mut folder_cache).await?;

    let existing_drive_id = ledger.files.get(&rel_path).map(|e| e.drive_id.as_str());

    let drive_id = upload_drive_file(&token, &filename, &content, &parent_id, existing_drive_id).await?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    ledger.files.insert(
        rel_path,
        LedgerFileEntry {
            drive_id,
            sha256: local_hash,
            last_synced: now,
            remote_md5: None,
        },
    );

    let _ = save_ledger(&vault_path, &ledger);
    Ok(())
}

#[tauri::command]
pub async fn google_delete_file(
    app: tauri::AppHandle,
    vault_path: String,
    file_path: String,
) -> Result<(), String> {
    let token = match get_valid_access_token(&app).await {
        Ok(t) => t,
        Err(_) => return Ok(()),
    };

    let vault_base = Path::new(&vault_path);
    let target_file = Path::new(&file_path);
    let rel_path = normalize_relative_path(target_file, vault_base);

    let mut ledger = load_ledger(&vault_path);

    // 1. Direct file match
    if let Some(entry) = ledger.files.remove(&rel_path) {
        let _ = trash_drive_file(&token, &entry.drive_id).await;
    }

    // 2. Folder match: remove all files under this folder prefix and trash them
    let folder_prefix = format!("{}/", rel_path);
    let to_trash: Vec<(String, String)> = ledger
        .files
        .iter()
        .filter(|(k, _)| k.starts_with(&folder_prefix))
        .map(|(k, v)| (k.clone(), v.drive_id.clone()))
        .collect();

    for (k, drive_id) in to_trash {
        ledger.files.remove(&k);
        let _ = trash_drive_file(&token, &drive_id).await;
    }

    let _ = save_ledger(&vault_path, &ledger);
    Ok(())
}

// Helper: collect remote files iteratively
async fn collect_remote_files(
    token: &str,
    root_id: &str,
    out: &mut HashMap<String, DriveItem>,
    folder_cache: &mut HashMap<String, String>,
) -> Result<(), String> {
    let mut queue = vec![(root_id.to_string(), String::new())];

    while let Some((folder_id, current_rel_prefix)) = queue.pop() {
        let items = list_drive_files(token, &folder_id).await?;
        for item in items {
            if item.mime_type == "application/vnd.google-apps.folder" {
                let subfolder_rel = if current_rel_prefix.is_empty() {
                    item.name.clone()
                } else {
                    format!("{}/{}", current_rel_prefix, item.name)
                };
                folder_cache.insert(subfolder_rel.clone(), item.id.clone());
                queue.push((item.id, subfolder_rel));
            } else if item.name.ends_with(".md") {
                let rel_path = if current_rel_prefix.is_empty() {
                    item.name.clone()
                } else {
                    format!("{}/{}", current_rel_prefix, item.name)
                };
                out.insert(rel_path, item);
            }
        }
    }
    Ok(())
}

fn chrono_lite_timestamp() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{}", now)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_iso8601_to_epoch() {
        let iso = "2026-10-14T03:40:28.798Z";
        let epoch = parse_iso8601_to_epoch(iso);
        assert_eq!(epoch, Some(1791949228));

        let epoch_1970 = parse_iso8601_to_epoch("1970-01-01T00:00:00Z");
        assert_eq!(epoch_1970, Some(0));
    }
}
