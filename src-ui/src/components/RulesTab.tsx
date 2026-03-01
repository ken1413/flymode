import { useState } from 'preact/hooks';
import { invoke } from '@tauri-apps/api/core';
import type { AppConfig, ScheduleRule, ActionType, TargetType } from '../App';
import { toast } from './Toast';

interface RuleForm {
  name: string;
  action: ActionType;
  target: TargetType;
  start_time: string;
  end_time: string;
  days: number[];
  command: string;
}

const DAYS = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'];

export function RulesTab({ config, onSave }: { config: AppConfig; onSave: (c: AppConfig) => void }) {
  const [showModal, setShowModal] = useState(false);
  const [editingRule, setEditingRule] = useState<ScheduleRule | null>(null);
  const [form, setForm] = useState<RuleForm>({
    name: '',
    action: 'Disable',
    target: 'Wifi',
    start_time: '22:00',
    end_time: '',
    days: [0, 1, 2, 3, 4, 5, 6],
    command: '',
  });

  const openAddModal = () => {
    setEditingRule(null);
    setForm({
      name: '',
      action: 'Disable',
      target: 'Wifi',
      start_time: '22:00',
      end_time: '',
      days: [0, 1, 2, 3, 4, 5, 6],
      command: '',
    });
    setShowModal(true);
  };

  const openEditModal = (rule: ScheduleRule) => {
    setEditingRule(rule);
    setForm({
      name: rule.name,
      action: rule.action,
      target: rule.target,
      start_time: rule.start_time,
      end_time: rule.end_time || '',
      days: rule.days,
      command: rule.command || '',
    });
    setShowModal(true);
  };

  const toggleDay = (day: number) => {
    setForm(prev => ({
      ...prev,
      days: prev.days.includes(day)
        ? prev.days.filter(d => d !== day)
        : [...prev.days, day].sort(),
    }));
  };

  const handleSubmit = async () => {
    if (!form.name.trim()) {
      toast.error('Rule name is required');
      return;
    }
    if (!form.start_time) {
      toast.error('Start time is required');
      return;
    }
    if (form.days.length === 0) {
      toast.error('Select at least one day');
      return;
    }
    if (form.target === 'CustomCommand' && !form.command?.trim()) {
      toast.error('Command is required for custom command rules');
      return;
    }
    const rule: ScheduleRule = {
      id: editingRule?.id || '',
      name: form.name,
      enabled: editingRule?.enabled ?? true,
      action: form.action,
      target: form.target,
      start_time: form.start_time,
      end_time: form.end_time || null,
      days: form.days,
      command: form.target === 'CustomCommand' ? form.command : null,
    };

    try {
      if (editingRule) {
        await invoke('update_rule', { rule });
      } else {
        await invoke('add_rule', { rule });
      }
      if (editingRule) {
        onSave({ ...config, rules: config.rules.map(r => r.id === rule.id ? rule : r) });
      } else {
        const freshConfig = await invoke<AppConfig>('get_config');
        const added = freshConfig.rules[freshConfig.rules.length - 1];
        onSave({ ...config, rules: [...config.rules, added] });
      }
      setShowModal(false);
    } catch (e) {
      toast.error('Failed to save rule');
    }
  };

  const handleToggle = async (ruleId: string) => {
    await invoke('toggle_rule', { ruleId });
    onSave({
      ...config,
      rules: config.rules.map(r => r.id === ruleId ? { ...r, enabled: !r.enabled } : r),
    });
  };

  const handleDelete = async (ruleId: string) => {
    await invoke('delete_rule', { ruleId });
    onSave({ ...config, rules: config.rules.filter(r => r.id !== ruleId) });
  };

  const handleExecuteNow = async (rule: ScheduleRule) => {
    try {
      await invoke('execute_rule_now', { rule });
    } catch (e) {
      toast.error('Failed to execute action');
    }
  };

  return (
    <>
      <div class="card">
        <div class="card-header">
          <span class="card-title">Scheduled Rules</span>
          <button class="btn btn-primary btn-sm" onClick={openAddModal}>
            + Add Rule
          </button>
        </div>

        {config.rules.length === 0 ? (
          <div class="empty-state">
            <p>No rules configured yet.</p>
            <p style={{ marginTop: '8px', fontSize: '12px' }}>
              Add a rule to automatically toggle wireless settings.
            </p>
          </div>
        ) : (
          config.rules.map(rule => (
            <div class="rule-item" key={rule.id}>
              <div
                class={`toggle ${rule.enabled ? 'on' : ''}`}
                onClick={() => handleToggle(rule.id)}
              />
              <div class="rule-info">
                <div class="rule-name">{rule.name}</div>
                <div class="rule-details">
                  {rule.action} {rule.target}
                  {' • '}
                  {rule.start_time}
                  {rule.end_time ? ` - ${rule.end_time}` : ''}
                  {' • '}
                  {rule.days.map(d => DAYS[d]).join(', ')}
                </div>
              </div>
              <button class="btn btn-icon btn-sm" onClick={() => handleExecuteNow(rule)}>
                Run
              </button>
              <button class="btn btn-icon btn-sm" onClick={() => openEditModal(rule)}>
                Edit
              </button>
              <button class="btn btn-danger btn-sm" onClick={() => handleDelete(rule.id)}>
                Delete
              </button>
            </div>
          ))
        )}
      </div>

      {showModal && (
        <div class="modal-overlay" onClick={() => setShowModal(false)}>
          <div class="modal" onClick={e => e.stopPropagation()}>
            <div class="modal-header">
              <span class="modal-title">{editingRule ? 'Edit Rule' : 'Add New Rule'}</span>
              <button class="modal-close" onClick={() => setShowModal(false)}>×</button>
            </div>

            <div class="form-group">
              <label>Rule Name</label>
              <input
                type="text"
                class="form-control"
                value={form.name}
                onInput={e => setForm({ ...form, name: e.currentTarget.value })}
                placeholder="e.g., Turn off WiFi at night"
              />
            </div>

            <div class="form-row">
              <div class="form-group">
                <label>Action</label>
                <select
                  class="form-control"
                  value={form.action}
                  onChange={e => setForm({ ...form, action: e.currentTarget.value as ActionType })}
                >
                  <option value="Enable">Enable</option>
                  <option value="Disable">Disable</option>
                  <option value="Toggle">Toggle</option>
                  <option value="RunCommand">Run Command</option>
                </select>
              </div>
              <div class="form-group">
                <label>Target</label>
                <select
                  class="form-control"
                  value={form.target}
                  onChange={e => setForm({ ...form, target: e.currentTarget.value as TargetType })}
                >
                  <option value="Wifi">WiFi</option>
                  <option value="Bluetooth">Bluetooth</option>
                  <option value="AirplaneMode">Airplane Mode</option>
                  <option value="CustomCommand">Custom Command</option>
                </select>
              </div>
            </div>

            {form.target === 'CustomCommand' && (
              <div class="form-group">
                <label>Command</label>
                <input
                  type="text"
                  class="form-control"
                  value={form.command}
                  onInput={e => setForm({ ...form, command: e.currentTarget.value })}
                  placeholder="e.g., systemctl suspend"
                />
              </div>
            )}

            <div class="form-row">
              <div class="form-group">
                <label>Start Time</label>
                <input
                  type="time"
                  class="form-control"
                  value={form.start_time}
                  onInput={e => setForm({ ...form, start_time: e.currentTarget.value })}
                />
              </div>
              <div class="form-group">
                <label>End Time (optional)</label>
                <input
                  type="time"
                  class="form-control"
                  value={form.end_time}
                  onInput={e => setForm({ ...form, end_time: e.currentTarget.value })}
                />
              </div>
            </div>

            <div class="form-group">
              <label>Days</label>
              <div class="days-selector">
                {DAYS.map((day, i) => (
                  <button
                    key={day}
                    class={`day-btn ${form.days.includes(i) ? 'selected' : ''}`}
                    onClick={() => toggleDay(i)}
                  >
                    {day}
                  </button>
                ))}
              </div>
            </div>

            <div class="modal-actions">
              <button class="btn btn-icon" onClick={() => setShowModal(false)}>Cancel</button>
              <button class="btn btn-primary" onClick={handleSubmit}>
                {editingRule ? 'Save Changes' : 'Add Rule'}
              </button>
            </div>
          </div>
        </div>
      )}
    </>
  );
}
