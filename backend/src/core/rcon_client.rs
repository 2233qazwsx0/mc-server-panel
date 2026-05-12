use anyhow::Result;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock as TokioRwLock;
use tracing::{debug, info};

#[derive(Debug, Clone, serde::Serialize)]
pub struct PlayerInfo {
    pub name: String,
    pub uuid: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ServerStats {
    pub tps: Option<f64>,
    pub mspt: Option<f64>,
    pub online_players: Vec<PlayerInfo>,
    pub max_players: u32,
}

#[derive(Clone)]
pub struct RconClient {
    conn: Arc<TokioRwLock<Option<rcon::Connection<tokio::net::TcpStream>>>>,
    address: String,
    password: String,
    command_validator: Arc<CommandValidator>,
    stats: Arc<TokioRwLock<ServerStats>>,
}

pub struct CommandValidator {
    allowed_commands: HashSet<String>,
    allowed_patterns: Vec<regex::Regex>,
    blocked_patterns: Vec<regex::Regex>,
}

impl CommandValidator {
    pub fn new() -> Self {
        let mut allowed = HashSet::new();
        allowed.insert("list".to_string());
        allowed.insert("help".to_string());
        allowed.insert("say".to_string());
        allowed.insert("tell".to_string());
        allowed.insert("msg".to_string());
        allowed.insert("w".to_string());
        allowed.insert("whitelist".to_string());
        allowed.insert("kick".to_string());
        allowed.insert("ban".to_string());
        allowed.insert("banlist".to_string());
        allowed.insert("pardon".to_string());
        allowed.insert("op".to_string());
        allowed.insert("deop".to_string());
        allowed.insert("time".to_string());
        allowed.insert("weather".to_string());
        allowed.insert("gamemode".to_string());
        allowed.insert("difficulty".to_string());
        allowed.insert("tp".to_string());
        allowed.insert("teleport".to_string());
        allowed.insert("give".to_string());
        allowed.insert("clear".to_string());
        allowed.insert("effect".to_string());
        allowed.insert("enchant".to_string());
        allowed.insert("experience".to_string());
        allowed.insert("xp".to_string());
        allowed.insert("spawnpoint".to_string());
        allowed.insert("setworldspawn".to_string());
        allowed.insert("gamerule".to_string());
        allowed.insert("defaultgamemode".to_string());
        allowed.insert("title".to_string());
        allowed.insert("me".to_string());
        allowed.insert("teammsg".to_string());
        allowed.insert("reload".to_string());
        allowed.insert("stop".to_string());
        allowed.insert("save".to_string());
        allowed.insert("seed".to_string());
        allowed.insert("plugins".to_string());
        allowed.insert("version".to_string());
        allowed.insert("timings".to_string());
        allowed.insert("spark".to_string());
        allowed.insert("lag".to_string());

        let allowed_patterns = vec![
            regex::Regex::new(r"^(list)\s*$").unwrap(),
            regex::Regex::new(r"^(tell|msg|w)\s+\S+\s+.+$").unwrap(),
            regex::Regex::new(r"^(whitelist)\s+(add|remove|list|on|off|reload|check)\s*\S*$").unwrap(),
            regex::Regex::new(r"^(kick)\s+\S+\s*.+$").unwrap(),
            regex::Regex::new(r"^(ban|pardon)\s+\S+\s*.+$").unwrap(),
            regex::Regex::new(r"^(op|deop)\s+\S+$").unwrap(),
            regex::Regex::new(r"^(time)\s+(set|add|query)\s+\d+$").unwrap(),
            regex::Regex::new(r"^(weather)\s+(clear|rain|thunder)(?:\s+\d+)?$").unwrap(),
            regex::Regex::new(r"^(gamemode|gm)\s+(survival|creative|spectator|adventure|0|1|2|3)(\s+\S+)?$").unwrap(),
            regex::Regex::new(r"^(tp|teleport)\s+\S+\s+\S+(\s+\S+)?$").unwrap(),
            regex::Regex::new(r"^(give)\s+\S+\s+\S+\s*\d*$").unwrap(),
            regex::Regex::new(r"^(effect)\s+(give|clear|remove)\s+\S+\s*\d*$").unwrap(),
            regex::Regex::new(r"^(title)\s+\S+\s+.+$").unwrap(),
            regex::Regex::new(r"^(gamerule)\s+\S+\s*.+$").unwrap(),
            regex::Regex::new(r"^(execute)\s+.+$").unwrap(),
        ];

        let blocked_patterns = vec![
            regex::Regex::new(r"(?i)(rm\s+-rf|mv\s+/dev|sudo\s+rm)").unwrap(),
            regex::Regex::new(r"(?i)(eval|exec|source)\s*\(").unwrap(),
            regex::Regex::new(r"[;&|`$]").unwrap(),
        ];

        Self {
            allowed_commands: allowed,
            allowed_patterns,
            blocked_patterns,
        }
    }

    pub fn validate(&self, command: &str) -> std::result::Result<(), String> {
        let trimmed = command.trim();

        if trimmed.is_empty() {
            return Err("Empty command".to_string());
        }

        if trimmed.len() > 500 {
            return Err("Command too long (max 500 characters)".to_string());
        }

        for pattern in &self.blocked_patterns {
            if pattern.is_match(trimmed) {
                return Err("Command contains blocked pattern".to_string());
            }
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.is_empty() {
            return Err("Invalid command format".to_string());
        }

        let base_cmd = parts[0].to_lowercase();

        if self.allowed_commands.contains(&base_cmd) {
            return Ok(());
        }

        for pattern in &self.allowed_patterns {
            if pattern.is_match(trimmed) {
                return Ok(());
            }
        }

        Err(format!("Command not allowed: {}", base_cmd))
    }
}

impl RconClient {
    pub fn new(address: &str, password: &str) -> Self {
        Self {
            conn: Arc::new(TokioRwLock::new(None)),
            address: address.to_string(),
            password: password.to_string(),
            command_validator: Arc::new(CommandValidator::new()),
            stats: Arc::new(TokioRwLock::new(ServerStats::default())),
        }
    }

