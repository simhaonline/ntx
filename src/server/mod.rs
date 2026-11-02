//! Nityaksha Pure Rust Multi-Threaded HTTP & REST API Server
//! Serves the embedded workstation WebUI and provides REST endpoints for memory, sandbox, and catalog.

pub mod webui;

use crate::anti_slop::AntiSlopFilter;
use crate::db::MemoryEngine;
use crate::sandbox::SandboxManager;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;

pub struct HttpServer {
    port: u16,
    engine: Arc<MemoryEngine>,
    sandbox: Arc<SandboxManager>,
}

impl HttpServer {
    pub fn new(port: u16, engine: Arc<MemoryEngine>, sandbox: Arc<SandboxManager>) -> Self {
        Self { port, engine, sandbox }
    }

    pub fn start(&self) -> std::io::Result<()> {
        let addr = format!("0.0.0.0:{}", self.port);
        let listener = TcpListener::bind(&addr)?;
        println!("Nityaksha Pure Rust WebUI & REST API running on http://{}", addr);

        for stream in listener.incoming() {
            if let Ok(stream) = stream {
                let engine = self.engine.clone();
                let sandbox = self.sandbox.clone();
                thread::spawn(move || {
                    let _ = handle_connection(stream, engine, sandbox);
                });
            }
        }
        Ok(())
    }
}

