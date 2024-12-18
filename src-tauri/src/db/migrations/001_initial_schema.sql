-- Table principale des jeux
CREATE TABLE IF NOT EXISTS games (
    id TEXT PRIMARY KEY,
    platform_id TEXT NOT NULL,
    platform TEXT NOT NULL,
    title TEXT NOT NULL,
    install_path TEXT NOT NULL,
    executable TEXT,
    install_size INTEGER NOT NULL DEFAULT 0,
    version TEXT,
    last_updated INTEGER,
    last_played INTEGER,
    last_scan INTEGER,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- Table pour les métadonnées (une seule entrée par jeu)
CREATE TABLE IF NOT EXISTS game_metadata (
    game_id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT,
    developer TEXT,
    publisher TEXT,
    release_date TEXT,
    last_fetched INTEGER NOT NULL,
    FOREIGN KEY(game_id) REFERENCES games(id) ON DELETE CASCADE
);

-- Table pour les genres
CREATE TABLE IF NOT EXISTS game_genres (
    game_id TEXT NOT NULL,
    genre TEXT NOT NULL,
    PRIMARY KEY(game_id, genre),
    FOREIGN KEY(game_id) REFERENCES games(id) ON DELETE CASCADE
);

-- Table pour les tags
CREATE TABLE IF NOT EXISTS game_tags (
    game_id TEXT NOT NULL,
    tag TEXT NOT NULL,
    PRIMARY KEY(game_id, tag),
    FOREIGN KEY(game_id) REFERENCES games(id) ON DELETE CASCADE
);

-- Table pour les médias
CREATE TABLE IF NOT EXISTS game_media (
    game_id TEXT PRIMARY KEY,
    thumbnail TEXT,
    cover TEXT,
    background TEXT,
    icon TEXT,
    logo TEXT,
    last_fetched INTEGER NOT NULL,
    FOREIGN KEY(game_id) REFERENCES games(id) ON DELETE CASCADE
);

-- Table pour les screenshots
CREATE TABLE IF NOT EXISTS game_screenshots (
    game_id TEXT NOT NULL,
    url TEXT NOT NULL,
    PRIMARY KEY(game_id, url),
    FOREIGN KEY(game_id) REFERENCES games(id) ON DELETE CASCADE
);

-- Table pour les statistiques de jeu
CREATE TABLE IF NOT EXISTS game_stats (
    game_id TEXT PRIMARY KEY,
    total_playtime INTEGER DEFAULT 0,
    last_session_duration INTEGER DEFAULT 0,
    sessions_count INTEGER DEFAULT 0,
    first_played INTEGER,
    last_played INTEGER,
    FOREIGN KEY(game_id) REFERENCES games(id) ON DELETE CASCADE
);

-- Table pour les sessions de jeu
CREATE TABLE IF NOT EXISTS game_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    game_id TEXT NOT NULL,
    start_time INTEGER NOT NULL,
    end_time INTEGER,
    duration INTEGER,
    FOREIGN KEY(game_id) REFERENCES games(id) ON DELETE CASCADE
);