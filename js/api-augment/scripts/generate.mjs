import { types } from '../dist/esm/index.js';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import { dirname } from 'path';

// Get the filename from the URL
const __filename = fileURLToPath(import.meta.url);
// Get the directory name from the filename
const __dirname = dirname(__filename);
const outdir = path.join(__dirname, '../dist/json/');

fs.mkdirSync(outdir, { recursive: true });

fs.writeFileSync(path.join(outdir, 'types.json'), JSON.stringify(types, null, 4));
