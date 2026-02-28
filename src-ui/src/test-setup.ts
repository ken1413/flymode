import '@testing-library/jest-dom/vitest';

// Mock @tauri-apps/api/core
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
  Channel: class MockChannel {
    onmessage: ((data: unknown) => void) | null = null;
  },
}));

// Mock @tauri-apps/api/event
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}));

// Mock xterm.js — jsdom has no canvas/webgl
vi.mock('@xterm/xterm', () => ({
  Terminal: class MockTerminal {
    cols = 80;
    rows = 24;
    options: Record<string, unknown> = {};
    open = vi.fn();
    write = vi.fn();
    writeln = vi.fn();
    dispose = vi.fn();
    focus = vi.fn();
    onData = vi.fn().mockReturnValue({ dispose: vi.fn() });
    onResize = vi.fn().mockReturnValue({ dispose: vi.fn() });
    onSelectionChange = vi.fn().mockReturnValue({ dispose: vi.fn() });
    getSelection = vi.fn().mockReturnValue('');
    loadAddon = vi.fn();
  },
}));

vi.mock('@xterm/addon-fit', () => ({
  FitAddon: class MockFitAddon {
    fit = vi.fn();
    dispose = vi.fn();
  },
}));

vi.mock('@xterm/addon-webgl', () => ({
  WebglAddon: class MockWebglAddon {
    dispose = vi.fn();
  },
}));

// Mock ResizeObserver (not in jsdom)
global.ResizeObserver = class MockResizeObserver {
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
} as unknown as typeof ResizeObserver;

// Mock clipboard API
Object.assign(navigator, {
  clipboard: {
    writeText: vi.fn().mockResolvedValue(undefined),
    readText: vi.fn().mockResolvedValue(''),
  },
});
