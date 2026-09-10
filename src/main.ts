/**
 * Point d'entrée de l'application Vue : résout le thème d'affichage avant
 * le premier rendu (pour éviter un flash du mauvais thème), puis monte le
 * composant racine `App` dans le conteneur `#app` de `index.html`.
 */

// Importation des modules Vue / framework
import { createApp } from "vue";

// Importation des modules internes
import "./styles.css";
import App from "./App.vue";
import { initTheme } from "./composables/useTheme";

async function bootstrap() {
  await initTheme();
  createApp(App).mount("#app");
}

bootstrap();
