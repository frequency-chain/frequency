/**
 * Build the needed directories for the code generation
 */
// eslint-disable-next-line
const fs = require('fs');
// eslint-disable-next-line
const path = require('path');

const interfacePath = path.join(__dirname, '../interfaces');
if (!fs.existsSync(interfacePath)) fs.mkdirSync(interfacePath);

const definitionsPath = path.join(__dirname, '../definitions');
const files = fs.readdirSync(definitionsPath);

// Create folders for each of the definitions
files.forEach((file) => {
  const folder = file.replace('.ts', '');
  const newDir = path.join(interfacePath, folder);
  console.log(`Making interfaces/${folder} if it doesn't exist...`);
  if (!fs.existsSync(newDir)) fs.mkdirSync(newDir);
});

// Pipe over the exports into the interfaces
const definitionsExportFile = path.join(interfacePath, 'definitions.ts');
fs.writeFileSync(definitionsExportFile, `export * from "../definitions/index.js";\n`);
