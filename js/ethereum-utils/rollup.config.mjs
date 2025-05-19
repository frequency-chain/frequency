import resolve from '@rollup/plugin-node-resolve';
import commonjs from '@rollup/plugin-commonjs';
import typescript from 'rollup-plugin-typescript2';
import { terser } from 'rollup-plugin-terser';
import pkg from './package.json' assert { type: 'json' };

// Dependencies to exclude in ESM
const esmExternals = Object.keys(pkg.dependencies || {}).filter((dep) => dep !== '@polkadot/util' && dep !== 'ethers');

// Dependencies to exclude in UMD
const umdExternals = Object.keys(pkg.dependencies || {}).filter((dep) => dep !== '@polkadot/util');

/**
 * Create external logic depending on format.
 * For ESM: all deps are external (except relative paths).
 * For UMD: only selected deps (e.g. bundle @polkadot/util).
 */
function makeExternal(format) {
  return (id) => {
    if (format === 'ESM') {
      return esmExternals.includes(id);
    }
    return umdExternals.includes(id);
  };
}

export default [
  {
    input: 'src/index.ts',
    output: {
      file: 'dist/browser/frequency-ethereum-utils.esm.min.js',
      format: 'esm',
      sourcemap: false,
    },
    external: makeExternal('ESM'),
    plugins: [
      resolve({ browser: true, preferBuiltins: false }),
      commonjs(),
      typescript({ tsconfig: './tsconfig.json', clean: true }),
      terser(),
    ],
  },
  {
    input: 'src/index.ts',
    output: {
      file: 'dist/browser/frequency-ethereum-utils.umd.min.js',
      format: 'umd',
      name: 'EthereumUtils',
      sourcemap: false,
    },
    external: makeExternal('UMD'),
    plugins: [
      resolve({ browser: true, preferBuiltins: false }),
      commonjs(),
      typescript({ tsconfig: './tsconfig.json', clean: true }),
      terser(),
    ],
  },
];
