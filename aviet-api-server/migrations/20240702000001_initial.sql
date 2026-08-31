CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    phone TEXT UNIQUE NOT NULL,
    password_hash TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    site_name TEXT NOT NULL DEFAULT 'Betika',
    is_active BOOLEAN DEFAULT true,
    session_data JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    last_active_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS strategies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    tuples JSONB NOT NULL,
    expected_amount DECIMAL(12, 2),
    start_value DECIMAL(12, 2),
    end_value DECIMAL(12, 2),
    profitier DECIMAL(12, 2),
    multiplier_base DECIMAL(12, 2),
    item_no INTEGER,
    choice INTEGER,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS game_rounds (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID REFERENCES sessions(id) ON DELETE SET NULL,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    round_id TEXT,
    bet_amount DECIMAL(12, 2) NOT NULL,
    odd_used DECIMAL(12, 2) NOT NULL,
    multiplier_used DECIMAL(12, 2) NOT NULL,
    cashout_target DECIMAL(12, 2) NOT NULL,
    actual_cashout DECIMAL(12, 2),
    result TEXT CHECK (result IN ('win', 'loss', 'pending', 'cancelled')),
    balance_before DECIMAL(12, 2),
    balance_after DECIMAL(12, 2),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS payouts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID REFERENCES sessions(id) ON DELETE SET NULL,
    value TEXT NOT NULL,
    multiplier DECIMAL(12, 4),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_active ON sessions(is_active) WHERE is_active = true;
CREATE INDEX IF NOT EXISTS idx_game_rounds_session ON game_rounds(session_id);
CREATE INDEX IF NOT EXISTS idx_game_rounds_user ON game_rounds(user_id);
CREATE INDEX IF NOT EXISTS idx_strategies_user ON strategies(user_id);
