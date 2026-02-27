import { useState } from 'preact/hooks';
import { invoke } from '@tauri-apps/api/core';

export function QuickActionsTab() {
  const [running, setRunning] = useState<string | null>(null);
  const [command, setCommand] = useState('');
  const [output, setOutput] = useState('');

  const executeToggle = async (action: string, enable: boolean) => {
    setRunning(action);
    setOutput('');
    try {
      const result = await invoke<string>(action, { enable });
      setOutput(result || 'Success');
    } catch (e) {
      setOutput(`Error: ${e}`);
    } finally {
      setRunning(null);
    }
  };

  const runCustomCommand = async () => {
    if (!command.trim()) return;
    setRunning('custom');
    setOutput('');
    try {
      const result = await invoke<string>('run_custom_command', { command });
      setOutput(result || 'Success');
    } catch (e) {
      setOutput(`Error: ${e}`);
    } finally {
      setRunning(null);
    }
  };

  return (
    <div>
      <div class="card">
        <div class="card-header">
          <span class="card-title">WiFi</span>
        </div>
        <div class="quick-actions">
          <div class="quick-action" onClick={() => executeToggle('toggle_wifi', true)}>
            <div class="quick-action-icon">📶</div>
            <div class="quick-action-label">Enable WiFi</div>
          </div>
          <div class="quick-action" onClick={() => executeToggle('toggle_wifi', false)}>
            <div class="quick-action-icon">📵</div>
            <div class="quick-action-label">Disable WiFi</div>
          </div>
        </div>
      </div>

      <div class="card">
        <div class="card-header">
          <span class="card-title">Bluetooth</span>
        </div>
        <div class="quick-actions">
          <div class="quick-action" onClick={() => executeToggle('toggle_bluetooth', true)}>
            <div class="quick-action-icon">🔵</div>
            <div class="quick-action-label">Enable Bluetooth</div>
          </div>
          <div class="quick-action" onClick={() => executeToggle('toggle_bluetooth', false)}>
            <div class="quick-action-icon">⬛</div>
            <div class="quick-action-label">Disable Bluetooth</div>
          </div>
        </div>
      </div>

      <div class="card">
        <div class="card-header">
          <span class="card-title">Airplane Mode</span>
        </div>
        <div class="quick-actions">
          <div class="quick-action" onClick={() => executeToggle('toggle_airplane_mode', true)}>
            <div class="quick-action-icon">✈️</div>
            <div class="quick-action-label">Enable Airplane</div>
          </div>
          <div class="quick-action" onClick={() => executeToggle('toggle_airplane_mode', false)}>
            <div class="quick-action-icon">🌐</div>
            <div class="quick-action-label">Disable Airplane</div>
          </div>
        </div>
      </div>

      <div class="card">
        <div class="card-header">
          <span class="card-title">Custom Command</span>
        </div>
        <div class="form-group" style={{ marginBottom: '12px' }}>
          <input
            type="text"
            class="form-control"
            value={command}
            onInput={e => setCommand(e.currentTarget.value)}
            placeholder="Enter command to execute..."
            onKeyDown={e => e.key === 'Enter' && runCustomCommand()}
          />
        </div>
        <button
          class="btn btn-primary"
          onClick={runCustomCommand}
          disabled={!command.trim() || running !== null}
        >
          {running === 'custom' ? 'Running...' : 'Execute'}
        </button>
      </div>

      {output && (
        <div class="card">
          <div class="card-header">
            <span class="card-title">Output</span>
          </div>
          <pre style={{ 
            background: 'var(--bg)', 
            padding: '12px', 
            borderRadius: '6px',
            overflow: 'auto',
            fontSize: '12px',
            whiteSpace: 'pre-wrap'
          }}>
            {output}
          </pre>
        </div>
      )}
    </div>
  );
}
