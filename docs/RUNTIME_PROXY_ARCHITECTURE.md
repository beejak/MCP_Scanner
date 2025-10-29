# Runtime Proxy Architecture - Phase 3.0

**Version**: v3.0.0 Design Document
**Status**: Draft Specification
**Last Updated**: 2025-10-29

---

## Executive Summary

The Runtime Proxy Engine transforms MCP Sentinel from a static scanner into a **real-time security platform**. By intercepting MCP protocol traffic, we can detect and prevent threats as they happen, not just after the fact.

**Key Innovation**: **Zero-touch** security - no code changes required by MCP server developers.

---

## Architecture Overview

### High-Level Design

```
┌─────────────┐                    ┌──────────────────┐                    ┌─────────────┐
│             │                    │  Runtime Proxy   │                    │             │
│ MCP Client  │───── Request ─────►│                  │───── Request ─────►│ MCP Server  │
│  (Claude)   │                    │  • Inspect       │                    │ (Upstream)  │
│             │◄──── Response ─────│  • Analyze       │◄──── Response ─────│             │
└─────────────┘                    │  • Alert/Block   │                    └─────────────┘
                                   └──────────────────┘
                                            │
                                            │ Threats
                                            ▼
                                   ┌──────────────────┐
                                   │  Alert System    │
                                   │  • Dashboard     │
                                   │  • Logs          │
                                   │  • Webhooks      │
                                   └──────────────────┘
```

### Component Breakdown

1. **Proxy Server** - TCP/HTTP server accepting client connections
2. **Protocol Inspector** - Parses and understands MCP JSON-RPC
3. **Runtime Detectors** - Analyze traffic for threats in real-time
4. **Guardrails Engine** - Enforce security policies (block/alert)
5. **Alert System** - Notify stakeholders of threats
6. **Metrics Collector** - Performance and security metrics

---

## Core Components

### 1. Proxy Server

**Purpose**: Accept connections and forward traffic bidirectionally

**Implementation**:
```rust
// src/engines/runtime_proxy/proxy.rs

use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::net::SocketAddr;

pub struct ProxyServer {
    listen_addr: SocketAddr,
    upstream_addr: SocketAddr,
    config: ProxyConfig,
}

pub struct ProxyConfig {
    pub mode: ProxyMode,           // Monitor or Enforce
    pub max_connections: usize,     // Connection limit
    pub timeout_ms: u64,            // Request timeout
    pub buffer_size: usize,         // Read buffer size
}

pub enum ProxyMode {
    Monitor,  // Log threats, don't block
    Enforce,  // Block threats, return error
}

impl ProxyServer {
    pub async fn start(&self) -> Result<()> {
        let listener = TcpListener::bind(&self.listen_addr).await?;
        info!("Runtime proxy listening on {}", self.listen_addr);

        loop {
            let (client_stream, client_addr) = listener.accept().await?;
            debug!("New connection from {}", client_addr);

            let upstream_stream = TcpStream::connect(&self.upstream_addr).await?;

            // Spawn task to handle this connection
            let config = self.config.clone();
            tokio::spawn(async move {
                if let Err(e) = handle_connection(client_stream, upstream_stream, config).await {
                    error!("Connection error: {}", e);
                }
            });
        }
    }
}

async fn handle_connection(
    mut client: TcpStream,
    mut upstream: TcpStream,
    config: ProxyConfig,
) -> Result<()> {
    let (mut client_read, mut client_write) = client.split();
    let (mut upstream_read, mut upstream_write) = upstream.split();

    // Bidirectional forwarding with inspection
    tokio::select! {
        // Client → Upstream
        result = proxy_direction(&mut client_read, &mut upstream_write, Direction::Request, &config) => {
            result?;
        }
        // Upstream → Client
        result = proxy_direction(&mut upstream_read, &mut client_write, Direction::Response, &config) => {
            result?;
        }
    }

    Ok(())
}
```

**Key Features**:
- Async I/O for high concurrency (1000+ connections)
- Connection pooling to upstream
- Graceful shutdown handling
- Circuit breaker for upstream failures

---

### 2. Protocol Inspector

