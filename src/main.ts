/**
 * Point d'entrée de l'application Vue : résout le thème d'affichage et la
 * langue de l'interface avant le premier rendu (pour éviter un flash du
 * mauvais thème ou de la mauvaise langue), puis monte le composant racine
 * `App` dans le conteneur `#app` de `index.html`.
 */

// Importation des modules Vue / framework
import { createApp } from "vue";

// Importation des modules internes
import "./styles.css";
import App from "./App.vue";
import { initTheme } from "./composables/useTheme";
import { initUiLanguage } from "./composables/useUiLanguage";
import { i18n } from "./i18n";

async function bootstrap() {
  await Promise.all([initTheme(), initUiLanguage()]);
  createApp(App).use(i18n).mount("#app");
}

bootstrap();
