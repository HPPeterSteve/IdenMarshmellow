mod window;
mod accessibility;
mod vault;
mod crypto;
mod log;

use gtk4::prelude::*;
use gtk4::{Application, gio};
use std::env;

fn main() {
    // Inicializa subsistema C do vault
    if let Err(e) = vault::vault_init() {
        eprintln!("Aviso: Falha ao inicializar o core C do Vault: {}", e);
    }

    let app = Application::builder()
        .application_id("com.idenmarshmellow.desktop")
        .build();

    app.connect_startup(|_| {
        libadwaita::init();
        load_css();
    });

    app.connect_activate(window::build_ui);
    
    app.connect_shutdown(|_| {
        let _ = vault::vault_shutdown();
    });

    // Roda a aplicação
    app.run();
}

fn load_css() {
    let display = gtk4::gdk::Display::default().expect("Could not get default display.");
    let provider = gtk4::CssProvider::new();
    
    // We would ideally load from GResource here, but for simplicity in compilation:
    // If running in development, load from file, otherwise from resource
    let _ = provider.load_from_path("data/style.css");

    gtk4::style_context_add_provider_for_display(
        &display,
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
