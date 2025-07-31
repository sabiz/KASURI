import { describe, it, expect, beforeEach, beforeAll, vi } from 'vitest';
import { Backend } from './backend';
import type { Application } from './backend';
import { mockIPC, clearMocks } from '@tauri-apps/api/mocks';

beforeAll(() => {
    if (!window.crypto) {
        Object.defineProperty(window, 'crypto', {
            value: {
                getRandomValues: (buffer: any) => {
                    // Simple polyfill for getRandomValues using Math.random (for testing only)
                    for (let i = 0; i < buffer.length; i++) {
                        buffer[i] = Math.floor(Math.random() * 256);
                    }
                    return buffer;
                },
            },
        });
    }
});

describe('Backend', () => {
    let backend: Backend;

    beforeEach(() => {
        backend = new Backend();
        clearMocks();
    });

    it('searchApplication calls invoke with correct args', async () => {
        mockIPC((cmd, args) => {
            if (cmd === 'search_application') {
                expect(args).toEqual({ query: 'query' });
                return [{ name: 'App', app_id: 'id', icon_path: 'icon.png' }];
            }
        });
        const result = await backend.searchApplication('query');
        expect(result).toEqual([{ name: 'App', app_id: 'id', icon_path: 'icon.png' }]);
    });

    it('sendContentSize skips if size is same', async () => {
        backend.lastContentSize = 100;
        let called = false;
        mockIPC(() => { called = true; });
        await backend.sendContentSize(100);
        expect(called).toBe(false);
    });

    it('sendContentSize calls invoke if size is different', async () => {
        backend.lastContentSize = 100;
        let called = false;
        mockIPC((cmd, args) => {
            if (cmd === 'changed_content_size') {
                called = true;
                expect(args).toEqual({ contentHeight: 200 });
            }
        });
        await backend.sendContentSize(200);
        expect(backend.lastContentSize).toBe(200);
        expect(called).toBe(true);
    });

    it('close calls invoke with correct args', async () => {
        let called = false;
        mockIPC((cmd) => {
            if (cmd === 'close_window') called = true;
        });
        await backend.close();
        expect(called).toBe(true);
    });

    it('launch calls invoke with correct args', async () => {
        let called = false;
        mockIPC((cmd, args) => {
            if (cmd === 'launch_application') {
                called = true;
                expect(args).toEqual({ appId: 'id' });
            }
        });
        const app: Application = { name: 'App', app_id: 'id', icon_path: 'icon.png' };
        await backend.launch(app);
        expect(called).toBe(true);
    });

    it('sendContentSize throws if contentSize is negative', async () => {
        await expect(backend.sendContentSize(-1)).rejects.toThrow('Content size cannot be negative');
    });

    it('sendContentSize allows 0 as contentSize', async () => {
        backend.lastContentSize = 1;
        let called = false;
        mockIPC((cmd, args) => {
            if (cmd === 'changed_content_size') {
                called = true;
                expect(args).toEqual({ contentHeight: 0 });
            }
        });
        await backend.sendContentSize(0);
        expect(backend.lastContentSize).toBe(0);
        expect(called).toBe(true);
    });

    it('launch throws if app_id is empty', async () => {
        const app: Application = { name: 'App', app_id: '', icon_path: 'icon.png' };
        await expect(backend.launch(app)).rejects.toThrow('Invalid application object');
    });

    it('launch throws if app_id is null', async () => {
        const app: Application = { name: 'App', app_id: null as any, icon_path: 'icon.png' };
        await expect(backend.launch(app)).rejects.toThrow('Invalid application object');
    });
});
