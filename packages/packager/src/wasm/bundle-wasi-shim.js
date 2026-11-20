const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

// WASI preview2-shim bundler (W1 reproducible-build rules):
// - The shim version comes from the workspace package.json +
//   package-lock.json (@bytecodealliance/preview2-shim, pinned ^0.31).
// - Only locally installed copies are bundled. There is deliberately NO
//   versionless `npm install` fallback and no CDN fallback: a missing
//   shim is a hard error, not a silent degradation.
//
// argv[2] = destination dir, argv[3] = the manifest dir of the project
// being built (passed by the Rust caller). Discovery walks up from BOTH
// the cwd and the manifest dir, because the packager process may be
// invoked from anywhere — the project root must anchor the search too.
const shimDir = process.argv[2];
const projectRoot = process.argv[3] || process.cwd();
fs.mkdirSync(shimDir, { recursive: true });

function localRoots() {
    const roots = [];
    const pushWalk = (start) => {
        let dir = path.resolve(start);
        for (;;) {
            const nm = path.join(dir, 'node_modules');
            if (!roots.includes(nm)) roots.push(nm);
            const parent = path.dirname(dir);
            if (parent === dir) break;
            dir = parent;
        }
    };
    pushWalk(process.cwd());
    pushWalk(projectRoot);

    // jco may carry its own nested node_modules; probe by path (NOT
    // require.resolve — this script runs from a temp copy inside the
    // output dir, so module resolution from here must not influence
    // discovery).
    for (const root of [...roots]) {
        const jcoNm = path.join(root, '@bytecodealliance', 'jco', 'node_modules');
        if (fs.existsSync(jcoNm) && !roots.includes(jcoNm)) roots.push(jcoNm);
    }
    return roots;
}

function findBrowserDir() {
    const probe = (root) => {
        // preview2-shim >=0.31 ships the browser build at dist/browser;
        // older lines (0.1x) kept it at lib/browser. The pin (^0.31)
        // resolves to the dist/ layout — lib/ stays as a compat probe.
        for (const sub of ['dist/browser', 'lib/browser']) {
            const dir = path.join(root, '@bytecodealliance', 'preview2-shim', sub);
            if (fs.existsSync(dir)) return dir;
        }
        return null;
    };

    for (const root of localRoots()) {
        const dir = probe(root);
        if (dir) return dir;
    }

    // Global npm root is the LAST resort: `npm root -g` costs ~100ms on
    // every build, and watch-mode rebuilds must not pay it when the
    // pinned local install already satisfies the lookup.
    try {
        const gRoot = execSync('npm root -g', { encoding: 'utf-8' }).trim();
        if (gRoot) {
            const dir = probe(gRoot);
            if (dir) return dir;
        }
    } catch {}

    return null;
}

const browserDir = findBrowserDir();

if (!browserDir) {
    console.error('WASI preview2-shim browser files not found in any local node_modules.');
    console.error('The shim is a pinned devDependency (@bytecodealliance/preview2-shim,');
    console.error('^0.31 in the tairitsu workspace package.json + package-lock.json).');
    console.error('Install it where the build can see it: `npm install` at the tairitsu');
    console.error('workspace root or your project root. Floating versionless installs are');
    console.error('not used (reproducible-build rule).');
    process.exit(1);
}

// Marker = <shim package version>:<hash of this bundler script>. A stale
// copy (shim bump or bundler change) must NOT short-circuit (R3 finding:
// same class as the wrapper cache) — only an exact match may skip.
const crypto = require('crypto');
function fingerprint() {
    let shimVersion = 'unknown';
    try {
        shimVersion = require(path.join(browserDir, '..', '..', 'package.json')).version || 'unknown';
    } catch {}
    const self = fs.readFileSync(__filename, 'utf-8');
    const scriptHash = crypto.createHash('sha256').update(self).digest('hex').slice(0, 16);
    return shimVersion + ':' + scriptHash;
}

const markerPath = path.join(shimDir, '.tairitsu-bundled');
const stamp = fingerprint();
if (fs.existsSync(markerPath) && fs.readFileSync(markerPath, 'utf-8').trim() === stamp) {
    console.log(`WASI shim already bundled (${stamp}) — copy is current, skipping`);
    process.exit(0);
}

for (const file of fs.readdirSync(browserDir)) {
    if (!file.endsWith('.js')) continue;
    fs.writeFileSync(path.join(shimDir, file), fs.readFileSync(path.join(browserDir, file), 'utf-8'));
}

fs.writeFileSync(markerPath, stamp);
