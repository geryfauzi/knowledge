use reqwest::multipart::{Form, Part};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DriveItem {
    pub id: String,
    pub name: String,
    #[serde(default, rename = "mimeType")]
    pub mime_type: String,
    #[serde(default, rename = "modifiedTime")]
    pub modified_time: Option<String>,
    #[serde(default, rename = "md5Checksum")]
    pub md5_checksum: Option<String>,
    #[serde(default)]
    pub parents: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct FileListResponse {
    #[serde(default)]
    files: Vec<DriveItem>,
}

#[derive(Deserialize)]
struct FolderItem {
    id: String,
}

#[derive(Deserialize)]
struct FolderListResponse {
    #[serde(default)]
    files: Vec<FolderItem>,
}

#[derive(Deserialize)]
struct CreateFileResponse {
    id: String,
}

pub async fn ensure_drive_folder(
    token: &str,
    folder_name: &str,
    parent_id: Option<&str>,
) -> Result<String, String> {
    let client = reqwest::Client::new();

    let query = if let Some(pid) = parent_id {
        format!(
            "mimeType = 'application/vnd.google-apps.folder' and name = '{}' and '{}' in parents and trashed = false",
            folder_name.replace('\'', "\\'"),
            pid
        )
    } else {
        format!(
            "mimeType = 'application/vnd.google-apps.folder' and name = '{}' and trashed = false",
            folder_name.replace('\'', "\\'")
        )
    };

    let res = client
        .get("https://www.googleapis.com/drive/v3/files")
        .header("Authorization", format!("Bearer {}", token))
        .query(&[("q", query.as_str()), ("fields", "files(id, name)")])
        .send()
        .await
        .map_err(|e| format!("Network error querying Drive folder: {}", e))?;

    if !res.status().is_success() {
        let err = res.text().await.unwrap_or_default();
        return Err(format!("Drive API query error: {}", err));
    }

    let folder_list: FolderListResponse = res
        .json()
        .await
        .map_err(|e| format!("Failed to parse Drive folder list: {}", e))?;

    if let Some(first) = folder_list.files.into_iter().next() {
        return Ok(first.id);
    }

    // Create folder
    let mut payload = serde_json::json!({
        "name": folder_name,
        "mimeType": "application/vnd.google-apps.folder"
    });

    if let Some(pid) = parent_id {
        payload["parents"] = serde_json::json!([pid]);
    }

    let create_res = client
        .post("https://www.googleapis.com/drive/v3/files")
        .header("Authorization", format!("Bearer {}", token))
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Network error creating Drive folder: {}", e))?;

    if !create_res.status().is_success() {
        let err = create_res.text().await.unwrap_or_default();
        return Err(format!("Failed to create folder on Drive: {}", err));
    }

    let created: CreateFileResponse = create_res
        .json()
        .await
        .map_err(|e| format!("Failed to parse created folder response: {}", e))?;

    Ok(created.id)
}

pub async fn list_drive_files(token: &str, parent_id: &str) -> Result<Vec<DriveItem>, String> {
    let client = reqwest::Client::new();
    let query = format!("'{}' in parents and trashed = false", parent_id);

    let res = client
        .get("https://www.googleapis.com/drive/v3/files")
        .header("Authorization", format!("Bearer {}", token))
        .query(&[
            ("q", query.as_str()),
            (
                "fields",
                "files(id, name, mimeType, modifiedTime, md5Checksum, parents)",
            ),
            ("pageSize", "1000"),
        ])
        .send()
        .await
        .map_err(|e| format!("Network error listing files: {}", e))?;

    if !res.status().is_success() {
        let err = res.text().await.unwrap_or_default();
        return Err(format!("Drive API list files error: {}", err));
    }

    let file_list: FileListResponse = res
        .json()
        .await
        .map_err(|e| format!("Failed to parse Drive file list: {}", e))?;

    Ok(file_list.files)
}

pub async fn upload_drive_file(
    token: &str,
    filename: &str,
    content: &[u8],
    parent_id: &str,
    existing_id: Option<&str>,
) -> Result<String, String> {
    let client = reqwest::Client::new();

    if let Some(file_id) = existing_id {
        // Update existing file content
        let url = format!(
            "https://www.googleapis.com/upload/drive/v3/files/{}?uploadType=media",
            file_id
        );
        let res = client
            .patch(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "text/markdown; charset=utf-8")
            .body(content.to_vec())
            .send()
            .await
            .map_err(|e| format!("Network error updating file: {}", e))?;

        if !res.status().is_success() {
            let err = res.text().await.unwrap_or_default();
            return Err(format!("Drive API update file error: {}", err));
        }

        let updated: CreateFileResponse = res
            .json()
            .await
            .map_err(|e| format!("Failed to parse update file response: {}", e))?;

        Ok(updated.id)
    } else {
        // Multipart upload for new file with metadata (parents + name)
        let metadata = serde_json::json!({
            "name": filename,
            "parents": [parent_id]
        });

        let form = Form::new()
            .part(
                "metadata",
                Part::text(metadata.to_string())
                    .mime_str("application/json; charset=UTF-8")
                    .map_err(|e| e.to_string())?,
            )
            .part(
                "media",
                Part::bytes(content.to_vec())
                    .mime_str("text/markdown; charset=utf-8")
                    .map_err(|e| e.to_string())?,
            );

        let res = client
            .post("https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart")
            .header("Authorization", format!("Bearer {}", token))
            .multipart(form)
            .send()
            .await
            .map_err(|e| format!("Network error creating file: {}", e))?;

        if !res.status().is_success() {
            let err = res.text().await.unwrap_or_default();
            return Err(format!("Drive API create file error: {}", err));
        }

        let created: CreateFileResponse = res
            .json()
            .await
            .map_err(|e| format!("Failed to parse create file response: {}", e))?;

        Ok(created.id)
    }
}

pub async fn download_drive_file(token: &str, file_id: &str) -> Result<String, String> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://www.googleapis.com/drive/v3/files/{}?alt=media",
        file_id
    );

    let res = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Network error downloading file: {}", e))?;

    if !res.status().is_success() {
        let err = res.text().await.unwrap_or_default();
        return Err(format!("Drive API download file error: {}", err));
    }

    let text = res
        .text()
        .await
        .map_err(|e| format!("Failed to read response body: {}", e))?;

    Ok(text)
}

