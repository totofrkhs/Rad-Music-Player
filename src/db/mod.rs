use rusqlite::{Connection, Result, params};
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Metadatos de una pista almacenada localmente.
#[derive(Debug, Clone)]
pub struct Track {
    pub id: Option<i64>,
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub path: String,
    pub cover_path: Option<String>,
    pub duration_secs: Option<u32>,
}

/// Una playlist.
#[derive(Debug, Clone)]
pub struct Playlist {
    pub id: Option<i64>,
    pub name: String,
}

/// Capa de persistencia con Rusqlite.
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    /// Abre (o crea) la base de datos en la ruta dada y aplica el esquema.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS tracks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                artist TEXT,
                album TEXT,
                path TEXT NOT NULL UNIQUE,
                cover_path TEXT,
                duration_secs INTEGER
            );
            CREATE TABLE IF NOT EXISTS playlists (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE
            );
            CREATE TABLE IF NOT EXISTS playlist_tracks (
                playlist_id INTEGER NOT NULL,
                track_id INTEGER NOT NULL,
                position INTEGER NOT NULL,
                PRIMARY KEY (playlist_id, track_id),
                FOREIGN KEY (playlist_id) REFERENCES playlists(id),
                FOREIGN KEY (track_id) REFERENCES tracks(id)
            );",
        )?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Inserta una pista y devuelve su ID asignado.
    pub fn insert_track(&self, track: &Track) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO tracks (title, artist, album, path, cover_path, duration_secs)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                track.title,
                track.artist,
                track.album,
                track.path,
                track.cover_path,
                track.duration_secs.map(|d| d as i64)
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// Inserta una playlist y devuelve su ID asignado.
    pub fn insert_playlist(&self, playlist: &Playlist) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO playlists (name) VALUES (?1)",
            params![playlist.name],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// Añade una pista a una playlist en una posición dada.
    pub fn add_track_to_playlist(&self, playlist_id: i64, track_id: i64, position: i32) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO playlist_tracks (playlist_id, track_id, position) VALUES (?1, ?2, ?3)",
            params![playlist_id, track_id, position],
        )?;
        Ok(())
    }

    /// Lista todas las pistas.
    pub fn list_tracks(&self) -> Result<Vec<Track>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, artist, album, path, cover_path, duration_secs FROM tracks"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Track {
                id: row.get(0)?,
                title: row.get(1)?,
                artist: row.get(2)?,
                album: row.get(3)?,
                path: row.get(4)?,
                cover_path: row.get(5)?,
                duration_secs: row.get::<_, Option<i64>>(6)?.map(|d| d as u32),
            })
        })?;
        let mut tracks = Vec::new();
        for row in rows {
            tracks.push(row?);
        }
        Ok(tracks)
    }

    /// Lista todas las playlists.
    pub fn list_playlists(&self) -> Result<Vec<Playlist>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name FROM playlists")?;
        let rows = stmt.query_map([], |row| {
            Ok(Playlist {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })?;
        let mut playlists = Vec::new();
        for row in rows {
            playlists.push(row?);
        }
        Ok(playlists)
    }
}

impl Default for Database {
    fn default() -> Self {
        Self::open(":memory:").expect("no se pudo abrir la base de datos en memoria")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_list_track() {
        let db = Database::open(":memory:").unwrap();
        let track = Track {
            id: None,
            title: "Test Song".into(),
            artist: Some("Test Artist".into()),
            album: Some("Test Album".into()),
            path: "/music/test.mp3".into(),
            cover_path: Some("/music/cover.jpg".into()),
            duration_secs: Some(240),
        };
        let id = db.insert_track(&track).unwrap();
        assert!(id > 0);

        let tracks = db.list_tracks().unwrap();
        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].title, "Test Song");
        assert_eq!(tracks[0].id, Some(id));
    }

    #[test]
    fn insert_and_list_playlist() {
        let db = Database::open(":memory:").unwrap();
        let playlist = Playlist { id: None, name: "Mi Playlist".into() };
        let id = db.insert_playlist(&playlist).unwrap();
        assert!(id > 0);

        let playlists = db.list_playlists().unwrap();
        assert_eq!(playlists.len(), 1);
        assert_eq!(playlists[0].name, "Mi Playlist");
    }
}
