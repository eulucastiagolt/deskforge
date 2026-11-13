import os
import re
import shlex
import shutil
import sys
from pathlib import Path

import gi

gi.require_version("Gtk", "4.0")
gi.require_version("Adw", "1")
from gi.repository import Adw, Gio, GLib, Gtk


APP_ID = "io.github.windowlens.WindowLens"
VERSION = "0.1.0"


def picker_command():
    image = os.environ.get("APPIMAGE")
    args = [image] if image else [sys.executable, str(Path(__file__).resolve())]
    return shlex.join([*args, "--pick"])


class WindowLens(Adw.Application):
    def __init__(self):
        super().__init__(application_id=APP_ID, flags=Gio.ApplicationFlags.HANDLES_COMMAND_LINE)
        self.add_main_option("pick", 0, GLib.OptionFlags.NONE, GLib.OptionArg.NONE,
                             "Inicia a seleção de uma janela", None)
        self.process = None
        self.classes = []
        self.qdbus = shutil.which("qdbus6") or shutil.which("qdbus")

        for name, callback in (("pick", self.pick), ("settings", self.settings),
                               ("about", self.about), ("quit", self.quit)):
            action = Gio.SimpleAction.new(name, None)
            action.connect("activate", lambda _action, _parameter, run=callback: run())
            self.add_action(action)
        self.set_accels_for_action("app.pick", ["<Ctrl>p"])
        self.set_accels_for_action("app.quit", ["<Ctrl>q"])

    def do_command_line(self, command):
        self.activate()
        if command.get_options_dict().contains("pick"):
            self.pick()
        return 0

    def do_activate(self):
        if self.get_active_window():
            self.get_active_window().present()
            return

        self.window = Adw.ApplicationWindow(application=self, title="Window Lens",
                                            default_width=630, default_height=670)
        self.window.connect("close-request", lambda *_: self.process is not None)
        header = Adw.HeaderBar()
        menu = Gio.Menu()
        menu.append("Configurações", "app.settings")
        menu.append("Sobre o Window Lens", "app.about")
        menu.append("Sair", "app.quit")
        header.pack_end(Gtk.MenuButton(icon_name="open-menu-symbolic", menu_model=menu))

        toolbar = Adw.ToolbarView()
        toolbar.add_top_bar(header)
        self.overlay = Adw.ToastOverlay()
        toolbar.set_content(self.overlay)
        self.window.set_content(toolbar)

        scroll = Gtk.ScrolledWindow(hscrollbar_policy=Gtk.PolicyType.NEVER)
        clamp = Adw.Clamp(maximum_size=650)
        self.content = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=18,
                               margin_top=24, margin_bottom=24, margin_start=24, margin_end=24)
        clamp.set_child(self.content)
        scroll.set_child(clamp)
        self.overlay.set_child(scroll)

        intro = Adw.StatusPage(title="Selecione uma janela",
                               description="Consulte a classe que o KDE usa nas regras de janela.",
                               icon_name="find-location-symbolic")
        intro.add_css_class("compact")
        self.pick_button = Gtk.Button(label="Selecionar janela", action_name="app.pick",
                                      halign=Gtk.Align.CENTER)
        self.pick_button.add_css_class("suggested-action")
        self.pick_button.add_css_class("pill")
        intro.set_child(self.pick_button)
        self.content.append(intro)

        self.status = Gtk.Label(wrap=True)
        self.status.add_css_class("dim-label")
        self.status.set_visible(False)
        self.content.append(self.status)

        self.collected = Adw.PreferencesGroup(title="Classes coletadas")
        self.content.append(self.collected)
        self.empty = Adw.ActionRow(title="Nenhuma classe coletada")
        self.collected.add(self.empty)

        regex_group = Adw.PreferencesGroup(title="Expressão regular")
        self.content.append(regex_group)
        self.expression = Gtk.Entry(editable=False, hexpand=True,
                                    placeholder_text="Selecione uma janela para gerar a regex")
        self.expression.add_css_class("monospace")
        self.copy_regex = Gtk.Button(icon_name="edit-copy-symbolic", tooltip_text="Copiar regex",
                                     sensitive=False)
        self.copy_regex.connect("clicked", lambda *_: self.copy(self.expression.get_text(),
                                                               "Regex copiada"))
        regex_box = Gtk.Box(spacing=8)
        regex_box.append(self.expression)
        regex_box.append(self.copy_regex)
        regex_group.add(regex_box)

        self.details = Adw.ExpanderRow(title="Dados da última janela")
        self.details.set_sensitive(False)
        self.fields = Gtk.TextView(editable=False, monospace=True, wrap_mode=Gtk.WrapMode.WORD_CHAR,
                                   left_margin=12, right_margin=12, top_margin=10, bottom_margin=10)
        details_scroll = Gtk.ScrolledWindow(min_content_height=220, child=self.fields)
        self.details.add_row(details_scroll)
        details_group = Adw.PreferencesGroup()
        details_group.add(self.details)
        self.content.append(details_group)

        if not self.qdbus:
            self.lookup_action("pick").set_enabled(False)
            self.set_status("O seletor precisa de qdbus6 ou qdbus na sessão do KDE.")
        self.window.present()

    def set_status(self, message):
        self.status.set_text(message)
        self.status.set_visible(bool(message))

    def pick(self):
        if self.process or not self.qdbus:
            return
        try:
            self.process = Gio.Subprocess.new(
                [self.qdbus, "org.kde.KWin", "/KWin", "org.kde.KWin.queryWindowInfo"],
                Gio.SubprocessFlags.STDOUT_PIPE | Gio.SubprocessFlags.STDERR_PIPE,
            )
        except GLib.Error as error:
            self.set_status(error.message)
            return
        self.lookup_action("pick").set_enabled(False)
        self.set_status("Clique na janela desejada ou pressione Esc para cancelar.")
        self.hold()
        self.process.communicate_utf8_async(None, None, self.show_result)

    def show_result(self, process, result):
        self.process = None
        self.lookup_action("pick").set_enabled(True)
        try:
            _, output, error = process.communicate_utf8_finish(result)
            if not process.get_successful():
                message = (error or "").strip()
                if "InvalidWindow" in message:
                    self.set_status("Seleção cancelada.")
                else:
                    self.set_status(message or "Não foi possível consultar o KWin.")
                return
            self.show_window_info(output)
        except GLib.Error as error:
            self.set_status(error.message)
        finally:
            self.release()

    def show_window_info(self, output):
        self.fields.get_buffer().set_text(output.strip())
        self.details.set_sensitive(True)
        match = re.search(r"^resourceClass:[ \t]*([^\r\n]*)", output, re.MULTILINE)
        value = match.group(1).strip() if match else ""
        if not value:
            self.set_status("O KWin não informou a classe dessa janela.")
            return
        if value not in self.classes:
            self.add_class(value)
        self.set_status(f"Última classe selecionada · {value}")

    def add_class(self, value):
        row = Adw.ActionRow(title=value)
        row.set_use_markup(False)
        copy = Gtk.Button(icon_name="edit-copy-symbolic", tooltip_text="Copiar classe",
                          valign=Gtk.Align.CENTER)
        copy.add_css_class("flat")
        copy.connect("clicked", lambda *_: self.copy(value, "Classe copiada"))
        remove = Gtk.Button(icon_name="list-remove-symbolic", tooltip_text="Remover classe",
                            valign=Gtk.Align.CENTER)
        remove.add_css_class("flat")
        remove.connect("clicked", lambda *_: self.remove_class(value, row))
        row.add_suffix(copy)
        row.add_suffix(remove)
        self.collected.add(row)
        self.classes.append(value)
        self.update_expression()

    def remove_class(self, value, row):
        self.classes.remove(value)
        self.collected.remove(row)
        self.update_expression()

    def update_expression(self):
        escaped = "|".join(map(re.escape, self.classes))
        self.expression.set_text(f"^({escaped})$" if escaped else "")
        self.empty.set_visible(not self.classes)
        self.copy_regex.set_sensitive(bool(self.classes))

    def copy(self, value, message):
        self.window.get_clipboard().set(value)
        self.overlay.add_toast(Adw.Toast(title=message))

    def settings(self):
        dialog = Adw.PreferencesDialog(title="Configurações")
        page = Adw.PreferencesPage()
        group = Adw.PreferencesGroup(
            title="Atalho global",
            description="No KDE, adicione um comando na tela de Atalhos. Cole o comando abaixo "
                        "e atribua a combinação de teclas que você quiser.",
        )
        command = Adw.EntryRow(title="Comando do seletor", text=picker_command())
        command.set_editable(False)
        copy = Gtk.Button(icon_name="edit-copy-symbolic", tooltip_text="Copiar comando",
                          valign=Gtk.Align.CENTER)
        copy.add_css_class("flat")
        copy.connect("clicked", lambda *_: self.copy_shortcut(dialog, command))
        command.add_suffix(copy)
        group.add(command)
        open_settings = Adw.ButtonRow(title="Abrir atalhos do KDE")
        open_settings.connect("activated", lambda *_: self.open_shortcuts(dialog))
        group.add(open_settings)
        page.add(group)
        dialog.add(page)
        dialog.present(self.window)

    def copy_shortcut(self, dialog, command):
        self.window.get_clipboard().set(command.get_text())
        dialog.add_toast(Adw.Toast(title="Comando copiado"))

    def open_shortcuts(self, dialog):
        try:
            Gio.Subprocess.new(["kcmshell6", "kcm_keys"], Gio.SubprocessFlags.NONE)
        except GLib.Error:
            dialog.add_toast(Adw.Toast(title="Abra Atalhos nas Configurações do Sistema."))

    def about(self):
        Adw.AboutDialog(application_name="Window Lens", version=VERSION,
                        application_icon="find-location-symbolic",
                        comments="Seletor de janelas e gerador de regex para regras do KDE Plasma."
                        ).present(self.window)


if __name__ == "__main__":
    sys.exit(WindowLens().run(sys.argv))
