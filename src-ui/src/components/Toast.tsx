import { useState, useEffect } from 'preact/hooks';

export type ToastType = 'success' | 'error' | 'info';

interface ToastItem {
  id: number;
  type: ToastType;
  message: string;
  action?: { label: string; onClick: () => void };
}

let nextId = 0;
let addToastFn: ((type: ToastType, message: string, action?: { label: string; onClick: () => void }) => void) | null = null;

/** Global toast function — call from anywhere after ToastContainer is mounted. */
export const toast = {
  success: (msg: string, action?: { label: string; onClick: () => void }) => addToastFn?.('success', msg, action),
  error: (msg: string, action?: { label: string; onClick: () => void }) => addToastFn?.('error', msg, action),
  info: (msg: string, action?: { label: string; onClick: () => void }) => addToastFn?.('info', msg, action),
};

const DURATION_MS = 4000;
const ACTION_DURATION_MS = 10000;

export function ToastContainer() {
  const [items, setItems] = useState<ToastItem[]>([]);

  useEffect(() => {
    addToastFn = (type: ToastType, message: string, action?: { label: string; onClick: () => void }) => {
      const id = ++nextId;
      setItems(prev => [...prev, { id, type, message, action }]);
      setTimeout(() => {
        setItems(prev => prev.filter(t => t.id !== id));
      }, action ? ACTION_DURATION_MS : DURATION_MS);
    };
    return () => { addToastFn = null; };
  }, []);

  if (items.length === 0) return null;

  return (
    <div class="toast-container">
      {items.map(item => (
        <div key={item.id} class={`toast toast-${item.type}`}>
          <span class="toast-icon">
            {item.type === 'success' ? '✓' : item.type === 'error' ? '✕' : 'ℹ'}
          </span>
          <span class="toast-message">{item.message}</span>
          {item.action && (
            <button
              class="toast-action"
              onClick={() => {
                item.action!.onClick();
                setItems(prev => prev.filter(t => t.id !== item.id));
              }}
            >
              {item.action.label}
            </button>
          )}
          <button
            class="toast-close"
            onClick={() => setItems(prev => prev.filter(t => t.id !== item.id))}
          >
            ×
          </button>
        </div>
      ))}
    </div>
  );
}
