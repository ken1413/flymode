import { useState, useEffect } from 'preact/hooks';
import { invoke } from '@tauri-apps/api/core';
import type { AppConfig } from '../App';

export function SettingsTab({ config, onSave }: { config: AppConfig; onSave: (c: AppConfig) => void }) {
  const [buildInfo, setBuildInfo] = useState<{ version: string; git_hash: string } | null>(null);

  useEffect(() => {
    invoke<Record<string, string>>('get_build_info').then(info => {
      setBuildInfo({ version: info.version, git_hash: info.git_hash });
    });
  }, []);

  const updateSetting = <K extends keyof AppConfig>(key: K, value: AppConfig[K]) => {
    onSave({ ...config, [key]: value });
  };

  return (
    <div>
      <div class="card">
        <div class="card-header">
          <span class="card-title">General Settings</span>
        </div>

        <div class="rule-item">
          <div class="rule-info">
            <div class="rule-name">Show Notifications</div>
            <div class="rule-details">Display notifications when rules are executed</div>
          </div>
          <div
            class={`toggle ${config.show_notifications ? 'on' : ''}`}
            onClick={() => updateSetting('show_notifications', !config.show_notifications)}
          />
        </div>

        <div class="rule-item">
          <div class="rule-info">
            <div class="rule-name">Minimize to System Tray</div>
            <div class="rule-details">Keep running in background when window is closed</div>
          </div>
          <div
            class={`toggle ${config.minimize_to_tray ? 'on' : ''}`}
            onClick={() => updateSetting('minimize_to_tray', !config.minimize_to_tray)}
          />
        </div>

        <div class="rule-item">
          <div class="rule-info">
            <div class="rule-name">Launch at Startup</div>
            <div class="rule-details">Automatically start FlyMode when you log in</div>
          </div>
          <div
            class={`toggle ${config.auto_start ? 'on' : ''}`}
            onClick={() => updateSetting('auto_start', !config.auto_start)}
          />
        </div>

        <div class="rule-item">
          <div class="rule-info">
            <div class="rule-name">Require Password</div>
            <div class="rule-details">Ask for system login password when opening the app</div>
          </div>
          <div
            class={`toggle ${config.require_password ? 'on' : ''}`}
            onClick={() => updateSetting('require_password', !config.require_password)}
          />
        </div>
      </div>

      <div class="card">
        <div class="card-header">
          <span class="card-title">Schedule Settings</span>
        </div>

        <div class="form-group">
          <label>Check Interval (seconds)</label>
          <input
            type="number"
            class="form-control"
            value={config.check_interval_seconds}
            min={10}
            max={3600}
            onInput={e => {
              const val = parseInt(e.currentTarget.value, 10);
              if (val >= 10 && val <= 3600) {
                updateSetting('check_interval_seconds', val);
              }
            }}
          />
          <p style={{ marginTop: '8px', fontSize: '12px', color: 'var(--text-muted)' }}>
            How often to check if a rule should be executed. Lower values are more precise but use more resources.
          </p>
        </div>
      </div>

      <div class="card">
        <div class="card-header">
          <span class="card-title">About</span>
        </div>
        <p style={{ color: 'var(--text-muted)', fontSize: '14px' }}>
          FlyMode v{buildInfo?.version || '...'} ({buildInfo?.git_hash || '...'})
        </p>
        <p style={{ color: 'var(--text-muted)', fontSize: '12px', marginTop: '8px' }}>
          A desktop application to automatically control wireless settings (WiFi, Bluetooth, Airplane Mode) 
          or run custom commands on a schedule.
        </p>
      </div>
    </div>
  );
}
