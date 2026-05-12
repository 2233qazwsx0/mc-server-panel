-- Player Statistics Table
CREATE TABLE player_stats (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    player_uuid VARCHAR(36) NOT NULL UNIQUE,
    player_name VARCHAR(16) NOT NULL,
    play_time_seconds BIGINT DEFAULT 0,
    blocks_placed INTEGER DEFAULT 0,
    blocks_broken INTEGER DEFAULT 0,
    deaths INTEGER DEFAULT 0,
    kills INTEGER DEFAULT 0,
    last_login TIMESTAMP,
    first_join TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Economy Table
CREATE TABLE economy (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    player_uuid VARCHAR(36) NOT NULL UNIQUE,
    balance DECIMAL(20, 2) NOT NULL DEFAULT 0.00,
    total_earned DECIMAL(20, 2) NOT NULL DEFAULT 0.00,
    total_spent DECIMAL(20, 2) NOT NULL DEFAULT 0.00,
    transaction_count INTEGER DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Transaction History Table
CREATE TABLE transactions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    player_uuid VARCHAR(36) NOT NULL,
    transaction_type VARCHAR(20) NOT NULL,
    amount DECIMAL(20, 2) NOT NULL,
    balance_before DECIMAL(20, 2) NOT NULL,
    balance_after DECIMAL(20, 2) NOT NULL,
    description TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_player_transactions (player_uuid),
    INDEX idx_transaction_time (created_at)
);

-- API Keys Table
CREATE TABLE api_keys (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    key_hash VARCHAR(64) NOT NULL UNIQUE,
    key_name VARCHAR(100) NOT NULL,
    permissions JSON NOT NULL DEFAULT '[]',
    rate_limit INTEGER DEFAULT 100,
    is_active BOOLEAN DEFAULT TRUE,
    expires_at TIMESTAMP,
    last_used TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Performance Metrics Table (for query performance analysis)
CREATE TABLE query_metrics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    query_hash VARCHAR(64) NOT NULL,
    query_type VARCHAR(50) NOT NULL,
    execution_time_ms INTEGER NOT NULL,
    rows_affected INTEGER DEFAULT 0,
    table_name VARCHAR(100),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_query_performance (query_type, execution_time_ms),
    INDEX idx_query_time (created_at)
);

-- Archive Metadata Table
CREATE TABLE archive_metadata (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    archive_name VARCHAR(255) NOT NULL,
    archive_type VARCHAR(50) NOT NULL,
    source_table VARCHAR(100) NOT NULL,
    record_count INTEGER NOT NULL,
    file_size_bytes BIGINT,
    compressed BOOLEAN DEFAULT FALSE,
    archived_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    retention_days INTEGER DEFAULT 90,
    auto_delete BOOLEAN DEFAULT TRUE
);

-- Backup Metadata Table
CREATE TABLE backup_metadata (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    backup_name VARCHAR(255) NOT NULL,
    backup_path VARCHAR(500) NOT NULL,
    backup_type VARCHAR(50) NOT NULL,
    file_size_bytes BIGINT,
    checksum VARCHAR(64),
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP,
    INDEX idx_backup_status (status),
    INDEX idx_backup_time (created_at)
);

-- Sync Status Table (for real-time data sync)
CREATE TABLE sync_status (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sync_type VARCHAR(50) NOT NULL,
    target_system VARCHAR(100),
    last_sync_at TIMESTAMP,
    status VARCHAR(20) NOT NULL DEFAULT 'idle',
    records_synced INTEGER DEFAULT 0,
    errors TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
