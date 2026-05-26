use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box, Button, FlowBox, Label, Image, Overlay, PopoverMenu, gio, GestureClick, TextView, TextBuffer, ScrolledWindow, Paned};
use crate::accessibility::{speak, is_blind_mode_active, set_blind_mode};
use crate::vault;

pub fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("IdenMarshmellow 🔒")
        .default_width(900)
        .default_height(700)
        .build();

    let main_paned = Paned::builder()
        .orientation(gtk4::Orientation::Vertical)
        .build();

    let top_box = Box::new(gtk4::Orientation::Vertical, 0);

    // Header bar
    let header = gtk4::HeaderBar::new();
    
    // Blind mode toggle
    let blind_mode_btn = gtk4::ToggleButton::with_label("👁 Modo Cego");
    blind_mode_btn.add_css_class("blind-mode-button");
    blind_mode_btn.connect_toggled(|btn| {
        let is_active = btn.is_active();
        set_blind_mode(is_active);
    });
    header.pack_end(&blind_mode_btn);
    window.set_titlebar(Some(&header));

    // Desktop area with wood wallpaper
    let overlay = Overlay::new();
    overlay.add_css_class("desktop-area");
    overlay.set_vexpand(true);

    // Right-click gesture for context menu
    let gesture = GestureClick::new();
    gesture.set_button(gtk4::gdk::BUTTON_SECONDARY);
    let popover = build_context_menu();
    popover.set_parent(&overlay);
    
    gesture.connect_pressed(move |gesture, n_press, x, y| {
        if n_press == 1 {
            let rect = gtk4::gdk::Rectangle::new(x as i32, y as i32, 1, 1);
            popover.set_pointing_to(Some(&rect));
            popover.popup();
            if is_blind_mode_active() {
                speak("Menu de contexto aberto.");
            }
        }
    });
    overlay.add_controller(gesture);

    let flow_box = FlowBox::new();
    flow_box.set_valign(gtk4::Align::Center);
    flow_box.set_halign(gtk4::Align::Center);
    flow_box.set_selection_mode(gtk4::SelectionMode::None);
    flow_box.set_max_children_per_line(4);
    flow_box.set_column_spacing(20);
    flow_box.set_row_spacing(20);

    let vaults = vault::vault_get_all_paths_pub();
    let has_vaults = !vaults.is_empty();

    let icons = vec![
        ("document-new-symbolic", "Criar\nVault", "criar", true),
        ("changes-prevent-symbolic", "Proteger\nVault", "proteger", has_vaults),
        ("changes-allow-symbolic", "Desbloquear\nVault", "desbloquear", has_vaults),
        ("document-export-symbolic", "Exportar\nArquivo", "exportar", has_vaults),
        ("system-search-symbolic", "Scan\nVault", "scan", has_vaults),
        ("user-trash-symbolic", "Deletar\nVault", "deletar", has_vaults),
        ("dialog-password-symbolic", "Mudar\nSenha", "senha", has_vaults),
        ("security-high-symbolic", "Sandbox\nVault", "sandbox", has_vaults),
    ];

    for (icon_name, label, action, enabled) in icons {
        let btn = create_desktop_icon(icon_name, label, action, enabled);
        flow_box.insert(&btn, -1);
    }

    overlay.set_child(Some(&flow_box));
    top_box.append(&overlay);

    main_paned.set_start_child(Some(&top_box));

    // Bottom pane: Log console
    let console_scrolled = ScrolledWindow::builder()
        .min_content_height(150)
        .vexpand(false)
        .build();
    let console_text = TextView::builder()
        .editable(false)
        .cursor_visible(false)
        .monospace(true)
        .build();
    console_text.add_css_class("console-view");
    let buffer = TextBuffer::new(None);
    buffer.set_text("IdenMarshmellow System Console Iniciado...\n[INFO] Carregando catálogo de cofres do sistema.\n");
    console_text.set_buffer(Some(&buffer));
    console_scrolled.set_child(Some(&console_text));
    
    let bottom_box = Box::new(gtk4::Orientation::Vertical, 0);
    bottom_box.append(&console_scrolled);

    // Status bar (bottom most)
    let status_bar = build_status_bar(vaults.len());
    bottom_box.append(&status_bar);

    main_paned.set_end_child(Some(&bottom_box));
    // Set pane position so the console is initially about 150px high
    main_paned.set_position(550);

    window.set_child(Some(&main_paned));
    window.present();
}

