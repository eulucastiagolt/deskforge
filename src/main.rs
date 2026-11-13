use gtk::{gio, glib, prelude::*};
use gtk4 as gtk;
use std::{cell::RefCell, collections::BTreeMap, path::PathBuf, rc::Rc};

const APP_ID: &str = "io.github.eulucastiagolt.Deskforge";

type Properties = BTreeMap<String, String>;

#[derive(Default)]
struct Inspector {
    classes: Vec<String>,
    properties: Properties,
}

impl Inspector {
    fn expression(&self) -> String {
        if self.classes.is_empty() {
            return String::new();
        }
        let escaped: Vec<_> = self
            .classes
            .iter()
            .map(|value| glib::Regex::escape_string(value.as_str()))
            .collect();
        format!("^({})$", escaped.join("|"))
    }
}

fn column(spacing: i32) -> gtk::Box {
    gtk::Box::new(gtk::Orientation::Vertical, spacing)
}

fn panel() -> gtk::Box {
    let panel = column(10);
    panel.set_margin_top(14);
    panel.set_margin_bottom(14);
    panel.set_margin_start(14);
    panel.set_margin_end(14);
    panel
}

fn label(text: &str) -> gtk::Label {
    gtk::Label::builder().label(text).xalign(0.0).build()
}

fn heading(text: &str) -> gtk::Label {
    let title = label(text);
    title.add_css_class("heading");
    title
}

fn scroll(child: &impl IsA<gtk::Widget>) -> gtk::ScrolledWindow {
    gtk::ScrolledWindow::builder()
        .child(child)
        .vexpand(true)
        .hexpand(true)
        .min_content_width(190)
        .min_content_height(150)
        .has_frame(true)
        .build()
}

fn clear(list: &gtk::ListBox) {
    while let Some(row) = list.first_child() {
        list.remove(&row);
    }
}

fn visible_properties(state: &Inspector, search: &str) -> Properties {
    let search = search.to_lowercase();
    state
        .properties
        .iter()
        .filter(|(key, value)| {
            key.to_lowercase().contains(&search) || value.to_lowercase().contains(&search)
        })
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect()
}

fn refresh_properties(list: &gtk::ListBox, state: &Inspector, search: &str) {
    clear(list);
    for (key, value) in visible_properties(state, search) {
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        row.set_margin_top(6);
        row.set_margin_bottom(6);
        row.set_margin_start(8);
        row.set_margin_end(8);
        let key_label = label(&key);
        key_label.set_width_chars(21);
        key_label.set_selectable(true);
        let value_label = label(&value);
        value_label.set_selectable(true);
        value_label.set_wrap(true);
        value_label.set_wrap_mode(gtk::pango::WrapMode::WordChar);
        row.append(&key_label);
        row.append(&value_label);
        list.append(&row);
    }
}

fn refresh_classes(list: &gtk::ListBox, expression: &gtk::Entry, state: &Rc<RefCell<Inspector>>) {
    clear(list);
    let classes = state.borrow().classes.clone();
    for value in classes {
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        let name = label(&value);
        name.set_hexpand(true);
        name.set_ellipsize(gtk::pango::EllipsizeMode::End);
        name.set_tooltip_text(Some(&value));
        let copy = gtk::Button::from_icon_name("edit-copy-symbolic");
        copy.set_tooltip_text(Some("Copiar classe"));
        copy.connect_clicked({
            let value = value.clone();
            move |button| button.clipboard().set_text(&value)
        });
        let remove = gtk::Button::from_icon_name("list-remove-symbolic");
        remove.set_tooltip_text(Some("Remover classe"));
        remove.connect_clicked(glib::clone!(
            #[weak]
            list,
            #[weak]
            expression,
            #[strong]
            state,
            move |_| {
                state.borrow_mut().classes.retain(|class| class != &value);
                refresh_classes(&list, &expression, &state);
            }
        ));
        row.append(&name);
        row.append(&copy);
        row.append(&remove);
        list.append(&row);
    }
    expression.set_text(&state.borrow().expression());
}

