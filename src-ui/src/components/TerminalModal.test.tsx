import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/preact';
import { invoke } from '@tauri-apps/api/core';
import { TerminalModal } from './TerminalModal';
import type { PeerDevice } from '../App';

const mockInvoke = vi.mocked(invoke);

function makePeer(overrides: Partial<PeerDevice> = {}): PeerDevice {
  return {
    id: 'peer-1',
    name: 'Server A',
    hostname: 'server-a',
    ip_address: '100.64.0.1',
    port: 22,
    connection_type: 'Tailscale',
    status: 'Online',
    last_seen: null,
    ssh_user: 'user',
    ssh_key_path: null,
    ssh_password: 'pass',
    is_trusted: true,
    tailscale_hostname: null,
    flymode_version: null,
    ...overrides,
  };
}

const peerA = makePeer({ id: 'peer-a', name: 'Server A', ip_address: '100.64.0.1' });
const peerB = makePeer({ id: 'peer-b', name: 'Server B', ip_address: '100.64.0.2' });
const peerC = makePeer({ id: 'peer-c', name: 'Server C', ip_address: '100.64.0.3' });

describe('TerminalModal', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // Default: open_terminal resolves with a session ID
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === 'open_terminal') return 'session-001';
      if (cmd === 'close_terminal') return undefined;
      if (cmd === 'send_terminal_input') return undefined;
      if (cmd === 'resize_terminal') return undefined;
      return undefined;
    });
  });

  it('renders modal with title and close button', () => {
    render(
      <TerminalModal
        openclawPeers={[peerA]}
        initialPeer={peerA}
        onClose={vi.fn()}
      />
    );

    expect(screen.getByText('OpenClaw Terminal')).toBeInTheDocument();
    expect(screen.getByText('x')).toBeInTheDocument();
  });

  it('hides device navbar when only one peer', () => {
    const { container } = render(
      <TerminalModal
        openclawPeers={[peerA]}
        initialPeer={peerA}
        onClose={vi.fn()}
      />
    );

    expect(container.querySelector('.terminal-devices')).toBeNull();
  });

  it('shows device navbar when multiple peers', () => {
    const { container } = render(
      <TerminalModal
        openclawPeers={[peerA, peerB, peerC]}
        initialPeer={peerA}
        onClose={vi.fn()}
      />
    );

    const navbar = container.querySelector('.terminal-devices');
    expect(navbar).not.toBeNull();

    // All three peer tabs should be visible
    expect(screen.getByText('Server A')).toBeInTheDocument();
    expect(screen.getByText('Server B')).toBeInTheDocument();
    expect(screen.getByText('Server C')).toBeInTheDocument();
  });

  it('marks initial peer tab as active', () => {
    render(
      <TerminalModal
        openclawPeers={[peerA, peerB]}
        initialPeer={peerA}
        onClose={vi.fn()}
      />
    );

    const tabA = screen.getByText('Server A').closest('button');
    const tabB = screen.getByText('Server B').closest('button');

    expect(tabA).toHaveClass('active');
    expect(tabB).not.toHaveClass('active');
  });

  it('connects to initial peer on mount', async () => {
    render(
      <TerminalModal
        openclawPeers={[peerA, peerB]}
        initialPeer={peerA}
        onClose={vi.fn()}
      />
    );

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith(
        'open_terminal',
        expect.objectContaining({ peer: peerA })
      );
    });
  });

  it('switches active tab on click without reconnecting', async () => {
    const { container } = render(
      <TerminalModal
        openclawPeers={[peerA, peerB]}
        initialPeer={peerA}
        onClose={vi.fn()}
      />
    );

    // Wait for initial connection
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith(
        'open_terminal',
        expect.objectContaining({ peer: peerA })
      );
    });

    // Click on peer B tab
    fireEvent.click(screen.getByText('Server B'));

    // Should connect to B
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith(
        'open_terminal',
        expect.objectContaining({ peer: peerB })
      );
    });

    // Tab B should now be active
    const tabB = screen.getByText('Server B').closest('button');
    expect(tabB).toHaveClass('active');

    // Clear and switch back to A — should NOT open_terminal again
    mockInvoke.mockClear();
    fireEvent.click(screen.getByText('Server A'));

    const tabA = screen.getByText('Server A').closest('button');
    expect(tabA).toHaveClass('active');

    // open_terminal should NOT be called again for peer A
    expect(mockInvoke).not.toHaveBeenCalledWith(
      'open_terminal',
      expect.objectContaining({ peer: peerA })
    );
  });

  it('calls onClose and close_terminal when close button clicked', async () => {
    const onClose = vi.fn();
    render(
      <TerminalModal
        openclawPeers={[peerA]}
        initialPeer={peerA}
        onClose={onClose}
      />
    );

    // Wait for connection
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith(
        'open_terminal',
        expect.objectContaining({ peer: peerA })
      );
    });

    // Click close
    fireEvent.click(screen.getByText('x'));

    expect(onClose).toHaveBeenCalled();
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith(
        'close_terminal',
        { sessionId: 'session-001' }
      );
    });
  });

  it('calls onClose when overlay is clicked', async () => {
    const onClose = vi.fn();
    const { container } = render(
      <TerminalModal
        openclawPeers={[peerA]}
        initialPeer={peerA}
        onClose={onClose}
      />
    );

    const overlay = container.querySelector('.terminal-overlay')!;
    fireEvent.click(overlay);

    expect(onClose).toHaveBeenCalled();
  });

  it('does not close when modal body is clicked', () => {
    const onClose = vi.fn();
    const { container } = render(
      <TerminalModal
        openclawPeers={[peerA]}
        initialPeer={peerA}
        onClose={onClose}
      />
    );

    const modal = container.querySelector('.terminal-modal')!;
    fireEvent.click(modal);

    expect(onClose).not.toHaveBeenCalled();
  });

  it('shows connecting dot status while connecting', async () => {
    // Make open_terminal hang (never resolve)
    mockInvoke.mockImplementation(() => new Promise(() => {}));

    const { container } = render(
      <TerminalModal
        openclawPeers={[peerA, peerB]}
        initialPeer={peerA}
        onClose={vi.fn()}
      />
    );

    await waitFor(() => {
      const dot = container.querySelector('.terminal-device-tab.active .device-dot');
      expect(dot).toHaveClass('connecting');
    });
  });

  it('shows error dot on connection failure', async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === 'open_terminal') throw new Error('SSH failed');
      return undefined;
    });

    const { container } = render(
      <TerminalModal
        openclawPeers={[peerA, peerB]}
        initialPeer={peerA}
        onClose={vi.fn()}
      />
    );

    await waitFor(() => {
      const dot = container.querySelector('.terminal-device-tab.active .device-dot');
      expect(dot).toHaveClass('error');
    });
  });

  it('renders terminal containers with correct visibility', async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === 'open_terminal') return 'session-001';
      return undefined;
    });

    const { container } = render(
      <TerminalModal
        openclawPeers={[peerA, peerB]}
        initialPeer={peerA}
        onClose={vi.fn()}
      />
    );

    // Wait for initial peer to connect
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith(
        'open_terminal',
        expect.objectContaining({ peer: peerA })
      );
    });

    // Active container should be visible
    const containers = container.querySelectorAll('.terminal-container');
    const visibleContainers = Array.from(containers).filter(
      (el) => (el as HTMLElement).style.display !== 'none'
    );
    expect(visibleContainers.length).toBe(1);
  });
});
