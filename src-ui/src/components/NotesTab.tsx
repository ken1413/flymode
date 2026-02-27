import { useState, useEffect, useCallback } from 'preact/hooks';
import { invoke } from '@tauri-apps/api/core';
import type { Note, NoteColor, NoteCategory } from '../App';
import { toast } from './Toast';

interface NoteFormData {
  title: string;
  content: string;
  color: NoteColor;
  category: NoteCategory;
  pinned: boolean;
  tags: string[];
}

const NOTE_COLORS: { value: NoteColor; label: string; hex: string }[] = [
  { value: 'Yellow', label: 'Yellow', hex: '#fef08a' },
  { value: 'Pink', label: 'Pink', hex: '#fbcfe8' },
  { value: 'Blue', label: 'Blue', hex: '#bfdbfe' },
  { value: 'Green', label: 'Green', hex: '#bbf7d0' },
  { value: 'Purple', label: 'Purple', hex: '#e9d5ff' },
  { value: 'Orange', label: 'Orange', hex: '#fed7aa' },
  { value: 'White', label: 'White', hex: '#ffffff' },
  { value: 'Gray', label: 'Gray', hex: '#e5e7eb' },
];

const CATEGORIES: NoteCategory[] = ['General', 'Work', 'Personal', 'Ideas', 'Tasks', 'Important'];