fn regex_page(stack: &gtk::Stack) -> gtk::Entry {
    let page = panel();
    let pattern = gtk::Entry::builder()
        .placeholder_text("Cole ou digite uma expressão")
        .build();
    let text = gtk::TextView::builder()
        .monospace(true)
        .wrap_mode(gtk::WrapMode::WordChar)
        .build();
    let result = label("Digite uma expressão para testar.");
    result.set_wrap(true);
    page.append(&heading("Expressão regular"));
    page.append(&pattern);
    page.append(&heading("Texto para testar · um valor por linha"));
    page.append(&scroll(&text));
    page.append(&result);
    let update = Rc::new(glib::clone!(
        #[weak]
        pattern,
        #[weak]
        text,
        #[weak]
        result,
        move || {
            let pattern = pattern.text();
            let buffer = text.buffer();
            let input = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
            let message = if pattern.is_empty() {
                "Digite uma expressão para testar.".into()
            } else {
                match glib::Regex::new(
                    &pattern,
                    glib::RegexCompileFlags::DEFAULT,
                    glib::RegexMatchFlags::DEFAULT,
                ) {
                    Ok(Some(regex)) => {
                        let count = input
                            .lines()
                            .filter(|line| {
                                regex
                                    .match_(
                                        glib::GString::from(*line).as_gstr(),
                                        glib::RegexMatchFlags::DEFAULT,
                                    )
                                    .is_ok_and(|info| info.matches())
                            })
                            .count();
                        format!("Linhas correspondentes: {count}")
                    }
                    Ok(None) => "Expressão inválida.".into(),
                    Err(error) => format!("Expressão inválida: {error}"),
                }
            };
            result.set_text(&message);
        }
    ));
    pattern.connect_changed({
        let update = update.clone();
        move |_| update()
    });
    text.buffer().connect_changed(move |_| update());
    stack.add_titled(&page, Some("regex"), "Testador de regex");
    pattern
}

fn shortcut_command() -> String {
    let executable = std::env::var_os("APPIMAGE")
        .map(PathBuf::from)
        .or_else(|| std::env::current_exe().ok())
        .unwrap_or_else(|| PathBuf::from("deskforge"));
    format!("{} --pick", glib::shell_quote(executable).to_string_lossy())
}

fn settings(window: &gtk::ApplicationWindow) {
    let dialog = gtk::Window::builder()
        .title("Configurações")
        .default_width(500)
        .transient_for(window)
        .modal(true)
        .build();
    let content = panel();
    content.append(&heading("Atalho global"));
    let instructions = label(
        "No KDE, adicione um atalho do tipo Comando ou script. Cole o comando abaixo e escolha uma combinação de teclas.",
    );
    instructions.set_wrap(true);
    content.append(&instructions);
    let command = gtk::Entry::builder()
        .text(shortcut_command())
        .editable(false)
        .build();
    content.append(&command);
    let actions = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let copy = gtk::Button::with_label("Copiar comando");
    copy.connect_clicked(move |button| button.clipboard().set_text(&shortcut_command()));
    let open = gtk::Button::with_label("Abrir atalhos do KDE");
    let status = label("");
    status.set_wrap(true);
    open.connect_clicked(glib::clone!(
        #[weak]
        status,
        move |_| {
            let mut process = std::process::Command::new("kcmshell6");
            process.args(["kcm_keys"]);
            if std::env::var_os("DESKFORGE_BUNDLED").is_some() {
                process
                    .env_remove("LD_LIBRARY_PATH")
                    .env_remove("GSETTINGS_SCHEMA_DIR")
                    .env_remove("GIO_MODULE_DIR")
                    .env_remove("GDK_PIXBUF_MODULE_FILE");
            }
            if let Err(error) = process.spawn() {
                status.set_text(&format!(
                    "Abra Atalhos nas Configurações do Sistema. {error}"
                ));
            }
        }
    ));
    actions.append(&copy);
    actions.append(&open);
    content.append(&actions);
    content.append(&status);
    dialog.set_child(Some(&content));
    dialog.present();
}

