import { useState, useEffect, useRef, useCallback } from 'preact/hooks';
import { invoke, Channel } from '@tauri-apps/api/core';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { WebglAddon } from '@xterm/addon-webgl';
import '@xterm/xterm/css/xterm.css';
import type { PeerDevice } from '../App';
import { toast } from './Toast';

interface TerminalModalProps {
  openclawPeers: PeerDevice[];
  initialPeer: PeerDevice;
  onClose: () => void;
}

type SessionStatus = 'idle' | 'connecting' | 'connected' | 'error';

interface TermSession {
  peer: PeerDevice;
  sessionId: string | null;
  terminal: Terminal | null;
  fitAddon: FitAddon | null;
  status: SessionStatus;
  pasteHandler: ((ev: KeyboardEvent) => void) | null;
  resizeObserver: ResizeObserver | null;
}

const TERM_OPTIONS = {
  cursorBlink: true,
  cursorStyle: 'block' as const,
  cursorInactiveStyle: 'outline' as const,
  fontSize: 14,
  fontFamily: "'JetBrains Mono', 'Fira Code', 'Cascadia Code', monospace",
  theme: {
    background: '#0f172a',
    foreground: '#f1f5f9',
    cursor: '#3b82f6',
    cursorAccent: '#0f172a',
    selectionBackground: '#334155',
    selectionInactiveBackground: '#1e293b',
    black: '#0f172a',
    red: '#ef4444',
    green: '#22c55e',
    yellow: '#eab308',
    blue: '#3b82f6',
    magenta: '#a855f7',
    cyan: '#06b6d4',
    white: '#f1f5f9',
    brightBlack: '#334155',
    brightRed: '#f87171',
    brightGreen: '#4ade80',
    brightYellow: '#facc15',
    brightBlue: '#60a5fa',
    brightMagenta: '#c084fc',
    brightCyan: '#22d3ee',
    brightWhite: '#ffffff',
  },
};

function needsPassword(peer: PeerDevice): boolean {
  return peer.id === '__local__' && !peer.ssh_key_path && !peer.ssh_password;
}

