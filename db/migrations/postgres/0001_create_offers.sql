CREATE EXTENSION IF NOT EXISTS vector;

CREATE TABLE offers (
    id              BIGSERIAL PRIMARY KEY,
    source          VARCHAR(50)  NOT NULL,  -- 'wttj', 'indeed', 'linkedin'...
    source_id       VARCHAR(255),           -- slug ou ID de la source
    title           TEXT         NOT NULL,
    company         TEXT         NOT NULL,
    location        TEXT         NOT NULL,
    url             TEXT         NOT NULL,
    contract_type   VARCHAR(100),
    posted_at       TIMESTAMPTZ,
    scraped_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    status          VARCHAR(20)  NOT NULL DEFAULT 'new',  -- new, reviewed, applied, dismissed
    score           REAL,                                 -- scoring LLM (étape 3)
    embedding       vector(1536),                         -- embeddings (étape 3)
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_offers_url UNIQUE (url)
);

CREATE INDEX idx_offers_status ON offers (status);
CREATE INDEX idx_offers_source ON offers (source);
CREATE INDEX idx_offers_posted_at ON offers (posted_at DESC);
