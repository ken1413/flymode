import { useState, useEffect } from 'preact/hooks';

export type ToastType = 'success' | 'error' | 'info';

interface ToastItem {
  id: number;
  type: ToastType;
  message: string;
}

let nextId = 0;
let addToastFn: ((type: ToastType, message: string) => void) | null = null;

/** Global toast function — call from anywhere after ToastContainer is mounted. */
export const toast = {
  success: (msg: string) => addToastFn?.('success', msg),
  error: (msg: string) => addToastFn?.('error', msg),
  info: (msg: string) => addToastFn?.('info', msg),
};

const DURATION_MS = 4000;

export function ToastContainer() {
  const [items, setItems] = useState<ToastItem[]>([]);

  useEffect(() => {
    addToastFn = (type: ToastType, message: string) => {
      const id = ++nextId;
      setItems(prev => [...prev, { id, type, message }]);
      setTimeout(() => {
        setItems(prev => prev.filter(t => t.id !== id));
      }, DURATION_MS);
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
