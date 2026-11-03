//! Google OAuth 2.0 Engine for Google Gemini and Generative Language APIs
//! Supports interactive browser redirect, headless code paste, auto-refresh, and ADC fallback.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::Command;

pub const DEFAULT_REDIRECT_URI: &str = "http://127.0.0.1:8085/oauth/callback";
pub const GOOGLE_AUTH_ENDPOINT: &str = "https://accounts.google.com/o/oauth2/v2/auth";
pub const GOOGLE_TOKEN_ENDPOINT: &str = "https://oauth2.googleapis.com/token";
pub const GOOGLE_USERINFO_ENDPOINT: &str = "https://www.googleapis.com/oauth2/v3/userinfo";
pub const GOOGLE_SCOPES: &str = "https://www.googleapis.com/auth/cloud-platform https://www.googleapis.com/auth/userinfo.email https://www.googleapis.com/auth/userinfo.profile openid";

// Default public client ID for Gemini CLI / Antigravity developer tooling
pub const DEFAULT_CLIENT_ID: &str = "764086051850-6qr4p6gpi6hn506pt8ejuq83di341hur.apps.googleusercontent.com";
pub const DEFAULT_CLIENT_SECRET: &str = "d-FL95Q19q7MQmDhgtTXo7GB";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleOAuthCredentials {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: i64, // Unix timestamp in seconds
    pub token_type: String,
    pub email: Option<String>,
    pub scopes: Vec<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub auth_source: String, // "oauth2_flow", "gcloud_adc", "manual_token"
}

pub struct GoogleOAuthEngine;

impl GoogleOAuthEngine {
    /// Returns the credential storage path (system or user level)
    pub fn credential_path() -> PathBuf {
        let system_path = PathBuf::from("/etc/nityaksha/auth/google_oauth.json");
        if system_path.parent().map(|p| p.exists()).unwrap_or(false) {
            return system_path;
        }

        if let Ok(home) = std::env::var("HOME") {
            let user_path = PathBuf::from(home).join(".config/nityaksha/auth/google_oauth.json");
            if let Some(parent) = user_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            return user_path;
        }

        PathBuf::from("google_oauth.json")
    }

    /// Returns path for custom GCP client ID and secret configuration
    pub fn client_config_path() -> PathBuf {
        let system_path = PathBuf::from("/etc/nityaksha/auth/google_client.json");
        if system_path.parent().map(|p| p.exists()).unwrap_or(false) {
            return system_path;
        }

        if let Ok(home) = std::env::var("HOME") {
            let user_path = PathBuf::from(home).join(".config/nityaksha/auth/google_client.json");
            if let Some(parent) = user_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            return user_path;
        }

        PathBuf::from("google_client.json")
    }

