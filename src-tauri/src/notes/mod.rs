use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NotesError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("Parse error: {0}")]
    Parse(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NoteColor {
    Yellow,
    Pink,
    Blue,
    Green,
    Purple,
    Orange,
    White,
    Gray,
}

impl Default for NoteColor {
    fn default() -> Self {
        Self::Yellow
    }
}

impl NoteColor {
    pub fn hex(&self) -> &'static str {
        match self {
            NoteColor::Yellow => "#fef08a",
            NoteColor::Pink => "#fbcfe8",
            NoteColor::Blue => "#bfdbfe",
            NoteColor::Green => "#bbf7d0",
            NoteColor::Purple => "#e9d5ff",
            NoteColor::Orange => "#fed7aa",
            NoteColor::White => "#ffffff",
            NoteColor::Gray => "#e5e7eb",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NoteCategory {
    General,
    Work,
    Personal,
    Ideas,
    Tasks,
    Important,
    Archive,
}

impl Default for NoteCategory {
    fn default() -> Self {
        Self::General
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub title: String,
    pub content: String,
    pub color: NoteColor,
    pub category: NoteCategory,
    pub pinned: bool,
    pub archived: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<String>,
    pub position_x: i32,
    pub position_y: i32,
    pub width: i32,
    pub height: i32,
    pub device_id: String,
    pub sync_hash: Option<String>,
    pub deleted: bool,
}

impl Note {
    pub fn new(title: String, content: String, device_id: String) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title,
            content,
            color: NoteColor::default(),
            category: NoteCategory::default(),
            pinned: false,
            archived: false,
            created_at: now,
            updated_at: now,
            tags: Vec::new(),
            position_x: 0,
            position_y: 0,
            width: 280,
            height: 200,
            device_id,
            sync_hash: None,
            deleted: false,
        }
    }

    pub fn compute_hash(&self) -> String {
        use sha2::{Digest, Sha256};
        let data = format!(
            "{}{}{}{:?}{:?}{}{}",
            self.id,
            self.title,
            self.content,
            self.color,
            self.category,
            self.pinned,
            self.updated_at.timestamp()
        );
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
        self.sync_hash = Some(self.compute_hash());
    }
}

#[derive(Debug, Clone)]
pub struct NotesStore {
    db_path: PathBuf,
    device_id: String,
}

impl NotesStore {
    pub fn new(device_id: String) -> Result<Self, NotesError> {
        let data_dir = Self::data_dir();
        if !data_dir.exists() {
            fs::create_dir_all(&data_dir)?;
        }

        let db_path = data_dir.join("notes.db");
        let store = Self { db_path, device_id };
        store.init_db()?;
        Ok(store)
    }

    pub fn with_path(db_path: PathBuf, device_id: String) -> Result<Self, NotesError> {
        if let Some(parent) = db_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }
        let store = Self { db_path, device_id };
        store.init_db()?;
        Ok(store)
    }

    fn data_dir() -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("flymode")
    }

    fn init_db(&self) -> Result<(), NotesError> {
        let conn = rusqlite::Connection::open(&self.db_path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS notes (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                color TEXT NOT NULL,
                category TEXT NOT NULL,
                pinned INTEGER NOT NULL DEFAULT 0,
                archived INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                tags TEXT NOT NULL DEFAULT '[]',
                position_x INTEGER NOT NULL DEFAULT 0,
                position_y INTEGER NOT NULL DEFAULT 0,
                width INTEGER NOT NULL DEFAULT 280,
                height INTEGER NOT NULL DEFAULT 200,
                device_id TEXT NOT NULL,
                sync_hash TEXT,
                deleted INTEGER NOT NULL DEFAULT 0
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_notes_updated ON notes(updated_at)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_notes_category ON notes(category)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_notes_deleted ON notes(deleted)",
            [],
        )?;

        Ok(())
    }

    fn get_conn(&self) -> Result<rusqlite::Connection, NotesError> {
        Ok(rusqlite::Connection::open(&self.db_path)?)
    }

    pub fn create(&self, title: String, content: String) -> Result<Note, NotesError> {
        let mut note = Note::new(title, content, self.device_id.clone());
        note.sync_hash = Some(note.compute_hash());

        let conn = self.get_conn()?;
        conn.execute(
            "INSERT INTO notes (
                id, title, content, color, category, pinned, archived,
                created_at, updated_at, tags, position_x, position_y,
                width, height, device_id, sync_hash, deleted
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
            rusqlite::params![
                note.id,
                note.title,
                note.content,
                serde_json::to_string(&note.color)?,
                serde_json::to_string(&note.category)?,
                note.pinned as i32,
                note.archived as i32,
                note.created_at.to_rfc3339(),
                note.updated_at.to_rfc3339(),
                serde_json::to_string(&note.tags)?,
                note.position_x,
                note.position_y,
                note.width,
                note.height,
                note.device_id,
                note.sync_hash,
                note.deleted as i32,
            ],
        )?;

        Ok(note)
    }

    pub fn update(&self, note: &Note) -> Result<(), NotesError> {
        // Always recompute sync_hash so the sync engine can detect changes
        let fresh_hash = note.compute_hash();

        let conn = self.get_conn()?;
        conn.execute(
            "UPDATE notes SET
                title = ?1, content = ?2, color = ?3, category = ?4,
                pinned = ?5, archived = ?6, updated_at = ?7, tags = ?8,
                position_x = ?9, position_y = ?10, width = ?11, height = ?12,
                sync_hash = ?13, deleted = ?14
            WHERE id = ?15",
            rusqlite::params![
                note.title,
                note.content,
                serde_json::to_string(&note.color)?,
                serde_json::to_string(&note.category)?,
                note.pinned as i32,
                note.archived as i32,
                note.updated_at.to_rfc3339(),
                serde_json::to_string(&note.tags)?,
                note.position_x,
                note.position_y,
                note.width,
                note.height,
                fresh_hash,
                note.deleted as i32,
                note.id,
            ],
        )?;

        Ok(())
    }

    pub fn delete(&self, id: &str) -> Result<(), NotesError> {
        let conn = self.get_conn()?;
        conn.execute(
            "UPDATE notes SET deleted = 1, updated_at = ?1 WHERE id = ?2",
            rusqlite::params![Utc::now().to_rfc3339(), id],
        )?;
        Ok(())
    }

    pub fn get(&self, id: &str) -> Result<Option<Note>, NotesError> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, title, content, color, category, pinned, archived,
                    created_at, updated_at, tags, position_x, position_y,
                    width, height, device_id, sync_hash, deleted
             FROM notes WHERE id = ?1 AND deleted = 0",
        )?;

        let note = stmt
            .query_row(rusqlite::params![id], |row| {
                Ok(Note {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    content: row.get(2)?,
                    color: serde_json::from_str(&row.get::<_, String>(3)?).unwrap_or_default(),
                    category: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or_default(),
                    pinned: row.get::<_, i32>(5)? != 0,
                    archived: row.get::<_, i32>(6)? != 0,
                    created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                        .map(|d| d.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                        .map(|d| d.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    tags: serde_json::from_str(&row.get::<_, String>(9)?).unwrap_or_default(),
                    position_x: row.get(10)?,
                    position_y: row.get(11)?,
                    width: row.get(12)?,
                    height: row.get(13)?,
                    device_id: row.get(14)?,
                    sync_hash: row.get(15)?,
                    deleted: row.get::<_, i32>(16)? != 0,
                })
            })
            .ok();

        Ok(note)
    }

    pub fn list(&self, include_archived: bool) -> Result<Vec<Note>, NotesError> {
        let conn = self.get_conn()?;
        let sql = if include_archived {
            "SELECT id, title, content, color, category, pinned, archived,
                    created_at, updated_at, tags, position_x, position_y,
                    width, height, device_id, sync_hash, deleted
             FROM notes WHERE deleted = 0 ORDER BY pinned DESC, updated_at DESC"
        } else {
            "SELECT id, title, content, color, category, pinned, archived,
                    created_at, updated_at, tags, position_x, position_y,
                    width, height, device_id, sync_hash, deleted
             FROM notes WHERE deleted = 0 AND archived = 0 ORDER BY pinned DESC, updated_at DESC"
        };

        let mut stmt = conn.prepare(sql)?;
        let notes = stmt
            .query_map([], |row| {
                Ok(Note {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    content: row.get(2)?,
                    color: serde_json::from_str(&row.get::<_, String>(3)?).unwrap_or_default(),
                    category: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or_default(),
                    pinned: row.get::<_, i32>(5)? != 0,
                    archived: row.get::<_, i32>(6)? != 0,
                    created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                        .map(|d| d.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                        .map(|d| d.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    tags: serde_json::from_str(&row.get::<_, String>(9)?).unwrap_or_default(),
                    position_x: row.get(10)?,
                    position_y: row.get(11)?,
                    width: row.get(12)?,
                    height: row.get(13)?,
                    device_id: row.get(14)?,
                    sync_hash: row.get(15)?,
                    deleted: row.get::<_, i32>(16)? != 0,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(notes)
    }

    pub fn search(&self, query: &str) -> Result<Vec<Note>, NotesError> {
        let conn = self.get_conn()?;
        let pattern = format!("%{}%", query);

        let mut stmt = conn.prepare(
            "SELECT id, title, content, color, category, pinned, archived,
                    created_at, updated_at, tags, position_x, position_y,
                    width, height, device_id, sync_hash, deleted
             FROM notes WHERE deleted = 0 AND (title LIKE ?1 OR content LIKE ?1)
             ORDER BY pinned DESC, updated_at DESC",
        )?;

        let notes = stmt
            .query_map(rusqlite::params![pattern], |row| {
                Ok(Note {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    content: row.get(2)?,
                    color: serde_json::from_str(&row.get::<_, String>(3)?).unwrap_or_default(),
                    category: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or_default(),
                    pinned: row.get::<_, i32>(5)? != 0,
                    archived: row.get::<_, i32>(6)? != 0,
                    created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                        .map(|d| d.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                        .map(|d| d.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    tags: serde_json::from_str(&row.get::<_, String>(9)?).unwrap_or_default(),
                    position_x: row.get(10)?,
                    position_y: row.get(11)?,
                    width: row.get(12)?,
                    height: row.get(13)?,
                    device_id: row.get(14)?,
                    sync_hash: row.get(15)?,
                    deleted: row.get::<_, i32>(16)? != 0,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(notes)
    }

    pub fn get_changes_since(&self, since: DateTime<Utc>) -> Result<Vec<Note>, NotesError> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, title, content, color, category, pinned, archived,
                    created_at, updated_at, tags, position_x, position_y,
                    width, height, device_id, sync_hash, deleted
             FROM notes WHERE updated_at > ?1
             ORDER BY updated_at ASC",
        )?;

        let notes = stmt
            .query_map(rusqlite::params![since.to_rfc3339()], |row| {
                Ok(Note {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    content: row.get(2)?,
                    color: serde_json::from_str(&row.get::<_, String>(3)?).unwrap_or_default(),
                    category: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or_default(),
                    pinned: row.get::<_, i32>(5)? != 0,
                    archived: row.get::<_, i32>(6)? != 0,
                    created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                        .map(|d| d.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                        .map(|d| d.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    tags: serde_json::from_str(&row.get::<_, String>(9)?).unwrap_or_default(),
                    position_x: row.get(10)?,
                    position_y: row.get(11)?,
                    width: row.get(12)?,
                    height: row.get(13)?,
                    device_id: row.get(14)?,
                    sync_hash: row.get(15)?,
                    deleted: row.get::<_, i32>(16)? != 0,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(notes)
    }

    pub fn apply_remote_changes(&self, notes: Vec<Note>) -> Result<usize, NotesError> {
        let conn = self.get_conn()?;
        let mut applied = 0;

        for note in notes {
            let existing: Option<(String, String)> = conn
                .query_row(
                    "SELECT sync_hash, updated_at FROM notes WHERE id = ?1",
                    rusqlite::params![note.id],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .ok();

            match existing {
                None => {
                    self.insert_note(&conn, &note)?;
                    applied += 1;
                }
                Some((hash, local_updated_str)) => {
                    let remote_hash = note.sync_hash.clone().unwrap_or_default();
                    if hash == remote_hash {
                        continue; // identical — nothing to do
                    }
                    // LWW: only apply if remote is strictly newer
                    let local_updated = DateTime::parse_from_rfc3339(&local_updated_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now());
                    if note.updated_at > local_updated {
                        self.insert_or_replace_note(&conn, &note)?;
                        applied += 1;
                    }
                }
            }
        }

        Ok(applied)
    }

    fn insert_note(&self, conn: &rusqlite::Connection, note: &Note) -> Result<(), NotesError> {
        conn.execute(
            "INSERT INTO notes (
                id, title, content, color, category, pinned, archived,
                created_at, updated_at, tags, position_x, position_y,
                width, height, device_id, sync_hash, deleted
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
            rusqlite::params![
                note.id,
                note.title,
                note.content,
                serde_json::to_string(&note.color)?,
                serde_json::to_string(&note.category)?,
                note.pinned as i32,
                note.archived as i32,
                note.created_at.to_rfc3339(),
                note.updated_at.to_rfc3339(),
                serde_json::to_string(&note.tags)?,
                note.position_x,
                note.position_y,
                note.width,
                note.height,
                note.device_id,
                note.sync_hash,
                note.deleted as i32,
            ],
        )?;
        Ok(())
    }

    fn insert_or_replace_note(
        &self,
        conn: &rusqlite::Connection,
        note: &Note,
    ) -> Result<(), NotesError> {
        conn.execute(
            "INSERT OR REPLACE INTO notes (
                id, title, content, color, category, pinned, archived,
                created_at, updated_at, tags, position_x, position_y,
                width, height, device_id, sync_hash, deleted
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
            rusqlite::params![
                note.id,
                note.title,
                note.content,
                serde_json::to_string(&note.color)?,
                serde_json::to_string(&note.category)?,
                note.pinned as i32,
                note.archived as i32,
                note.created_at.to_rfc3339(),
                note.updated_at.to_rfc3339(),
                serde_json::to_string(&note.tags)?,
                note.position_x,
                note.position_y,
                note.width,
                note.height,
                note.device_id,
                note.sync_hash,
                note.deleted as i32,
            ],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use tempfile::TempDir;

    struct TestFixture {
        _temp_dir: TempDir,
        store: NotesStore,
    }

    impl TestFixture {
        fn new() -> Self {
            let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
            let db_path = temp_dir.path().join("notes.db");
            let store = NotesStore::with_path(db_path, "test-device".to_string())
                .expect("Failed to create store");
            Self {
                _temp_dir: temp_dir,
                store,
            }
        }
    }

    #[test]
    fn test_note_creation() {
        let note = Note::new(
            "Test Title".to_string(),
            "Test Content".to_string(),
            "device-123".to_string(),
        );

        assert!(!note.id.is_empty());
        assert_eq!(note.title, "Test Title");
        assert_eq!(note.content, "Test Content");
        assert_eq!(note.device_id, "device-123");
        assert_eq!(note.color, NoteColor::Yellow);
        assert_eq!(note.category, NoteCategory::General);
        assert!(!note.pinned);
        assert!(!note.archived);
        assert!(!note.deleted);
        assert!(note.tags.is_empty());
    }

    #[test]
    fn test_note_hash_computation() {
        let mut note1 = Note::new(
            "Title".to_string(),
            "Content".to_string(),
            "dev".to_string(),
        );
        let note2 = note1.clone();

        let hash1 = note1.compute_hash();
        let hash2 = note2.compute_hash();

        assert_eq!(hash1, hash2);

        note1.title = "Modified Title".to_string();
        let hash3 = note1.compute_hash();

        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_note_touch() {
        let mut note = Note::new(
            "Title".to_string(),
            "Content".to_string(),
            "dev".to_string(),
        );
        let original_updated = note.updated_at;

        std::thread::sleep(std::time::Duration::from_millis(10));
        note.touch();

        assert!(note.updated_at > original_updated);
        assert!(note.sync_hash.is_some());
    }

    #[test]
    fn test_note_color_hex() {
        assert_eq!(NoteColor::Yellow.hex(), "#fef08a");
        assert_eq!(NoteColor::Pink.hex(), "#fbcfe8");
        assert_eq!(NoteColor::Blue.hex(), "#bfdbfe");
        assert_eq!(NoteColor::Green.hex(), "#bbf7d0");
        assert_eq!(NoteColor::Purple.hex(), "#e9d5ff");
        assert_eq!(NoteColor::Orange.hex(), "#fed7aa");
        assert_eq!(NoteColor::White.hex(), "#ffffff");
        assert_eq!(NoteColor::Gray.hex(), "#e5e7eb");
    }

    #[test]
    fn test_note_serialization() {
        let note = Note::new(
            "Title".to_string(),
            "Content".to_string(),
            "dev".to_string(),
        );

        let json = serde_json::to_string(&note).expect("Failed to serialize");
        let deserialized: Note = serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(note.id, deserialized.id);
        assert_eq!(note.title, deserialized.title);
        assert_eq!(note.content, deserialized.content);
    }

    #[test]
    fn test_store_create_note() {
        let fixture = TestFixture::new();

        let note = fixture
            .store
            .create("My Note".to_string(), "Note content".to_string())
            .expect("Failed to create note");

        assert!(!note.id.is_empty());
        assert_eq!(note.title, "My Note");
        assert_eq!(note.content, "Note content");
        assert!(note.sync_hash.is_some());
    }

    #[test]
    fn test_store_get_note() {
        let fixture = TestFixture::new();

        let created = fixture
            .store
            .create("Test".to_string(), "Content".to_string())
            .expect("Failed to create");

        let retrieved = fixture
            .store
            .get(&created.id)
            .expect("Failed to get")
            .expect("Note not found");

        assert_eq!(created.id, retrieved.id);
        assert_eq!(created.title, retrieved.title);
        assert_eq!(created.content, retrieved.content);
    }

    #[test]
    fn test_store_get_nonexistent_note() {
        let fixture = TestFixture::new();

        let result = fixture.store.get("nonexistent-id").expect("Query failed");

        assert!(result.is_none());
    }

    #[test]
    fn test_store_update_note() {
        let fixture = TestFixture::new();

        let mut note = fixture
            .store
            .create("Original".to_string(), "Original content".to_string())
            .expect("Failed to create");

        note.title = "Updated".to_string();
        note.content = "Updated content".to_string();
        note.color = NoteColor::Blue;
        note.category = NoteCategory::Work;
        note.touch();

        fixture.store.update(&note).expect("Failed to update");

        let retrieved = fixture
            .store
            .get(&note.id)
            .expect("Failed to get")
            .expect("Note not found");

        assert_eq!(retrieved.title, "Updated");
        assert_eq!(retrieved.content, "Updated content");
        assert_eq!(retrieved.color, NoteColor::Blue);
        assert_eq!(retrieved.category, NoteCategory::Work);
    }

    #[test]
    fn test_store_delete_note() {
        let fixture = TestFixture::new();

        let note = fixture
            .store
            .create("To Delete".to_string(), "Content".to_string())
            .expect("Failed to create");

        fixture.store.delete(&note.id).expect("Failed to delete");

        let retrieved = fixture.store.get(&note.id).expect("Failed to get");

        assert!(retrieved.is_none(), "Note should be soft-deleted");
    }

    #[test]
    fn test_store_list_notes() {
        let fixture = TestFixture::new();

        fixture
            .store
            .create("Note 1".to_string(), "Content".to_string())
            .unwrap();
        fixture
            .store
            .create("Note 2".to_string(), "Content".to_string())
            .unwrap();
        fixture
            .store
            .create("Note 3".to_string(), "Content".to_string())
            .unwrap();

        let notes = fixture.store.list(false).expect("Failed to list");

        assert_eq!(notes.len(), 3);
    }

    #[test]
    fn test_store_list_notes_exclude_archived() {
        let fixture = TestFixture::new();

        fixture
            .store
            .create("Note 1".to_string(), "Content".to_string())
            .unwrap();

        let mut archived = fixture
            .store
            .create("Archived Note".to_string(), "Content".to_string())
            .unwrap();
        archived.archived = true;
        fixture.store.update(&archived).unwrap();

        let notes_excluding = fixture.store.list(false).expect("Failed to list");
        let notes_including = fixture.store.list(true).expect("Failed to list");

        assert_eq!(notes_excluding.len(), 1);
        assert_eq!(notes_including.len(), 2);
    }

    #[test]
    fn test_store_search_notes() {
        let fixture = TestFixture::new();

        fixture
            .store
            .create("Hello World".to_string(), "Content".to_string())
            .unwrap();
        fixture
            .store
            .create("Test Note".to_string(), "Hello there".to_string())
            .unwrap();
        fixture
            .store
            .create("Another".to_string(), "Different".to_string())
            .unwrap();

        let results = fixture.store.search("Hello").expect("Failed to search");

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_store_pin_note() {
        let fixture = TestFixture::new();

        let note1 = fixture
            .store
            .create("Regular".to_string(), "Content".to_string())
            .unwrap();
        let mut note2 = fixture
            .store
            .create("Pinned".to_string(), "Content".to_string())
            .unwrap();

        note2.pinned = true;
        fixture.store.update(&note2).unwrap();

        let notes = fixture.store.list(false).expect("Failed to list");

        assert_eq!(notes[0].id, note2.id, "Pinned note should be first");
        assert_eq!(notes[1].id, note1.id);
    }

    #[test]
    fn test_store_apply_remote_changes_new() {
        let fixture = TestFixture::new();

        let mut remote_note = Note::new(
            "Remote".to_string(),
            "Content".to_string(),
            "remote-dev".to_string(),
        );
        remote_note.sync_hash = Some(remote_note.compute_hash());

        let applied = fixture
            .store
            .apply_remote_changes(vec![remote_note.clone()])
            .expect("Failed to apply");

        assert_eq!(applied, 1);

        let retrieved = fixture.store.get(&remote_note.id).expect("Failed to get");
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_store_apply_remote_changes_no_change() {
        let fixture = TestFixture::new();

        let local = fixture
            .store
            .create("Same".to_string(), "Content".to_string())
            .expect("Failed to create");

        let mut remote = local.clone();
        remote.device_id = "remote-dev".to_string();
        remote.sync_hash = local.sync_hash.clone();

        let applied = fixture
            .store
            .apply_remote_changes(vec![remote])
            .expect("Failed to apply");

        assert_eq!(applied, 0, "Should not apply when hash is same");
    }

    #[test]
    fn test_store_apply_remote_changes_with_change() {
        let fixture = TestFixture::new();

        let local = fixture
            .store
            .create("Original".to_string(), "Content".to_string())
            .expect("Failed to create");

        let mut remote = local.clone();
        remote.title = "Updated Title".to_string();
        remote.device_id = "remote-dev".to_string();
        remote.touch();

        let applied = fixture
            .store
            .apply_remote_changes(vec![remote.clone()])
            .expect("Failed to apply");

        assert_eq!(applied, 1);

        let retrieved = fixture
            .store
            .get(&local.id)
            .expect("Failed to get")
            .expect("Not found");
        assert_eq!(retrieved.title, "Updated Title");
    }

    #[test]
    fn test_store_get_changes_since() {
        let fixture = TestFixture::new();

        let before = Utc::now() - chrono::Duration::seconds(1);

        std::thread::sleep(std::time::Duration::from_millis(10));

        fixture
            .store
            .create("New 1".to_string(), "Content".to_string())
            .unwrap();
        fixture
            .store
            .create("New 2".to_string(), "Content".to_string())
            .unwrap();

        let changes = fixture
            .store
            .get_changes_since(before)
            .expect("Failed to get changes");

        assert_eq!(changes.len(), 2);
    }

    #[test]
    fn test_note_with_tags() {
        let fixture = TestFixture::new();

        let mut note = fixture
            .store
            .create("Tagged".to_string(), "Content".to_string())
            .expect("Failed to create");

        note.tags = vec!["work".to_string(), "important".to_string()];
        fixture.store.update(&note).expect("Failed to update");

        let retrieved = fixture
            .store
            .get(&note.id)
            .expect("Failed to get")
            .expect("Not found");

        assert_eq!(retrieved.tags.len(), 2);
        assert!(retrieved.tags.contains(&"work".to_string()));
        assert!(retrieved.tags.contains(&"important".to_string()));
    }
}