**Purpose**: Parse MCP JSON-RPC messages from TCP byte stream

**Challenge**: HTTP/JSON-RPC messages can span multiple TCP packets

**Solution**: Stateful parser with buffering

**Implementation**:
```rust
// src/engines/runtime_proxy/inspector.rs

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,      // "2.0"
    pub method: String,       // "tools/call", "tools/list", etc.
    pub params: Value,        // Method parameters
    pub id: Option<Value>,    // Request ID
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub result: Option<Value>,
    pub error: Option<JsonRpcError>,
    pub id: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

pub struct ProtocolInspector {
    buffer: Vec<u8>,
    max_buffer_size: usize,
}

impl ProtocolInspector {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            max_buffer_size: 10 * 1024 * 1024,  // 10MB max message
        }
    }

    /// Feed bytes into the inspector, returns complete messages
    pub fn feed(&mut self, data: &[u8]) -> Result<Vec<Message>> {
        self.buffer.extend_from_slice(data);

        if self.buffer.len() > self.max_buffer_size {
            anyhow::bail!("Message too large: {} bytes", self.buffer.len());
        }

        let mut messages = Vec::new();

        // Try to extract complete HTTP messages
        while let Some(message) = self.extract_message()? {
            messages.push(message);
        }

        Ok(messages)
    }

    fn extract_message(&mut self) -> Result<Option<Message>> {
        // Look for HTTP request/response pattern
        if let Some(pos) = find_http_message_end(&self.buffer) {
            let raw = self.buffer.drain(..=pos).collect::<Vec<u8>>();
            let message = parse_http_message(&raw)?;
            Ok(Some(message))
        } else {
            Ok(None)  // Need more data
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Request {
        headers: HashMap<String, String>,
        body: JsonRpcRequest,
        raw: Vec<u8>,
    },
    Response {
        status_code: u16,
        headers: HashMap<String, String>,
        body: JsonRpcResponse,
        raw: Vec<u8>,
    },
}

impl Message {
    pub fn is_tool_call(&self) -> bool {
        match self {
            Message::Request { body, .. } => body.method == "tools/call",
            _ => false,
        }
    }

    pub fn tool_name(&self) -> Option<&str> {
        match self {
            Message::Request { body, .. } => {
                body.params.get("name")
                    .and_then(|v| v.as_str())
            }
            _ => None,
        }
    }

    pub fn extract_strings(&self) -> Vec<String> {
        // Recursively extract all strings from JSON for scanning
        match self {
            Message::Request { body, .. } => extract_json_strings(&body.params),
            Message::Response { body, .. } => {
                if let Some(result) = &body.result {
                    extract_json_strings(result)
                } else {
                    Vec::new()
                }
            }
        }
    }
}
```

**Edge Cases Handled**:
- Partial messages (buffering)
- Malformed JSON (error handling)
- Large messages (size limits)
- Chunked transfer encoding (HTTP)

---

### 3. Runtime Detectors

**Purpose**: Analyze traffic in real-time for security threats

**Detector Trait**:
```rust
// src/engines/runtime_proxy/detectors.rs

#[async_trait]
pub trait RuntimeDetector: Send + Sync {
    /// Unique detector identifier
    fn name(&self) -> &str;

    /// Analyze a request before it's sent to upstream
    async fn analyze_request(&self, msg: &Message) -> Result<Vec<ThreatAlert>>;

    /// Analyze a response before it's sent to client
    async fn analyze_response(&self, msg: &Message, request: &Message) -> Result<Vec<ThreatAlert>>;

    /// Should this detector run for this message?
    fn should_analyze(&self, msg: &Message) -> bool {
        true  // Default: analyze all messages
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ThreatAlert {
    pub id: String,              // Unique alert ID
    pub detector: String,        // Which detector found it
    pub severity: Severity,      // Critical/High/Medium/Low
    pub category: ThreatCategory,
    pub title: String,
    pub description: String,
    pub evidence: HashMap<String, Value>,
    pub timestamp: DateTime<Utc>,
    pub action: AlertAction,
}

#[derive(Debug, Clone, Serialize)]
pub enum ThreatCategory {
    DataExfiltration,
    CommandInjection,
    RugPull,
    BehavioralAnomaly,
    RateLimitViolation,
    UnauthorizedAccess,
}

#[derive(Debug, Clone, Serialize)]
pub enum AlertAction {
    Monitor,     // Log only
    Alert,       // Log + notify
    Block,       // Prevent request/response
}
```

