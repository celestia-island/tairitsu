const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const shimDir = process.argv[2];
fs.mkdirSync(shimDir, { recursive: true });

function findBrowserDir() {
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
        const dir = path.join(root, '@bytecodealliance', 'preview2-shim', 'lib', 'browser');
        if (fs.existsSync(dir)) return dir;
    }

    return null;
}

let browserDir = findBrowserDir();

if (!browserDir) {
    const tempDir = path.join(shimDir, '_tmp_install');
    fs.rmSync(tempDir, { recursive: true, force: true });
    fs.mkdirSync(tempDir, { recursive: true });
    try {
        execSync('npm install --no-save --no-package-lock --prefix "' + tempDir + '" @bytecodealliance/preview2-shim', {
            stdio: 'pipe',
            timeout: 60000,
        });
        browserDir = path.join(tempDir, 'node_modules', '@bytecodealliance', 'preview2-shim', 'lib', 'browser');
    } catch (e) {
        console.error('npm install fallback failed:', e.message);
    }
    if (!browserDir || !fs.existsSync(browserDir)) {
        fs.rmSync(tempDir, { recursive: true, force: true });
        console.error('Failed to locate WASI preview2-shim browser files');
        process.exit(1);
    }
}

for (const file of fs.readdirSync(browserDir)) {
    if (!file.endsWith('.js')) continue;
    fs.writeFileSync(path.join(shimDir, file), fs.readFileSync(path.join(browserDir, file), 'utf-8'));
}

if (browserDir.includes('_tmp_install')) {
    const tempDir = path.join(shimDir, '_tmp_install');
    fs.rmSync(tempDir, { recursive: true, force: true });
}

fs.writeFileSync(path.join(shimDir, '.tairitsu-bundled'), '');
