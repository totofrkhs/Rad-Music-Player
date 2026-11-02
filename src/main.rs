slint::include_modules!();

mod audio;
mod downloader;
mod db;

use std::rc::Rc;
use std::cell::RefCell;
use crate::audio::equalizer::BiquadType;
use crate::audio::{AudioEngine, EqBand};
use crate::downloader::DownloadQueue;
use crate::db::Database;
use std::path::PathBuf;

struct EqState {
    frequency: f32,
    q: f32,
    gain: f32,
}

impl EqState {
    fn new() -> Self {
        Self {
            frequency: 1000.0,
            q: 1.0,
            gain: 0.0,
        }
    }

    fn update(&mut self, frequency: f32, q: f32, gain: f32) {
        self.frequency = frequency;
        self.q = q;
        self.gain = gain;
    }

    fn as_band(&self) -> EqBand {
        EqBand {
            filter_type: BiquadType::Peaking,
            frequency: self.frequency,
            q: self.q,
            gain_db: self.gain,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = Rc::new(RefCell::new(AudioEngine::new()?));
    let eq_state = Rc::new(RefCell::new(EqState::new()));
    let downloader = Rc::new(RefCell::new(DownloadQueue::new(3)));
    let database = Rc::new(RefCell::new(Database::open("data/rad.db")?));
    let last_downloaded_path = Rc::new(RefCell::new(None::<PathBuf>));
    let window = MainWindow::new()?;

    // --- Audio controls ---
    let engine_play = Rc::clone(&engine);
    let eq_play = Rc::clone(&eq_state);
    let play_path = Rc::clone(&last_downloaded_path);
    window.on_play_clicked(move || {
        let band = eq_play.borrow().as_band();
        engine_play.borrow_mut().set_eq_band(band);

        let path_to_play = play_path
            .borrow()
            .clone()
            .unwrap_or_else(|| PathBuf::from("assets/example.mp3"));
        if let Err(e) = engine_play.borrow_mut().play(&path_to_play) {
            eprintln!("error al reproducir {:?}: {}", path_to_play, e);
        }
        let eq_for_log = Rc::clone(&eq_play);
        println!(
            "Reproduciendo {:?} | EQ params: freq={:.1} Hz, q={:.2}, gain={:.1} dB",
            path_to_play,
            eq_for_log.borrow().frequency,
            eq_for_log.borrow().q,
            eq_for_log.borrow().gain,
        );
    });

    let engine_pause = Rc::clone(&engine);
    window.on_pause_clicked(move || {
        engine_pause.borrow().pause();
    });

    let engine_stop = Rc::clone(&engine);
    window.on_stop_clicked(move || {
        engine_stop.borrow().stop();
    });

    // --- Equalizer ---
    let eq_ui = Rc::clone(&eq_state);
    let engine_eq = Rc::clone(&engine);
    window.on_eq_params_changed(move |frequency, q, gain| {
        let band = {
            let mut state = eq_ui.borrow_mut();
            state.update(frequency, q, gain);
            state.as_band()
        };
        engine_eq.borrow_mut().set_eq_band(band);
    });

    // --- Downloader ---
    let dl_ui = Rc::clone(&downloader);
    let dl_path = Rc::clone(&last_downloaded_path);
    window.on_download_clicked(move |url| {
        if url.is_empty() {
            eprintln!("URL vacía, no se encola descarga");
            return;
        }
        let out_dir = std::path::PathBuf::from("downloads");
        let _ = std::fs::create_dir_all(&out_dir);
        let id = dl_ui.borrow_mut().push(url.to_string(), out_dir.clone());
        *dl_path.borrow_mut() = Some(out_dir.clone());
        println!("Descarga encolada: id={} url={} destino={:?}", id, url, out_dir);
    });

    // --- Database / playlists ---
    let db_ui = Rc::clone(&database);
    window.on_refresh_playlists_clicked(move || {
        let db = db_ui.borrow();
        match db.list_playlists() {
            Ok(playlists) => {
                println!("Playlists ({}):", playlists.len());
                for p in &playlists {
                    println!("  - {} (id={})", p.name, p.id.unwrap_or(0));
                }
            }
            Err(e) => eprintln!("error listando playlists: {}", e),
        }
        match db.list_tracks() {
            Ok(tracks) => {
                println!("Pistas ({}):", tracks.len());
                for t in &tracks {
                    println!(
                        "  - {} - {} ({})",
                        t.title,
                        t.artist.as_deref().unwrap_or(""),
                        t.path
                    );
                }
            }
            Err(e) => eprintln!("error listando pistas: {}", e),
        }
    });

    window.show()?;
    let _ = slint::run_event_loop();
    Ok(())
}
