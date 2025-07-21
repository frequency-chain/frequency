// @ts-check

import eslint from '@eslint/js';
import globals from 'globals';
import tseslint from 'typescript-eslint';
import mochaPlugin from 'eslint-plugin-mocha';

export default tseslint.config(
  eslint.configs.recommended,
  mochaPlugin.configs.recommended,
  ...tseslint.configs.strict,
  ...tseslint.configs.stylistic,
  {
    ignores: ['dist/'],
  },
  {
    languageOptions: {
      globals: { ...globals.node },
    },
    rules: {
      '@typescript-eslint/no-empty-interface': 'off',
      '@typescript-eslint/no-unused-vars': 'off',
      '@typescript-eslint/no-explicit-any': 'off',
      '@typescript-eslint/no-non-null-assertion': 'off',
      '@typescript-eslint/no-inferrable-types': 'off',
      '@typescript-eslint/no-extraneous-class': 'off',
      semi: ['error', 'always'],
      'mocha/no-setup-in-describe': 'off',
      'no-use-before-define': 'off',
      'no-unused-vars': 'off',
      'no-var': 'error',
      'id-length': [
        'error',
        {
          exceptionPatterns: ['[i-k]', 'e', 'c', 'x', 'y', 'r', 's', 'v', 'f', '_'],
          properties: 'never',
        },
      ],
      'allow-namespace': 'off',
      'no-restricted-syntax': [
        'error',
        {
          message:
            'Direct usage of keyPair.address is not allowed in this file. please use getUnifiedAddress function.',
          selector: 'MemberExpression[property.name="address"]',
        },
        {
          message:
            'Direct usage of keyPair.publicKey is not allowed in this file. please use getUnifiedPublicKey function',
          selector: 'MemberExpression[property.name="publicKey"]',
        },
      ],
    },
  }
);
