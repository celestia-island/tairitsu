// Injected into the debug WebView before page load.
// Captures console output and exposes utility functions.

(function() {
    if (window.__TAIRITSU_DEBUG_INJECTED) return;
    window.__TAIRITSU_DEBUG_INJECTED = true;

    // Capture console output for the /console endpoint
    const _origLog = console.log;
    const _origWarn = console.warn;
    const _origError = console.error;
    const _origInfo = console.info;
    const __debugConsole = [];

    function formatArgs(args) {
        return Array.from(args).map(a => {
            if (a === undefined) return 'undefined';
            if (a === null) return 'null';
            if (typeof a === 'object') {
                try { return JSON.stringify(a); }
                catch { return String(a); }
            }
            return String(a);
        }).join(' ');
    }

    console.log = function() {
        __debugConsole.push({ level: 'log', text: formatArgs(arguments), timestamp: new Date().toISOString(), source: 'page' });
        _origLog.apply(console, arguments);
    };
    console.warn = function() {
        __debugConsole.push({ level: 'warn', text: formatArgs(arguments), timestamp: new Date().toISOString(), source: 'page' });
        _origWarn.apply(console, arguments);
    };
    console.error = function() {
        __debugConsole.push({ level: 'error', text: formatArgs(arguments), timestamp: new Date().toISOString(), source: 'page' });
        _origError.apply(console, arguments);
    };
    console.info = function() {
        __debugConsole.push({ level: 'info', text: formatArgs(arguments), timestamp: new Date().toISOString(), source: 'page' });
        _origInfo.apply(console, arguments);
    };

    // Expose captured log for polling
    window.__tairitsu_debug_console = __debugConsole;

    // Unhandled error capture
    window.addEventListener('error', function(e) {
        __debugConsole.push({
            level: 'error',
            text: (e.message || '') + (e.filename ? ' at ' + e.filename + ':' + e.lineno : ''),
            timestamp: new Date().toISOString(),
            source: 'uncaught'
        });
    });

    window.addEventListener('unhandledrejection', function(e) {
        __debugConsole.push({
            level: 'error',
            text: 'Unhandled Promise Rejection: ' + (e.reason ? String(e.reason) : 'unknown'),
            timestamp: new Date().toISOString(),
            source: 'promise'
        });
    });
})();
