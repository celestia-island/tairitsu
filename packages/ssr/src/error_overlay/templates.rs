/// HTML templates for the error overlay component
pub struct Templates;

impl Templates {
    /// Get the CSS styles for the error overlay
    pub fn overlay_styles() -> &'static str {
        r#"
.tairitsu-error-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.85);
    z-index: 999999;
    display: flex;
    align-items: center;
    justify-content: center;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
    padding: 20px;
    box-sizing: border-box;
}

.tairitsu-error-container {
    background: #1a1a1a;
    border-radius: 12px;
    max-width: 800px;
    width: 100%;
    max-height: 90vh;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.5);
}

.tairitsu-error-header {
    padding: 20px 24px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    border-bottom: 1px solid #333;
}

.tairitsu-error-title {
    display: flex;
    align-items: center;
    gap: 12px;
    margin: 0;
    font-size: 18px;
    font-weight: 600;
}

.tairitsu-error-icon {
    width: 24px;
    height: 24px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 14px;
    font-weight: bold;
}

.tairitsu-error-icon.compile {
    background: #f59e0b;
    color: #000;
}

.tairitsu-error-icon.runtime {
    background: #ef4444;
    color: #fff;
}

.tairitsu-error-icon.network {
    background: #3b82f6;
    color: #fff;
}

.tairitsu-error-icon.type {
    background: #8b5cf6;
    color: #fff;
}

.tairitsu-error-close {
    background: none;
    border: none;
    color: #888;
    cursor: pointer;
    padding: 8px;
    border-radius: 6px;
    font-size: 20px;
    line-height: 1;
    transition: all 0.2s;
}

.tairitsu-error-close:hover {
    background: #333;
    color: #fff;
}

.tairitsu-error-body {
    padding: 24px;
    overflow-y: auto;
    flex: 1;
}

.tairitsu-error-message {
    margin: 0 0 16px 0;
    font-size: 16px;
    line-height: 1.6;
    color: #e5e5e5;
}

.tairitsu-error-location {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px;
    background: #252525;
    border-radius: 8px;
    margin-bottom: 16px;
    font-family: 'Monaco', 'Menlo', monospace;
    font-size: 13px;
    color: #a3a3a3;
}

.tairitsu-error-location-icon {
    color: #666;
}

.tairitsu-error-stack {
    background: #0d0d0d;
    border-radius: 8px;
    padding: 16px;
    overflow-x: auto;
}

.tairitsu-error-stack-title {
    margin: 0 0 12px 0;
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: #666;
}

.tairitsu-error-stack-content {
    margin: 0;
    font-family: 'Monaco', 'Menlo', 'Courier New', monospace;
    font-size: 12px;
    line-height: 1.6;
    color: #a3a3a3;
    white-space: pre-wrap;
    word-break: break-all;
}

.tairitsu-error-footer {
    padding: 16px 24px;
    border-top: 1px solid #333;
    display: flex;
    justify-content: flex-end;
    gap: 12px;
}

.tairitsu-error-button {
    padding: 10px 20px;
    border-radius: 6px;
    border: none;
    cursor: pointer;
    font-size: 14px;
    font-weight: 500;
    transition: all 0.2s;
}

.tairitsu-error-button.primary {
    background: #3b82f6;
    color: #fff;
}

.tairitsu-error-button.primary:hover {
    background: #2563eb;
}

.tairitsu-error-button.secondary {
    background: #333;
    color: #e5e5e5;
}

.tairitsu-error-button.secondary:hover {
    background: #404040;
}

.tairitsu-error-hidden {
    display: none !important;
}

/* Animation for overlay appearance */
@keyframes tairitsu-fade-in {
    from {
        opacity: 0;
    }
    to {
        opacity: 1;
    }
}

.tairitsu-error-overlay {
    animation: tairitsu-fade-in 0.15s ease-out;
}
"#
    }

    /// Get the HTML template for the overlay container
    pub fn overlay_container() -> &'static str {
        r#"
<div id="tairitsu-error-overlay" class="tairitsu-error-overlay tairitsu-error-hidden">
    <div class="tairitsu-error-container">
        <div class="tairitsu-error-header">
            <div class="tairitsu-error-title">
                <span class="tairitsu-error-icon" data-icon-type></span>
                <span data-title>Error</span>
            </div>
            <button class="tairitsu-error-close" data-close aria-label="Close">&times;</button>
        </div>
        <div class="tairitsu-error-body">
            <p class="tairitsu-error-message" data-message></p>
            <div class="tairitsu-error-location tairitsu-error-hidden" data-location>
                <span class="tairitsu-error-location-icon"></span>
                <span data-location-text></span>
            </div>
            <div class="tairitsu-error-stack tairitsu-error-hidden" data-stack>
                <h4 class="tairitsu-error-stack-title">Stack Trace</h4>
                <pre class="tairitsu-error-stack-content" data-stack-content></pre>
            </div>
        </div>
        <div class="tairitsu-error-footer">
            <button class="tairitsu-error-button secondary" data-copy>Copy Error</button>
            <button class="tairitsu-error-button primary" data-reload>Reload</button>
        </div>
    </div>
</div>
"#
    }

    /// Get the JavaScript for client-side error handling
    pub fn client_script() -> &'static str {
        r#"
