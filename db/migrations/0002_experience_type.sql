-- Ajoute la colonne type_contrat à la table experiences
ALTER TABLE experiences ADD COLUMN type_contrat TEXT
  CHECK(type_contrat IN ('stage', 'alternance', 'interim', 'cdd', 'cdi'));
