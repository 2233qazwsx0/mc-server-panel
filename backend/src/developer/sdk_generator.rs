use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::state::AppState;

#[derive(Debug, Serialize, ToSchema)]
pub struct SdkGenerateRequest {
    pub language: String,
    pub include_examples: Option<bool>,
    pub base_url: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SdkGenerateResponse {
    pub language: String,
    pub code: String,
    pub filename: String,
    pub imports: Vec<String>,
}

lazy_static::lazy_static! {
    static ref SDK_TEMPLATES: std::collections::HashMap<String, &'static str> = {
        let mut m = std::collections::HashMap::new();
        m.insert("typescript".to_string(), TYPESCRIPT_TEMPLATE);
        m.insert("javascript".to_string(), JAVASCRIPT_TEMPLATE);
        m.insert("python".to_string(), PYTHON_TEMPLATE);
        m.insert("rust".to_string(), RUST_TEMPLATE);
        m.insert("go".to_string(), GO_TEMPLATE);
        m
    };
}

const TYPESCRIPT_TEMPLATE: &str = r#"/**
 * Minecraft Server Admin Panel SDK
 * Generated TypeScript SDK for MC Server Panel API
 */

export interface ServerStatus {
  online: boolean;
  cpu: number;
  memory: number;
  tps: number;
  players: number;
  maxPlayers: number;
}

export interface CommandRequest {
  command: string;
}

export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
}

export interface MetricsResponse {
  cpu: number;
  memory: number;
  uptimeSeconds: number;
  timestamp: string;
}

export class McServerClient {
  private baseUrl: string;
  private apiKey?: string;

  constructor(baseUrl: string, apiKey?: string) {
    this.baseUrl = baseUrl.replace(/\/$/, '');
    this.apiKey = apiKey;
  }

  private async request<T>(
    method: string,
    path: string,
    body?: any
  ): Promise<ApiResponse<T>> {
    const headers: HeadersInit = {
      'Content-Type': 'application/json',
    };
    if (this.apiKey) {
      headers['Authorization'] = `Bearer ${this.apiKey}`;
    }

    const response = await fetch(`${this.baseUrl}${path}`, {
      method,
      headers,
      body: body ? JSON.stringify(body) : undefined,
    });

    return response.json();
  }

  async getStatus(): Promise<ApiResponse<ServerStatus>> {
    return this.request<ServerStatus>('GET', '/api/status');
  }

  async startServer(): Promise<ApiResponse<void>> {
    return this.request<void>('POST', '/api/start');
  }

  async stopServer(): Promise<ApiResponse<void>> {
    return this.request<void>('POST', '/api/stop');
  }

  async restartServer(): Promise<ApiResponse<void>> {
    return this.request<void>('POST', '/api/restart');
  }

  async sendCommand(command: string): Promise<ApiResponse<string>> {
    return this.request<string>('POST', '/api/command', { command });
  }

  async getMetrics(): Promise<ApiResponse<MetricsResponse>> {
    return this.request<MetricsResponse>('GET', '/api/metrics');
  }

  async getLogs(): Promise<ApiResponse<string[]>> {
    return this.request<string[]>('GET', '/api/logs');
  }

  async connectRcon(password: string): Promise<ApiResponse<void>> {
    return this.request<void>('POST', '/api/rcon/connect', { password });
  }

  async disconnectRcon(): Promise<ApiResponse<void>> {
    return this.request<void>('POST', '/api/rcon/disconnect');
  }

  async getPlayerList(): Promise<ApiResponse<string[]>> {
    return this.request<string[]>('GET', '/api/rcon/players');
  }
}

export default McServerClient;
"#;

const JAVASCRIPT_TEMPLATE: &str = r#"/**
 * Minecraft Server Admin Panel SDK
 * Generated JavaScript SDK for MC Server Panel API
 */

class McServerClient {
  constructor(baseUrl, apiKey) {
    this.baseUrl = baseUrl.replace(/\/$/, '');
    this.apiKey = apiKey;
  }

  async request(method, path, body) {
    const headers = {
      'Content-Type': 'application/json',
    };
    if (this.apiKey) {
      headers['Authorization'] = `Bearer ${this.apiKey}`;
    }

    const response = await fetch(`${this.baseUrl}${path}`, {
      method,
      headers,
      body: body ? JSON.stringify(body) : undefined,
    });

    return response.json();
  }

  async getStatus() {
    return this.request('GET', '/api/status');
  }

  async startServer() {
    return this.request('POST', '/api/start');
  }

  async stopServer() {
    return this.request('POST', '/api/stop');
  }

  async restartServer() {
    return this.request('POST', '/api/restart');
  }

  async sendCommand(command) {
    return this.request('POST', '/api/command', { command });
  }

  async getMetrics() {
    return this.request('GET', '/api/metrics');
  }

  async getLogs() {
    return this.request('GET', '/api/logs');
  }
}

module.exports = McServerClient;
"#;

