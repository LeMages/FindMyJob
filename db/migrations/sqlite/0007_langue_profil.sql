-- FindMyJob : langue dans laquelle le profil est rédigé (migration 0007).
-- Sert de langue source à la traduction des textes du profil quand un
-- document est généré dans une autre langue. Défaut : français.

ALTER TABLE users ADD COLUMN langue_profil TEXT NOT NULL DEFAULT 'fr' CHECK (langue_profil IN ('fr', 'en'));