fn create_desktop_icon(icon_name: &str, text: &str, action: &str, enabled: bool) -> Button {
    let btn = Button::new();
    btn.add_css_class("desktop-icon");
    
    let vbox = Box::new(gtk4::Orientation::Vertical, 4);
    vbox.set_halign(gtk4::Align::Center);
    
    let icon_img = Image::from_icon_name(icon_name);
    icon_img.set_pixel_size(48);
    icon_img.add_css_class("desktop-icon-image");
    vbox.append(&icon_img);
    
    let text_label = Label::new(Some(text));
    text_label.add_css_class("desktop-icon-label");
    text_label.set_justify(gtk4::Justification::Center);
    vbox.append(&text_label);
    
    btn.set_child(Some(&vbox));
    
    if !enabled {
        btn.set_sensitive(false);
        btn.set_tooltip_text(Some("Oops! Não há nenhum vault criado, tente criar um primeiro."));
    }

    let text_clone = text.replace('\n', " ").to_string();
    let action_clone = action.to_string();
    
    btn.connect_clicked(move |b| {
        if is_blind_mode_active() {
            speak(&format!("Ação selecionada: {}", text_clone));
        }
        
        if !b.is_sensitive() {
            speak("Ação indisponível. Crie um vault primeiro.");
        } else {
            println!("Action clicked: {}", action_clone);
        }
    });

    btn
}

fn build_context_menu() -> PopoverMenu {
    let menu = gio::Menu::new();
    
    // Group 1: Basic actions
    let basic_section = gio::Menu::new();
    basic_section.append(Some("🆕 Criar Novo Vault"), Some("app.create_vault"));
    basic_section.append(Some("📋 Listar Vaults"), Some("app.list_vaults"));
    menu.append_section(None, &basic_section);
    
    // Group 2: Security actions
    let sec_section = gio::Menu::new();
    sec_section.append(Some("🔐 Criptografar Vault"), Some("app.encrypt_vault"));
    sec_section.append(Some("🔓 Descriptografar Vault"), Some("app.decrypt_vault"));
    sec_section.append(Some("📦 Exportar Arquivo"), Some("app.export_file"));
    sec_section.append(Some("🔍 Scan de Integridade"), Some("app.scan_vault"));
    menu.append_section(None, &sec_section);
    
    // Group 3: Advanced
    let adv_section = gio::Menu::new();
    adv_section.append(Some("🛡️ Abrir Sandbox"), Some("app.open_sandbox"));
    adv_section.append(Some("🔑 Mudar Senha"), Some("app.change_password"));
    adv_section.append(Some("✏️ Renomear Vault"), Some("app.rename_vault"));
    adv_section.append(Some("🗑️ Deletar Vault"), Some("app.delete_vault"));
    menu.append_section(None, &adv_section);
    
    // Group 4: System
    let sys_section = gio::Menu::new();
    sys_section.append(Some("📊 Informações do Sistema"), Some("app.system_info"));
    sys_section.append(Some("⚙️ Regras de Bloqueio"), Some("app.vault_rules"));
    sys_section.append(Some("🔑 Derivar Master Key"), Some("app.derive_key"));
    menu.append_section(None, &sys_section);

    let popover = PopoverMenu::from_model(Some(&menu));
    popover.set_has_arrow(false);
    popover
}

fn build_status_bar(vault_count: usize) -> Box {
    let status_box = Box::new(gtk4::Orientation::Horizontal, 10);
    status_box.add_css_class("status-bar");
    
    let ok_lbl = Label::new(Some("🟢 Tudo OK"));
    ok_lbl.add_css_class("status-box");
    ok_lbl.add_css_class("status-ok");
    
    let warn_lbl = Label::new(Some("🟡 0 avisos"));
    warn_lbl.add_css_class("status-box");
    warn_lbl.add_css_class("status-warn");
    
    let attack_lbl = Label::new(Some("🔴 0 ataques"));
    attack_lbl.add_css_class("status-box");
    attack_lbl.add_css_class("status-attack");
    
    let count_lbl = Label::new(Some(&format!("{} vaults", vault_count)));
    count_lbl.set_hexpand(true);
    count_lbl.set_halign(gtk4::Align::End);
    
    status_box.append(&ok_lbl);
    status_box.append(&warn_lbl);
    status_box.append(&attack_lbl);
    status_box.append(&count_lbl);
    
    status_box
}
