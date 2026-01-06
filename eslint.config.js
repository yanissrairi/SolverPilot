import js from '@eslint/js';
import svelte from 'eslint-plugin-svelte';
import globals from 'globals';
import ts from 'typescript-eslint';
import svelteConfig from './svelte.config.js';

export default ts.config(
  // Base configs
  js.configs.recommended,

  // TypeScript STRICT type-checked (le plus strict)
  ...ts.configs.strictTypeChecked,
  ...ts.configs.stylisticTypeChecked,

  // Svelte recommended
  ...svelte.configs.recommended,

  // Global settings
  {
    languageOptions: {
      globals: {
        ...globals.browser,
        ...globals.node,
      },
      parserOptions: {
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
      },
    },
  },

  // Svelte files config
  {
    files: ['**/*.svelte', '**/*.svelte.ts', '**/*.svelte.js'],
    languageOptions: {
      parserOptions: {
        projectService: true,
        extraFileExtensions: ['.svelte'],
        parser: ts.parser,
        svelteConfig,
      },
    },
  },

  // Custom strict rules
  {
    rules: {
      // TypeScript strict
      '@typescript-eslint/no-unused-vars': ['error', { argsIgnorePattern: '^_' }],
      '@typescript-eslint/consistent-type-imports': 'error',
      '@typescript-eslint/no-explicit-any': 'error',
      '@typescript-eslint/strict-boolean-expressions': 'warn',
      '@typescript-eslint/no-floating-promises': 'error',
      '@typescript-eslint/await-thenable': 'error',

      // Core ESLint strict
      eqeqeq: ['error', 'always'],
      'no-console': 'warn',
      'prefer-const': 'error',
      'no-var': 'error',

      // Svelte
      'svelte/no-unused-svelte-ignore': 'error',
      'svelte/valid-compile': 'off', // Let Svelte compiler handle
    },
  },

  // Ignores
  {
    ignores: [
      'build/**',
      'dist/**',
      '.svelte-kit/**',
      'node_modules/**',
      'target/**',
      'src-tauri/**',
      '*.config.js',
      '*.config.ts',
    ],
  },
);