**Detector Implementations**:

#### A. Data Exfiltration Detector
```rust
pub struct DataExfiltrationDetector {
    max_response_size: usize,        // e.g., 1MB
    sensitive_patterns: Vec<Regex>,  // SSN, credit cards, etc.
}

#[async_trait]
impl RuntimeDetector for DataExfiltrationDetector {
    fn name(&self) -> &str { "data_exfiltration" }

    async fn analyze_response(&self, response: &Message, _request: &Message) -> Result<Vec<ThreatAlert>> {
        let mut alerts = Vec::new();

        // Check 1: Response size
        if let Message::Response { raw, .. } = response {
            if raw.len() > self.max_response_size {
                alerts.push(ThreatAlert {
                    id: Uuid::new_v4().to_string(),
                    detector: self.name().to_string(),
                    severity: Severity::High,
                    category: ThreatCategory::DataExfiltration,
                    title: "Large response detected".to_string(),
                    description: format!("Response size {} exceeds limit {}",
                        raw.len(), self.max_response_size),
                    evidence: hashmap! {
                        "response_size".to_string() => json!(raw.len()),
                        "limit".to_string() => json!(self.max_response_size),
                    },
                    timestamp: Utc::now(),
                    action: AlertAction::Alert,
                });
            }
        }

        // Check 2: Sensitive patterns
        let strings = response.extract_strings();
        for pattern in &self.sensitive_patterns {
            for string in &strings {
                if pattern.is_match(string) {
                    alerts.push(ThreatAlert {
                        id: Uuid::new_v4().to_string(),
                        detector: self.name().to_string(),
                        severity: Severity::Critical,
                        category: ThreatCategory::DataExfiltration,
                        title: "Sensitive data in response".to_string(),
                        description: "Response contains pattern matching sensitive data".to_string(),
                        evidence: hashmap! {
                            "pattern".to_string() => json!(pattern.as_str()),
                            "matched".to_string() => json!(string),
                        },
                        timestamp: Utc::now(),
                        action: AlertAction::Block,  // Block this!
                    });
                }
            }
        }

        Ok(alerts)
    }
}
```

#### B. Rug Pull Detector
```rust
pub struct RugPullDetector {
    tool_registry: Arc<Mutex<HashMap<String, ToolSnapshot>>>,
}

#[derive(Clone)]
struct ToolSnapshot {
    name: String,
    description: String,
    parameters_schema: Value,
    checksum: String,  // SHA-256
    first_seen: DateTime<Utc>,
    last_seen: DateTime<Utc>,
}

#[async_trait]
impl RuntimeDetector for RugPullDetector {
    fn name(&self) -> &str { "rug_pull" }

    async fn analyze_response(&self, response: &Message, request: &Message) -> Result<Vec<ThreatAlert>> {
        // Only analyze "tools/list" responses
        if !matches!(request, Message::Request { body, .. } if body.method == "tools/list") {
            return Ok(Vec::new());
        }

        let mut alerts = Vec::new();
        let mut registry = self.tool_registry.lock().await;

        // Parse tool list from response
        if let Message::Response { body, .. } = response {
            if let Some(tools) = body.result.as_ref()
                .and_then(|r| r.get("tools"))
                .and_then(|t| t.as_array()) {

                for tool_value in tools {
                    let tool_name = tool_value.get("name")
                        .and_then(|n| n.as_str())
                        .unwrap_or("unknown");

                    let description = tool_value.get("description")
                        .and_then(|d| d.as_str())
                        .unwrap_or("");

                    let schema = tool_value.get("inputSchema").cloned()
                        .unwrap_or(Value::Null);

                    // Compute checksum of tool definition
                    let checksum = compute_tool_checksum(tool_value);

                    // Check if we've seen this tool before
                    if let Some(snapshot) = registry.get(tool_name) {
                        if snapshot.checksum != checksum {
                            // RUG PULL DETECTED!
                            alerts.push(ThreatAlert {
                                id: Uuid::new_v4().to_string(),
                                detector: self.name().to_string(),
                                severity: Severity::Critical,
                                category: ThreatCategory::RugPull,
                                title: format!("Tool '{}' definition changed", tool_name),
                                description: format!(
                                    "Tool '{}' has been redefined. This could indicate a rug pull attack.",
                                    tool_name
                                ),
                                evidence: hashmap! {
                                    "tool_name".to_string() => json!(tool_name),
                                    "old_checksum".to_string() => json!(&snapshot.checksum),
                                    "new_checksum".to_string() => json!(checksum),
                                    "old_description".to_string() => json!(&snapshot.description),
                                    "new_description".to_string() => json!(description),
                                    "first_seen".to_string() => json!(snapshot.first_seen),
                                },
                                timestamp: Utc::now(),
                                action: AlertAction::Block,
                            });
                        }
                    } else {
                        // First time seeing this tool, record it
                        registry.insert(tool_name.to_string(), ToolSnapshot {
                            name: tool_name.to_string(),
                            description: description.to_string(),
                            parameters_schema: schema,
                            checksum,
                            first_seen: Utc::now(),
                            last_seen: Utc::now(),
                        });
                    }
                }
            }
        }

        Ok(alerts)
    }
}

fn compute_tool_checksum(tool: &Value) -> String {
    use sha2::{Sha256, Digest};
    let canonical = serde_json::to_string(tool).unwrap();
    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    format!("{:x}", hasher.finalize())
}
```