export function NotesTab() {
  const [notes, setNotes] = useState<Note[]>([]);
  const [showModal, setShowModal] = useState(false);
  const [editingNote, setEditingNote] = useState<Note | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedCategory, setSelectedCategory] = useState<NoteCategory | 'All'>('All');
  const [form, setForm] = useState<NoteFormData>({
    title: '',
    content: '',
    color: 'Yellow',
    category: 'General',
    pinned: false,
    tags: [],
  });
  const [tagInput, setTagInput] = useState('');
  const [viewMode, setViewMode] = useState<'grid' | 'list'>('grid');

  const loadNotes = useCallback(async () => {
    try {
      const list = await invoke<Note[]>('list_notes', { includeArchived: false });
      setNotes(list);
    } catch (e) {
      toast.error('Failed to load notes');
    }
  }, []);

  useEffect(() => {
    loadNotes();
  }, [loadNotes]);

  const searchNotes = async () => {
    if (!searchQuery.trim()) {
      loadNotes();
      return;
    }
    try {
      const results = await invoke<Note[]>('search_notes', { query: searchQuery });
      setNotes(results);
    } catch (e) {
      toast.error('Failed to search notes');
    }
  };

  const openAddModal = () => {
    setEditingNote(null);
    setForm({
      title: '',
      content: '',
      color: 'Yellow',
      category: 'General',
      pinned: false,
      tags: [],
    });
    setTagInput('');
    setShowModal(true);
  };

  const openEditModal = (note: Note) => {
    setEditingNote(note);
    setForm({
      title: note.title,
      content: note.content,
      color: note.color,
      category: note.category,
      pinned: note.pinned,
      tags: note.tags,
    });
    setTagInput('');
    setShowModal(true);
  };

  const handleSubmit = async () => {
    if (!form.title.trim()) {
      toast.error('Title is required');
      return;
    }
    if (form.title.length > 200) {
      toast.error('Title must be 200 characters or less');
      return;
    }
    try {
      if (editingNote) {
        const updated: Note = {
          ...editingNote,
          ...form,
          updated_at: new Date().toISOString(),
        };
        await invoke('update_note', { note: updated });
      } else {
        const created = await invoke<Note>('create_note', { title: form.title, content: form.content });
        // Update with full form data (color, category, tags, pinned)
        const updated: Note = {
          ...created,
          color: form.color,
          category: form.category,
          tags: form.tags,
          pinned: form.pinned,
          updated_at: new Date().toISOString(),
        };
        await invoke('update_note', { note: updated });
      }
      await loadNotes();
      setShowModal(false);
    } catch (e) {
      toast.error('Failed to save note');
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm('Delete this note?')) return;
    try {
      await invoke('delete_note', { id });
      await loadNotes();
    } catch (e) {
      toast.error('Failed to delete note');
    }
  };

  const togglePin = async (note: Note) => {
    try {
      const updated = { ...note, pinned: !note.pinned };
      await invoke('update_note', { note: updated });
      await loadNotes();
    } catch (e) {
      toast.error('Failed to toggle pin');
    }
  };

  const addTag = () => {
    const tag = tagInput.trim();
    if (tag && !form.tags.includes(tag)) {
      setForm({ ...form, tags: [...form.tags, tag] });
      setTagInput('');
    }
  };

  const removeTag = (tag: string) => {
    setForm({ ...form, tags: form.tags.filter(t => t !== tag) });
  };

  const getColorHex = (color: NoteColor): string => {
    return NOTE_COLORS.find(c => c.value === color)?.hex || '#fef08a';
  };

  const filteredNotes = notes.filter(note => {
    if (selectedCategory !== 'All' && note.category !== selectedCategory) {
      return false;
    }
    return true;
  });

  const pinnedNotes = filteredNotes.filter(n => n.pinned);
  const unpinnedNotes = filteredNotes.filter(n => !n.pinned);

  const renderNote = (note: Note) => (
    <div
      class={`note-card ${note.pinned ? 'pinned' : ''}`}
      style={{ backgroundColor: getColorHex(note.color) }}
      key={note.id}
    >
      <div class="note-header">
        <span class="note-category">{note.category}</span>
        <div class="note-actions">
          <button class="note-btn" onClick={() => togglePin(note)} title={note.pinned ? 'Unpin' : 'Pin'}>
            {note.pinned ? '📌' : '📍'}
          </button>
          <button class="note-btn" onClick={() => openEditModal(note)}>✏️</button>
          <button class="note-btn delete" onClick={() => handleDelete(note.id)}>🗑️</button>
        </div>
      </div>
      <h3 class="note-title">{note.title || 'Untitled'}</h3>
      <p class="note-content">{note.content}</p>
      {note.tags.length > 0 && (
        <div class="note-tags">
          {note.tags.map(tag => (
            <span class="note-tag" key={tag}>#{tag}</span>
          ))}
        </div>
      )}
      <div class="note-footer">
        <span class="note-date">
          {new Date(note.updated_at).toLocaleDateString()}
        </span>
      </div>
    </div>
  );

  return (
    <div>
      <div class="notes-toolbar">
        <div class="search-box">
          <input
            type="text"
            class="form-control"
            placeholder="Search notes..."
            value={searchQuery}
            onInput={e => setSearchQuery(e.currentTarget.value)}
            onKeyDown={e => e.key === 'Enter' && searchNotes()}
          />
        </div>
        <select
          class="form-control"
          value={selectedCategory}
          onChange={e => setSelectedCategory(e.currentTarget.value as NoteCategory | 'All')}
          style={{ width: 'auto' }}
        >
          <option value="All">All Categories</option>
          {CATEGORIES.map(cat => (
            <option key={cat} value={cat}>{cat}</option>
          ))}
        </select>
        <div class="view-toggle">
          <button
            class={`btn btn-icon ${viewMode === 'grid' ? 'active' : ''}`}
            onClick={() => setViewMode('grid')}
          >
            ▦
          </button>
          <button
            class={`btn btn-icon ${viewMode === 'list' ? 'active' : ''}`}
            onClick={() => setViewMode('list')}
          >
            ≡
          </button>
        </div>
        <button class="btn btn-primary" onClick={openAddModal}>
          + New Note
        </button>
      </div>

      {notes.length === 0 ? (
        <div class="empty-state">
          <p style={{ fontSize: '48px', marginBottom: '16px' }}>📝</p>
          <p>No notes yet. Create your first sticky note!</p>
        </div>
      ) : (
        <div>
          {pinnedNotes.length > 0 && (
            <>
              <h4 style={{ marginBottom: '12px', color: 'var(--text-muted)' }}>📌 Pinned</h4>
              <div class={`notes-${viewMode}`}>
                {pinnedNotes.map(renderNote)}
              </div>
            </>
          )}
          {unpinnedNotes.length > 0 && (
            <>
              {pinnedNotes.length > 0 && <h4 style={{ marginTop: '20px', marginBottom: '12px', color: 'var(--text-muted)' }}>Notes</h4>}
              <div class={`notes-${viewMode}`}>
                {unpinnedNotes.map(renderNote)}
              </div>
            </>
          )}
        </div>
      )}

      {showModal && (
        <div class="modal-overlay" onClick={() => setShowModal(false)}>
          <div class="modal" onClick={e => e.stopPropagation()} style={{ maxWidth: '560px' }}>
            <div class="modal-header">
              <span class="modal-title">{editingNote ? 'Edit Note' : 'New Note'}</span>
              <button class="modal-close" onClick={() => setShowModal(false)}>×</button>
            </div>

            <div class="form-group">
              <label>Title</label>
              <input
                type="text"
                class="form-control"
                value={form.title}
                onInput={e => setForm({ ...form, title: e.currentTarget.value })}
                placeholder="Note title..."
              />
            </div>

            <div class="form-group">
              <label>Content</label>
              <textarea
                class="form-control"
                value={form.content}
                onInput={e => setForm({ ...form, content: e.currentTarget.value })}
                placeholder="Write your note..."
                rows={6}
              />
            </div>

            <div class="form-row">
              <div class="form-group">
                <label>Color</label>
                <div class="color-picker">
                  {NOTE_COLORS.map(c => (
                    <button
                      key={c.value}
                      class={`color-btn ${form.color === c.value ? 'selected' : ''}`}
                      style={{ backgroundColor: c.hex }}
                      onClick={() => setForm({ ...form, color: c.value })}
                      title={c.label}
                    />
                  ))}
                </div>
              </div>
              <div class="form-group">
                <label>Category</label>
                <select
                  class="form-control"
                  value={form.category}
                  onChange={e => setForm({ ...form, category: e.currentTarget.value as NoteCategory })}
                >
                  {CATEGORIES.map(cat => (
                    <option key={cat} value={cat}>{cat}</option>
                  ))}
                </select>
              </div>
            </div>

            <div class="form-group">
              <label>Tags</label>
              <div class="tags-input">
                {form.tags.map(tag => (
                  <span class="tag-chip" key={tag}>
                    #{tag}
                    <button onClick={() => removeTag(tag)}>×</button>
                  </span>
                ))}
                <input
                  type="text"
                  class="form-control"
                  value={tagInput}
                  onInput={e => setTagInput(e.currentTarget.value)}
                  onKeyDown={e => {
                    if (e.key === 'Enter') {
                      e.preventDefault();
                      addTag();
                    }
                  }}
                  placeholder="Add tag..."
                />
              </div>
            </div>

            <div class="form-group">
              <label>
                <input
                  type="checkbox"
                  checked={form.pinned}
                  onChange={e => setForm({ ...form, pinned: e.currentTarget.checked })}
                />
                {' '}Pin this note
              </label>
            </div>

            <div class="modal-actions">
              <button class="btn btn-icon" onClick={() => setShowModal(false)}>Cancel</button>
              <button class="btn btn-primary" onClick={handleSubmit}>
                {editingNote ? 'Save Changes' : 'Create Note'}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
