/**
 * Build the package.json for the actual publishing
 */
// eslint-disable-next-line
const fs = require("fs");
// eslint-disable-next-line
const path = require("path");

// eslint-disable-next-line
const rootPackage = require("../package.json");

// Remove test related work
delete rootPackage["jest-junit"];
delete rootPackage["jest"];

// Don't keep scripts
delete rootPackage["scripts"];

// Don't keep dev dependencies
delete rootPackage["devDependencies"];

// Setup the main and types correctly
rootPackage["main"] = "./cjs/index.js";
rootPackage["module"] = "./esm/index.js";
rootPackage["types"] = "index.d.ts";
rootPackage["exports"] = {
  ".": {
    "types": "./index.d.ts",
    "require": "./cjs/index.js",
    "import": "./esm/index.js",
    "default": "./esm/index.js"
  },
},

// Write it out
fs.writeFileSync(`${path.join(__dirname, "../dist", "package.json")}`, JSON.stringify(rootPackage, null, 2), (err) => {
  if (err) throw new Error(err);
});

// Write out a simple type override for the esm side of things
fs.writeFileSync(`${path.join(__dirname, "../dist/esm", "package.json")}`, JSON.stringify({ "type": "module" }, null, 2), (err) => {
  if (err) throw new Error(err);
});

// Write out a simple type override for the cjs side of things
fs.writeFileSync(`${path.join(__dirname, "../dist/cjs", "package.json")}`, JSON.stringify({ "type": "commonjs" }, null, 2), (err) => {
  if (err) throw new Error(err);
});
