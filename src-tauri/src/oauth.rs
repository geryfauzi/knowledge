use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::Manager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthTokens {
    pub client_id: String,
    pub client_secret: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: i64,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthStatus {
    pub is_authenticated: bool,
    pub email: Option<String>,
    pub expires_at: Option<i64>,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: i64,
    refresh_token: Option<String>,
}

#[derive(Deserialize)]
struct UserInfoResponse {
    email: Option<String>,
}

fn get_auth_file_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Could not get app data dir: {}", e))?;
    std::fs::create_dir_all(&app_dir)
        .map_err(|e| format!("Could not create app data dir: {}", e))?;
    Ok(app_dir.join("google_auth.json"))
}

pub fn load_tokens(app: &tauri::AppHandle) -> Result<Option<AuthTokens>, String> {
    let path = get_auth_file_path(app)?;
    if !path.exists() {
        return Ok(None);
    }
    let data = std::fs::read_to_string(&path)
        .map_err(|e| format!("Could not read auth file: {}", e))?;
    let tokens: AuthTokens = serde_json::from_str(&data)
        .map_err(|e| format!("Could not parse auth file: {}", e))?;
    Ok(Some(tokens))
}

pub fn save_tokens(app: &tauri::AppHandle, tokens: &AuthTokens) -> Result<(), String> {
    let path = get_auth_file_path(app)?;
    let json = serde_json::to_string_pretty(tokens)
        .map_err(|e| format!("Could not serialize tokens: {}", e))?;
    std::fs::write(&path, json)
        .map_err(|e| format!("Could not save auth tokens: {}", e))?;
    Ok(())
}

pub fn delete_tokens(app: &tauri::AppHandle) -> Result<(), String> {
    let path = get_auth_file_path(app)?;
    if path.exists() {
        let _ = std::fs::remove_file(path);
    }
    Ok(())
}

pub fn generate_pkce() -> (String, String) {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    let verifier = URL_SAFE_NO_PAD.encode(bytes);

    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let challenge = URL_SAFE_NO_PAD.encode(hasher.finalize());

    (verifier, challenge)
}

fn extract_query_param(request: &str, param: &str) -> Option<String> {
    let first_line = request.lines().next()?;
    let path = first_line.split_whitespace().nth(1)?;
    let query = path.split('?').nth(1)?;

    for pair in query.split('&') {
        let mut parts = pair.split('=');
        if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
            if key == param {
                return urlencoding_decode(value);
            }
        }
    }
    None
}

fn urlencoding_decode(s: &str) -> Option<String> {
    let mut result = String::new();
    let mut chars = s.chars();
    while let Some(ch) = chars.next() {
        if ch == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                result.push(byte as char);
            }
        } else if ch == '+' {
            result.push(' ');
        } else {
            result.push(ch);
        }
    }
    Some(result)
}

pub async fn get_valid_access_token(app: &tauri::AppHandle) -> Result<String, String> {
    let mut tokens = load_tokens(app)?.ok_or_else(|| "Not logged in to Google Drive".to_string())?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    // Buffer 60 seconds before expiration
    if tokens.expires_at > now + 60 {
        return Ok(tokens.access_token);
    }

    let refresh_token = tokens
        .refresh_token
        .as_ref()
        .ok_or_else(|| "No refresh token available. Please reconnect Google Drive.".to_string())?;

    let client = reqwest::Client::new();
    let res = client
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("client_id", tokens.client_id.as_str()),
            ("client_secret", tokens.client_secret.as_str()),
            ("refresh_token", refresh_token.as_str()),
            ("grant_type", "refresh_token"),
        ])
        .send()
        .await
        .map_err(|e| format!("Network error while refreshing token: {}", e))?;

    if !res.status().is_success() {
        let err_body = res.text().await.unwrap_or_default();
        return Err(format!("Google rejected token refresh: {}", err_body));
    }

    let token_res: TokenResponse = res
        .json()
        .await
        .map_err(|e| format!("Failed to parse refreshed token: {}", e))?;

    tokens.access_token = token_res.access_token;
    tokens.expires_at = now + token_res.expires_in;
    if let Some(new_rt) = token_res.refresh_token {
        tokens.refresh_token = Some(new_rt);
    }

    save_tokens(app, &tokens)?;
    Ok(tokens.access_token)
}

