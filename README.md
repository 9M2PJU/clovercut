<p align="center">
  <img src="logo.jpg" alt="Clovercut — editor de vídeo livre e de código aberto" width="800" />
</p>

<h1 align="center">Clovercut</h1>
<p align="center"><strong>O teu vídeo. A tua máquina. O teu editor.</strong></p>
<p align="center">Um editor de vídeo livre, com processamento local e espaço para evoluir.</p>

<p align="center">
  <a href="https://github.com/tulcoturo/clovercut"><img src="https://img.shields.io/badge/Projeto-Clovercut-16b86a" alt="Projeto Clovercut" /></a>
  <a href="https://github.com/tulcoturo/clovercut/issues"><img src="https://img.shields.io/badge/Feedback-Issues-00bcd4" alt="Reportar problemas e sugerir melhorias" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/Licen%C3%A7a-AGPL--3.0--or--later-16b86a" alt="Licença AGPL-3.0-or-later" /></a>
</p>

## O que é o Clovercut?

O **Clovercut** é um fork independente do [Concat](https://github.com/jub0t/Concat), criado para desenvolver uma experiência de edição de vídeo mais fluida e com identidade própria. O foco inicial está no desempenho, na facilidade de utilização e nas melhorias que surgem ao editar projetos reais.

A edição corre localmente, num motor nativo escrito em Rust, com composição acelerada por GPU. Os modelos opcionais de inteligência artificial precisam de ser descarregados antes de poderem ser usados offline.

## O que podes fazer

A base herdada do Concat inclui:

- **Editar vídeo e áudio em várias pistas:** cortar, aparar, mover e organizar clips na timeline.
- **Criar títulos:** escolher fontes, cores, contornos, sombras e fundos para o texto.
- **Animar e aplicar efeitos:** keyframes, filtros, transições e ajustes de imagem e som.
- **Usar ferramentas de IA locais:** legendagem com Whisper, síntese de voz e remoção de fundo, com os respetivos modelos instalados.
- **Exportar vídeo:** formatos como H.264, HEVC e AV1, incluindo projetos em 4K.
- **Automatizar a edição:** ferramentas de linha de comandos, API JSON-RPC e servidor MCP.

Estas funcionalidades vêm do projeto original. A validação específica do Clovercut está em curso.

## Estado do projeto

O Clovercut está numa fase inicial de desenvolvimento. A compilação e os primeiros testes de utilização foram feitos em **Linux**. A base original também contém suporte para Windows, macOS e plataformas móveis, que ainda precisam de validação neste fork.

As primeiras melhorias incluem:

- Correção das dimensões reportadas pelas caixas de texto com bordas fracionárias.
- Renderização mais rápida das sombras dos títulos, limitando o desfoque à área necessária.
- Otimização do módulo de texto nas compilações de desenvolvimento.

Os nomes internos dos executáveis e de vários componentes ainda são `concat`. A adoção da identidade Clovercut na aplicação está em curso.

## Compilar e executar

### Dependências

- Rust **1.93.1**, fixado em [`src/rust-toolchain.toml`](src/rust-toolchain.toml), com Rustfmt e Clippy.
- Bibliotecas de desenvolvimento do **FFmpeg 7 ou superior**.
- CMake, um compilador C++ e libclang.
- No Linux, bibliotecas de desenvolvimento de fontconfig, FreeType, ALSA, GTK 3 e xkbcommon, além dos drivers gráficos necessários.
- ONNX Runtime e sherpa-onnx, configurados para a compilação das ferramentas de IA.

O guia do motor em [`src/README.md`](src/README.md) e o workflow em [`.github/workflows/ci.yml`](.github/workflows/ci.yml) descrevem a configuração das dependências nativas. As versões e os hashes dos binários usados pelo CI estão em [`scripts/ci/deps.lock`](scripts/ci/deps.lock).

### Obter o código

```bash
git clone https://github.com/tulcoturo/clovercut.git
cd clovercut
```

Depois de preparar as dependências e as variáveis de ambiente:

```bash
cd src
cargo run --locked -p concat -j 1
```

Se tiveres um ficheiro `.env.local` com a configuração da tua máquina, carrega-o com `source .env.local` a partir da raiz do repositório, antes de entrar em `src/`. Esse ficheiro é local e não é incluído no Git.

Para compilar o editor com menos informação de depuração e menos unidades de geração de código:

```bash
cargo build --locked -p concat -j 1 \
  --config 'profile.dev.package.concat.debug=0' \
  --config 'profile.dev.package.concat.codegen-units=16'
```

Para abrir o executável já compilado, a partir da raiz do repositório:

```bash
./src/target/debug/concat
```

### Verificações

A partir de `src/`:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --locked --no-fail-fast -j 1
```

A compilação inicial da interface pode consumir bastante memória, mesmo com um único processo. Reserva tempo e memória para esta etapa antes de executar a suíte completa.

## Participar

Encontraste um problema ou tens uma ideia? Abre uma [issue no Clovercut](https://github.com/tulcoturo/clovercut/issues). Nos problemas de edição, indica o sistema operativo, os passos para reproduzir e, se possível, as características do vídeo utilizado.

Para alterações de código, consulta [`CONTRIBUTING.md`](CONTRIBUTING.md). A documentação técnica herdada encontra-se em [`src/README.md`](src/README.md) e [`docs/MCP.md`](docs/MCP.md); parte dela ainda usa a identidade e os serviços do Concat original.

## Origem e licença

O Clovercut parte do trabalho de **Jareer e dos contribuidores do Concat**. É um projeto independente e não uma distribuição oficial do Concat.

O código mantém a licença **AGPL-3.0-or-later**. Consulta [`LICENSE`](LICENSE) e os avisos das dependências em [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md).