fn handle_connection(
    mut stream: TcpStream,
    engine: Arc<MemoryEngine>,
    sandbox: Arc<SandboxManager>,
) -> std::io::Result<()> {
    let mut buffer = [0u8; 8192];
    let bytes_read = stream.read(&mut buffer)?;
    if bytes_read == 0 {
        return Ok(());
    }

    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    let mut lines = request.lines();
    let request_line = lines.next().unwrap_or("");
    let parts: Vec<&str> = request_line.split_whitespace().collect();

    if parts.len() < 2 {
        return Ok(());
    }

    let method = parts[0];
    let uri = parts[1];

    // Find body if any
    let body = if let Some(idx) = request.find("\r\n\r\n") {
        &request[idx + 4..]
    } else {
        ""
    };

    if method == "OPTIONS" {
        let resp = "HTTP/1.1 204 No Content\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\n\r\n";
        stream.write_all(resp.as_bytes())?;
        return Ok(());
    }

    if (method == "GET" || method == "HEAD") && (uri == "/" || uri == "/index.html") {
        let html = include_str!("../../index.html");
        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n",
            html.len()
        );
        stream.write_all(resp.as_bytes())?;
        if method == "GET" {
            stream.write_all(html.as_bytes())?;
        }
        return Ok(());
    }

    if (method == "GET" || method == "HEAD") && (uri == "/app" || uri == "/dashboard") {
        let html = webui::EMBEDDED_WEBUI_HTML;
        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n",
            html.len()
        );
        stream.write_all(resp.as_bytes())?;
        if method == "GET" {
            stream.write_all(html.as_bytes())?;
        }
        return Ok(());
    }

    if (method == "GET" || method == "HEAD") && (uri == "/NITYAKSHA_LOGO.png" || uri == "/logo.png" || uri == "/logo-64.png" || uri == "/logo-180.png") {
        let bytes = include_bytes!("../../brand/NITYAKSHA_LOGO.png");
        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: image/png\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nCache-Control: public, max-age=86400\r\n\r\n",
            bytes.len()
        );
        stream.write_all(resp.as_bytes())?;
        if method == "GET" {
            stream.write_all(bytes)?;
        }
        return Ok(());
    }

    if (method == "GET" || method == "HEAD") && (uri == "/logo.svg") {
        let svg = include_str!("../../brand/logo.svg");
        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: image/svg+xml\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nCache-Control: public, max-age=86400\r\n\r\n",
            svg.len()
        );
        stream.write_all(resp.as_bytes())?;
        if method == "GET" {
            stream.write_all(svg.as_bytes())?;
        }
        return Ok(());
    }

    if (method == "GET" || method == "HEAD") && (uri == "/favicon.ico") {
        let bytes = include_bytes!("../../brand/logo-32.png");
        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: image/png\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n",
            bytes.len()
        );
        stream.write_all(resp.as_bytes())?;
        if method == "GET" {
            stream.write_all(bytes)?;
        }
        return Ok(());
    }

    if (method == "GET" || method == "HEAD") && uri == "/install.sh" {
        let script = include_str!("../../install.sh");
        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n",
            script.len()
        );
        stream.write_all(resp.as_bytes())?;
        if method == "GET" {
            stream.write_all(script.as_bytes())?;
        }
        return Ok(());
    }

    if (method == "GET" || method == "HEAD") && uri == "/install.ps1" {
        let script = include_str!("../../install.ps1");
        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n",
            script.len()
        );
        stream.write_all(resp.as_bytes())?;
        if method == "GET" {
            stream.write_all(script.as_bytes())?;
        }
        return Ok(());
    }

    if (method == "GET" || method == "HEAD") && uri == "/install-ios.sh" {
        let script = include_str!("../../install-ios.sh");
        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n",
            script.len()
        );
        stream.write_all(resp.as_bytes())?;
        if method == "GET" {
            stream.write_all(script.as_bytes())?;
        }
        return Ok(());
    }

    if method == "GET" && (uri == "/api/status" || uri == "/api/health") {
        let payload = serde_json::json!({
            "status": "healthy",
            "runtime": "pure-rust",
            "version": "1.0.0",
            "capabilities_cached": engine.relational.count_capabilities(),
            "multimodal_assets": engine.multimodal.list_assets().len(),
        });
        send_json(&mut stream, 200, &payload)?;
        return Ok(());
    }

    if method == "GET" && uri.starts_with("/api/catalog") {
        let query = if let Some(idx) = uri.find("q=") {
            &uri[idx + 2..]
        } else {
            ""
        };
        let items = engine.relational.search_capabilities(query, None);
        let payload = serde_json::to_value(items).unwrap_or_default();
        send_json(&mut stream, 200, &payload)?;
        return Ok(());
    }

    if method == "POST" && uri == "/api/memory/query" {
        let query_val: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
        let query_text = query_val["query"].as_str().unwrap_or("");
        let top_k = query_val["top_k"].as_u64().unwrap_or(5) as usize;
        let alpha = query_val["alpha"].as_f64().unwrap_or(0.5) as f32;

        let results = engine.hybrid_rag.search(query_text, None, top_k, alpha);
        let payload = serde_json::to_value(results).unwrap_or_default();
        send_json(&mut stream, 200, &payload)?;
        return Ok(());
    }

    if method == "POST" && uri == "/api/sandbox/run" {
        let cmd_val: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
        let cmd = cmd_val["command"].as_str().unwrap_or("echo 'empty command'");
        let dry_run = cmd_val["dry_run"].as_bool().unwrap_or(false);

        let res = sandbox.execute("web_session", cmd, dry_run);
        let payload = match res {
            Ok(r) => serde_json::to_value(r).unwrap_or_default(),
            Err(e) => serde_json::json!({ "error": e.to_string(), "exit_code": 1 }),
        };
        send_json(&mut stream, 200, &payload)?;
        return Ok(());
    }

    if method == "POST" && uri == "/api/filter/slop" {
        let text = body.trim();
        let filtered = AntiSlopFilter::filter(text);
        let payload = serde_json::json!({ "filtered": filtered });
        send_json(&mut stream, 200, &payload)?;
        return Ok(());
    }

    // 404 Fallback
    let not_found = "HTTP/1.1 404 Not Found\r\nContent-Length: 9\r\n\r\nNot Found";
    stream.write_all(not_found.as_bytes())?;
    Ok(())
}

fn send_json(stream: &mut TcpStream, status: u16, val: &serde_json::Value) -> std::io::Result<()> {
    let json_bytes = serde_json::to_vec(val).unwrap_or_default();
    let resp = format!(
        "HTTP/1.1 {} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n",
        status,
        json_bytes.len()
    );
    stream.write_all(resp.as_bytes())?;
    stream.write_all(&json_bytes)?;
    Ok(())
}
