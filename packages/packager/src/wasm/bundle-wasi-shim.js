const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');
const { createRequire } = require('module');

const modules = ['io', 'cli', 'random', 'clocks', 'filesystem', 'sockets'];
const shimDir = process.argv[2];

fs.mkdirSync(shimDir, { recursive: true });

function findBrowserShimFile(mod) {
    const searchRoots = [
        process.cwd(),
        path.resolve(process.cwd(), 'node_modules'),
    ];

    try {
        const jcoDir = path.dirname(require.resolve('@bytecodealliance/jco/package.json'));
        searchRoots.push(path.join(jcoDir, 'node_modules'));
    } catch {}

    try {
        const gRoot = execSync('npm root -g', { encoding: 'utf-8' }).trim();
        searchRoots.push(gRoot);
        searchRoots.push(path.join(gRoot, '@bytecodealliance', 'jco', 'node_modules'));
    } catch {}

    for (const root of searchRoots) {
        const direct = path.join(root, '@bytecodealliance', 'preview2-shim', 'lib', 'browser', mod + '.js');
        if (fs.existsSync(direct)) return direct;
    }

    try {
        const req = createRequire(path.join(process.cwd(), 'package.json'));
        const resolved = req.resolve('@bytecodealliance/preview2-shim/' + mod, {
            conditions: ['default', 'browser', 'import'],
        });
        if (fs.existsSync(resolved)) return resolved;
    } catch {}

    return null;
}

for (const mod of modules) {
    const outPath = path.join(shimDir, mod + '.js');
    const resolved = findBrowserShimFile(mod);
    if (resolved) {
        fs.writeFileSync(outPath, fs.readFileSync(resolved, 'utf-8'));
        continue;
    }

    const tempDir = path.join(shimDir, '_tmp_install');
    fs.rmSync(tempDir, { recursive: true, force: true });
    fs.mkdirSync(tempDir, { recursive: true });
    try {
        execSync('npm install --no-save --no-package-lock --prefix "' + tempDir + '" @bytecodealliance/preview2-shim', {
            stdio: 'pipe',
            timeout: 60000,
        });
        const browserFile = path.join(tempDir, 'node_modules', '@bytecodealliance', 'preview2-shim', 'lib', 'browser', mod + '.js');
        if (fs.existsSync(browserFile)) {
            fs.writeFileSync(outPath, fs.readFileSync(browserFile, 'utf-8'));
        }
    } catch (e) {
        console.error('npm install fallback failed:', e.message);
    }
    fs.rmSync(tempDir, { recursive: true, force: true });

    if (!fs.existsSync(outPath)) {
        console.error('Failed to bundle WASI shim module: ' + mod);
        process.exit(1);
    }
}

fs.writeFileSync(path.join(shimDir, '.tairitsu-bundled'), '');
