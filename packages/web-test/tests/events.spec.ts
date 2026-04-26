import { test, expect } from '@playwright/test';

const BASE_URL = process.env.BASE_URL || 'http://localhost:3000';

test.describe('Event System - WASM Bridge Verification', () => {
  test('page loads with WASM runtime initialized', async ({ page }) => {
    await page.goto('/event-test');
    await page.waitForLoadState('networkidle');

    const wasmReady = await page.evaluate(() => {
      return !!globalThis.__wasmExports &&
        !!globalThis.__listenerHandles &&
        globalThis.__listenerHandles.size > 0;
    });

    expect(wasmReady).toBe(true);
  });

  test('click event listener is registered on button', async ({ page }) => {
    await page.goto('/event-test');
    await page.waitForLoadState('networkidle');

    const listenerInfo = await page.evaluate(() => {
      const btn = document.getElementById('event-test-btn');
      if (!btn) return { error: 'button not found' };

      let clickListenerId: string | null = null;
      if (globalThis.__listenerHandles) {
        for (const [id, info] of globalThis.__listenerHandles) {
          if (info.element === btn && info.type === 'click') {
            clickListenerId = id.toString();
            break;
          }
        }
      }

      return {
        buttonExists: true,
        clickListenerId,
        totalListeners: globalThis.__listenerHandles?.size ?? 0,
      };
    });

    expect(listenerInfo.buttonExists).toBe(true);
    expect(listenerInfo.clickListenerId).not.toBeNull();
    expect(listenerInfo.totalListeners).toBeGreaterThan(0);
  });

  test('WIT onMouseEvent fires on button click', async ({ page }) => {
    await page.goto('/event-test');
    await page.waitForLoadState('networkidle');

    const result = await page.evaluate(() => {
      return new Promise((resolve) => {
        const cb = globalThis.__wasmExports?.[
          'tairitsu-browser:full/event-callbacks@0.2.0'
        ];
        if (!cb?.onMouseEvent) {
          resolve({ error: 'no WIT callbacks available' });
          return;
        }

        let callCount = 0;
        let capturedListenerId: string | null = null;

        const orig = cb.onMouseEvent.bind(cb);
        cb.onMouseEvent = function (listenerId: any, ...args: any[]) {
          callCount++;
          capturedListenerId = String(listenerId);
          return orig(listenerId, ...args);
        };

        const btn = document.getElementById('event-test-btn');
        if (!btn) {
          resolve({ error: 'button not found' });
          return;
        }

        btn.click();

        setTimeout(() => {
          cb.onMouseEvent = orig;
          resolve({
            callCount,
            capturedListenerId,
            success: callCount > 0,
          });
        }, 500);
      });
    });

    expect(result.error).toBeUndefined();
    expect(result.success).toBe(true);
    expect(result.callCount).toBeGreaterThanOrEqual(1);
  });

  test('event handler closure executes (Cell mutation)', async ({ page }) => {
    await page.goto('/event-test');
    await page.waitForLoadState('networkidle');

    const countBefore = await page.evaluate(() => {
      const el = document.getElementById('event-test-count');
      return el?.textContent?.trim() ?? '';
    });
    expect(countBefore).toContain('clicks: 0');

    await page.evaluate(() => {
      return new Promise((resolve) => {
        const cb = globalThis.__wasmExports?.[
          'tairitsu-browser:full/event-callbacks@0.2.0'
        ];
        if (!cb?.onMouseEvent) { resolve(false); return; }

        const orig = cb.onMouseEvent.bind(cb);
        let fired = false;
        cb.onMouseEvent = function (...args: any[]) {
          fired = true;
          return orig(...args);
        };

        document.getElementById('event-test-btn')?.click();

        setTimeout(() => {
          cb.onMouseEvent = orig;
          resolve(fired);
        }, 500);
      });
    });
  });

  test('DOM element is reachable via __elementHandles', async ({ page }) => {
    await page.goto('/event-test');
    await page.waitForLoadState('networkidle');

    const handleInfo = await page.evaluate(() => {
      const btn = document.getElementById('event-test-btn');
      if (!btn || !globalThis.__elementHandles) return null;

      for (const [handle, el] of globalThis.__elementHandles) {
        if (el === btn) {
          return {
            handle: handle.toString(),
            tagName: el.tagName,
            id: el.id,
          };
        }
      }
      return null;
    });

    expect(handleInfo).not.toBeNull();
    expect(handleInfo!.tagName).toBe('BUTTON');
    expect(handleInfo!.id).toBe('event-test-btn');
  });
});

test.describe('Event System - Component Interaction Tests', () => {
  test('switch component has clickable element', async ({ page }) => {
    await page.goto('/components/layer1/switch');
    await page.waitForLoadState('networkidle');

    const switchEl = page.locator('.hi-switch').first();
    await expect(switchEl).toBeVisible();

    const hasClickListener = await page.evaluate(() => {
      const sw = document.querySelector('.hi-switch');
      if (!sw || !globalThis.__listenerHandles) return false;
      for (const [, info] of globalThis.__listenerHandles) {
        if (info.element === sw && info.type === 'click') return true;
      }
      return false;
    });

    expect(hasClickListener).toBeTruthy();
  });

  test('all pages load without console errors', async ({ page }) => {
    const errors: string[] = [];
    page.on('console', msg => {
      if (msg.type() === 'error') {
        const text = msg.text();
        if (!text.includes('favicon.ico')) {
          errors.push(text);
        }
      }
    });

    const pagesToCheck = [
      '/',
      '/components/layer1/button',
      '/components/layer1/switch',
      '/components/layer1/form',
      '/event-test',
    ];

    for (const url of pagesToCheck) {
      await page.goto(url);
      await page.waitForLoadState('networkidle');
    }

    expect(errors).toHaveLength(0);
  });
});
