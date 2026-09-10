// Configuration ESLint (flat config) — lint TypeScript et Vue 3 (script setup).
// Voir rules/typescript_vue.md : strict TS, pas de `any` non justifié, Composition API uniquement.

import js from '@eslint/js'
import tseslint from 'typescript-eslint'
import pluginVue from 'eslint-plugin-vue'
import globals from 'globals'

export default tseslint.config(
  {
    ignores: ['dist/**', 'node_modules/**', 'src-tauri/**', 'dev-data/**'],
  },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  // 'essential' (Vue Style Guide Priority A) plutôt que 'recommended' : ce dernier
  // impose aussi des règles de mise en forme (retours à la ligne des attributs...)
  // qui contredisent le style compact déjà en place dans ce projet (SVG en ligne, etc.).
  ...pluginVue.configs['flat/essential'],
  {
    languageOptions: {
      globals: globals.browser,
    },
  },
  {
    files: ['**/*.vue'],
    languageOptions: {
      parserOptions: {
        parser: tseslint.parser,
      },
    },
  },
  {
    rules: {
      // Un fichier = une responsabilité (rules/typescript_vue.md) : Options API bannie.
      'vue/component-api-style': ['error', ['script-setup']],
      // Vite gère le HMR sans exiger un nom de composant explicite par fichier.
      'vue/multi-word-component-names': 'off',
      '@typescript-eslint/no-explicit-any': 'error',
      '@typescript-eslint/no-unused-vars': ['warn', { argsIgnorePattern: '^_' }],
    },
  },
)