export function TerminalModal({ openclawPeers, initialPeer, onClose }: TerminalModalProps) {
  const [sessions, setSessions] = useState<Map<string, TermSession>>(new Map());
  const [activePeerId, setActivePeerId] = useState<string>(initialPeer.id);
  const [peersWithCreds, setPeersWithCreds] = useState<Map<string, PeerDevice>>(new Map());
  const [passwordPrompt, setPasswordPrompt] = useState<PeerDevice | null>(null);
  const [passwordInput, setPasswordInput] = useState('');
  const sessionsRef = useRef<Map<string, TermSession>>(new Map());
  const closingRef = useRef(false);
  const containerRefs = useRef<Map<string, HTMLDivElement>>(new Map());
  const awaitingPasswordRef = useRef(false);

  // Keep ref in sync with state
  sessionsRef.current = sessions;

  const cleanupSession = useCallback(async (peerId: string) => {
    const session = sessionsRef.current.get(peerId);
    if (!session) return;

    if (session.sessionId) {
      try {
        await invoke('close_terminal', { sessionId: session.sessionId });
      } catch {
        // Session may already be closed
      }
    }
    if (session.pasteHandler) {
      const container = containerRefs.current.get(peerId);
      container?.removeEventListener('keydown', session.pasteHandler);
    }
    if (session.resizeObserver) {
      session.resizeObserver.disconnect();
    }
    if (session.terminal) {
      session.terminal.dispose();
    }
  }, []);

  const cleanupAll = useCallback(async () => {
    if (closingRef.current) return;
    closingRef.current = true;

    const promises: Promise<void>[] = [];
    for (const peerId of sessionsRef.current.keys()) {
      promises.push(cleanupSession(peerId));
    }
    await Promise.all(promises);
  }, [cleanupSession]);

  // Cleanup on unmount
  useEffect(() => {
    return () => { cleanupAll(); };
  }, [cleanupAll]);

  const connectPeer = useCallback((peer: PeerDevice, containerEl: HTMLDivElement) => {
    // Mark connecting
    setSessions(prev => {
      const next = new Map(prev);
      next.set(peer.id, {
        peer,
        sessionId: null,
        terminal: null,
        fitAddon: null,
        status: 'connecting',
        pasteHandler: null,
        resizeObserver: null,
      });
      return next;
    });

    const term = new Terminal(TERM_OPTIONS);
    const fitAddon = new FitAddon();
    term.loadAddon(fitAddon);
    term.open(containerEl);
    fitAddon.fit();

    // Try WebGL renderer
    try {
      const webglAddon = new WebglAddon();
      term.loadAddon(webglAddon);
    } catch {
      console.warn('WebGL renderer not available, using default canvas renderer');
    }

    // Force cursor options after open
    term.options.cursorStyle = 'block';
    term.options.cursorBlink = true;

    term.writeln(`Connecting to ${peer.name} (${peer.ip_address})...`);

    // Create output channel
    const onData = new Channel<Uint8Array>();
    onData.onmessage = (data: Uint8Array) => {
      term.write(data);
    };

    const cols = term.cols;
    const rows = term.rows;

    // Paste handler
    const pasteHandler = (ev: KeyboardEvent) => {
      if (ev.ctrlKey && ev.shiftKey && ev.key === 'V') {
        ev.preventDefault();
        const currentSession = sessionsRef.current.get(peer.id);
        navigator.clipboard.readText().then((text) => {
          if (text && currentSession?.sessionId) {
            const encoded = new TextEncoder().encode(text);
            invoke('send_terminal_input', {
              sessionId: currentSession.sessionId,
              data: Array.from(encoded),
            }).catch(() => {});
          }
        });
      }
    };
    containerEl.addEventListener('keydown', pasteHandler);

    // ResizeObserver
    const resizeObserver = new ResizeObserver(() => {
      // Only fit if this is the active terminal
      const currentActive = sessionsRef.current.get(peer.id);
      if (currentActive?.fitAddon) {
        currentActive.fitAddon.fit();
      }
    });
    resizeObserver.observe(containerEl);

    invoke<string>('open_terminal', { peer, cols, rows, onData })
      .then((sid) => {
        // Wire input with dedup
        let lastSentData = '';
        let lastSentTime = 0;
        term.onData((data: string) => {
          const now = Date.now();
          if (data === lastSentData && now - lastSentTime < 50) return;
          lastSentData = data;
          lastSentTime = now;
          const currentSession = sessionsRef.current.get(peer.id);
          if (currentSession?.sessionId) {
            const encoded = new TextEncoder().encode(data);
            invoke('send_terminal_input', {
              sessionId: currentSession.sessionId,
              data: Array.from(encoded),
            }).catch(() => {});
          }
        });

        // Clear textarea after composition (IME fix)
        const xtermTextarea = containerEl.querySelector('textarea') as HTMLTextAreaElement | null;
        if (xtermTextarea) {
          xtermTextarea.addEventListener('compositionend', () => {
            setTimeout(() => { xtermTextarea.value = ''; }, 50);
          });
        }

        // Copy on selection
        term.onSelectionChange(() => {
          const sel = term.getSelection();
          if (sel) navigator.clipboard.writeText(sel).catch(() => {});
        });

        // Forward resize events
        term.onResize(({ cols, rows }: { cols: number; rows: number }) => {
          const currentSession = sessionsRef.current.get(peer.id);
          if (currentSession?.sessionId) {
            invoke('resize_terminal', {
              sessionId: currentSession.sessionId,
              cols,
              rows,
            }).catch(() => {});
          }
        });

        // Update session with connected state
        setSessions(prev => {
          const next = new Map(prev);
          next.set(peer.id, {
            peer,
            sessionId: sid,
            terminal: term,
            fitAddon,
            status: 'connected',
            pasteHandler,
            resizeObserver,
          });
          return next;
        });

        requestAnimationFrame(() => term.focus());
      })
      .catch((e) => {
        toast.error(`Terminal connection failed: ${e}`);

        // If localhost failed, clear cached password so user can re-enter
        if (peer.id === '__local__') {
          // Set ref FIRST (synchronous) to block auto-reconnect before any setState
          awaitingPasswordRef.current = true;
          try {
            term.writeln(`\r\nConnection failed: ${e}`);
          } catch { /* terminal may already be disposed */ }
          try {
            if (pasteHandler) containerEl.removeEventListener('keydown', pasteHandler);
            if (resizeObserver) resizeObserver.disconnect();
            term.dispose();
          } catch { /* cleanup best-effort */ }
          setPeersWithCreds(prev => {
            const next = new Map(prev);
            next.delete('__local__');
            return next;
          });
          setSessions(prev => {
            const next = new Map(prev);
            next.delete(peer.id);
            return next;
          });
          // Always re-prompt for password
          setPasswordInput('');
          setPasswordPrompt(peer);
        } else {
          setSessions(prev => {
            const next = new Map(prev);
            next.set(peer.id, {
              peer,
              sessionId: null,
              terminal: term,
              fitAddon,
              status: 'error',
              pasteHandler,
              resizeObserver,
            });
            return next;
          });
        }
      });
  }, []);

  // Connect initial peer on mount
  const initialConnectedRef = useRef(false);
  useEffect(() => {
    if (initialConnectedRef.current) return;
    const container = containerRefs.current.get(initialPeer.id);
    if (container) {
      initialConnectedRef.current = true;
      connectPeer(initialPeer, container);
    }
  }, [initialPeer, connectPeer]);

  const resolvePeer = useCallback((peer: PeerDevice): PeerDevice => {
    return peersWithCreds.get(peer.id) || peer;
  }, [peersWithCreds]);

  const handleDeviceClick = useCallback((peer: PeerDevice) => {
    const session = sessionsRef.current.get(peer.id);
    if (session && (session.status === 'connected' || session.status === 'connecting' || session.status === 'error')) {
      // Just switch display
      setActivePeerId(peer.id);
      // Fit and focus active terminal
      requestAnimationFrame(() => {
        const s = sessionsRef.current.get(peer.id);
        if (s?.fitAddon) s.fitAddon.fit();
        if (s?.terminal) s.terminal.focus();
      });
    } else {
      const resolved = peersWithCreds.get(peer.id) || peer;
      if (needsPassword(resolved)) {
        // Show password prompt before connecting
        setPasswordInput('');
        setPasswordPrompt(peer);
      } else {
        // Connect directly
        setActivePeerId(peer.id);
      }
    }
  }, [peersWithCreds]);

  // When activePeerId changes and there's no session, connect once container mounts
  useEffect(() => {
    // Don't auto-connect while awaiting password re-entry
    if (awaitingPasswordRef.current) return;

    const session = sessionsRef.current.get(activePeerId);
    if (session) return; // already exists

    const peer = openclawPeers.find(p => p.id === activePeerId);
    if (!peer) return;

    const resolved = peersWithCreds.get(peer.id) || peer;
    // Don't auto-connect if password is needed
    if (needsPassword(resolved)) return;

    // Wait for container ref via requestAnimationFrame
    requestAnimationFrame(() => {
      if (awaitingPasswordRef.current) return; // double-check after frame
      const container = containerRefs.current.get(activePeerId);
      if (container && !sessionsRef.current.has(activePeerId)) {
        connectPeer(resolved, container);
      }
    });
  }, [activePeerId, openclawPeers, connectPeer, peersWithCreds, passwordPrompt]);

  // Fit active terminal on window resize
  useEffect(() => {
    const handleResize = () => {
      const session = sessionsRef.current.get(activePeerId);
      if (session?.fitAddon) {
        session.fitAddon.fit();
      }
    };
    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, [activePeerId]);

  const handlePasswordSubmit = useCallback(() => {
    if (!passwordPrompt || !passwordInput) return;
    awaitingPasswordRef.current = false;
    const peerWithPass = { ...passwordPrompt, ssh_password: passwordInput };
    setPeersWithCreds(prev => {
      const next = new Map(prev);
      next.set(passwordPrompt.id, peerWithPass);
      return next;
    });
    setPasswordPrompt(null);
    setPasswordInput('');
    setActivePeerId(passwordPrompt.id);
  }, [passwordPrompt, passwordInput]);

  const handleOverlayClick = (e: MouseEvent) => {
    if ((e.target as HTMLElement).classList.contains('terminal-overlay')) {
      cleanupAll();
      onClose();
    }
  };

  const handleClose = () => {
    cleanupAll();
    onClose();
  };

  const getStatusDotClass = (peerId: string): string => {
    const session = sessions.get(peerId);
    if (!session) return 'off';
    switch (session.status) {
      case 'connected': return 'on';
      case 'connecting': return 'connecting';
      case 'error': return 'error';
      default: return 'off';
    }
  };

  // Build the set of peer IDs that need container divs (active + already-connected)
  const renderedPeerIds = new Set<string>();
  renderedPeerIds.add(activePeerId);
  for (const [peerId, session] of sessions) {
    if (session.status === 'connected' || session.status === 'connecting' || session.status === 'error') {
      renderedPeerIds.add(peerId);
    }
  }

  return (
    <div class="terminal-overlay" onClick={handleOverlayClick}>
      <div class="terminal-modal" onClick={(e) => e.stopPropagation()}>
        <div class="terminal-header">
          <span class="terminal-title">OpenClaw Terminal</span>
          <button class="modal-close" onClick={handleClose}>x</button>
        </div>

        {openclawPeers.length > 1 && (
          <div class="terminal-devices">
            {openclawPeers.map(peer => (
              <button
                key={peer.id}
                class={`terminal-device-tab ${activePeerId === peer.id ? 'active' : ''}`}
                onClick={() => handleDeviceClick(peer)}
              >
                <span class={`device-dot ${getStatusDotClass(peer.id)}`} />
                {peer.name || peer.hostname}
              </button>
            ))}
          </div>
        )}

        <div class="terminal-body">
          {Array.from(renderedPeerIds).map(peerId => (
            <div
              key={peerId}
              class="terminal-container"
              ref={(el: HTMLDivElement | null) => {
                if (el) containerRefs.current.set(peerId, el);
              }}
              style={{ display: activePeerId === peerId ? 'block' : 'none' }}
            />
          ))}
        </div>

        {passwordPrompt && (
          <div class="terminal-password-overlay">
            <div class="terminal-password-dialog">
              <div style={{ marginBottom: '12px', fontWeight: 500 }}>
                SSH Password ({passwordPrompt.ssh_user}@{passwordPrompt.ip_address})
              </div>
              <input
                type="password"
                class="form-control"
                value={passwordInput}
                onInput={e => setPasswordInput(e.currentTarget.value)}
                onKeyDown={e => { if (e.key === 'Enter' && passwordInput) handlePasswordSubmit(); }}
                placeholder="SSH password"
                autoFocus
              />
              <div style={{ display: 'flex', gap: '8px', justifyContent: 'flex-end', marginTop: '12px' }}>
                <button class="btn btn-icon btn-sm" onClick={() => { awaitingPasswordRef.current = false; setPasswordPrompt(null); }}>Cancel</button>
                <button class="btn btn-primary btn-sm" disabled={!passwordInput} onClick={handlePasswordSubmit}>Connect</button>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
