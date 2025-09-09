#!/usr/bin/env node
/**
 * Scan a repo for package-lock.json files and report if/where certain packages appear.
 *
 * Supports npm lockfileVersion 1, 2, and 3.
 * - Traverses nested dependencies (including dev and optional deps).
 * - By default, matches by package name only.
 * - Use --exact to require the version to match the target version list below.
 *
 * Usage:
 *   node check-locks.js [pathToRepo=. ] [--exact] [--json]
 *
 * Exit codes:
 *   0 = ran successfully; packages may or may not have been found (see output)
 *   1 = unexpected error
 */

const fs = require('fs');
const path = require('path');

// ---- Targets: name -> (optional) target version
const TARGETS = new Map([
  ['backslash', '0.2.1'],
  ['chalk-template', '1.1.1'],
  ['supports-hyperlinks', '4.1.1'],
  ['has-ansi', '6.0.1'],
  ['simple-swizzle', '0.2.3'],
  ['color-string', '2.1.1'],
  ['error-ex', '1.3.3'],
  ['color-name', '2.0.1'],
  ['is-arrayish', '0.3.3'],
  ['slice-ansi', '7.1.1'],
  ['color-convert', '3.1.1'],
  ['wrap-ansi', '9.0.1'],
  ['ansi-regex', '6.2.1'],
  ['supports-color', '10.2.1'],
  ['strip-ansi', '7.1.1'],
  ['chalk', '5.6.1'],
  ['debug', '4.4.2'],
  ['ansi-styles', '6.2.2'],
]);

// -------------- CLI args --------------
const root = path.resolve(process.argv[2] && !process.argv[2].startsWith('--') ? process.argv[2] : '.');
const EXACT = process.argv.includes('--exact');
const AS_JSON = process.argv.includes('--json');

// -------------- Helpers --------------
function* walk(dir) {
  const stack = [dir];
  const SKIP = new Set(['node_modules', '.git', '.hg', '.svn', '.next', 'dist', 'build', 'out', 'coverage']);
  while (stack.length) {
    const cur = stack.pop();
    let ents = [];
    try { ents = fs.readdirSync(cur, { withFileTypes: true }); } catch { continue; }
    for (const e of ents) {
      if (e.isDirectory()) {
        if (!SKIP.has(e.name)) stack.push(path.join(cur, e.name));
      } else if (e.isFile() && e.name === 'package-lock.json') {
        yield path.join(cur, e.name);
      }
    }
  }
}

function safeReadJSON(file) {
  try {
    const text = fs.readFileSync(file, 'utf8');
    return JSON.parse(text);
  } catch (err) {
    return { __error: String(err) };
  }
}

// Collect match details in a consistent shape
function recordHit(results, lockPath, hit) {
  if (!results[lockPath]) results[lockPath] = [];
  results[lockPath].push(hit);
}

// lockfileVersion 2/3: prefer `packages` object, fall back to `dependencies` map if needed
function scanV2V3(lock, lockPath, results, exact) {
  const targets = new Set(TARGETS.keys());

  // 1) packages: { "": {name,version}, "node_modules/x": {name,version,dev}, ... }
  if (lock.packages && typeof lock.packages === 'object') {
    for (const [pkgPath, meta] of Object.entries(lock.packages)) {
      if (!meta || typeof meta !== 'object') continue;
      
      // Extract package name from path or meta.name
      let name;
      if (pkgPath === '') {
        // Root package
        name = meta.name;
      } else if (pkgPath.startsWith('node_modules/')) {
        // Extract name from node_modules path
        const pathParts = pkgPath.split('/');
        if (pathParts[1].startsWith('@')) {
          // Scoped package like @scope/package
          name = pathParts[1] + '/' + pathParts[2];
        } else {
          // Regular package
          name = pathParts[1];
        }
      } else {
        // Other path format, use meta.name if available
        name = meta.name;
      }
      
      const version = meta.version;
      if (!name || !targets.has(name)) continue;

      const want = TARGETS.get(name);
      const ok = exact ? (want && version === want) : true;
      if (ok) {
        recordHit(results, lockPath, {
          name,
          version,
          dev: !!meta.dev,
          optional: !!meta.optional,
          where: pkgPath || '(root)',
          via: 'packages',
        });
      }
    }
  }

  // 2) dependencies: { pkgName: { version, requires, dependencies } }
  if (lock.dependencies && typeof lock.dependencies === 'object') {
    const visit = (deps, trail = []) => {
      for (const [name, meta] of Object.entries(deps)) {
        if (!meta || typeof meta !== 'object') continue;
        const version = meta.version;
        if (TARGETS.has(name)) {
          const want = TARGETS.get(name);
          const ok = exact ? (want && version === want) : true;
          if (ok) {
            recordHit(results, lockPath, {
              name,
              version,
              dev: !!meta.dev,
              optional: !!meta.optional,
              where: trail.concat(name).join(' > '),
              via: 'dependencies',
            });
          }
        }
        if (meta.dependencies) visit(meta.dependencies, trail.concat(name));
      }
    };
    visit(lock.dependencies, []);
  }
}

