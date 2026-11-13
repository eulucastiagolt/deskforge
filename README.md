# Window Lens

Seletor de janelas para KDE Plasma. Clique em uma janela para consultar seus dados, reunir classes de aplicativos e copiar a expressão regular para as regras do KWin.

## Instalação

O AppImage é para Linux x86_64 com KDE Plasma 6. Ele usa o Python 3, PyGObject, GTK4, Libadwaita 1.6 ou mais recente e o utilitário `qdbus6` (ou `qdbus`) do sistema.

No Arch Linux, instale as dependências com:

```bash
sudo pacman -S --needed python python-gobject gtk4 libadwaita qt6-tools
```

Baixe o `Window-Lens-x86_64.AppImage` no [release mais recente](https://github.com/eulucastiagolt/window-lens/releases/latest). Integre o arquivo pelo AppManager ou abra-o diretamente:

```bash
chmod +x Window-Lens-x86_64.AppImage
./Window-Lens-x86_64.AppImage
```

Para executar pelo código-fonte, use `python3 window_lens.py` na pasta do projeto com as mesmas dependências.

## Uso

Clique em **Selecionar janela** e depois na janela que deseja consultar. Repita para cada aplicativo. Em **Classes coletadas**, copie uma classe individual ou remova entradas da lista. Use o botão ao lado de **Expressão regular** para copiar todas as classes em uma única regex.

No editor de regras do KDE, cole a regex em **Classe da janela (aplicativo)**, selecione **Expressão regular** e deixe **Corresponder à classe da janela completa** em **Não**.

### Atalho global

Em **Configurações**, copie o comando do seletor e clique em **Abrir atalhos do KDE**. Na tela de atalhos, use **Adicionar novo → Comando ou script**, cole o comando e atribua uma combinação. O comando abre diretamente o seletor mesmo com o app fechado. Caso mova o AppImage, atualize o caminho do comando no KDE.

## Gerar AppImage

Execute `./build-appimage.sh` com `appimagetool` e `rsvg-convert` instalados. O arquivo será criado em `dist/`. Use `APPIMAGE_RUNTIME` para indicar um runtime Type 2 local se necessário.

## Publicar com atualizações

O workflow do GitHub publica o AppImage e o arquivo `.zsync` ao receber uma tag anotada como `v0.1.0`. Escreva as notas do release na mensagem da tag. Ele usa o dono e nome reais do repositório para embutir a informação de atualização. Integre o AppImage publicado pelo AppManager para que ele possa consultar os releases seguintes. Mantenha os nomes dos dois arquivos iguais nas novas versões.

Para empacotar manualmente com o mesmo formato, instale `zsyncmake` e execute:

```bash
UPDATE_INFORMATION='gh-releases-zsync|DONO|REPOSITORIO|latest|Window-Lens-x86_64.AppImage.zsync' ./build-appimage.sh
```

Em cada release, atualize também a versão em `window_lens.py` e os dados de lançamento em `data/*.metainfo.xml`.
