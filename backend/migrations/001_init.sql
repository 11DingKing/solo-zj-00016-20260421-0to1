CREATE TABLE IF NOT EXISTS short_links (
    id SERIAL PRIMARY KEY,
    short_code VARCHAR(6) UNIQUE NOT NULL,
    original_url TEXT NOT NULL,
    user_cookie VARCHAR(64) NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    is_active BOOLEAN DEFAULT TRUE
);

CREATE INDEX IF NOT EXISTS idx_short_links_code ON short_links(short_code);
CREATE INDEX IF NOT EXISTS idx_short_links_user ON short_links(user_cookie);
CREATE INDEX IF NOT EXISTS idx_short_links_expires ON short_links(expires_at);

CREATE TABLE IF NOT EXISTS link_visits (
    id SERIAL PRIMARY KEY,
    short_link_id INTEGER NOT NULL REFERENCES short_links(id) ON DELETE CASCADE,
    visited_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    ip_address VARCHAR(45),
    user_agent TEXT,
    referer TEXT
);

CREATE INDEX IF NOT EXISTS idx_link_visits_short_link ON link_visits(short_link_id);
CREATE INDEX IF NOT EXISTS idx_link_visits_visited_at ON link_visits(visited_at);