// lockfileVersion 1
function scanV1(lock, lockPath, results, exact) {
  if (!lock.dependencies || typeof lock.dependencies !== 'object') return;
  const visit = (deps, trail = []) => {
    for (const [name, meta] of Object.entries(deps)) {
      if (!meta || typeof meta !== 'object') continue;
      const version = meta.version;
      if (TARGETS.has(name)) {
        const want = TARGETS.get(name);
        const ok = exact ? (want && version === want) : true;
        if (ok) {
          recordHit(results, lockPath, {
            name,
            version,
            dev: !!meta.dev,
            optional: !!meta.optional,
            where: trail.concat(name).join(' > '),
            via: 'dependencies',
          });
        }
      }
      if (meta.dependencies) visit(meta.dependencies, trail.concat(name));
    }
  };
  visit(lock.dependencies, []);
}

function scanLock(lockPath, exact) {
  const lock = safeReadJSON(lockPath);
  const results = {};
  if (lock.__error) {
    recordHit(results, lockPath, { error: `Failed to parse JSON: ${lock.__error}` });
    return results;
  }

  const v = Number(lock.lockfileVersion || 1);
  if (v >= 2) scanV2V3(lock, lockPath, results, exact);
  else scanV1(lock, lockPath, results, exact);

  // If no hits, still record an empty array for consistency
  if (!results[lockPath]) results[lockPath] = [];
  return results;
}

// -------------- Run --------------
(async function main() {
  try {
    const allResults = {};
    const files = Array.from(walk(root));

    for (const f of files) {
      const res = scanLock(f, EXACT);
      Object.assign(allResults, res);
    }

    if (AS_JSON) {
      console.log(JSON.stringify({ root, exact: EXACT, files: Object.keys(allResults).length, results: allResults }, null, 2));
      return;
    }

    // Pretty print
    if (files.length === 0) {
      console.log(`No package-lock.json files found under: ${root}`);
      return;
    }

    console.log(`Scanned ${files.length} package-lock.json file(s) under: ${root}`);
    console.log(EXACT ? '(Exact version matching enabled)\n' : '(Name-only matching)\n');

    let totalHits = 0;
    for (const f of files) {
      const hits = allResults[f] || [];
      if (hits.length === 0) continue;
      totalHits += hits.length;
      console.log(`\n${f}`);
      console.log('-'.repeat(f.length));
      for (const h of hits) {
        if (h.error) {
          console.log(`  [ERROR] ${h.error}`);
          continue;
        }
        const flags = [
          h.dev ? 'dev' : null,
          h.optional ? 'optional' : null,
          h.via ? `via:${h.via}` : null,
        ].filter(Boolean).join(', ');
        const want = TARGETS.get(h.name);
        const exactNote = EXACT ? (want ? ` (target ${want})` : ' (no target version listed)') : '';
        console.log(`  - ${h.name}@${h.version}${exactNote}`);
        console.log(`      where: ${h.where}`);
        if (flags) console.log(`      flags: ${flags}`);
      }
    }

    if (totalHits === 0) {
      console.log('No target packages were found.');
    } else {
      console.log(`\nFound ${totalHits} match(es) across ${files.length} lock file(s).`);
    }
  } catch (err) {
    console.error('Fatal error:', err && err.stack || err);
    process.exit(1);
  }
})();