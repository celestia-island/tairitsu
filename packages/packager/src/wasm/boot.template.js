        // W3: ONE place marks a failure. `data-tairitsu-error` is the machine
        // signal every consumer keys on, so it is set here rather than in each
        // catch — the outermost catch used to show the card without it, which
        // made a stale dist (missing /component-boot.js) invisible to anything
        // watching the attribute.
        const markBootFailure = (kind) => {
            document.documentElement.dataset.tairitsuError = kind;
        };

        const bootError = (msg, err, kind) => {
            if (kind) markBootFailure(kind);
            console.error('[tairitsu] Boot failed:', msg, err || '');
            const appRoot = document.getElementById('app');
            if (appRoot) appRoot.innerHTML =
                '<div style="padding:2em;font-family:monospace;color:#e06c75">' +
                '<b>Tairitsu boot error</b><br><pre style="margin-top:.5em;white-space:pre-wrap">' +
                (err ? String(err.message || err) : String(msg)) +
                '</pre></div>';
            document.body.removeAttribute('data-booting');
        };

        try {
        // A boot that hangs must not stay invisible forever. This used to be an
        // inline <script> in the page, which the CSP this build targets blocks.
        setTimeout(() => {
            document.body.removeAttribute('data-booting');
        }, 10000);

        const { instantiateWithWrapper } = await import('/component-wrapper-loader.js?v={{ v }}');
        const { bootComponent } = await import('/component-boot.js?v={{ v }}');

        const appRoot = document.getElementById('app');
        // W3: `data-tairitsu-ready` is set ONLY after the guest's boot entry
        // returned. Failures go to `data-tairitsu-error` + the visible card
        // and never set the ready attribute.
        let settleBoot;
        const bootSettled = new Promise((resolve, reject) => {
            settleBoot = { resolve, reject };
        });
        // A rejected lifecycle promise that nobody awaits would surface as an
        // unhandled rejection; the boot path awaits it below.
        bootSettled.catch(() => {});

        const bootSurface = {
            ready: (entryName) => {
                document.documentElement.removeAttribute('data-tairitsu-error');
                console.info('[tairitsu] component ready (entry: ' + entryName + ')');
                settleBoot.resolve(entryName);
            },
            fail: (kind, error) => {
                // W4: a failed boot reclaims its instance (handles, listeners,
                // observers, timers) instead of leaving them on the page.
                if (instanceApi && bootInstance) instanceApi.dispose(bootInstance);
                bootError('Component ' + kind.replace('-', ' '), error, kind);
                settleBoot.reject(error);
            },
        };

        // W7: the worker lane is OPT-IN (`<html data-tairitsu-worker>`): the
        // component is instantiated and run in a dedicated Worker, so a
        // component that never yields can be terminated without killing the
        // page. The default (no attribute) is unchanged — main thread.
        const wantsWorker =
            document.documentElement.dataset.tairitsuWorker !== undefined &&
            typeof Worker === 'function';
        let workerHost = null;
        let workerCancelled = false;

        const startWorkerBoot = async (capabilities) => {
            if (workerCancelled) return;
            // Relative to THIS module, like the loader's own resolution: a page
            // served from a sub-path gets its own host (and its own wrapper).
            const worker = new Worker(
                new URL('glue-worker.js', import.meta.url),
                { type: 'module' },
            );
            workerHost = worker;
            let settledFlag = false;
            const settled = new Promise((resolve) => {
                worker.onmessage = (event) => {
                    const message = event.data || {};
                    if (settledFlag) return;
                    if (message.type === 'ready') {
                        settledFlag = true;
                        bootSurface.ready(message.entry);
                        resolve({ ok: true });
                    } else if (message.type === 'error') {
                        settledFlag = true;
                        bootSurface.fail(message.kind || 'boot-failed', new Error(message.message));
                        resolve({ ok: false });
                    }
                };
                worker.onerror = (event) => {
                    // Only while the boot is still undecided: a late uncaught
                    // error must not tear down a page that already reported
                    // ready (and leave ready + error set at once).
                    if (settledFlag) return;
                    settledFlag = true;
                    bootSurface.fail('instantiate-failed', new Error(event.message || 'worker error'));
                    resolve({ ok: false });
                };
            });
            worker.postMessage({ type: 'boot', capabilities });
            await settled;
        };

        // The hard kill. `stop` asks a yielding component to wind down; only
        // `terminate()` can interrupt a synchronous loop, so that is what this
        // does — plus the main-thread half of the reclaim (the W4 instance).
        const terminateWorkerHost = () => {
            // Before the host exists there is nothing to kill, but the boot must
            // still be cancelled — otherwise a "killed" page boots afterwards.
            workerCancelled = true;
            if (!workerHost) return false;
            workerHost.terminate();
            workerHost = null;
            if (instanceApi && bootInstance) instanceApi.dispose(bootInstance);
            return true;
        };
        globalThis.__TAIRITSU_WORKER__ = {
            terminate: terminateWorkerHost,
            active: () => workerHost !== null,
        };

        // W6: the component's capability declaration decides what is injected.
        // No declaration → the conservative default set the build emitted; a
        // declaration → exactly that. Nothing else is provided.
        const loadCapabilities = async () => {
            try {
                const response = await fetch('/tairitsu-capabilities.json', { cache: 'no-store' });
                if (!response.ok) {
                    throw new Error(`HTTP ${response.status}`);
                }
                return await response.json();
            } catch (error) {
                throw new Error(
                    'the capability declaration is missing or unreadable ' +
                    '(/tairitsu-capabilities.json): ' + (error && error.message ? error.message : error),
                );
            }
        };

        const normalizeInterface = (name) =>
            String(name || '')
                .replace(/^@tairitsu-glue\//, '')
                .replace(/^tairitsu-browser:full\//, '')
                .replace(/@\d+\.\d+\.\d+$/, '');

        // Build imports object from the DECLARED capabilities only.
        const buildImports = async (capabilities) => {
            const imports = {};
            const declaredInterfaces = new Set(
                (capabilities.interfaces || []).map(normalizeInterface),
            );
            const declaredWasi = new Set(capabilities.wasiModules || []);
            const skipped = [];

            // Browser glue interfaces — injected only when declared.
            if (globalThis.__TAIRITSU_GLUE__ && globalThis.__TAIRITSU_GLUE__.INTERFACES) {
                for (const [shortName, exports] of Object.entries(globalThis.__TAIRITSU_GLUE__.INTERFACES)) {
                    const ifaceName = shortName.replace('@tairitsu-glue/', '');
                    const fullName = `tairitsu-browser:full/${ifaceName}@0.2.0`;
                    if (!declaredInterfaces.has(ifaceName)) {
                        skipped.push(fullName);
                        continue;
                    }
                    imports[fullName] = exports;
                    declaredInterfaces.delete(ifaceName);
                }
            }
            // Declared but the host does not provide it: fail BEFORE instantiate
            // (a silently missing import would surface as an opaque LinkError).
            if (declaredInterfaces.size > 0) {
                throw new Error(
                    'the component declares capabilities the host does not provide: ' +
                    [...declaredInterfaces].sort().join(', '),
                );
            }
            globalThis.__TAIRITSU_SKIPPED_IMPORTS__ = skipped;

            // WASI preview2 shim — always the locally bundled, pinned copy
            // (the build fails loudly when the shim is missing, so there is
            // no CDN fallback anymore: unversioned esm.sh imports are
            // unreproducible and would be blocked by the CSP posture anyway).
            if (declaredWasi.size > 0 && !globalThis.__wasiShimModules) {
                try {
                    const shimModules = ['io', 'cli', 'random', 'clocks', 'filesystem', 'sockets'];
                    const localImports = await Promise.all(shimModules.map(m => import('/wasi-shim/' + m + '.js')));
                    globalThis.__wasiShimModules = {
                        wasiIo: localImports[0], wasiCli: localImports[1], wasiRandom: localImports[2],
                        wasiClocks: localImports[3], wasiFilesystem: localImports[4], wasiSockets: localImports[5],
                    };
                } catch (e) {
                    console.error('[tairitsu] Could not load the locally bundled WASI preview2 shim (/wasi-shim/*). ' +
                        'The build bundles the pinned shim (@bytecodealliance/preview2-shim ^0.31) — a missing bundle ' +
                        'means the packaged output is incomplete; rebuild with `tairitsu build`.', e);
                }
            }
            const KNOWN_WASI = ['cli', 'clocks', 'filesystem', 'io', 'random', 'sockets'];
            const unknownWasi = [...declaredWasi].filter((name) => !KNOWN_WASI.includes(name));
            if (unknownWasi.length > 0) {
                throw new Error(
                    'the component declares WASI modules the host does not provide: ' +
                    unknownWasi.sort().join(', '),
                );
            }
            if (globalThis.__wasiShimModules && declaredWasi.size > 0) {
                const s = globalThis.__wasiShimModules;
                if (declaredWasi.has('cli')) imports['wasi:cli/environment@0.2.0'] = s.wasiCli.environment;
                if (declaredWasi.has('cli')) imports['wasi:cli/exit@0.2.0'] = s.wasiCli.exit;
                if (declaredWasi.has('cli')) imports['wasi:cli/stdin@0.2.0'] = s.wasiCli.stdin;
                if (declaredWasi.has('cli')) imports['wasi:cli/stdout@0.2.0'] = s.wasiCli.stdout;
                if (declaredWasi.has('cli')) imports['wasi:cli/stderr@0.2.0'] = s.wasiCli.stderr;
                if (declaredWasi.has('clocks')) imports['wasi:clocks/monotonic-clock@0.2.0'] = s.wasiClocks.monotonicClock;
                if (declaredWasi.has('clocks')) imports['wasi:clocks/wall-clock@0.2.0'] = s.wasiClocks.wallClock;
                if (declaredWasi.has('filesystem')) imports['wasi:filesystem/preopens@0.2.0'] = s.wasiFilesystem.preopens;
                if (declaredWasi.has('io')) imports['wasi:io/error@0.2.0'] = s.wasiIo.error;
                if (declaredWasi.has('io')) imports['wasi:io/poll@0.2.0'] = s.wasiIo.poll;
                if (declaredWasi.has('io')) imports['wasi:io/streams@0.2.0'] = s.wasiIo.streams;
                if (declaredWasi.has('random')) imports['wasi:random/random@0.2.0'] = s.wasiRandom.random;
                if (declaredWasi.has('sockets')) imports['wasi:sockets/tcp@0.2.0'] = s.wasiSockets.tcp;
                if (declaredWasi.has('sockets')) imports['wasi:sockets/udp@0.2.0'] = s.wasiSockets.udp;
                if (declaredWasi.has('sockets')) imports['wasi:sockets/network@0.2.0'] = s.wasiSockets.network;
                if (declaredWasi.has('sockets')) imports['wasi:sockets/instance-network@0.2.0'] = s.wasiSockets.instanceNetwork;
            }

            return imports;
        };


        // Strategy: always try the jco component wrapper first (avoids a
        // redundant 50MB fetch of the raw WASM just to inspect magic bytes).
        // The wrapper internally fetches only the core module it needs.
        // Direct WASM loading stays as the no-wrapper fallback. W3: whichever
        // path produced the instance, the SAME boot lifecycle decides the
        // entry, the ready attribute and the error surface — and a failure on
        // either path is terminal (never a blank page).
        // W4: the guest's handle tables belong to THIS boot's instance. The
        // glue bundle defines the API; without it (a page built before W4) the
        // boot still works, it just has no per-instance container to own.
        // ONE instance per booted component: `current` is a page-level pointer,
        // so a second component on the same page displaces the first unless the
        // host interposes per call (`withInstance`). The dual-instance test
        // drives exactly that interposition; a second guest boot would not.
        const instanceApi = globalThis.__TAIRITSU_INSTANCES__;
        const bootInstance = instanceApi ? instanceApi.create() : null;
        // W4 contract: ONE active guest per page. `claim` refuses when another
        // component already owns the glue surface, instead of silently
        // displacing it; the refusal reaches the error surface below.
        let guestRefused = null;
        if (instanceApi && bootInstance) {
            try {
                instanceApi.claim(bootInstance);
            } catch (claimErr) {
                // A refusal is TERMINAL: reporting it and then booting anyway
                // would run the second guest's entry on the first guest's
                // tables. `guestRefused` aborts the boot below.
                guestRefused = claimErr;
                bootSurface.fail('guest-active', claimErr);
            }
        }

        const startBoot = async (result) => {
            if (globalThis.__setWasmExports && result) {
                globalThis.__setWasmExports(result);
            }
            await bootComponent({ result, surface: bootSurface });
        };

        // Nothing below may run after a refusal: the surface already carries
        // the error. (A thrown refusal would land in the outermost catch and be
        // re-reported as a generic 'boot-failed'.)
        if (!guestRefused && wantsWorker) {
            // Detached on purpose: the component may never return (that is what
            // this lane is for), so the page's own module must finish evaluating
            // — otherwise the `load` event never fires and the page looks hung.
            void (async () => {
                try {
                    const capabilities = await loadCapabilities();
                    globalThis.__TAIRITSU_CAPABILITIES__ = capabilities;
                    await startWorkerBoot(capabilities);
                } catch (error) {
                    bootSurface.fail('worker-boot-failed', error);
                }
            })();
        } else if (!guestRefused) try {
            const capabilities = await loadCapabilities();
            // W6: the static glue modules check this before they hand out a
            // function, so a declaration reaches them too (W5 made their imports
            // static, invisible to the import filter below).
            globalThis.__TAIRITSU_CAPABILITIES__ = capabilities;
            const imports = await buildImports(capabilities);
            let wrapperErr = null;
            let wrapperResult = null;
            try {
                wrapperResult = await instantiateWithWrapper(imports);
            } catch (error) {
                wrapperErr = error;
                console.error('[tairitsu] Component wrapper failed, falling back to direct WASM load:', error);
            }

            if (wrapperResult) {
                await startBoot(wrapperResult);
            } else {
                const directImports = await buildImports(capabilities);
                const response = await fetch('/{{ wasm_file }}');
                const bytes = await response.arrayBuffer();
                const magic = new Uint8Array(bytes, 0, 8);
                const isWasm =
                    magic[0] === 0x00 && magic[1] === 0x61 && magic[2] === 0x73 && magic[3] === 0x6d;
                const isComponent = isWasm &&
                    magic[4] === 0x0d && magic[5] === 0x00 && magic[6] === 0x01 && magic[7] === 0x00;

                if (isComponent) {
                    // A component without a working wrapper is exactly the
                    // "wrapper failed" case, not a load we can fake.
                    throw (wrapperErr || new Error(
                        'WASM Component requires the jco wrapper, and the wrapper could not be loaded.',
                    ));
                }
                const module = await WebAssembly.compile(bytes);
                const moduleResult = await WebAssembly.instantiate(module, directImports);
                await startBoot(moduleResult);
            }
        } catch (bootErr) {
            // The lifecycle already reported through `bootSurface.fail` when it
            // got that far; this covers instantiation/fetch failures that never
            // reached it (a bad magic header, a 404 for the wasm). A missing
            // /component-boot.js is caught EARLIER, by the outermost catch —
            // both land on `markBootFailure`, so the attribute is always set.
            if (!document.documentElement.dataset.tairitsuError) {
                // W6: a capability problem must not read as a generic load
                // failure — the operator needs to know the component asked for
                // something the declaration does not allow.
                const message = String((bootErr && bootErr.message) || bootErr || '');
                // A dist that lacks the declaration is an incomplete build, not
                // a grant decision — say so instead of blaming the component.
                const kind = /capability declaration is missing/i.test(message)
                    ? 'capability-declaration-missing'
                    : /capabilit|not satisfied|missing import|WASI modules/i.test(message)
                        ? 'capability-denied'
                        : 'instantiate-failed';
                bootSurface.fail(kind, bootErr);
            }
        }

        // Wait for all stylesheets to finish loading before running any
        // post-boot DOM fixups.  CSS <link>s in <head> block rendering but
        // a deferred module script can execute before every stylesheet has
        // fully parsed (edge-case on slow networks / large bundles).
        const stylesReady = () => {
            if (!document.styleSheets) return Promise.resolve();
            const pending = [];
            for (const sheet of document.styleSheets) {
                try { void sheet.cssRules; } catch (_) {
                    // cssRules throws when the stylesheet is still loading
                    // (cross-origin sheets always throw, but those are fine
                    // since they are not our component styles).
                    pending.push(sheet);
                }
            }
            if (pending.length === 0) return Promise.resolve();
            return new Promise(resolve => {
                let count = 0;
                const check = () => {
                    count++;
                    if (count > 50) { resolve(); return; }
                    let stillPending = 0;
                    for (const s of pending) {
                        try { void s.cssRules; } catch (_) { stillPending++; }
                    }
                    if (stillPending === 0) resolve();
                    else setTimeout(check, 40);
                };
                check();
            });
        };

        // Fix SVG elements created with wrong namespace (HTML instead of SVG).
        // WIT document::createElement always creates HTML-namespaced elements,
        // which makes SVG graphics invisible. This post-process step replaces
        // any HTML-namespaced <svg> elements with proper SVG namespaced ones.
        const fixSvgNamespaces = () => {
            const SVG_NS = 'http://www.w3.org/2000/svg';
            document.querySelectorAll('svg').forEach(svg => {
                if (svg.namespaceURI === SVG_NS) return;
                const parent = svg.parentNode;
                if (!parent) return;
                const newSvg = document.createElementNS(SVG_NS, 'svg');
                for (const attr of svg.attributes) {
                    if (attr.name.toLowerCase() === 'viewbox') {
                        newSvg.setAttribute('viewBox', attr.value);
                    } else {
                        newSvg.setAttribute(attr.name, attr.value);
                    }
                }
                newSvg.innerHTML = svg.innerHTML;
                parent.replaceChild(newSvg, svg);
            });
        };

        // WASM just replaced #app content; re-run SPA router so the
        // correct page is activated for the current URL path.
        // The WASM boot function may spawn async DOM work (rAF, setTimeout,
        // etc.) that finishes *after* the await resolves.  A single rAF
        // is not enough — we wait for DOM mutations inside #app to settle
        // before calling navigate(), so every .ts-page element exists.
        const waitForDomSettle = (root, maxWait) => {
            return new Promise(resolve => {
                const deadline = Date.now() + maxWait;
                let timer = null;
                let settled = false;

                const tryResolve = () => {
                    if (settled) return;
                    settled = true;
                    if (timer) { clearTimeout(timer); timer = null; }
                    observer.disconnect();
                    resolve();
                };

                const observer = new MutationObserver(() => {
                    // mutations happening — reset the settle timer
                    if (timer) clearTimeout(timer);
                    if (Date.now() >= deadline) { tryResolve(); return; }
                    timer = setTimeout(tryResolve, 80);
                });

                observer.observe(root, {
                    childList: true,
                    subtree: true,
                    attributes: false,
                });

                // If no mutations fire within a short window, resolve.
                timer = setTimeout(tryResolve, 120);

                // Absolute safety net.
                setTimeout(tryResolve, maxWait);
            });
        };

        (async () => {
            await stylesReady();

            const appRoot = document.getElementById('app');
            if (appRoot) {
                await waitForDomSettle(appRoot, 2000);
            }

            fixSvgNamespaces();

            const appEl = document.getElementById('app');
            if (appEl) appEl.classList.remove('tairitsu-boot-screen');

            // Sync data-theme attribute so CSS custom properties resolve
            // correctly for dark mode (spa.css uses [data-theme=dark]).
            const tsApp = document.getElementById('ts-app');
            if (tsApp && tsApp.classList.contains('hi-layout-dark')) {
                document.documentElement.setAttribute('data-theme', 'tairitsu');
                document.body.classList.add('hi-layout-dark');
            }

            if (typeof navigate === 'function') {
                navigate();
            } else {
                window.dispatchEvent(new PopStateEvent('popstate'));
            }

            // W3: the ready attribute is the lifecycle's success signal. The
            // post-boot fixups above still run (styles, theme sync, routing),
            // but a component that never booted stays "not ready" — with the
            // error attribute set by the boot surface.
            try {
                await bootSettled;
            } catch (_bootFailure) {
                return;
            }
            // The successful instance stays current until the page goes away;
            // the dual-instance test drives the API to add and dispose peers.
            document.body.removeAttribute('data-booting');
            document.documentElement.dataset.tairitsuReady='hydrated';
        })();
        (function initGlow() {
            let rafId = null;
            const activeGlow = new Map();

            document.addEventListener('mouseenter', function(e) {
                const glow = e.target.closest ? e.target.closest('[data-glow]') : null;
                if (!glow || activeGlow.has(glow)) return;
                activeGlow.set(glow, { x: 50, y: 50 });
                if (!rafId) scheduleFrame();
            }, true);

            document.addEventListener('mousemove', function(e) {
                const glow = e.target.closest ? e.target.closest('[data-glow]') : null;
                if (!glow || !activeGlow.has(glow)) return;
                const rect = glow.getBoundingClientRect();
                if (rect.width > 0 && rect.height > 0) {
                    activeGlow.get(glow).x = ((e.clientX - rect.left) / rect.width * 100).toFixed(1);
                    activeGlow.get(glow).y = ((e.clientY - rect.top) / rect.height * 100).toFixed(1);
                }
            }, true);

            document.addEventListener('mouseleave', function(e) {
                const glow = e.target.closest ? e.target.closest('[data-glow]') : null;
                if (!glow) return;
                activeGlow.delete(glow);
                glow.style.setProperty('--glow-x', '50%');
                glow.style.setProperty('--glow-y', '50%');
            }, true);

            function scheduleFrame() {
                rafId = requestAnimationFrame(function tick() {
                    for (const [el, pos] of activeGlow) {
                        el.style.setProperty('--glow-x', pos.x + '%');
                        el.style.setProperty('--glow-y', pos.y + '%');
                    }
                    if (activeGlow.size > 0) {
                        rafId = requestAnimationFrame(tick);
                    } else {
                        rafId = null;
                    }
                });
            }
        })();

        // Hot-reload client: connects to dev server SSE endpoint.
        // On rebuild notification, reloads the page. Gives up after 3
        // consecutive failures and suggests using release builds.
        (function initHotReload() {
            let failures = 0;
            const MAX_FAILURES = 3;
            function connect() {
                const es = new EventSource('/__tairitsu_reload');
                es.onmessage = function(ev) {
                    if (ev.data === 'reload') {
                        console.log('[tairitsu] rebuild detected, reloading…');
                        location.reload();
                    }
                };
                es.onerror = function() {
                    failures++;
                    es.close();
                    if (failures >= MAX_FAILURES) {
                        console.warn(
                            '[tairitsu] hot-reload connection failed ' + MAX_FAILURES +
                            ' times. Dev server may not be running.' +
                            '\nFor production builds, use: tairitsu build --release'
                        );
                        return;
                    }
                    setTimeout(connect, 3000);
                };
            }
            connect();
        })();

        // Scrollbar auto-retract: track expands to 8px on scroll,
        // retracts to 4px with CSS transition after ~1s of inactivity.
        (function initScrollbarAutoRetract() {
            var RETRACT_DELAY = 1000;
            var timers = new WeakMap();
            function setupTrack(contentEl) {
                var wrapper = contentEl.closest('.custom-scrollbar-wrapper');
                if (!wrapper) return;
                var track = wrapper.querySelector('.custom-scrollbar-track');
                if (!track) return;
                if (timers.has(track)) return;
                timers.set(track, null);
                contentEl.addEventListener('scroll', function() {
                    track.style.width = '8px';
                    track.style.transition = 'none';
                    var t = timers.get(track);
                    if (t) clearTimeout(t);
                    timers.set(track, setTimeout(function() {
                        track.style.transition = 'width 300ms cubic-bezier(0.25, 0.1, 0.25, 1)';
                        track.style.width = '4px';
                        timers.set(track, null);
                    }, RETRACT_DELAY));
                }, { passive: true });
            }
            function initAll() {
                document.querySelectorAll('.custom-scrollbar-content').forEach(setupTrack);
            }
            initAll();
            new MutationObserver(initAll).observe(document.body, { childList: true, subtree: true });
        })();

        } catch (topErr) {
            // Covers the paths that never reached `bootSurface` at all — most
            // importantly a dist whose /component-boot.js or glue bundle is
            // missing (deploy skew): the card shows AND the attribute is set.
            bootError('Unhandled error', topErr, 'boot-failed');
        }
    