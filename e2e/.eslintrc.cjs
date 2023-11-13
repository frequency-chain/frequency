/* eslint-env node */
module.exports = {
  ignorePatterns: ['dist'],
  extends: ['eslint:recommended', 'plugin:@typescript-eslint/recommended', 'plugin:mocha/recommended'],
  parser: '@typescript-eslint/parser',
  plugins: ['@typescript-eslint', 'mocha'],
  root: true,
  env: {
    mocha: true,
    node: true,
  },
  rules: {
    '@typescript-eslint/no-explicit-any': 'off', // Don't worry about an any in a test
    '@typescript-eslint/no-unused-vars': 'warn',
    '@typescript-eslint/no-unused-vars': [
      'error',
      {
        argsIgnorePattern: '^_',
        varsIgnorePattern: '^_',
        caughtErrorsIgnorePattern: '^_',
      },
    ],
  },
};
