// @ts-check

import eslint from '@eslint/js';
import tseslint from 'typescript-eslint';
import mocha from 'eslint-plugin-mocha';
import globals from 'globals';

// Needed for eslint 9
const mochaConfig = [
  {
    name: 'mocha/recommended',
    languageOptions: {
      globals: globals.mocha,
    },
    plugins: {
      mocha,
    },
    rules: mocha.configs.flat.recommended.rules,
  },
];

export default tseslint.config(
  eslint.configs.recommended,
  ...tseslint.configs.strict,
  ...tseslint.configs.stylistic,
  ...mochaConfig,
  {
    ignores: ['dist/', 'scripts/'],
  },
  {
    linterOptions: {
      // Needed as the generated code uses this
      reportUnusedDisableDirectives: false,
    },
  },
  {
    languageOptions: {
      parserOptions: {
        projectService: {
          allowDefaultProject: ['eslint.config.mjs', 'test/*.ts'],
          defaultProject: './tsconfig.eslint.json',
        },
      },
    },
  },
  {
    rules: {
      '@typescript-eslint/no-empty-interface': 'off',
      '@typescript-eslint/no-unused-vars': 'off',
      '@typescript-eslint/no-explicit-any': 'off',
      '@typescript-eslint/no-unused-expressions': 'off',
      '@typescript-eslint/no-empty-function': 'off',
      semi: ['error', 'always'],
      'mocha/no-setup-in-describe': 'off',
      'no-use-before-define': 'off',
      'no-unused-vars': 'off',
      'no-var': 'error',
      'id-length': [
        'error',
        {
          exceptionPatterns: ['[i-k]', 'e', 'c', 'x', 'y', 'r', 's', 'v'],
          properties: 'never',
        },
      ],
      'allow-namespace': 'off',
    },
  }
);
