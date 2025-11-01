use axum::{
    body::{Body, to_bytes},
    extract::{Request, State},
    response::{IntoResponse, Response},
    routing::any,
    Router,
};
use http::{uri::Uri, StatusCode};
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use std::net::SocketAddr;
use tracing::{error, info, instrument, warn};
use serde_json::Value;
use crate::detectors;

type HttpClient = Client<HttpConnector, Body>;

#[instrument]
pub async fn start_proxy(port: u16) {
    let client = Client::builder(hyper_util::rt::TokioExecutor::new()).build_http();

    let app = Router::new()
        .route("/*path", any(handler))
        .with_state(client);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    info!("Starting proxy on http://{}", addr);

    if let Ok(listener) = tokio::net::TcpListener::bind(addr).await {
        axum::serve(listener, app).await.unwrap();
    } else {
        error!("Failed to bind to port {}", port);
    }
}

#[instrument(skip(client, req))]
async fn handler(
    State(client): State<HttpClient>,
    req: Request,
) -> Response {
    let path = req.uri().path();
    let path_query = req
        .uri()
        .path_and_query()
        .map(|v| v.as_str())
        .unwrap_or(path);

    // Hardcoded upstream server
    let upstream_uri_str = format!("http://localhost:8080{}", path_query);

    info!("Forwarding request to {}", upstream_uri_str);

    // --- MCP Tool Call Analysis ---
    let (parts, body) = req.into_parts();
    let bytes = match to_bytes(body, usize::MAX).await {
        Ok(bytes) => bytes,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };

    if let Ok(json) = serde_json::from_slice::<Value>(&bytes) {
        if let Some(tool_calls) = json.get("tool_calls").and_then(|tc| tc.as_array()) {
            for tool_call in tool_calls {
                if let Some(code) = tool_call.get("code").and_then(|c| c.as_str()) {
                    info!("Scanning tool call code snippet...");
                    scan_tool_call(code);
                }
            }
        }
    }

    let mut req = Request::from_parts(parts, Body::from(bytes));
    // --- End Analysis ---

    let upstream_uri = match Uri::try_from(upstream_uri_str) {
        Ok(uri) => uri,
        Err(e) => {
            error!("Failed to parse upstream URI: {}", e);
            return StatusCode::BAD_REQUEST.into_response();
        }
    };

    *req.uri_mut() = upstream_uri;

    match client.request(req).await {
        Ok(res) => {
            info!("Request forwarded successfully");
            res.into_response()
        }
        Err(e) => {
            error!("Failed to forward request: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

fn scan_tool_call(code: &str) {
    let file_path = "proxy_tool_call";
    let mut vulnerabilities = vec![];

    if let Ok(vulns) = detectors::secrets::detect(code, file_path) {
        vulnerabilities.extend(vulns);
    }
    if let Ok(vulns) = detectors::command_injection::detect(code, file_path) {
        vulnerabilities.extend(vulns);
    }

    if !vulnerabilities.is_empty() {
        warn!("Found {} vulnerabilities in tool call:", vulnerabilities.len());
        for vuln in vulnerabilities {
            warn!("  - {}: {}", vuln.title, vuln.description);
        }
    }
}