fn build(app: &gtk::Application) -> gtk::ApplicationWindow {
    let state = Rc::new(RefCell::new(Inspector::default()));
    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("Deskforge")
        .default_width(1070)
        .default_height(640)
        .build();
    let header = gtk::HeaderBar::new();
    window.set_titlebar(Some(&header));

    let layout = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let stack = gtk::Stack::builder().hexpand(true).vexpand(true).build();
    let sidebar = column(5);
    sidebar.set_width_request(166);
    sidebar.add_css_class("sidebar");
    let nav = gtk::StackSidebar::builder()
        .stack(&stack)
        .vexpand(true)
        .build();
    sidebar.append(&nav);
    let preferences = gtk::Button::with_label("Configurações");
    preferences.connect_clicked(glib::clone!(
        #[weak]
        window,
        move |_| settings(&window)
    ));
    sidebar.append(&preferences);
    let about = gtk::Button::with_label("Sobre");
    about.connect_clicked(glib::clone!(
        #[weak]
        window,
        move |_| {
            gtk::AboutDialog::builder()
                .transient_for(&window)
                .modal(true)
                .program_name("Deskforge")
                .version(env!("CARGO_PKG_VERSION"))
                .comments("Ferramentas de janelas e expressões regulares para KDE Plasma.")
                .website("https://github.com/eulucastiagolt/deskforge")
                .build()
                .present();
        }
    ));
    sidebar.append(&about);
    layout.append(&sidebar);
    layout.append(&gtk::Separator::new(gtk::Orientation::Vertical));
    layout.append(&stack);
    window.set_child(Some(&layout));

    let inspector = panel();
    stack.add_titled(&inspector, Some("inspector"), "Seletor de janelas");
    let toolbar = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    let pick = gtk::Button::builder()
        .label("Selecionar janela")
        .action_name("win.pick")
        .build();
    pick.add_css_class("suggested-action");
    let status = label("Clique em Selecionar janela para começar.");
    status.set_wrap(true);
    status.add_css_class("dim-label");
    toolbar.append(&pick);
    toolbar.append(&status);
    inspector.append(&toolbar);

    let panes = gtk::Paned::builder()
        .orientation(gtk::Orientation::Horizontal)
        .position(285)
        .wide_handle(true)
        .vexpand(true)
        .build();
    let classes = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::None)
        .build();
    let empty = label("Nenhuma classe coletada");
    empty.set_margin_start(8);
    empty.set_margin_top(8);
    classes.set_placeholder(Some(&empty));
    let class_panel = panel();
    class_panel.append(&heading("Classes coletadas"));
    class_panel.append(&scroll(&classes));
    panes.set_start_child(Some(&class_panel));
    panes.set_shrink_start_child(false);

    let property_panel = panel();
    property_panel.append(&heading("Propriedades da janela"));
    let filter = gtk::SearchEntry::builder()
        .placeholder_text("Filtrar propriedades")
        .build();
    property_panel.append(&filter);
    let fields = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::None)
        .show_separators(true)
        .build();
    let columns = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    let property_header = heading("Propriedade");
    property_header.set_width_chars(21);
    columns.append(&property_header);
    columns.append(&heading("Valor"));
    property_panel.append(&columns);
    property_panel.append(&scroll(&fields));
    filter.connect_search_changed(glib::clone!(
        #[weak]
        fields,
        #[strong]
        state,
        move |filter| refresh_properties(&fields, &state.borrow(), &filter.text())
    ));
    let copy_fields = gtk::Button::with_label("Copiar propriedades");
    copy_fields.set_halign(gtk::Align::End);
    copy_fields.connect_clicked(glib::clone!(
        #[weak]
        filter,
        #[strong]
        state,
        move |button| {
            let values: Vec<_> = visible_properties(&state.borrow(), &filter.text())
                .into_iter()
                .map(|(key, value)| format!("{key}: {value}"))
                .collect();
            if !values.is_empty() {
                button.clipboard().set_text(&values.join("\n"));
            }
        }
    ));
    property_panel.append(&copy_fields);
    panes.set_end_child(Some(&property_panel));
    panes.set_shrink_end_child(false);
    inspector.append(&panes);

    let expression = gtk::Entry::builder()
        .editable(false)
        .hexpand(true)
        .placeholder_text("Selecione uma janela para gerar a regex")
        .build();
    let regex_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    regex_row.append(&expression);
    let copy_regex = gtk::Button::from_icon_name("edit-copy-symbolic");
    copy_regex.set_tooltip_text(Some("Copiar regex"));
    copy_regex.connect_clicked(glib::clone!(
        #[weak]
        expression,
        move |button| {
            if !expression.text().is_empty() {
                button.clipboard().set_text(&expression.text());
            }
        }
    ));
    regex_row.append(&copy_regex);
    let pattern = regex_page(&stack);
    let test = gtk::Button::with_label("Testar");
    test.connect_clicked(glib::clone!(
        #[weak]
        stack,
        #[weak]
        expression,
        #[weak]
        pattern,
        move |_| {
            pattern.set_text(&expression.text());
            stack.set_visible_child_name("regex");
        }
    ));
    regex_row.append(&test);
    inspector.append(&heading("Expressão regular para o KDE"));
    inspector.append(&regex_row);

    let action = gio::SimpleAction::new("pick", None);
    action.connect_activate(glib::clone!(
        #[weak]
        stack,
        #[weak]
        status,
        #[weak]
        classes,
        #[weak]
        expression,
        #[weak]
        fields,
        #[weak]
        filter,
        #[strong]
        state,
        move |action, _| {
            stack.set_visible_child_name("inspector");
            action.set_enabled(false);
            status.set_text("Clique na janela desejada ou pressione Esc para cancelar.");
            glib::MainContext::default().spawn_local(glib::clone!(
                #[weak]
                action,
                #[weak]
                status,
                #[weak]
                classes,
                #[weak]
                expression,
                #[weak]
                fields,
                #[weak]
                filter,
                #[strong]
                state,
                async move {
                    let result = query_window().await;
                    action.set_enabled(true);
                    match result {
                        Ok(properties) => {
                            let class =
                                properties.get("resourceClass").cloned().unwrap_or_default();
                            {
                                let mut state = state.borrow_mut();
                                state.properties = properties;
                                if !class.is_empty() && !state.classes.contains(&class) {
                                    state.classes.push(class.clone());
                                }
                            }
                            refresh_classes(&classes, &expression, &state);
                            refresh_properties(&fields, &state.borrow(), &filter.text());
                            if class.is_empty() {
                                status.set_text("O KWin não informou a classe dessa janela.");
                            } else {
                                status.set_text(&format!("Última classe selecionada · {class}"));
                            }
                        }
                        Err(error)
                            if gio::DBusError::remote_error(&error).as_deref()
                                == Some("org.kde.KWin.Error.InvalidWindow") =>
                        {
                            status.set_text("Seleção cancelada.");
                        }
                        Err(error) => {
                            status.set_text(&format!("Não foi possível consultar o KWin. {error}"))
                        }
                    }
                }
            ));
        }
    ));
    window.add_action(&action);
    window
}

