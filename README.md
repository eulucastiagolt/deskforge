# Deskforge

Ferramentas de desktop para KDE Plasma. O Deskforge começou como Window Lens e mantém o seletor de janelas e o testador de expressões regulares.

## Instalação

Baixe o AppImage x86_64 nos [releases](https://github.com/eulucastiagolt/deskforge/releases). O arquivo Deskforge inclui as bibliotecas GTK4 e usa a sessão do KDE Plasma para consultar janelas.

```bash
chmod +x Deskforge-x86_64.AppImage
./Deskforge-x86_64.AppImage
```

## Uso

Em **Seletor de janelas**, clique em **Selecionar janela** e depois na janela que deseja consultar. Filtre a tabela por nome ou valor e copie as propriedades exibidas. Cada classe coletada pode ser copiada ou removida. O rodapé gera uma única regex com todas as classes.

No KDE, cole a expressão em **Classe da janela (aplicativo)**, selecione **Expressão regular** e deixe **Corresponder à classe da janela completa** em **Não**.

Em **Testador de regex**, digite um valor por linha para conferir quantas linhas correspondem à expressão.

### Atalho global

Em **Configurações**, copie o comando do seletor e abra **Atalhos do KDE**. Adicione um **Comando ou script** e atribua a combinação desejada. Se você usava um atalho do Window Lens, atualize o caminho para o AppImage do Deskforge.

## Desenvolvimento

O código usa Rust e GTK4. Instale as ferramentas de desenvolvimento da sua distribuição e execute `cargo run --locked`. O seletor consulta o KWin pelo D-Bus da sessão.

Para gerar o AppImage, execute `./build-appimage.sh` com `appimagetool`, `rsvg-convert`, `glib-compile-schemas` e o tema Adwaita disponíveis no ambiente de build. O resultado fica em `dist/`. A compatibilidade com glibc depende do ambiente de build. O workflow de distribuição usa Ubuntu 24.04.

## Publicação

Atualize a versão em `Cargo.toml` e os dados em `data/*.metainfo.xml`. Use versionamento semântico `MAJOR.MINOR.PATCH`. Durante a série 0.x, adições e mudanças incompatíveis avançam o MINOR e correções compatíveis avançam o PATCH.

O workflow gera o AppImage e os metadados `.zsync` a partir de uma tag anotada, como `v0.3.0`. O nome do arquivo do Deskforge é fixo entre releases para facilitar as atualizações pelo AppManager. Usuários do AppImage antigo do Window Lens devem baixar o Deskforge na primeira migração.