#### C. Behavioral Anomaly Detector
```rust
pub struct BehavioralAnomalyDetector {
    request_history: Arc<Mutex<VecDeque<RequestRecord>>>,
    max_history: usize,
    rate_limit_window: Duration,
    max_requests_per_window: usize,
}

#[derive(Clone)]
struct RequestRecord {
    timestamp: DateTime<Utc>,
    method: String,
    tool_name: Option<String>,
    client_addr: SocketAddr,
}

#[async_trait]
impl RuntimeDetector for BehavioralAnomalyDetector {
    fn name(&self) -> &str { "behavioral_anomaly" }

    async fn analyze_request(&self, msg: &Message) -> Result<Vec<ThreatAlert>> {
        let mut alerts = Vec::new();
        let mut history = self.request_history.lock().await;

        // Record this request
        if let Message::Request { body, .. } = msg {
            history.push_back(RequestRecord {
                timestamp: Utc::now(),
                method: body.method.clone(),
                tool_name: body.params.get("name")
                    .and_then(|n| n.as_str())
                    .map(|s| s.to_string()),
                client_addr: "0.0.0.0:0".parse().unwrap(),  // TODO: Get from context
            });

            // Trim old records
            let cutoff = Utc::now() - self.rate_limit_window;
            while let Some(record) = history.front() {
                if record.timestamp < cutoff {
                    history.pop_front();
                } else {
                    break;
                }
            }
        }

        // Check rate limit
        if history.len() > self.max_requests_per_window {
            alerts.push(ThreatAlert {
                id: Uuid::new_v4().to_string(),
                detector: self.name().to_string(),
                severity: Severity::Medium,
                category: ThreatCategory::RateLimitViolation,
                title: "Rate limit exceeded".to_string(),
                description: format!(
                    "{} requests in {:?} exceeds limit of {}",
                    history.len(),
                    self.rate_limit_window,
                    self.max_requests_per_window
                ),
                evidence: hashmap! {
                    "request_count".to_string() => json!(history.len()),
                    "window_seconds".to_string() => json!(self.rate_limit_window.as_secs()),
                    "limit".to_string() => json!(self.max_requests_per_window),
                },
                timestamp: Utc::now(),
                action: AlertAction::Alert,
            });
        }

        Ok(alerts)
    }
}
```

---

### 4. Guardrails Engine

**Purpose**: Enforce security policies based on threat alerts