(function() {
    'use strict';

    // Error tracking state
    const state = {
        errors: [],
        maxErrors: 50,
        overlay: null,
        isEnabled: true
    };

    // Initialize the overlay
    function initOverlay() {
        if (state.overlay) return state.overlay;

        // Check if overlay already exists in DOM
        let overlay = document.getElementById('tairitsu-error-overlay');
        if (!overlay) {
            // Create overlay from template
            const template = document.createElement('div');
            template.innerHTML = `__OVERLAY_HTML__`;
            overlay = template.firstElementChild;
            document.body.appendChild(overlay);
        }

        // Attach event listeners
        attachOverlayListeners(overlay);
        state.overlay = overlay;
        return overlay;
    }

    function attachOverlayListeners(overlay) {
        // Close button
        const closeBtn = overlay.querySelector('[data-close]');
        if (closeBtn) {
            closeBtn.addEventListener('click', hideOverlay);
        }

        // Reload button
        const reloadBtn = overlay.querySelector('[data-reload]');
        if (reloadBtn) {
            reloadBtn.addEventListener('click', () => {
                window.location.reload();
            });
        }

        // Copy button
        const copyBtn = overlay.querySelector('[data-copy]');
        if (copyBtn) {
            copyBtn.addEventListener('click', copyErrorToClipboard);
        }

        // Close on overlay click (outside container)
        overlay.addEventListener('click', (e) => {
            if (e.target === overlay) {
                hideOverlay();
            }
        });

        // Close on Escape key
        document.addEventListener('keydown', (e) => {
            if (e.key === 'Escape' && !overlay.classList.contains('tairitsu-error-hidden')) {
                hideOverlay();
            }
        });
    }

    function showError(error) {
        if (!state.isEnabled) return;

        const overlay = initOverlay();

        // Update icon type
        const icon = overlay.querySelector('[data-icon-type]');
        if (icon) {
            icon.className = 'tairitsu-error-icon ' + (error.type || 'runtime');
            icon.textContent = getIconForType(error.type);
        }

        // Update title
        const title = overlay.querySelector('[data-title]');
        if (title) {
            title.textContent = getTitleForType(error.type);
        }

        // Update message
        const message = overlay.querySelector('[data-message]');
        if (message) {
            message.textContent = error.message || 'An unknown error occurred';
        }

        // Update location
        const location = overlay.querySelector('[data-location]');
        const locationText = overlay.querySelector('[data-location-text]');
        if (error.location) {
            location.classList.remove('tairitsu-error-hidden');
            if (locationText) {
                locationText.textContent = error.location.file + ':' +
                                         error.location.line + ':' +
                                         error.location.column;
            }
        } else {
            location.classList.add('tairitsu-error-hidden');
        }

        // Update stack trace
        const stack = overlay.querySelector('[data-stack]');
        const stackContent = overlay.querySelector('[data-stack-content]');
        if (error.stack) {
            stack.classList.remove('tairitsu-error-hidden');
            if (stackContent) {
                stackContent.textContent = error.stack;
            }
        } else {
            stack.classList.add('tairitsu-error-hidden');
        }

        // Store current error for copying
        state.currentError = error;

        // Show the overlay
        overlay.classList.remove('tairitsu-error-hidden');
    }

    function hideOverlay() {
        const overlay = state.overlay || document.getElementById('tairitsu-error-overlay');
        if (overlay) {
            overlay.classList.add('tairitsu-error-hidden');
        }
    }

    function getIconForType(type) {
        const icons = {
            compile: '',
            runtime: '',
            network: '',
            type: ''
        };
        return icons[type] || '';
    }

    function getTitleForType(type) {
        const titles = {
            compile: 'Compile Error',
            runtime: 'Runtime Error',
            network: 'Network Error',
            type: 'Type Error'
        };
        return titles[type] || 'Error';
    }

    function copyErrorToClipboard() {
        if (!state.currentError) return;

        const error = state.currentError;
        let text = getTitleForType(error.type) + '\n';
        text += error.message + '\n';

        if (error.location) {
            text += '\nLocation: ' + error.location.file + ':' +
                   error.location.line + ':' + error.location.column + '\n';
        }

        if (error.stack) {
            text += '\nStack Trace:\n' + error.stack;
        }

        navigator.clipboard.writeText(text).then(() => {
            const copyBtn = state.overlay.querySelector('[data-copy]');
            const originalText = copyBtn.textContent;
            copyBtn.textContent = 'Copied!';
            setTimeout(() => {
                copyBtn.textContent = originalText;
            }, 2000);
        }).catch(err => {
            console.error('Failed to copy error:', err);
        });
    }

    function trackError(error) {
        state.errors.push(error);
        if (state.errors.length > state.maxErrors) {
            state.errors.shift();
        }
    }

    // Global error handlers
    window.addEventListener('error', (event) => {
        const error = {
            type: 'runtime',
            message: event.message,
            stack: event.error ? event.error.stack : null,
            location: {
                file: event.filename,
                line: event.lineno,
                column: event.colno
            }
        };
        trackError(error);
        showError(error);
    });

    window.addEventListener('unhandledrejection', (event) => {
        const error = {
            type: 'runtime',
            message: event.reason ? String(event.reason) : 'Unhandled promise rejection',
            stack: event.reason && event.reason.stack ? event.reason.stack : null
        };
        trackError(error);
        showError(error);
    });

    // Public API
    window.__TAIRITSU_ERROR_OVERLAY__ = {
        show: showError,
        hide: hideOverlay,
        enable: function() { state.isEnabled = true; },
        disable: function() { state.isEnabled = false; },
        getErrors: function() { return state.errors.slice(); },
        clearErrors: function() { state.errors = []; }
    };

    // Listen for custom error events from the framework
    window.addEventListener('tairitsu-error', (event) => {
        showError(event.detail);
    });
})();
"#
    }
}
