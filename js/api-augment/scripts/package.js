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

// Setup the main and types correctly
rootPackage["main"] = "index.cjs";
rootPackage["module"] = "index.js";
rootPackage["types"] = "index.d.ts";
rootPackage["exports"] = {
  ".": {
    "types": "./index.d.ts",
    "require": "./index.cjs",
    "default": "./index.js"
  },
},

// Write it out
fs.writeFileSync(`${path.join(__dirname, "../dist", "package.json")}`, JSON.stringify(rootPackage, null, 2), (err) => {
  if (err) throw new Error(err);
});