pub async fn trash_drive_file(token: &str, file_id: &str) -> Result<(), String> {
    let client = reqwest::Client::new();
    let url = format!("https://www.googleapis.com/drive/v3/files/{}", file_id);

    let res = client
        .patch(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({ "trashed": true }))
        .send()
        .await
        .map_err(|e| format!("Network error trashing file: {}", e))?;

    if !res.status().is_success() {
        let err = res.text().await.unwrap_or_default();
        return Err(format!("Drive API trash file error: {}", err));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_folder_list_response_deserialization() {
        let json = r#"{"files": [{"id": "1RnhXtr_0tk7h2c_yaU_X-qhZDiH718hV", "name": "Knowledge-Vault"}]}"#;
        let parsed: Result<FolderListResponse, _> = serde_json::from_str(json);
        assert!(parsed.is_ok());
        let res = parsed.unwrap();
        assert_eq!(res.files.len(), 1);
        assert_eq!(res.files[0].id, "1RnhXtr_0tk7h2c_yaU_X-qhZDiH718hV");
    }

    #[test]
    fn test_drive_item_with_missing_optional_fields() {
        let json = r#"{"id": "123", "name": "test.md"}"#;
        let parsed: Result<DriveItem, _> = serde_json::from_str(json);
        assert!(parsed.is_ok());
        let item = parsed.unwrap();
        assert_eq!(item.id, "123");
        assert_eq!(item.name, "test.md");
        assert_eq!(item.mime_type, "");
        assert_eq!(item.modified_time, None);
    }
}
