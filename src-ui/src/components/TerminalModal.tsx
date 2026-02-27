import { useEffect, useRef, useCallback } from 'preact/hooks';
import { invoke, Channel } from '@tauri-apps/api/core';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import '@xterm/xterm/css/xterm.css';
import type { PeerDevice } from '../App';
import { toast } from './Toast';

interface TerminalModalProps {
  peer: PeerDevice;
  onClose: () => void;
}

export function TerminalModal({ peer, onClose }: TerminalModalProps) {
  const termRef = useRef<HTMLDivElement>(null);
  const terminalRef = useRef<Terminal | null>(null);
  const fitAddonRef = useRef<FitAddon | null>(null);
  const sessionIdRef = useRef<string | null>(null);
  const closingRef = useRef(false);

  const cleanup = useCallback(async () => {
    if (closingRef.current) return;
    closingRef.current = true;

    if (sessionIdRef.current) {
      try {
        await invoke('close_terminal', { sessionId: sessionIdRef.current });
      } catch {
        // Session may already be closed
      }
      sessionIdRef.current = null;
    }
    if (terminalRef.current) {
      terminalRef.current.dispose();
      terminalRef.current = null;
    }
  }, []);

  useEffect(() => {
    if (!termRef.current) return;

    const term = new Terminal({
      cursorBlink: true,
      cursorStyle: 'block',
      cursorInactiveStyle: 'outline',
      fontSize: 14,
      fontFamily: "'JetBrains Mono', 'Fira Code', 'Cascadia Code', monospace",
      theme: {
        background: '#0f172a',
        foreground: '#f1f5f9',
        cursor: '#3b82f6',
        selectionBackground: '#334155',
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
    });

    const fitAddon = new FitAddon();
    term.loadAddon(fitAddon);
    term.open(termRef.current);
    fitAddon.fit();

    terminalRef.current = term;
    fitAddonRef.current = fitAddon;

    term.writeln(`Connecting to ${peer.name} (${peer.ip_address})...`);

    // Create output channel
    const onData = new Channel<Uint8Array>();
    onData.onmessage = (data: Uint8Array) => {
      term.write(data);
    };

    const cols = term.cols;
    const rows = term.rows;

    // Open terminal session
    invoke<string>('open_terminal', {
      peer,
      cols,
      rows,
      onData,
    })
      .then((sid) => {
        sessionIdRef.current = sid;
        term.focus();

        // Forward all keystrokes (including IME composed text) to backend.
        // xterm.js's CompositionHelper handles IME and fires onData with
        // the final composed text — no need to send from compositionend.
        term.onData((data: string) => {
          if (sessionIdRef.current) {
            const encoded = new TextEncoder().encode(data);
            invoke('send_terminal_input', {
              sessionId: sessionIdRef.current,
              data: Array.from(encoded),
            }).catch(() => {
              // Session may have closed
            });
          }
        });

        // Clear textarea after each composition to prevent text accumulation.
        // In WebKitGTK, backspace sends \x7f to remote but doesn't clear
        // the textarea, so old composed text replays on next composition.
        const xtermTextarea = termRef.current?.querySelector('textarea') as HTMLTextAreaElement | null;
        if (xtermTextarea) {
          xtermTextarea.addEventListener('compositionend', () => {
            setTimeout(() => { xtermTextarea.value = ''; }, 50);
          });
        }

        // Forward resize events
        term.onResize(({ cols, rows }: { cols: number; rows: number }) => {
          if (sessionIdRef.current) {
            invoke('resize_terminal', {
              sessionId: sessionIdRef.current,
              cols,
              rows,
            }).catch(() => {});
          }
        });
      })
      .catch((e) => {
        term.writeln(`\r\nConnection failed: ${e}`);
        toast.error(`Terminal connection failed: ${e}`);
      });

    // ResizeObserver for container size changes
    const observer = new ResizeObserver(() => {
      if (fitAddonRef.current) {
        fitAddonRef.current.fit();
      }
    });
    observer.observe(termRef.current);

    return () => {
      observer.disconnect();
      cleanup();
    };
  }, [peer, cleanup]);

  const handleOverlayClick = (e: MouseEvent) => {
    if ((e.target as HTMLElement).classList.contains('terminal-overlay')) {
      cleanup();
      onClose();
    }
  };

  const handleClose = () => {
    cleanup();
    onClose();
  };

  return (
    <div class="terminal-overlay" onClick={handleOverlayClick}>
      <div class="terminal-modal" onClick={(e) => e.stopPropagation()}>
        <div class="terminal-header">
          <span class="terminal-title">
            {peer.name} ({peer.ip_address})
          </span>
          <button class="modal-close" onClick={handleClose}>
            x
          </button>
        </div>
        <div
          class="terminal-container"
          ref={termRef}
          onClick={() => terminalRef.current?.focus()}
        />
      </div>
    </div>
  );
}