    /// Save custom GCP client ID and secret
    pub fn save_custom_client(client_id: &str, client_secret: &str) -> Result<(), String> {
        let path = Self::client_config_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let val = serde_json::json!({
            "client_id": client_id.trim(),
            "client_secret": client_secret.trim(),
        });
        fs::write(&path, serde_json::to_string_pretty(&val).unwrap_or_default())
            .map_err(|e| format!("Failed to save client credentials: {}", e))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
        }
        Ok(())
    }

    /// Load persisted credentials
    pub fn load_credentials() -> Option<GoogleOAuthCredentials> {
        let path = Self::credential_path();
        if path.exists() {
            if let Ok(data) = fs::read_to_string(&path) {
                if let Ok(creds) = serde_json::from_str::<GoogleOAuthCredentials>(&data) {
                    return Some(creds);
                }
            }
        }

        // Fallback: Check standard gcloud ADC credentials if available
        if let Ok(home) = std::env::var("HOME") {
            let gcloud_adc = PathBuf::from(home).join(".config/gcloud/application_default_credentials.json");
            if gcloud_adc.exists() {
                if let Ok(data) = fs::read_to_string(&gcloud_adc) {
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&data) {
                        let refresh_token = v.get("refresh_token").and_then(|s| s.as_str()).map(|s| s.to_string());
                        let client_id = v.get("client_id").and_then(|s| s.as_str()).map(|s| s.to_string());
                        let client_secret = v.get("client_secret").and_then(|s| s.as_str()).map(|s| s.to_string());

                        if refresh_token.is_some() {
                            let creds = GoogleOAuthCredentials {
                                access_token: String::new(),
                                refresh_token,
                                expires_at: 0,
                                token_type: "Bearer".to_string(),
                                email: Some("gcloud-adc@developer.gserviceaccount.com".to_string()),
                                scopes: vec!["https://www.googleapis.com/auth/cloud-platform".to_string()],
                                client_id,
                                client_secret,
                                auth_source: "gcloud_adc".to_string(),
                            };
                            return Some(creds);
                        }
                    }
                }
            }
        }

        None
    }

    /// Save credentials to disk
    pub fn save_credentials(creds: &GoogleOAuthCredentials) -> Result<(), String> {
        let path = Self::credential_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create auth directory: {}", e))?;
        }
        let json = serde_json::to_string_pretty(creds)
            .map_err(|e| format!("Serialization error: {}", e))?;
        fs::write(&path, json).map_err(|e| format!("Failed to write credentials: {}", e))?;

        // Restrict file permissions on Unix to 0600
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
        }

        Ok(())
    }

    /// Remove credentials (logout)
    pub fn clear_credentials() -> Result<(), String> {
        let path = Self::credential_path();
        if path.exists() {
            fs::remove_file(&path).map_err(|e| format!("Failed to remove credentials: {}", e))?;
        }
        Ok(())
    }

    /// Resolves active client id
    pub fn get_client_id(custom_id: Option<&str>) -> String {
        if let Some(id) = custom_id {
            if !id.trim().is_empty() {
                return id.trim().to_string();
            }
        }
        if let Ok(id) = std::env::var("GOOGLE_CLIENT_ID") {
            if !id.trim().is_empty() {
                return id.trim().to_string();
            }
        }
        if let Ok(data) = fs::read_to_string(Self::client_config_path()) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&data) {
                if let Some(cid) = val.get("client_id").and_then(|v| v.as_str()) {
                    if !cid.trim().is_empty() {
                        return cid.trim().to_string();
                    }
                }
            }
        }
        DEFAULT_CLIENT_ID.to_string()
    }

    /// Resolves active client secret
    pub fn get_client_secret(custom_secret: Option<&str>) -> String {
        if let Some(secret) = custom_secret {
            if !secret.trim().is_empty() {
                return secret.trim().to_string();
            }
        }
        if let Ok(secret) = std::env::var("GOOGLE_CLIENT_SECRET") {
            if !secret.trim().is_empty() {
                return secret.trim().to_string();
            }
        }
        if let Ok(data) = fs::read_to_string(Self::client_config_path()) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&data) {
                if let Some(csec) = val.get("client_secret").and_then(|v| v.as_str()) {
                    if !csec.trim().is_empty() {
                        return csec.trim().to_string();
                    }
                }
            }
        }
        DEFAULT_CLIENT_SECRET.to_string()
    }

    /// Generates the Google OAuth authorization URL
    pub fn build_authorization_url(client_id: &str, redirect_uri: &str) -> String {
        format!(
            "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&access_type=offline&prompt=consent",
            GOOGLE_AUTH_ENDPOINT,
            url_encode(client_id),
            url_encode(redirect_uri),
            url_encode(GOOGLE_SCOPES)
        )
    }

    /// Exchange authorization code for OAuth tokens
    pub fn exchange_authorization_code(
        code: &str,
        client_id: &str,
        client_secret: &str,
        redirect_uri: &str,
    ) -> Result<GoogleOAuthCredentials, String> {
        let mut body = format!(
            "code={}&client_id={}&redirect_uri={}&grant_type=authorization_code",
            url_encode(code),
            url_encode(client_id),
            url_encode(redirect_uri)
        );
        if !client_secret.is_empty() {
            body.push_str(&format!("&client_secret={}", url_encode(client_secret)));
        }

        let output = Command::new("curl")
            .arg("-s")
            .arg("-X")
            .arg("POST")
            .arg(GOOGLE_TOKEN_ENDPOINT)
            .arg("-H")
            .arg("Content-Type: application/x-www-form-urlencoded")
            .arg("-d")
            .arg(&body)
            .output()
            .map_err(|e| format!("Failed to execute curl: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "Token exchange failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let resp_str = String::from_utf8_lossy(&output.stdout);
        let resp_json: serde_json::Value = serde_json::from_str(&resp_str)
            .map_err(|e| format!("Invalid JSON token response: {}. Raw: {}", e, resp_str))?;

        if let Some(err) = resp_json.get("error") {
            let desc = resp_json.get("error_description").and_then(|d| d.as_str()).unwrap_or("");
            return Err(format!("Google OAuth Error: {} - {}", err, desc));
        }

        let access_token = resp_json
            .get("access_token")
            .and_then(|t| t.as_str())
            .ok_or("Missing access_token in Google response")?
            .to_string();

        let refresh_token = resp_json
            .get("refresh_token")
            .and_then(|t| t.as_str())
            .map(|t| t.to_string());

        let expires_in = resp_json
            .get("expires_in")
            .and_then(|e| e.as_i64())
            .unwrap_or(3600);

        let expires_at = Utc::now().timestamp() + expires_in;

        // Fetch user email via UserInfo endpoint
        let email = Self::fetch_user_email(&access_token).ok();

        let creds = GoogleOAuthCredentials {
            access_token,
            refresh_token,
            expires_at,
            token_type: "Bearer".to_string(),
            email,
            scopes: GOOGLE_SCOPES.split_whitespace().map(|s| s.to_string()).collect(),
            client_id: Some(client_id.to_string()),
            client_secret: Some(client_secret.to_string()),
            auth_source: "oauth2_flow".to_string(),
        };

        Self::save_credentials(&creds)?;
        Ok(creds)
    }

    /// Refreshes expired access token using refresh_token
    pub fn refresh_access_token(creds: &mut GoogleOAuthCredentials) -> Result<String, String> {
        let refresh_token = creds
            .refresh_token
            .as_ref()
            .ok_or("No refresh_token present in credentials")?;

        let client_id = creds.client_id.as_deref().unwrap_or(DEFAULT_CLIENT_ID);
        let client_secret = creds.client_secret.as_deref().unwrap_or(DEFAULT_CLIENT_SECRET);

        let body = format!(
            "refresh_token={}&client_id={}&client_secret={}&grant_type=refresh_token",
            url_encode(refresh_token),
            url_encode(client_id),
            url_encode(client_secret)
        );

        let output = Command::new("curl")
            .arg("-s")
            .arg("-X")
            .arg("POST")
            .arg(GOOGLE_TOKEN_ENDPOINT)
            .arg("-H")
            .arg("Content-Type: application/x-www-form-urlencoded")
            .arg("-d")
            .arg(&body)
            .output()
            .map_err(|e| format!("Failed to execute curl: {}", e))?;

        if !output.status.success() {
            return Err("Failed to contact Google OAuth token endpoint".to_string());
        }

        let resp_str = String::from_utf8_lossy(&output.stdout);
        let resp_json: serde_json::Value = serde_json::from_str(&resp_str)
            .map_err(|e| format!("Invalid JSON token response: {}", e))?;

        if let Some(err) = resp_json.get("error") {
            return Err(format!("Failed to refresh Google token: {}", err));
        }

        let access_token = resp_json
            .get("access_token")
            .and_then(|t| t.as_str())
            .ok_or("Missing access_token in refresh response")?
            .to_string();

        let expires_in = resp_json
            .get("expires_in")
            .and_then(|e| e.as_i64())
            .unwrap_or(3600);

        creds.access_token = access_token.clone();
        creds.expires_at = Utc::now().timestamp() + expires_in;

        // Persist refreshed credentials
        let _ = Self::save_credentials(creds);

        Ok(access_token)
    }

    /// Fetches a valid access token, auto-refreshing if expired
    pub fn get_valid_access_token() -> Result<String, String> {
        if let Some(mut creds) = Self::load_credentials() {
            let now = Utc::now().timestamp();
            // If valid for at least 90 more seconds, return current token
            if creds.expires_at > now + 90 && !creds.access_token.is_empty() {
                return Ok(creds.access_token.clone());
            }

            // Attempt token refresh
            if creds.refresh_token.is_some() {
                match Self::refresh_access_token(&mut creds) {
                    Ok(fresh_token) => return Ok(fresh_token),
                    Err(e) => eprintln!("[WARN] Auto-refresh failed: {}. Retrying alternatives...", e),
                }
            }
        }

        // Fallback: check gcloud auth print-access-token if gcloud CLI is available
        if let Ok(output) = Command::new("gcloud").arg("auth").arg("print-access-token").output() {
            if output.status.success() {
                let token = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !token.is_empty() && !token.contains("ERROR") {
                    return Ok(token);
                }
            }
        }

        Err("No valid Google OAuth session found. Please run 'ntx auth login google' to authenticate.".to_string())
    }

    /// Fetch authenticated user email from Google UserInfo endpoint
    pub fn fetch_user_email(access_token: &str) -> Result<String, String> {
        let auth_header = format!("Authorization: Bearer {}", access_token);
        let output = Command::new("curl")
            .arg("-s")
            .arg(GOOGLE_USERINFO_ENDPOINT)
            .arg("-H")
            .arg(&auth_header)
            .output()
            .map_err(|e| format!("Failed to curl userinfo: {}", e))?;

        if !output.status.success() {
            return Err("Failed to query Google UserInfo".to_string());
        }

        let resp_str = String::from_utf8_lossy(&output.stdout);
        let resp_json: serde_json::Value = serde_json::from_str(&resp_str)
            .map_err(|e| format!("Invalid JSON from userinfo: {}", e))?;

        if let Some(email) = resp_json.get("email").and_then(|e| e.as_str()) {
            Ok(email.to_string())
        } else {
            Err("No email field returned by Google UserInfo".to_string())
        }
    }

    /// Extracts an authorization code from input, handling raw codes or full redirect URLs
    pub fn extract_code_from_input(input: &str) -> String {
        let mut trimmed = input.trim();
        if (trimmed.starts_with('"') && trimmed.ends_with('"'))
            || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
        {
            if trimmed.len() >= 2 {
                trimmed = &trimmed[1..trimmed.len() - 1].trim();
            }
        }

        if let Some(code_idx) = trimmed.find("code=") {
            let sub = &trimmed[code_idx + 5..];
            let raw_code = sub.split(['&', ' ', '#', '"', '\'']).next().unwrap_or("");
            url_decode(raw_code)
        } else {
            url_decode(trimmed)
        }
    }

    /// Interactive CLI OAuth flow with non-blocking local TCP listener and concurrent manual fallback
    pub fn interactive_login(
        custom_client_id: Option<&str>,
        custom_client_secret: Option<&str>,
    ) -> Result<GoogleOAuthCredentials, String> {
        let client_id = Self::get_client_id(custom_client_id);
        let client_secret = Self::get_client_secret(custom_client_secret);
        let redirect_uri = DEFAULT_REDIRECT_URI;

        let auth_url = Self::build_authorization_url(&client_id, redirect_uri);

        println!("================================================================================");
        println!("             NITYAKSHA — GOOGLE OAUTH 2.0 FOR GEMINI AUTHENTICATION             ");
        println!("================================================================================");
        println!("1. Open this URL in your web browser to grant Gemini permissions:\n");
        println!("   \x1b[36m{}\x1b[0m\n", auth_url);
        println!("2. Sign in with your Google account and grant access.");
        println!("3. Redirect Handling:");
        println!("   • Local Machine: Browser redirects automatically to http://127.0.0.1:8085.");
        println!("   • Remote / VPS : Your browser will show '127.0.0.1 refused to connect'.");
        println!("                    This is NORMAL because the server is remote. Simply COPY the");
        println!("                    full URL from your browser address bar and PASTE it below.");
        println!("--------------------------------------------------------------------------------");
        print!("Paste full redirect URL or authorization code here: ");
        let _ = io::stdout().flush();

        // Attempt to open browser automatically
        let _ = open_browser(&auth_url);

        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::mpsc;
        use std::sync::Arc;

        let done = Arc::new(AtomicBool::new(false));
        let done_tcp = done.clone();
        let (tx, rx) = mpsc::channel();
        let tx_tcp = tx.clone();

        // Spawn background TCP listener on 127.0.0.1:8085
        std::thread::spawn(move || {
            if let Ok(listener) = TcpListener::bind("127.0.0.1:8085") {
                let _ = listener.set_nonblocking(true);
                let start = std::time::Instant::now();
                while !done_tcp.load(Ordering::Relaxed) && start.elapsed().as_secs() < 180 {
                    match listener.accept() {
                        Ok((mut stream, _)) => {
                            let mut buf = [0u8; 4096];
                            if let Ok(n) = stream.read(&mut buf) {
                                let req = String::from_utf8_lossy(&buf[..n]);
                                if let Some(line) = req.lines().next() {
                                    if let Some(code_idx) = line.find("code=") {
                                        let sub = &line[code_idx + 5..];
                                        let raw = sub.split(['&', ' ']).next().unwrap_or("");
                                        let code = url_decode(raw);
                                        if !code.is_empty() {
                                            let resp = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n\
                                                <html><body style='font-family:sans-serif;background:#0D2340;color:#F4F6F9;text-align:center;padding:50px;'>\
                                                <h2 style='color:#C05A28;'>✓ Nityaksha Gemini Authentication Successful</h2>\
                                                <p>Google OAuth credentials exchanged. You can safely close this window and return to your terminal.</p>\
                                                </body></html>";
                                            let _ = stream.write_all(resp.as_bytes());
                                            let _ = tx_tcp.send(code);
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                            std::thread::sleep(std::time::Duration::from_millis(100));
                        }
                        Err(_) => break,
                    }
                }
            }
        });

        // Spawn background stdin reader
        let (stdin_tx, stdin_rx) = mpsc::channel();
        let done_stdin = done.clone();
        std::thread::spawn(move || {
            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_ok() {
                if !done_stdin.load(Ordering::Relaxed) {
                    let _ = stdin_tx.send(input);
                }
            }
        });

        // Main thread polls both channels
        let start = std::time::Instant::now();
        let code = loop {
            if let Ok(browser_code) = rx.try_recv() {
                println!("\n[INFO] Callback received automatically via local browser redirect.");
                done.store(true, Ordering::Relaxed);
                break browser_code;
            }
            if let Ok(user_input) = stdin_rx.try_recv() {
                let parsed = Self::extract_code_from_input(&user_input);
                if !parsed.is_empty() {
                    println!("\n[INFO] Authorization code received from terminal input.");
                    done.store(true, Ordering::Relaxed);
                    break parsed;
                }
            }
            if start.elapsed().as_secs() > 180 {
                done.store(true, Ordering::Relaxed);
                return Err("Authentication timed out after 3 minutes.".to_string());
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        };

        if code.is_empty() {
            return Err("Authorization cancelled or empty code provided.".to_string());
        }

        println!("\n[INFO] Exchanging authorization code with Google OAuth servers...");
        match Self::exchange_authorization_code(&code, &client_id, &client_secret, redirect_uri) {
            Ok(creds) => {
                println!("[SUCCESS] Google OAuth Token Issued!");
                if let Some(ref email) = creds.email {
                    println!("  Authenticated Account: \x1b[32m{}\x1b[0m", email);
                }
                println!("  Expires In           : ~{} minutes", (creds.expires_at - Utc::now().timestamp()) / 60);
                println!("  Credential Vault     : {}", Self::credential_path().display());
                println!("  Status               : Gemini Provider Activated with Google OAuth");
                Ok(creds)
            }
            Err(e) => {
                eprintln!("\n[ERROR] Google OAuth Token Exchange failed: {}", e);
                if e.contains("invalid_client") {
                    eprintln!("\n┌────────────────────────────────────────────────────────────────────────┐");
                    eprintln!("│ ⚠️  GOOGLE OAUTH CLIENT CONFIGURATION NOTICE                           │");
                    eprintln!("├────────────────────────────────────────────────────────────────────────┤");
                    eprintln!("│ Google requires an authorized GCP OAuth 2.0 Client Secret for this flow. │");
                    eprintln!("│ For zero-friction setup without GCP console configuration, use your    │");
                    eprintln!("│ Google AI Studio API Key (instant, free, all Gemini models).           │");
                    eprintln!("│ Get your key at: https://aistudio.google.com/app/apikey                │");
                    eprintln!("└────────────────────────────────────────────────────────────────────────┘");
                    print!("Enter Gemini API Key (or press Enter to cancel): ");
                    let _ = io::stdout().flush();
                    let mut fallback_key = String::new();
                    if io::stdin().read_line(&mut fallback_key).is_ok() {
                        let clean = fallback_key.trim();
                        if !clean.is_empty() {
                            use crate::provider::ProviderRegistry;
                            let _ = ProviderRegistry::set_key("gemini", clean);
                            let _ = ProviderRegistry::set_active("gemini");
                            println!("✓ Successfully configured and activated Google Gemini with API Key!");
                            return Ok(GoogleOAuthCredentials {
                                access_token: clean.to_string(),
                                refresh_token: None,
                                expires_at: i64::MAX,
                                token_type: "ApiKey".to_string(),
                                email: Some("aistudio-apikey@google.com".to_string()),
                                scopes: vec!["gemini".to_string()],
                                client_id: None,
                                client_secret: None,
                                auth_source: "api_key".to_string(),
                            });
                        }
                    }
                }
                Err(e)
            }
        }
    }
}

fn open_browser(url: &str) -> io::Result<()> {
    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("xdg-open").arg(url).spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("open").arg(url).spawn();
    }
    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("cmd").args(["/C", "start", url]).spawn();
    }
    Ok(())
}

pub fn url_encode(input: &str) -> String {
    let mut encoded = String::new();
    for b in input.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(b as char);
            }
            b' ' => encoded.push_str("%20"),
            _ => {
                encoded.push_str(&format!("%{:02X}", b));
            }
        }
    }
    encoded
}

pub fn url_decode(s: &str) -> String {
    let mut result = String::new();
    let mut bytes = s.bytes().peekable();
    while let Some(b) = bytes.next() {
        if b == b'%' {
            let h1 = bytes.next().unwrap_or(b'0');
            let h2 = bytes.next().unwrap_or(b'0');
            let hex_str = format!("{}{}", h1 as char, h2 as char);
            if let Ok(byte) = u8::from_str_radix(&hex_str, 16) {
                result.push(byte as char);
            } else {
                result.push('%');
                result.push(h1 as char);
                result.push(h2 as char);
            }
        } else if b == b'+' {
            result.push(' ');
        } else {
            result.push(b as char);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_code_from_full_url() {
        let url = "http://127.0.0.1:8085/oauth/callback?iss=https://accounts.google.com&code=4/0AUn6Hd8kQpBySe2lVen9ETKEv2JkLn43oV1JUAMUQAc_Pvl6psOgT1isCno-Dp58Alg1xw&scope=email%20profile%20openid";
        let code = GoogleOAuthEngine::extract_code_from_input(url);
        assert_eq!(code, "4/0AUn6Hd8kQpBySe2lVen9ETKEv2JkLn43oV1JUAMUQAc_Pvl6psOgT1isCno-Dp58Alg1xw");
    }

    #[test]
    fn test_extract_code_from_encoded_url() {
        let url = "http://127.0.0.1:8085/oauth/callback?code=4%2F0AUn6Hd8kQpBySe2lVen&scope=email";
        let code = GoogleOAuthEngine::extract_code_from_input(url);
        assert_eq!(code, "4/0AUn6Hd8kQpBySe2lVen");
    }

    #[test]
    fn test_extract_code_from_raw_code() {
        let raw = "4/0AUn6Hd8kQpBySe2lVen9ETKEv2JkLn43oV1JUAMUQAc_Pvl6psOgT1isCno-Dp58Alg1xw";
        let code = GoogleOAuthEngine::extract_code_from_input(raw);
        assert_eq!(code, raw);
    }

    #[test]
    fn test_extract_code_quoted() {
        let quoted = "\"4/0AUn6Hd8kQpBySe2lVen9ETKEv2JkLn43oV1JUAMUQAc_Pvl6psOgT1isCno-Dp58Alg1xw\"";
        let code = GoogleOAuthEngine::extract_code_from_input(quoted);
        assert_eq!(code, "4/0AUn6Hd8kQpBySe2lVen9ETKEv2JkLn43oV1JUAMUQAc_Pvl6psOgT1isCno-Dp58Alg1xw");
    }
}