**Implementation**:
```rust
// src/engines/runtime_proxy/guardrails.rs

pub struct GuardrailsEngine {
    mode: ProxyMode,
    policy: SecurityPolicy,
}

pub struct SecurityPolicy {
    pub block_on_critical: bool,
    pub block_on_high: bool,
    pub alert_webhook: Option<String>,
    pub custom_rules: Vec<GuardrailRule>,
}

pub struct GuardrailRule {
    pub name: String,
    pub condition: RuleCondition,
    pub action: GuardrailAction,
}

pub enum RuleCondition {
    DetectorMatch(String),           // Detector name
    CategoryMatch(ThreatCategory),   // Threat category
    SeverityGte(Severity),           // Severity >= threshold
    And(Vec<RuleCondition>),
    Or(Vec<RuleCondition>),
}

pub enum GuardrailAction {
    Allow,
    Alert,
    Block,
    CustomResponse(String),  // Return custom error message
}

impl GuardrailsEngine {
    pub fn evaluate(&self, alerts: &[ThreatAlert]) -> GuardrailDecision {
        // If no alerts, allow
        if alerts.is_empty() {
            return GuardrailDecision::Allow;
        }

        // Check critical alerts
        let has_critical = alerts.iter().any(|a| a.severity == Severity::Critical);
        if has_critical && self.policy.block_on_critical {
            return GuardrailDecision::Block {
                reason: "Critical threat detected".to_string(),
                alerts: alerts.to_vec(),
            };
        }

        // Check high alerts
        let has_high = alerts.iter().any(|a| a.severity == Severity::High);
        if has_high && self.policy.block_on_high {
            return GuardrailDecision::Block {
                reason: "High severity threat detected".to_string(),
                alerts: alerts.to_vec(),
            };
        }

        // Check custom rules
        for rule in &self.policy.custom_rules {
            if self.rule_matches(rule, alerts) {
                match &rule.action {
                    GuardrailAction::Block => {
                        return GuardrailDecision::Block {
                            reason: format!("Policy '{}' triggered", rule.name),
                            alerts: alerts.to_vec(),
                        };
                    }
                    GuardrailAction::Alert => {
                        return GuardrailDecision::AlertOnly {
                            alerts: alerts.to_vec(),
                        };
                    }
                    _ => {}
                }
            }
        }

        // Default: alert but allow
        GuardrailDecision::AlertOnly {
            alerts: alerts.to_vec(),
        }
    }
}

pub enum GuardrailDecision {
    Allow,
    AlertOnly { alerts: Vec<ThreatAlert> },
    Block { reason: String, alerts: Vec<ThreatAlert> },
}
```

**Error Response When Blocked**:
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32000,
    "message": "Request blocked by MCP Sentinel runtime proxy",
    "data": {
      "reason": "Critical threat detected",
      "detector": "data_exfiltration",
      "severity": "critical",
      "contact": "security@example.com"
    }
  },
  "id": 123
}
```

---

## Performance Optimization

### Async I/O Strategy

- Use `tokio` for async runtime
- Connection pooling for upstream
- Zero-copy forwarding where possible
- Buffering tuned for MCP message sizes

### Detector Parallelization

```rust
async fn run_detectors(
    detectors: &[Box<dyn RuntimeDetector>],
    message: &Message,
) -> Vec<ThreatAlert> {
    // Run all detectors in parallel
    let futures = detectors.iter()
        .filter(|d| d.should_analyze(message))
        .map(|d| d.analyze_request(message));

    let results = futures::future::join_all(futures).await;

    results.into_iter()
        .filter_map(|r| r.ok())
        .flatten()
        .collect()
}
```

### Caching

- Tool snapshots cached in memory (rug pull detector)
- Request history in circular buffer (behavioral detector)
- Detector results NOT cached (must be real-time)

---

## Configuration

### Example Config

```yaml
# .mcp-sentinel.yaml
runtime_proxy:
  enabled: true
  listen: "127.0.0.1:8080"
  upstream: "127.0.0.1:3000"
  mode: enforce  # or: monitor

  # Performance
  max_connections: 1000
  timeout_ms: 30000
  buffer_size: 65536

  # Detectors
  detectors:
    data_exfiltration:
      enabled: true
      max_response_size: 1048576  # 1MB
      sensitive_patterns:
        - '\d{3}-\d{2}-\d{4}'  # SSN
        - '\d{16}'              # Credit card

    rug_pull:
      enabled: true

    behavioral_anomaly:
      enabled: true
      rate_limit_window: 60s
      max_requests_per_window: 100

  # Policy
  policy:
    block_on_critical: true
    block_on_high: false
    alert_webhook: "https://alerts.example.com/webhook"

    custom_rules:
      - name: "Block filesystem access"
        condition:
          tool_name_matches: ".*file.*"
        action: block
