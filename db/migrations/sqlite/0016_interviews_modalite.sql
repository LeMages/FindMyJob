-- Modalité de l'entretien (visio / présentiel / téléphone), distincte de `lieu`
-- qui reste du texte libre (adresse, lien de visio, numéro).
-- Colonne optionnelle : les entretiens déjà en base restent à NULL.

ALTER TABLE interviews ADD COLUMN modalite TEXT
    CHECK(modalite IN ('visio', 'presentiel', 'telephone')) DEFAULT NULL;