const PYTHON_TEMPLATE: &str = r#"# Minecraft Server Admin Panel SDK
# Generated Python SDK for MC Server Panel API

import json
from typing import Optional, Dict, Any, List
from dataclasses import dataclass

try:
    import requests
except ImportError:
    raise ImportError("Please install requests: pip install requests")

@dataclass
class ServerStatus:
    online: bool
    cpu: float
    memory: float
    tps: float
    players: int
    max_players: int

@dataclass
class ApiResponse:
    success: bool
    data: Optional[Any] = None
    error: Optional[str] = None

class McServerClient:
    def __init__(self, base_url: str, api_key: Optional[str] = None):
        self.base_url = base_url.rstrip('/')
        self.api_key = api_key
        self.session = requests.Session()
        if api_key:
            self.session.headers.update({'Authorization': f'Bearer {api_key}'})
        self.session.headers.update({'Content-Type': 'application/json'})

    def request(self, method: str, path: str, data: Optional[Dict] = None) -> Dict:
        url = f"{self.base_url}{path}"
        response = self.session.request(method, url, json=data)
        return response.json()

    def get_status(self) -> ApiResponse:
        return self.request('GET', '/api/status')

    def start_server(self) -> ApiResponse:
        return self.request('POST', '/api/start')

    def stop_server(self) -> ApiResponse:
        return self.request('POST', '/api/stop')

    def restart_server(self) -> ApiResponse:
        return self.request('POST', '/api/restart')

    def send_command(self, command: str) -> ApiResponse:
        return self.request('POST', '/api/command', {'command': command})

    def get_metrics(self) -> ApiResponse:
        return self.request('GET', '/api/metrics')

    def get_logs(self) -> ApiResponse:
        return self.request('GET', '/api/logs')

    def connect_rcon(self, password: str) -> ApiResponse:
        return self.request('POST', '/api/rcon/connect', {'password': password})

    def disconnect_rcon(self) -> ApiResponse:
        return self.request('POST', '/api/rcon/disconnect')

    def get_player_list(self) -> ApiResponse:
        return self.request('GET', '/api/rcon/players')
"#;

const RUST_TEMPLATE: &str = r#"// Minecraft Server Admin Panel SDK
// Generated Rust SDK for MC Server Panel API

use serde::{Deserialize, Serialize};
use reqwest::Client;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerStatus {
    pub online: bool,
    pub cpu: f64,
    pub memory: f64,
    pub tps: f64,
    pub players: u32,
    #[serde(rename = "maxPlayers")]
    pub max_players: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandRequest {
    pub command: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetricsResponse {
    pub cpu: f64,
    pub memory: f64,
    #[serde(rename = "uptimeSeconds")]
    pub uptime_seconds: u64,
    pub timestamp: String,
}

#[derive(Clone)]
pub struct McServerClient {
    client: Client,
    base_url: String,
    api_key: Option<String>,
}

impl McServerClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: None,
        }
    }

    pub fn with_api_key(mut self, api_key: &str) -> Self {
        self.api_key = Some(api_key.to_string());
        self
    }

    pub async fn get_status(&self) -> Result<ApiResponse<ServerStatus>, reqwest::Error> {
        self.client
            .get(format!("{}/api/status", self.base_url))
            .send()
            .await?
            .json()
            .await
    }

    pub async fn start_server(&self) -> Result<ApiResponse<()>, reqwest::Error> {
        self.client
            .post(format!("{}/api/start", self.base_url))
            .send()
            .await?
            .json()
            .await
    }

    pub async fn stop_server(&self) -> Result<ApiResponse<()>, reqwest::Error> {
        self.client
            .post(format!("{}/api/stop", self.base_url))
            .send()
            .await?
            .json()
            .await
    }

    pub async fn send_command(&self, command: &str) -> Result<ApiResponse<String>, reqwest::Error> {
        self.client
            .post(format!("{}/api/command", self.base_url))
            .json(&CommandRequest { command: command.to_string() })
            .send()
            .await?
            .json()
            .await
    }

    pub async fn get_metrics(&self) -> Result<ApiResponse<MetricsResponse>, reqwest::Error> {
        self.client
            .get(format!("{}/api/metrics", self.base_url))
            .send()
            .await?
            .json()
            .await
    }
}
"#;

const GO_TEMPLATE: &str = r#"// Minecraft Server Admin Panel SDK
// Generated Go SDK for MC Server Panel API

package mcserver

import (
    "bytes"
    "encoding/json"
    "net/http"
)

type ServerStatus struct {
    Online     bool    `json:"online"`
    Cpu        float64 `json:"cpu"`
    Memory     float64 `json:"memory"`
    Tps        float64 `json:"tps"`
    Players    uint32  `json:"players"`
    MaxPlayers uint32  `json:"maxPlayers"`
}

type CommandRequest struct {
    Command string `json:"command"`
}

