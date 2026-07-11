/**
 * Point d'entrée de l'application Vue : monte le composant racine `App`
 * dans le conteneur `#app` de `index.html`.
 */

// Importation des modules Vue / framework
import { createApp } from "vue";

// Importation des modules internes
import "./styles.css";
import App from "./App.vue";

createApp(App).mount("#app");
