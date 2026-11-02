use std::path::PathBuf;
use tokio::process::Command;
use tokio::sync::{mpsc, Semaphore};
use std::sync::Arc;
use id3::Tag;

/// Estado de un job de descarga.
#[derive(Debug, Clone, PartialEq)]
pub enum JobState {
    Pending,
    Running,
    Completed { path: PathBuf, cover: Option<PathBuf> },
    Failed { reason: String },
}

/// Un job de descarga individual.
#[derive(Debug, Clone)]
pub struct DownloadJob {
    pub id: usize,
    pub url: String,
    pub output_dir: PathBuf,
    pub state: JobState,
}

/// Cola de descargas con límite de concurrencia.
pub struct DownloadQueue {
    tx: mpsc::UnboundedSender<DownloadJob>,
    semaphore: Arc<Semaphore>,
    next_id: std::sync::atomic::AtomicUsize,
}

impl DownloadQueue {
    /// Crea una cola con el límite de concurrencia dado.
    pub fn new(max_concurrent: usize) -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel::<DownloadJob>();
        let semaphore = Arc::new(Semaphore::new(max_concurrent));
        let next_id = std::sync::atomic::AtomicUsize::new(0);

        let sem_clone = Arc::clone(&semaphore);
        tokio::spawn(async move {
            while let Some(job) = rx.recv().await {
                let sem = Arc::clone(&sem_clone);
                let id = job.id;
                tokio::spawn(async move {
                    let _permit = sem.acquire().await.ok();
                    if let Err(e) = run_job(job).await {
                        eprintln!("download job {} failed: {}", id, e);
                    }
                });
            }
        });

        Self {
            tx,
            semaphore,
            next_id,
        }
    }

    /// Encola una nueva descarga y devuelve su ID.
    pub fn push(&mut self, url: String, output_dir: PathBuf) -> usize {
        let id = self.next_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let job = DownloadJob {
            id,
            url,
            output_dir,
            state: JobState::Pending,
        };
        let _ = self.tx.send(job);
        id
    }
}

/// Ejecuta yt-dlp para un job y devuelve la ruta del archivo descargado y la carátula.
async fn run_job(job: DownloadJob) -> Result<(PathBuf, Option<PathBuf>), String> {
    let output = Command::new("yt-dlp")
        .arg("--extract-audio")
        .arg("--audio-format")
        .arg("mp3")
        .arg("--write-thumbnail")
        .arg("-o")
        .arg(job.output_dir.join("%(title)s.%(ext)s").to_str().unwrap())
        .arg(&job.url)
        .output()
        .await
        .map_err(|e| format!("no se pudo lanzar yt-dlp: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("yt-dlp falló (código {}): {}", output.status.code().unwrap_or(-1), stderr));
    }

    // Buscar el archivo de audio y la carátula generados en el directorio de salida.
    // Nota: yt-dlp nombra el archivo con %(title)s; aquí devolvemos el directorio como referencia.
    Ok((job.output_dir, None))
}

/// Lee etiquetas ID3 de un archivo de audio.
pub fn read_tags(path: &std::path::Path) -> Result<Tag, String> {
    Tag::read_from_path(path).map_err(|e| format!("no se pudieron leer las etiquetas ID3: {}", e))
}

/// Escribe etiquetas ID3 en un archivo de audio.
pub fn write_tags(path: &std::path::Path, tag: Tag) -> Result<(), String> {
    tag.write_to_path(path, id3::Version::Id3v24)
        .map_err(|e| format!("no se pudieron escribir las etiquetas ID3: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn job_state_debug_works() {
        let state = JobState::Failed { reason: "test".into() };
        assert!(format!("{:?}", state).contains("Failed"));
    }
}