#[tauri::command]
pub async fn google_get_auth_status(app: tauri::AppHandle) -> Result<AuthStatus, String> {
    let tokens = load_tokens(&app)?;
    if let Some(t) = tokens {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        Ok(AuthStatus {
            is_authenticated: t.refresh_token.is_some() || t.expires_at > now,
            email: t.email,
            expires_at: Some(t.expires_at),
        })
    } else {
        Ok(AuthStatus {
            is_authenticated: false,
            email: None,
            expires_at: None,
        })
    }
}

#[tauri::command]
pub async fn google_logout(app: tauri::AppHandle) -> Result<(), String> {
    delete_tokens(&app)?;
    Ok(())
}

#[tauri::command]
pub async fn google_start_oauth(
    app: tauri::AppHandle,
    client_id: String,
    client_secret: String,
) -> Result<AuthStatus, String> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .map_err(|e| format!("Failed to bind local loopback listener: {}", e))?;
    let port = listener
        .local_addr()
        .map_err(|e| format!("Failed to get local port: {}", e))?
        .port();

    let redirect_uri = format!("http://127.0.0.1:{}", port);
    let (code_verifier, code_challenge) = generate_pkce();

    let auth_url = format!(
        "https://accounts.google.com/o/oauth2/v2/auth?\
client_id={}&\
redirect_uri={}&\
response_type=code&\
scope=https://www.googleapis.com/auth/drive.file%20email&\
code_challenge={}&\
code_challenge_method=S256&\
access_type=offline&\
prompt=consent",
        client_id, redirect_uri, code_challenge
    );

    // Open user's default browser
    open::that(&auth_url).map_err(|e| format!("Failed to open system browser: {}", e))?;

    // Wait for incoming redirect
    let (mut stream, _) = listener
        .accept()
        .map_err(|e| format!("Failed to accept connection from browser: {}", e))?;

    let mut buffer = [0u8; 4096];
    let bytes_read = stream
        .read(&mut buffer)
        .map_err(|e| format!("Failed to read HTTP request: {}", e))?;

    let request_str = String::from_utf8_lossy(&buffer[..bytes_read]);
    let code = extract_query_param(&request_str, "code")
        .ok_or_else(|| "Authorization code not found in callback request".to_string())?;

    // Send friendly HTML response to browser
    let response_body = "<!DOCTYPE html><html><head><meta charset='utf-8'><title>Knowledge</title>\
<style>body{font-family:system-ui,-apple-system,sans-serif;background:#121215;color:#f4f4f5;display:flex;align-items:center;justify-content:center;height:100vh;margin:0;}\
.card{background:#18181b;border:1px solid #27272a;border-radius:12px;padding:32px;text-align:center;box-shadow:0 8px 30px rgba(0,0,0,0.5);max-width:400px;}\
h2{color:#38bdf8;margin-top:0;}p{color:#a1a1aa;font-size:14px;line-height:1.5;}</style></head>\
<body><div class='card'><h2>Google Drive Connected!</h2><p>You can close this tab and return to the Knowledge app.</p></div></body></html>";

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        response_body.len(),
        response_body
    );
    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();

    // Exchange authorization code for tokens
    let client = reqwest::Client::new();
    let token_res = client
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("code", code.as_str()),
            ("code_verifier", code_verifier.as_str()),
            ("grant_type", "authorization_code"),
            ("redirect_uri", redirect_uri.as_str()),
        ])
        .send()
        .await
        .map_err(|e| format!("Failed to exchange authorization code: {}", e))?;

    if !token_res.status().is_success() {
        let err_text = token_res.text().await.unwrap_or_default();
        return Err(format!("Google rejected token exchange: {}", err_text));
    }

    let tokens: TokenResponse = token_res
        .json()
        .await
        .map_err(|e| format!("Failed to parse token response: {}", e))?;

    // Fetch user profile email
    let user_info_res = client
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .header("Authorization", format!("Bearer {}", tokens.access_token))
        .send()
        .await;

    let email = if let Ok(info_res) = user_info_res {
        if info_res.status().is_success() {
            info_res
                .json::<UserInfoResponse>()
                .await
                .ok()
                .and_then(|u| u.email)
        } else {
            None
        }
    } else {
        None
    };

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let auth_tokens = AuthTokens {
        client_id,
        client_secret,
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
        expires_at: now + tokens.expires_in,
        email: email.clone(),
    };

    save_tokens(&app, &auth_tokens)?;

    Ok(AuthStatus {
        is_authenticated: true,
        email,
        expires_at: Some(auth_tokens.expires_at),
    })
}
