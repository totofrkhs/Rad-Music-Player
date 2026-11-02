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
    let output = build_command(&job)
        .output()
        .await
        .map_err(|e| format!("no se pudo lanzar yt-dlp: {}", e))?;

    interpret_output(&job, output)
}

/// Construye el comando de yt-dlp para un job.
fn build_command(job: &DownloadJob) -> Command {
    let mut cmd = Command::new("yt-dlp");
    cmd.arg("--extract-audio")
        .arg("--audio-format")
        .arg("mp3")
        .arg("--write-thumbnail")
        .arg("-o")
        .arg(job.output_dir.join("%(title)s.%(ext)s"))
        .arg(&job.url);
    cmd
}

/// Traduce la salida de yt-dlp al resultado del job.
///
/// Nota: yt-dlp nombra el archivo con %(title)s; aquí devolvemos el directorio como referencia.
fn interpret_output(
    job: &DownloadJob,
    output: std::process::Output,
) -> Result<(PathBuf, Option<PathBuf>), String> {
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "yt-dlp falló (código {}): {}",
            output.status.code().unwrap_or(-1),
            stderr
        ));
    }

    Ok((job.output_dir.clone(), None))
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
    use std::process::ExitStatus;

    #[cfg(windows)]
    fn exit_status(code: i32) -> ExitStatus {
        use std::os::windows::process::ExitStatusExt;
        ExitStatus::from_raw(code as u32)
    }

    #[cfg(unix)]
    fn exit_status(code: i32) -> ExitStatus {
        use std::os::unix::process::ExitStatusExt;
        ExitStatus::from_raw(code)
    }

    fn test_job() -> DownloadJob {
        DownloadJob {
            id: 7,
            url: "https://example.com/watch?v=abc".to_string(),
            output_dir: PathBuf::from("downloads"),
            state: JobState::Pending,
        }
    }

    fn output_with(status: ExitStatus, stderr: &str) -> std::process::Output {
        std::process::Output {
            status,
            stdout: Vec::new(),
            stderr: stderr.as_bytes().to_vec(),
        }
    }

    #[test]
    fn job_state_debug_works() {
        let state = JobState::Failed { reason: "test".into() };
        assert!(format!("{:?}", state).contains("Failed"));
    }

    #[test]
    fn build_command_uses_expected_ytdlp_flags() {
        let job = test_job();
        let cmd = build_command(&job);
        let std_cmd = cmd.as_std();
        let args: Vec<String> = std_cmd
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();

        assert_eq!(std_cmd.get_program().to_string_lossy(), "yt-dlp");
        assert!(args.contains(&"--extract-audio".to_string()));
        assert!(args.contains(&"mp3".to_string()));
        assert!(args.contains(&"--write-thumbnail".to_string()));
        assert!(args.contains(&"--audio-format".to_string()));
        assert!(args.iter().any(|a| a.contains("%(title)s.%(ext)s")));
        assert_eq!(args.last().unwrap(), &job.url);
    }

    #[test]
    fn interpret_output_maps_failure_to_error() {
        let job = test_job();
        let err = interpret_output(&job, output_with(exit_status(1), "boom")).unwrap_err();
        assert!(err.contains("yt-dlp falló"), "mensaje inesperado: {err}");
        assert!(err.contains("boom"), "mensaje inesperado: {err}");
        assert!(err.contains("(código 1)"), "mensaje inesperado: {err}");
    }

    #[test]
    fn interpret_output_maps_success_to_output_dir() {
        let job = test_job();
        let (path, cover) = interpret_output(&job, output_with(exit_status(0), "")).unwrap();
        assert_eq!(path, job.output_dir);
        assert_eq!(cover, None);
    }
}