async fn query_window() -> Result<Properties, glib::Error> {
    let bus = gio::bus_get_future(gio::BusType::Session).await?;
    let reply = bus
        .call_future(
            Some("org.kde.KWin"),
            "/KWin",
            "org.kde.KWin",
            "queryWindowInfo",
            None,
            Some(glib::VariantTy::new("(a{sv})").unwrap()),
            gio::DBusCallFlags::NONE,
            i32::MAX,
        )
        .await?;
    let (values,) = reply
        .get::<(BTreeMap<String, glib::Variant>,)>()
        .ok_or_else(|| {
            glib::Error::new(gio::IOErrorEnum::InvalidData, "Resposta inesperada do KWin")
        })?;
    Ok(values
        .into_iter()
        .map(|(key, value)| {
            let text = value
                .str()
                .map(str::to_owned)
                .unwrap_or_else(|| value.print(false).to_string());
            (key, text)
        })
        .collect())
}

fn main() -> glib::ExitCode {
    let app = gtk::Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::HANDLES_COMMAND_LINE)
        .build();
    app.add_main_option(
        "pick",
        glib::Char::from(b'p'),
        glib::OptionFlags::NONE,
        glib::OptionArg::None,
        "Inicia a seleção de uma janela",
        None,
    );
    app.connect_command_line(|app, command| {
        let window = app
            .windows()
            .into_iter()
            .find_map(|window| window.downcast::<gtk::ApplicationWindow>().ok())
            .unwrap_or_else(|| build(app));
        window.present();
        if command.options_dict().contains("pick") {
            gio::prelude::ActionGroupExt::activate_action(&window, "pick", None);
        }
        glib::ExitCode::SUCCESS
    });
    app.set_accels_for_action("win.pick", &["<Ctrl>p"]);
    app.run()
}
