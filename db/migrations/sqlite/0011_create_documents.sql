-- Trace en base des documents de référence (CV, lettre de motivation) dont le
-- contenu vit sur le filesystem. Prépare le terrain pour le cache d'analyse
-- de style LLM (`style_analysis`, rempli plus tard par le pipeline IA).

CREATE TABLE documents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER NOT NULL,
    doc_type TEXT NOT NULL CHECK (doc_type IN ('cv', 'lettre_motivation')),
    original_filename TEXT NOT NULL,
    file_path TEXT NOT NULL,
    pdf_path TEXT,
    style_analysis TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE,
    UNIQUE(session_id, doc_type)
);
