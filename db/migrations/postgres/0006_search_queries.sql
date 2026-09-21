-- Requêtes de recherche d'offres, rattachées à une session : remplace la
-- config statique `[[searches]]` d'agent.toml par une liste éditable par
-- session (mots-clés, localisation, pays).
CREATE TABLE search.search_queries (
    id BIGSERIAL PRIMARY KEY,
    session_id BIGINT NOT NULL REFERENCES search.sessions(id) ON DELETE CASCADE,
    keywords TEXT NOT NULL,
    location TEXT,
    country_code TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_search_queries_session_id ON search.search_queries(session_id);