```

---

## Testing Strategy

### Unit Tests
- Protocol parser (malformed JSON, partial messages)
- Each detector in isolation
- Guardrails policy evaluation

### Integration Tests
- Full proxy with mock upstream
- Multiple concurrent connections
- Error conditions (upstream down, timeout)
- Performance benchmarks

### Stress Tests
- 10,000 concurrent connections
- Sustained 1,000 req/s for 1 hour
- Memory leak detection
- Connection pool exhaustion

---

## Deployment Guide

### Standalone Mode

```bash
# Start proxy
mcp-sentinel proxy \
  --listen 127.0.0.1:8080 \
  --upstream 127.0.0.1:3000 \
  --mode enforce

# Configure MCP client to use proxy
export MCP_SERVER_URL="http://127.0.0.1:8080"
```

### Docker Compose

```yaml
version: '3.8'
services:
  mcp-sentinel-proxy:
    image: ghcr.io/beejak/mcp-sentinel:3.0.0
    ports:
      - "8080:8080"
    environment:
      - PROXY_UPSTREAM=http://mcp-server:3000
      - PROXY_MODE=enforce
    volumes:
      - ./config.yaml:/etc/mcp-sentinel/config.yaml

  mcp-server:
    image: my-mcp-server:latest
    ports:
      - "3000:3000"
```

### Kubernetes

```yaml
apiVersion: v1
kind: Service
metadata:
  name: mcp-sentinel-proxy
spec:
  selector:
    app: mcp-sentinel-proxy
  ports:
    - port: 8080
      targetPort: 8080
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: mcp-sentinel-proxy
spec:
  replicas: 3
  selector:
    matchLabels:
      app: mcp-sentinel-proxy
  template:
    metadata:
      labels:
        app: mcp-sentinel-proxy
    spec:
      containers:
      - name: proxy
        image: ghcr.io/beejak/mcp-sentinel:3.0.0
        ports:
        - containerPort: 8080
        env:
        - name: PROXY_UPSTREAM
          value: "http://mcp-server:3000"
        - name: PROXY_MODE
          value: "enforce"
```

---

## Monitoring & Observability

### Metrics Exposed

```
# Prometheus format
mcp_sentinel_proxy_connections_active 45
mcp_sentinel_proxy_requests_total{method="tools/call"} 1234
mcp_sentinel_proxy_threats_detected{category="data_exfiltration"} 5
mcp_sentinel_proxy_latency_seconds{quantile="0.95"} 0.008
mcp_sentinel_proxy_blocked_requests_total 3
```

### Logging

```
2025-10-29T12:34:56Z INFO  Runtime proxy started on 127.0.0.1:8080
2025-10-29T12:35:01Z DEBUG New connection from 192.168.1.100:54321
2025-10-29T12:35:02Z WARN  Threat detected: data_exfiltration, severity=high
2025-10-29T12:35:02Z ERROR Request blocked: Critical threat
```

---

## Security Considerations

### Proxy Security
- No secrets in logs
- TLS termination support (for HTTPS)
- Authentication for proxy admin API
- Rate limiting per client IP

### Attack Surface
- Proxy itself is an attack surface
- Must validate all input
- DoS protection (connection limits, timeouts)
- No code execution in detectors

---

## Future Enhancements

- [ ] HTTP/2 and HTTP/3 support
- [ ] gRPC protocol inspection
- [ ] Machine learning for anomaly detection
- [ ] Distributed tracing integration (OpenTelemetry)
- [ ] Replay attacks from logs
- [ ] Shadow mode (duplicate traffic to test server)

---

**Document Status**: Draft for Review
**Next Steps**: Prototype implementation in Phase 3.1
