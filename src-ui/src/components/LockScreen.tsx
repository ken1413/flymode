import { useState, useRef, useEffect } from 'preact/hooks';
import { invoke } from '@tauri-apps/api/core';

export function LockScreen({ onUnlock }: { onUnlock: () => void }) {
  const [password, setPassword] = useState('');
  const [error, setError] = useState('');
  const [verifying, setVerifying] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  const handleSubmit = async () => {
    if (!password || verifying) return;

    setVerifying(true);
    setError('');

    try {
      const ok = await invoke<boolean>('verify_system_password', { password });
      if (ok) {
        onUnlock();
      } else {
        setError('Password incorrect');
        setPassword('');
        inputRef.current?.focus();
      }
    } catch (e) {
      setError('Verification failed: ' + e);
    } finally {
      setVerifying(false);
    }
  };

  return (
    <div class="lock-screen">
      <div class="lock-card">
        <div class="lock-icon">🔒</div>
        <h2 class="lock-title">FlyMode Locked</h2>
        <p class="lock-subtitle">Enter your system password to unlock</p>

        <div class="form-group" style={{ marginTop: '20px' }}>
          <input
            ref={inputRef}
            type="password"
            class="form-control"
            value={password}
            placeholder="System password"
            onInput={e => {
              setPassword(e.currentTarget.value);
              setError('');
            }}
            onKeyDown={e => e.key === 'Enter' && handleSubmit()}
            disabled={verifying}
          />
        </div>

        {error && (
          <p class="lock-error">{error}</p>
        )}

        <button
          class="btn btn-primary"
          style={{ width: '100%', marginTop: '12px' }}
          onClick={handleSubmit}
          disabled={!password || verifying}
        >
          {verifying ? 'Verifying...' : 'Unlock'}
        </button>
      </div>
    </div>
  );
}
