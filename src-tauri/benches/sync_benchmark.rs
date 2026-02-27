use criterion::{black_box, Criterion};
use flymode::notes::{Note, NotesStore, NoteColor, NoteCategory};
use flymode::sync::{SyncEngine, SyncPayload, SyncState};
use std::sync::Arc;
use std::time::Duration;

fn sync_benchmark(c: &mut Criterion) {
    c.bench_function("create_100 notes", |b| {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("bench_notes.db");
        
        let store = NotesStore::with_path(db_path, "bench-device".to_string())
            .expect("Failed to create store");
        
        b.iter(|| {
                let title = format!("Note {}", i);
                let content = format!("Content for note {}", i);
                store.create(title, content).expect("Failed to create note");
            });
    });
    
    c.bench_function("export notes to |b| {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("bench_notes.db");
        
        let store = NotesStore::with_path(db_path, "bench-device".to_string())
            .expect("Failed to create store");
        
        for i in 0..10 {
            store.create(format!("Note {}", i), format!("Content {}", i)).expect("Failed to create");
        }
        
        store.export_notes().expect("Failed to export");
    });
    
    c.bench_function("import notes", |b| {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("bench_notes.db");
        
        let store = NotesStore::with_path(db_path, "bench-device".to_string())
            .expect("Failed to create store");
        
        let notes: Vec<Note> = (0..100).map(|i| {
            let mut note = Note::new(format!("Note {}", i), format!("Content {}", i), "bench-device".to_string());
            note.sync_hash = Some(note.compute_hash());
            note
        }).collect();
        
        let json = serde_json::to_string(&notes).expect("Failed to serialize");
        store.import_notes(&json).expect("Failed to import");
    });
    
    c.bench_function("search notes", |b| {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("bench_notes.db");
        
        let store = NotesStore::with_path(db_path, "bench-device".to_string())
            .expect("Failed to create store");
        
        for i in 0..100 {
            store.create(format!("Note {}", i), format!("Content {}", i)).expect("Failed to create");
        }
        
        store.search("Note 50").expect("Failed to search");
    });
    
    c.bench_function("apply remote changes", |b| {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("bench_notes.db");
        
        let store = NotesStore::with_path(db_path, "bench-device".to_string())
            .expect("Failed to create store");
        
        for i in 0..50 {
            store.create(format!("Note {}", i), format!("Content {}", i)).expect("Failed to create");
        }
        
        let remote_notes: Vec<Note> = (50..100).map(|i| {
            let mut note = Note::new(format!("Remote {}", i), format!("Remote content {}", i), "remote-device".to_string());
            note.sync_hash = Some(note.compute_hash());
            note
        }).collect();
        
        store.apply_remote_changes(remote_notes).expect("Failed to apply changes");
    });
}

criterion_group! {
    sync_benchmark();
}