    pub async fn connect(&self) -> Result<()> {
        let mut guard = self.conn.write().await;

        if guard.is_some() {
            return Ok(());
        }

        info!("Connecting to RCON at {}", self.address);

        let conn = rcon::Builder::new()
            .connect(&self.address, &self.password)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect to {}: {}", self.address, e))?;

        *guard = Some(conn);
        info!("RCON connected successfully");

        Ok(())
    }

    pub async fn disconnect(&self) {
        let mut guard = self.conn.write().await;
        if let Some(_conn) = guard.take() {
            info!("RCON disconnected");
        }
    }

    pub async fn send_command(&self, command: &str) -> Result<String> {
        self.command_validator.validate(command)
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        let mut guard = self.conn.write().await;
        let conn = guard.as_mut()
            .ok_or_else(|| anyhow::anyhow!("RCON not connected"))?;

        let response = conn.cmd(command).await
            .map_err(|e| anyhow::anyhow!("Failed to execute command {}: {}", command, e))?;

        debug!("RCON command executed: {} -> {}", command, response);
        Ok(response)
    }

    pub async fn is_connected(&self) -> bool {
        self.conn.read().await.is_some()
    }

    pub async fn get_tps(&self) -> Result<f64> {
        let responses = [
            self.send_command("forge tps").await,
            self.send_command("tps").await,
            self.send_command("spark tps").await,
        ];

        for response in responses {
            if let Ok(resp) = response {
                if let Some(tps) = parse_tps(&resp) {
                    let mut stats = self.stats.write().await;
                    stats.tps = Some(tps);
                    return Ok(tps);
                }
            }
        }

        Err(anyhow::anyhow!("Failed to get TPS"))
    }

    pub async fn get_player_list(&self) -> Result<Vec<PlayerInfo>> {
        let response = self.send_command("list").await?;
        let players = parse_player_list(&response);

        let mut stats = self.stats.write().await;
        stats.online_players = players.clone();
        stats.max_players = 20;

        Ok(players)
    }

    pub async fn get_server_stats(&self) -> Result<ServerStats> {
        let mut stats = ServerStats::default();

        if let Ok(tps) = self.get_tps().await {
            stats.tps = Some(tps);
            stats.mspt = Some((1_000_000.0 / tps).round() / 1000.0);
        }

        if let Ok(players) = self.get_player_list().await {
            stats.online_players = players;
        }

        Ok(stats)
    }

    pub async fn get_cached_stats(&self) -> ServerStats {
        self.stats.read().await.clone()
    }
}

fn parse_tps(response: &str) -> Option<f64> {
    let patterns = [
        r"TPS from last 1m, 5m, 15m:\s*([\d.]+),\s*([\d.]+),\s*([\d.]+)",
        r"Mean tick time: ([\d.]+) ms",
        r"Overall TPS:\s*([\d.]+)",
        r"tps:\s*([\d.]+)",
    ];

    for pattern in &patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            if let Some(caps) = re.captures(response) {
                if let Some(tps) = caps.get(1) {
                    if let Ok(tps_val) = tps.as_str().parse::<f64>() {
                        if tps_val > 0.0 && tps_val <= 20.0 {
                            return Some(tps_val);
                        }
                    }
                }
            }
        }
    }

    for line in response.lines() {
        let lower = line.to_lowercase();
        if lower.contains("tps") {
            let numbers: Vec<f64> = line.split(|c: char| !c.is_numeric() && c != '.')
                .filter_map(|s| s.parse().ok())
                .collect();
            for num in numbers {
                if num > 0.0 && num <= 20.0 {
                    return Some(num);
                }
            }
        }
    }

    None
}

fn parse_player_list(response: &str) -> Vec<PlayerInfo> {
    let mut players = Vec::new();

    let patterns = [
        r"There are (\d+) of a max of (\d+) players? online:",
        r"Online players:\s*(.*)",
        r"players online:\s*(.*)",
        r"(\d+) of (\d+) players",
    ];

    let mut player_line = String::new();

    for pattern in &patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            if let Some(caps) = re.captures(response) {
                if caps.len() > 2 {
                    player_line = caps[caps.len() - 1].to_string();
                }
                break;
            }
        }
    }

    if player_line.is_empty() {
        player_line = response.to_string();
    }

    let names: Vec<String> = player_line
        .split(|c: char| ![',', ' '].contains(&c))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .filter(|s| !s.chars().all(|c| c.is_numeric()))
        .filter(|s| {
            !s.to_lowercase().contains("online")
            && !s.to_lowercase().contains("players")
            && s != ":"
            && !s.starts_with('[')
        })
        .collect();

    for name in names {
        let clean_name = name.trim_matches(|c: char| c.is_whitespace() || c == ':' || c == '\n' || c == '\t');
        if !clean_name.is_empty() && clean_name.len() <= 16 {
            players.push(PlayerInfo {
                name: clean_name.to_string(),
                uuid: None,
            });
        }
    }

    players
}
