-- Users table: stores OIDC user profiles
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,                          -- OIDC sub claim
    email TEXT NOT NULL,
    name TEXT NOT NULL DEFAULT '',
    preferred_username TEXT NOT NULL DEFAULT '',
    oidc_issuer TEXT NOT NULL,
    first_seen_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_login_at TEXT NOT NULL DEFAULT (datetime('now')),
    is_active INTEGER NOT NULL DEFAULT 1
);

-- Projects table: stores user projects with calculation data
CREATE TABLE IF NOT EXISTS projects (
    id TEXT PRIMARY KEY,                          -- UUID v4
    user_id TEXT NOT NULL REFERENCES users(id),
    name TEXT NOT NULL DEFAULT 'Naamloos project',
    project_data TEXT NOT NULL,                   -- Full Project JSON
    result_data TEXT,                             -- Last calculation result (nullable)
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    is_archived INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_projects_user_id ON projects(user_id);
CREATE INDEX IF NOT EXISTS idx_projects_updated_at ON projects(updated_at);