type ApiResponse struct {
    Success bool        `json:"success"`
    Data    interface{} `json:"data,omitempty"`
    Error   string      `json:"error,omitempty"`
}

type Client struct {
    BaseURL string
    APIKey  string
    Client  *http.Client
}

func NewClient(baseURL string) *Client {
    return &Client{
        BaseURL: baseURL,
        Client:  &http.Client{},
    }
}

func (c *Client) SetAPIKey(apiKey string) {
    c.APIKey = apiKey
}

func (c *Client) request(method, path string, body interface{}) (*ApiResponse, error) {
    var reqBody *bytes.Buffer
    if body != nil {
        jsonData, err := json.Marshal(body)
        if err != nil {
            return nil, err
        }
        reqBody = bytes.NewBuffer(jsonData)
    }

    req, err := http.NewRequest(method, c.BaseURL+path, reqBody)
    if err != nil {
        return nil, err
    }

    req.Header.Set("Content-Type", "application/json")
    if c.APIKey != "" {
        req.Header.Set("Authorization", "Bearer "+c.APIKey)
    }

    resp, err := c.Client.Do(req)
    if err != nil {
        return nil, err
    }
    defer resp.Body.Close()

    var apiResp ApiResponse
    if err := json.NewDecoder(resp.Body).Decode(&apiResp); err != nil {
        return nil, err
    }

    return &apiResp, nil
}

func (c *Client) GetStatus() (*ApiResponse, error) {
    return c.request("GET", "/api/status", nil)
}

func (c *Client) StartServer() (*ApiResponse, error) {
    return c.request("POST", "/api/start", nil)
}

func (c *Client) StopServer() (*ApiResponse, error) {
    return c.request("POST", "/api/stop", nil)
}

func (c *Client) RestartServer() (*ApiResponse, error) {
    return c.request("POST", "/api/restart", nil)
}

func (c *Client) SendCommand(command string) (*ApiResponse, error) {
    return c.request("POST", "/api/command", CommandRequest{Command: command})
}

func (c *Client) GetMetrics() (*ApiResponse, error) {
    return c.request("GET", "/api/metrics", nil)
}

func (c *Client) GetLogs() (*ApiResponse, error) {
    return c.request("GET", "/api/logs", nil)
}
"#;

#[utoipa::path(
    post,
    path = "/api/developer/sdk/generate",
    request_body = SdkGenerateRequest,
    responses(
        (status = 200, description = "Generate SDK code", body = SdkGenerateResponse)
    ),
    tag = "Developer"
)]
pub async fn generate_sdk(
    State(state): State<AppState>,
    Json(req): Json<SdkGenerateRequest>,
) -> Result<Json<SdkGenerateResponse>, crate::error::AppError> {
    let base_url = req.base_url.unwrap_or_else(|| "http://localhost:8080".to_string());
    let include_examples = req.include_examples.unwrap_or(true);
    
    let language = req.language.to_lowercase();
    
    let template = SDK_TEMPLATES.get(&language)
        .ok_or_else(|| crate::error::AppError::Internal(format!("Unsupported language: {}", language)))?;
    
    let code = if include_examples {
        format!("// Base URL: {}\n\n{}", base_url, template)
    } else {
        template.to_string()
    };
    
    let filename = match language.as_str() {
        "typescript" => "mc-server-client.ts",
        "javascript" => "mc-server-client.js",
        "python" => "mc_server_client.py",
        "rust" => "mc_server_client.rs",
        "go" => "mc_server_client.go",
        _ => "client.sdk",
    };
    
    let imports = match language.as_str() {
        "typescript" | "javascript" => vec!["fetch or request library".to_string()],
        "python" => vec!["requests".to_string()],
        "rust" => vec!["reqwest".to_string(), "serde".to_string()],
        "go" => vec!["net/http".to_string(), "encoding/json".to_string()],
        _ => vec![],
    };
    
    Ok(Json(SdkGenerateResponse {
        language: language.clone(),
        code,
        filename: filename.to_string(),
        imports,
    }))
}

#[utoipa::path(
    get,
    path = "/api/developer/sdk/languages",
    responses(
        (status = 200, description = "List supported SDK languages")
    ),
    tag = "Developer"
)]
pub async fn list_sdk_languages() -> Result<impl IntoResponse, crate::error::AppError> {
    let languages = vec![
        serde_json::json!({
            "id": "typescript",
            "name": "TypeScript",
            "extension": "ts",
            "featured": true
        }),
        serde_json::json!({
            "id": "javascript",
            "name": "JavaScript",
            "extension": "js",
            "featured": true
        }),
        serde_json::json!({
            "id": "python",
            "name": "Python",
            "extension": "py",
            "featured": true
        }),
        serde_json::json!({
            "id": "rust",
            "name": "Rust",
            "extension": "rs",
            "featured": false
        }),
        serde_json::json!({
            "id": "go",
            "name": "Go",
            "extension": "go",
            "featured": false
        }),
    ];
    
    Ok(Json(languages))
}
