//! Nityaksha Multi-Threaded HTTP & REST API Server
//! Serves the embedded workstation WebUI and provides REST endpoints for memory, sandbox, and catalog.

pub mod webui;

use crate::anti_slop::AntiSlopFilter;
use crate::bot::DynamicPromptWrapper;
use crate::db::MemoryEngine;
use crate::provider::{GoogleOAuthEngine, ProviderRegistry};
use crate::sandbox::SandboxManager;
use crate::workflow::{GraphRunner, MultiModelGateway};
use std::collections::HashMap;
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
        println!("Nityaksha WebUI & REST API running on http://46.62.188.249:{} (and http://{})", self.port, addr);

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

    let req_host = request.lines()
        .find(|l| l.to_lowercase().starts_with("host:"))
        .map(|l| l[5..].trim())
        .unwrap_or("46.62.188.249:8080");

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

    if (method == "GET" || method == "HEAD") && (uri == "/" || uri == "/index.html" || uri == "/app" || uri == "/dashboard") {
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

    if (method == "GET" || method == "HEAD") && (uri == "/landing" || uri == "/welcome" || uri == "/marketing") {
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

    if (method == "GET" || method == "HEAD") && (uri == "/logo-dark.svg") {
        let svg = include_str!("../../brand/logo-dark.svg");
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

    if (method == "GET" || method == "HEAD") && (uri.starts_with("/assets/") || uri == "/hero.png" || uri == "/brand/hero.png") {
        let clean_filename = if uri == "/hero.png" || uri == "/brand/hero.png" {
            "hero.png".to_string()
        } else {
            let without_prefix = uri.trim_start_matches("/assets/");
            without_prefix.split('?').next().unwrap_or("").replace("..", "")
        };

        let p1 = std::path::Path::new("/root/install/assets").join(&clean_filename);
        let p2 = std::path::Path::new("src/assets").join(&clean_filename);
        let p3 = std::path::Path::new("/root/install/nityaksha-core/src/assets").join(&clean_filename);
        let p4 = std::path::Path::new("brand").join(&clean_filename);

        let target_path = if p1.exists() {
            Some(p1)
        } else if p2.exists() {
            Some(p2)
        } else if p3.exists() {
            Some(p3)
        } else if p4.exists() {
            Some(p4)
        } else {
            None
        };

        if let Some(path) = target_path {
            if let Ok(bytes) = std::fs::read(&path) {
                let mime = if clean_filename.ends_with(".png") {
                    "image/png"
                } else if clean_filename.ends_with(".svg") {
                    "image/svg+xml"
                } else if clean_filename.ends_with(".ico") {
                    "image/x-icon"
                } else if clean_filename.ends_with(".jpg") || clean_filename.ends_with(".jpeg") {
                    "image/jpeg"
                } else if clean_filename.ends_with(".css") {
                    "text/css"
                } else if clean_filename.ends_with(".js") {
                    "application/javascript"
                } else {
                    "application/octet-stream"
                };

                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nCache-Control: public, max-age=86400\r\n\r\n",
                    mime, bytes.len()
                );
                stream.write_all(resp.as_bytes())?;
                if method == "GET" {
                    stream.write_all(&bytes)?;
                }
                return Ok(());
            }
        }
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

    if (method == "GET" || method == "HEAD") && (uri == "/ntx.exe" || uri == "/download/ntx.exe") {
        let p1 = std::path::Path::new("ntx.exe");
        let p2 = std::path::Path::new("/root/install/nityaksha-core/ntx.exe");
        let bytes = if p1.exists() {
            std::fs::read(p1).unwrap_or_default()
        } else if p2.exists() {
            std::fs::read(p2).unwrap_or_default()
        } else {
            Vec::new()
        };
        if !bytes.is_empty() {
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/vnd.microsoft.portable-executable\r\nContent-Disposition: attachment; filename=\"ntx.exe\"\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n",
                bytes.len()
            );
            stream.write_all(resp.as_bytes())?;
            if method == "GET" {
                stream.write_all(&bytes)?;
            }
            return Ok(());
        }
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

    // --- Bot App Store API ---
    if method == "GET" && uri.starts_with("/api/bots") {
        let id_opt = if let Some(idx) = uri.find("id=") {
            Some(uri[idx + 3..].split('&').next().unwrap_or("").to_string())
        } else if uri.len() > 10 && uri.starts_with("/api/bots/") {
            Some(uri[10..].to_string())
        } else {
            None
        };

        if let Some(bot_id) = id_opt {
            if let Some(b) = engine.relational.get_bot(&bot_id) {
                let payload = serde_json::to_value(b).unwrap_or_default();
                send_json(&mut stream, 200, &payload)?;
                return Ok(());
            } else {
                let payload = serde_json::json!({ "error": "Bot not found" });
                send_json(&mut stream, 404, &payload)?;
                return Ok(());
            }
        }

        let bots = engine.relational.list_bots();
        let payload = serde_json::to_value(bots).unwrap_or_default();
        send_json(&mut stream, 200, &payload)?;
        return Ok(());
    }

    if method == "POST" && (uri == "/api/bots/run" || uri.starts_with("/api/bots/")) {
        let req_val: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
        let bot_id = req_val["bot_id"]
            .as_str()
            .or_else(|| {
                if uri.starts_with("/api/bots/") && !uri.ends_with("/run") {
                    Some(&uri[10..])
                } else {
                    None
                }
            })
            .unwrap_or("ntx-bot-concierge-prime");

        let prompt = req_val["prompt"].as_str().unwrap_or("Audit workspace health");
        let user_name = req_val["user_name"].as_str().unwrap_or("simha");
        let workspace_root = req_val["workspace_root"].as_str().unwrap_or("/root/install");
        let domain_context = req_val["domain_context"].as_str().unwrap_or("production");

        if let Some(bot) = engine.relational.get_bot(bot_id) {
            let mut vars = HashMap::new();
            vars.insert("user_name".to_string(), user_name.to_string());
            vars.insert("workspace_root".to_string(), workspace_root.to_string());
            vars.insert("domain_context".to_string(), domain_context.to_string());

            let hydrated_sys = DynamicPromptWrapper::hydrate(
                &bot.system_prompt_template,
                &vars,
                &bot.allowed_tools,
                &bot.sandbox_policy,
            );

            let execution_output = MultiModelGateway::dispatch(
                &bot.model_tier,
                prompt,
                Some(&hydrated_sys),
                bot.temperature,
                &vars,
            );

            let payload = serde_json::json!({
                "bot_id": bot.id,
                "bot_name": bot.name,
                "model_tier": bot.model_tier,
                "hydrated_system_prompt": hydrated_sys,
                "output": execution_output,
            });
            send_json(&mut stream, 200, &payload)?;
            return Ok(());
        } else {
            let payload = serde_json::json!({ "error": format!("Bot '{}' not found", bot_id) });
            send_json(&mut stream, 404, &payload)?;
            return Ok(());
        }
    }

    // --- Workflow DAG Graph Engine API ---
    if method == "GET" && (uri == "/api/workflows" || uri.starts_with("/api/workflows?")) {
        let id_opt = if let Some(idx) = uri.find("id=") {
            Some(uri[idx + 3..].split('&').next().unwrap_or("").to_string())
        } else {
            None
        };

        if let Some(wf_id) = id_opt {
            if let Some(wf) = engine.relational.get_workflow(&wf_id) {
                let payload = serde_json::to_value(wf).unwrap_or_default();
                send_json(&mut stream, 200, &payload)?;
                return Ok(());
            } else {
                let payload = serde_json::json!({ "error": "Workflow not found" });
                send_json(&mut stream, 404, &payload)?;
                return Ok(());
            }
        }

        let wfs = engine.relational.list_workflows();
        let payload = serde_json::to_value(wfs).unwrap_or_default();
        send_json(&mut stream, 200, &payload)?;
        return Ok(());
    }

    if method == "POST" && uri == "/api/workflows/execute" {
        let req_val: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
        let wf_id = req_val["workflow_id"].as_str().unwrap_or("wf-code-security-pipeline");
        
        let mut custom_inputs = HashMap::new();
        if let Some(obj) = req_val["inputs"].as_object() {
            for (k, v) in obj {
                if let Some(s) = v.as_str() {
                    custom_inputs.insert(k.clone(), s.to_string());
                }
            }
        }

        if let Some(wf) = engine.relational.get_workflow(wf_id) {
            match GraphRunner::init_execution(&wf, custom_inputs) {
                Ok(mut exec) => {
                    GraphRunner::run_to_completion_or_suspend(&mut exec, &wf, &engine, &sandbox);
                    let _ = engine.relational.save_execution(exec.clone());
                    let payload = serde_json::to_value(exec).unwrap_or_default();
                    send_json(&mut stream, 200, &payload)?;
                    return Ok(());
                }
                Err(err) => {
                    let payload = serde_json::json!({ "error": err });
                    send_json(&mut stream, 400, &payload)?;
                    return Ok(());
                }
            }
        } else {
            let payload = serde_json::json!({ "error": format!("Workflow '{}' not found", wf_id) });
            send_json(&mut stream, 404, &payload)?;
            return Ok(());
        }
    }

    if method == "GET" && uri.starts_with("/api/workflows/executions") {
        let id_opt = if let Some(idx) = uri.find("id=") {
            Some(uri[idx + 3..].split('&').next().unwrap_or("").to_string())
        } else {
            None
        };

        if let Some(exec_id) = id_opt {
            if let Some(exec) = engine.relational.get_execution(&exec_id) {
                let payload = serde_json::to_value(exec).unwrap_or_default();
                send_json(&mut stream, 200, &payload)?;
                return Ok(());
            } else {
                let payload = serde_json::json!({ "error": "Execution not found" });
                send_json(&mut stream, 404, &payload)?;
                return Ok(());
            }
        }

        let execs = engine.relational.list_executions();
        let payload = serde_json::to_value(execs).unwrap_or_default();
        send_json(&mut stream, 200, &payload)?;
        return Ok(());
    }

    if method == "POST" && uri == "/api/workflows/executions/approve" {
        let req_val: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
        let exec_id = req_val["id"].as_str().unwrap_or("");
        let token = req_val["token"].as_str().unwrap_or("");

        if let Some(mut exec) = engine.relational.get_execution(exec_id) {
            if let Some(wf) = engine.relational.get_workflow(&exec.workflow_id) {
                match GraphRunner::approve_hitl(&mut exec, token) {
                    Ok(_) => {
                        // Resume execution after approval
                        GraphRunner::run_to_completion_or_suspend(&mut exec, &wf, &engine, &sandbox);
                        let _ = engine.relational.save_execution(exec.clone());
                        let payload = serde_json::to_value(exec).unwrap_or_default();
                        send_json(&mut stream, 200, &payload)?;
                        return Ok(());
                    }
                    Err(err) => {
                        let payload = serde_json::json!({ "error": err });
                        send_json(&mut stream, 400, &payload)?;
                        return Ok(());
                    }
                }
            } else {
                let payload = serde_json::json!({ "error": "Workflow definition missing for execution" });
                send_json(&mut stream, 500, &payload)?;
                return Ok(());
            }
        } else {
            let payload = serde_json::json!({ "error": format!("Execution '{}' not found", exec_id) });
            send_json(&mut stream, 404, &payload)?;
            return Ok(());
        }
    }

    // --- AI Providers Gateway API ---
    if method == "GET" && uri.starts_with("/api/providers") {
        let providers = ProviderRegistry::list_all();
        let payload = serde_json::to_value(providers).unwrap_or_default();
        send_json(&mut stream, 200, &payload)?;
        return Ok(());
    }

    if method == "POST" && uri == "/api/providers/select" {
        let req_val: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
        let provider_id = req_val["id"].as_str().or_else(|| req_val["provider"].as_str()).unwrap_or("gemini");
        match ProviderRegistry::set_active(provider_id) {
            Ok(()) => {
                let active = ProviderRegistry::active();
                let payload = serde_json::json!({
                    "status": "success",
                    "active_provider": active.id,
                    "name": active.name,
                    "default_model": active.default_model
                });
                send_json(&mut stream, 200, &payload)?;
                return Ok(());
            }
            Err(e) => {
                let payload = serde_json::json!({ "error": e });
                send_json(&mut stream, 400, &payload)?;
                return Ok(());
            }
        }
    }

    if method == "POST" && uri == "/api/providers/config" {
        let req_val: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
        let provider_id = req_val["id"].as_str().or_else(|| req_val["provider"].as_str()).unwrap_or("");
        let key_opt = req_val["api_key"].as_str();
        let url_opt = req_val["base_url"].as_str();
        let model_opt = req_val["model"].as_str();
        let effort_opt = req_val["reasoning_effort"].as_str();
        let set_active = req_val["set_active"].as_bool().unwrap_or(false);

        if let Some(key) = key_opt {
            let _ = ProviderRegistry::set_key(provider_id, key);
        }
        if let Some(url) = url_opt {
            let _ = ProviderRegistry::set_base_url(provider_id, url);
        }
        if let Some(model) = model_opt {
            let _ = ProviderRegistry::set_model(provider_id, model);
        }
        if let Some(effort) = effort_opt {
            let _ = ProviderRegistry::set_reasoning_effort(effort);
        }
        if set_active {
            let _ = ProviderRegistry::set_active(provider_id);
        }

        let updated = ProviderRegistry::get(provider_id);
        let payload = serde_json::json!({ "status": "success", "provider": updated });
        send_json(&mut stream, 200, &payload)?;
        return Ok(());
    }

    if method == "POST" && uri == "/api/providers/fetch-models" {
        let req_val: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
        let provider_id = req_val["id"].as_str().or_else(|| req_val["provider"].as_str()).unwrap_or("");
        let key_opt = req_val["api_key"].as_str();
        let url_opt = req_val["base_url"].as_str();

        match ProviderRegistry::fetch_remote_models_with_key(provider_id, key_opt, url_opt) {
            Ok(models) => {
                let payload = serde_json::json!({
                    "status": "ok",
                    "models": models,
                    "count": models.len()
                });
                send_json(&mut stream, 200, &payload)?;
                return Ok(());
            }
            Err(e) => {
                let fallback = ProviderRegistry::get(provider_id).map(|p| p.available_models).unwrap_or_default();
                let payload = serde_json::json!({
                    "status": "error",
                    "message": e,
                    "fallback_models": fallback
                });
                send_json(&mut stream, 200, &payload)?;
                return Ok(());
            }
        }
    }

    if method == "POST" && uri == "/api/providers/test" {
        let req_val: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
        let provider_id = req_val["id"].as_str().or_else(|| req_val["provider"].as_str()).unwrap_or("gemini");
        match ProviderRegistry::test_connectivity(provider_id) {
            Ok(msg) => {
                let payload = serde_json::json!({ "status": "ok", "message": msg });
                send_json(&mut stream, 200, &payload)?;
                return Ok(());
            }
            Err(e) => {
                let payload = serde_json::json!({ "status": "error", "message": e });
                send_json(&mut stream, 400, &payload)?;
                return Ok(());
            }
        }
    }

    // --- Drop-in OpenAI API Router & Gateway (/v1/chat/completions, /v1/models, /v1/health, /api/router/stats) ---
    if method == "GET" && (uri == "/v1/health" || uri == "/health" || uri == "/api/health") {
        let payload = serde_json::json!({
            "status": "healthy",
            "server": "Nityaksha AI Gateway",
            "version": "1.0.0",
            "timestamp": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0),
        });
        send_json(&mut stream, 200, &payload)?;
        return Ok(());
    }

    if method == "GET" && (uri == "/api/router/stats" || uri == "/v1/router/stats") {
        use crate::provider::get_router_state;
        let state = get_router_state();
        let records = state.list_health();
        let saved_tokens = state.get_total_tokens_saved();
        let rec_json: Vec<serde_json::Value> = records.iter().map(|r| {
            serde_json::json!({
                "id": r.id,
                "name": r.name,
                "is_healthy": r.is_healthy(),
                "cooldown_remaining_secs": r.cooldown_remaining_secs(),
                "successful_requests": r.successful_requests,
                "failed_requests": r.failed_requests,
                "total_requests": r.total_requests,
                "avg_latency_ms": r.avg_latency_ms,
                "last_error": r.last_error,
            })
        }).collect();
        let payload = serde_json::json!({
            "total_tokens_saved": saved_tokens,
            "providers": rec_json,
        });
        send_json(&mut stream, 200, &payload)?;
        return Ok(());
    }

    if method == "POST" && uri == "/api/router/compress" {
        use crate::provider::RtkTokenOptimizer;
        let req_val: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
        let prompt = req_val["prompt"].as_str().or_else(|| req_val["text"].as_str()).unwrap_or("");
        let (compressed, orig, comp) = RtkTokenOptimizer::compress(prompt);
        let saved = orig.saturating_sub(comp);
        let pct = if orig > 0 { (saved as f64 / orig as f64) * 100.0 } else { 0.0 };
        let payload = serde_json::json!({
            "status": "ok",
            "original_text": prompt,
            "compressed_text": compressed,
            "original_tokens": orig,
            "compressed_tokens": comp,
            "tokens_saved": saved,
            "savings_percent": pct
        });
        send_json(&mut stream, 200, &payload)?;
        return Ok(());
    }

    if method == "POST" && uri == "/api/router/classify" {
        use crate::provider::AdaptiveComplexityRouter;
        let req_val: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
        let prompt = req_val["prompt"].as_str().or_else(|| req_val["text"].as_str()).unwrap_or("");
        let (tier, rationale) = AdaptiveComplexityRouter::classify(prompt);
        let rec = AdaptiveComplexityRouter::recommended_model(tier);
        let payload = serde_json::json!({
            "status": "ok",
            "prompt": prompt,
            "tier": format!("{:?}", tier),
            "tier_name": tier.name(),
            "rationale": rationale,
            "recommended_model": rec
        });
        send_json(&mut stream, 200, &payload)?;
        return Ok(());
    }

    if method == "POST" && uri == "/api/router/reset" {
        use crate::provider::get_router_state;
        get_router_state().reset();
        let payload = serde_json::json!({
            "status": "ok",
            "message": "All circuit breakers and provider telemetry counters have been reset"
        });
        send_json(&mut stream, 200, &payload)?;
        return Ok(());
    }

    if method == "GET" && (uri == "/v1/models" || uri == "/api/models") {
        let providers = ProviderRegistry::list_all();
        let mut model_entries = Vec::new();
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(1700000000);
        for p in &providers {
            for m in &p.available_models {
                model_entries.push(serde_json::json!({
                    "id": m,
                    "object": "model",
                    "created": now,
                    "owned_by": p.id,
                    "permission": [],
                    "root": m,
                    "parent": null
                }));
            }
        }
        let payload = serde_json::json!({
            "object": "list",
            "data": model_entries
        });
        send_json(&mut stream, 200, &payload)?;
        return Ok(());
    }

    if method == "POST" && (uri == "/v1/chat/completions" || uri == "/api/chat/completions") {
        let req_val: serde_json::Value = match serde_json::from_str(body) {
            Ok(v) => v,
            Err(e) => {
                let err_payload = serde_json::json!({
                    "error": {
                        "message": format!("Invalid JSON request payload: {}", e),
                        "type": "invalid_request_error",
                        "param": null,
                        "code": "bad_request"
                    }
                });
                send_json(&mut stream, 400, &err_payload)?;
                return Ok(());
            }
        };

        let temp = req_val["temperature"].as_f64().map(|f| f as f32).unwrap_or(0.7);
        let mut sys_prompt = String::new();
        let mut user_prompt = String::new();
        let mut history: Vec<(String, String)> = Vec::new();

        if let Some(messages) = req_val["messages"].as_array() {
            for msg in messages {
                let role = msg["role"].as_str().unwrap_or("");
                let content = if let Some(s) = msg["content"].as_str() {
                    s.to_string()
                } else if let Some(arr) = msg["content"].as_array() {
                    let mut text_parts = Vec::new();
                    for part in arr {
                        if part["type"].as_str() == Some("text") {
                            if let Some(t) = part["text"].as_str() {
                                text_parts.push(t);
                            }
                        }
                    }
                    text_parts.join("\n")
                } else {
                    String::new()
                };

                match role {
                    "system" => {
                        if !sys_prompt.is_empty() {
                            sys_prompt.push('\n');
                        }
                        sys_prompt.push_str(&content);
                    }
                    "user" => {
                        if !user_prompt.is_empty() {
                            history.push(("user".to_string(), user_prompt.clone()));
                        }
                        user_prompt = content;
                    }
                    "assistant" => {
                        history.push(("assistant".to_string(), content));
                    }
                    _ => {}
                }
            }
        }

        if user_prompt.is_empty() {
            let err_payload = serde_json::json!({
                "error": {
                    "message": "No user message found in 'messages' array",
                    "type": "invalid_request_error",
                    "param": "messages",
                    "code": "missing_required_field"
                }
            });
            send_json(&mut stream, 400, &err_payload)?;
            return Ok(());
        }

        let sys_opt = if !sys_prompt.is_empty() { Some(sys_prompt.as_str()) } else { None };

        match ProviderRegistry::dispatch_prompt_detailed(&user_prompt, sys_opt, temp, &history) {
            Ok(res) => {
                let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(1700000000);
                let unique_id = format!("chatcmpl-ntx-{:x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0));
                let resp_payload = serde_json::json!({
                    "id": unique_id,
                    "object": "chat.completion",
                    "created": now,
                    "model": res.model,
                    "choices": [
                        {
                            "index": 0,
                            "message": {
                                "role": "assistant",
                                "content": res.text
                            },
                            "finish_reason": "stop"
                        }
                    ],
                    "usage": {
                        "prompt_tokens": res.input_tokens,
                        "completion_tokens": res.output_tokens,
                        "total_tokens": res.total_tokens
                    }
                });
                send_json(&mut stream, 200, &resp_payload)?;
                return Ok(());
            }
            Err(e) => {
                let err_payload = serde_json::json!({
                    "error": {
                        "message": e,
                        "type": "api_error",
                        "param": null,
                        "code": "gateway_execution_error"
                    }
                });
                send_json(&mut stream, 502, &err_payload)?;
                return Ok(());
            }
        }
    }

    // --- Google OAuth & Auth Status API ---
    if method == "GET" && uri == "/api/auth/status" {
        let google_creds = GoogleOAuthEngine::load_credentials();
        let active_p = ProviderRegistry::active();

        let payload = serde_json::json!({
            "google_oauth": {
                "authenticated": google_creds.is_some(),
                "email": google_creds.as_ref().and_then(|c| c.email.clone()),
                "expires_at": google_creds.as_ref().map(|c| c.expires_at),
                "auth_source": google_creds.as_ref().map(|c| c.auth_source.clone()),
            },
            "active_provider": active_p.id,
            "provider_name": active_p.name,
            "auth_summary": active_p.auth_summary
        });
        send_json(&mut stream, 200, &payload)?;
        return Ok(());
    }

    if method == "GET" && uri.starts_with("/api/auth/google/url") {
        let client_id = GoogleOAuthEngine::get_client_id(None);
        let redirect_uri = format!("http://{}/api/auth/google/callback", req_host);
        let auth_url = GoogleOAuthEngine::build_authorization_url(&client_id, &redirect_uri);
        let payload = serde_json::json!({
            "url": auth_url,
            "client_id": client_id,
            "redirect_uri": redirect_uri
        });
        send_json(&mut stream, 200, &payload)?;
        return Ok(());
    }

    if method == "GET" && uri.starts_with("/api/auth/google/callback") {
        let code_opt = if let Some(idx) = uri.find("code=") {
            let sub = &uri[idx + 5..];
            Some(sub.split('&').next().unwrap_or("").to_string())
        } else {
            None
        };

        if let Some(code) = code_opt {
            let client_id = GoogleOAuthEngine::get_client_id(None);
            let client_secret = GoogleOAuthEngine::get_client_secret(None);
            let redirect_uri = format!("http://{}/api/auth/google/callback", req_host);

            match GoogleOAuthEngine::exchange_authorization_code(&code, &client_id, &client_secret, &redirect_uri) {
                Ok(creds) => {
                    let _ = ProviderRegistry::set_active("gemini");
                    let email = creds.email.unwrap_or_else(|| "Google User".to_string());
                    let html = format!(
                        "<!DOCTYPE html><html><head><meta charset='utf-8'><title>Google OAuth Success</title>\
                        <meta http-equiv='refresh' content='2;url=/app'>\
                        <style>body{{background:#0D2340;color:#FFFFFF;font-family:sans-serif;text-align:center;padding:60px;}} \
                        h2{{color:#C05A28;}} a{{color:#0EA5E9;}}</style></head><body>\
                        <h2>✓ Google OAuth Successful</h2>\
                        <p>Authenticated as <strong>{}</strong>. Gemini Provider Activated.</p>\
                        <p>Redirecting to Nityaksha Workstation in 2 seconds... <a href='/app'>Click here to return</a></p>\
                        </body></html>",
                        email
                    );
                    let resp = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}",
                        html.len(),
                        html
                    );
                    stream.write_all(resp.as_bytes())?;
                    return Ok(());
                }
                Err(err) => {
                    let html = format!(
                        "<!DOCTYPE html><html><body style='background:#0D2340;color:#FF6B6B;font-family:sans-serif;text-align:center;padding:50px;'>\
                        <h2>Authentication Failed</h2><p>{}</p><p><a href='/app' style='color:#FFFFFF;'>Return to Workstation</a></p></body></html>",
                        err
                    );
                    let resp = format!(
                        "HTTP/1.1 400 Bad Request\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",
                        html.len(),
                        html
                    );
                    stream.write_all(resp.as_bytes())?;
                    return Ok(());
                }
            }
        }
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
