const fs = require('fs');
const path = require('path');

function patchNextDirectory(nextDir) {
  try {
    const pkgPath = path.join(nextDir, 'package.json');
    if (!fs.existsSync(pkgPath)) return;

    const pkg = JSON.parse(fs.readFileSync(pkgPath, 'utf8'));
    if (pkg.name !== 'next') return;

    let modified = false;

    if (pkg.exports && !pkg.exports['./config']) {
      pkg.exports['./config'] = './config.js';
      fs.writeFileSync(pkgPath, JSON.stringify(pkg, null, 2), 'utf8');
      modified = true;
    }

    const configJsPath = path.join(nextDir, 'config.js');
    if (!fs.existsSync(configJsPath)) {
      fs.writeFileSync(
        configJsPath,
        'module.exports = function getConfig() { return { publicRuntimeConfig: {}, serverRuntimeConfig: {} }; };\nmodule.exports.default = module.exports;\n',
        'utf8'
      );
      modified = true;
    }

    if (modified) {
      console.log(`[fix-storybook-next] Patched next package at ${nextDir}`);
    }
  } catch (err) {
    console.error(`[fix-storybook-next] Failed to patch ${nextDir}:`, err.message);
  }
}

function searchNodeModules(dir, depth = 0) {
  if (depth > 6 || !fs.existsSync(dir)) return;

  try {
    const entries = fs.readdirSync(dir, { withFileTypes: true });
    for (const entry of entries) {
      if (!entry.isDirectory()) continue;
      const fullPath = path.join(dir, entry.name);

      if (entry.name === 'next') {
        patchNextDirectory(fullPath);
      } else if (
        entry.name === '.pnpm' ||
        entry.name.startsWith('@') ||
        entry.name === 'node_modules'
      ) {
        searchNodeModules(fullPath, depth + 1);
      }
    }
  } catch {
    // Ignore read errors
  }
}

const rootNodeModules = path.join(__dirname, '..', 'node_modules');
searchNodeModules(rootNodeModules);

const webNodeModules = path.join(__dirname, '..', 'apps', 'web', 'node_modules');
searchNodeModules(webNodeModules);
