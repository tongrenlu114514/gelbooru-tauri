import js from '@eslint/js'
import vuePlugin from 'eslint-plugin-vue'
import vueParser from 'vue-eslint-parser'
import prettier from 'eslint-config-prettier'
import tseslint from 'typescript-eslint'

export default tseslint.config(
  {
    ignores: ['dist/**', 'node_modules/**', 'src-tauri/**', '*.min.js']
  },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  prettier,
  {
    files: ['**/*.vue'],
    plugins: {
      vue: vuePlugin
    },
    languageOptions: {
      parser: vueParser,
      parserOptions: {
        parser: tseslint.parser,
        ecmaVersion: 'latest',
        sourceType: 'module'
      },
      globals: {
        browser: true,
        console: true,
        document: true,
        localStorage: true,
        navigator: true,
        Promise: true,
        URL: true,
        window: true,
        setTimeout: true,
        KeyboardEvent: true,
        Event: true,
        HTMLImageElement: true,
        HTMLElement: true,
        Element: true
      }
    },
    rules: {
      'vue/multi-word-component-names': 'off',
      'vue/no-v-html': 'warn',
      'vue/require-default-prop': 'off',
      'vue/require-explicit-emits': 'warn',
      'vue/no-parsing-error': ['error', { 'x-invalid-end-tag': false }]
    }
  },
  {
    files: ['**/*.ts', '**/*.js'],
    languageOptions: {
      ecmaVersion: 'latest',
      sourceType: 'module',
      globals: {
        browser: true,
        console: true,
        document: true,
        localStorage: true,
        navigator: true,
        Promise: true,
        URL: true,
        window: true,
        setTimeout: true,
        KeyboardEvent: true,
        Event: true,
        HTMLImageElement: true,
        HTMLElement: true,
        Element: true
      }
    },
    rules: {
      '@typescript-eslint/no-explicit-any': 'warn',
      '@typescript-eslint/no-unused-vars': ['warn', { argsIgnorePattern: '^_' }],
      '@typescript-eslint/explicit-function-return-type': 'off',
      '@typescript-eslint/explicit-module-boundary-types': 'off',
      'no-console': ['warn', { allow: ['warn', 'error', 'debug'] }],
      'no-debugger': 'warn',
      'no-unused-vars': 'off'
    }
  }
)
