use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};

static BLIND_MODE_ACTIVE: AtomicBool = AtomicBool::new(false);

pub fn is_blind_mode_active() -> bool {
    BLIND_MODE_ACTIVE.load(Ordering::SeqCst)
}

pub fn set_blind_mode(active: bool) {
    BLIND_MODE_ACTIVE.store(active, Ordering::SeqCst);
    if active {
        speak("Modo para pessoas cegas ativado. A partir de agora, lerei as ações em voz alta.");
    } else {
        speak("Modo de acessibilidade desativado.");
    }
}

pub fn speak(text: &str) {
    if !is_blind_mode_active() {
        return;
    }
    
    let text_clone = text.to_string();
    std::thread::spawn(move || {
        // Usa espeak-ng no Linux
        let _ = Command::new("espeak-ng")
            .arg("-v")
            .arg("pt-br")
            .arg(&text_clone)
            .status();
    });
}
