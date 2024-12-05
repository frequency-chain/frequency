// A quick file used by CI to list all the test files as an array for the matrix
import process from 'node:process';
import { globSync } from 'glob';

const files = globSync('**/*.test.ts', { ignore: 'node_modules/**' });
process.stdout.write(JSON.stringify(files));
