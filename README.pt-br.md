<p align="center">
  <img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" alt="Rust"/>
  <img src="https://img.shields.io/badge/MCP-Protocol-blue?style=for-the-badge" alt="MCP"/>
  <img src="https://img.shields.io/badge/Windows-UI_Automation-0078D6?style=for-the-badge&logo=windows&logoColor=white" alt="Windows"/>
  <img src="https://img.shields.io/badge/License-MIT-yellow?style=for-the-badge" alt="MIT License"/>
</p>

<h1 align="center">Desktop Automation MCP</h1>

<p align="center">
  <strong>Controle qualquer aplicativo Windows desktop atraves de IA.</strong><br/>
  WinForms, WPF, Win32, Electron — se tem interface, este MCP controla.
</p>

<p align="center">
  <code>clicar botoes</code> · <code>ler valores</code> · <code>preencher formularios</code> · <code>capturar telas</code> · <code>navegar arvores</code> · <code>ler grids</code>
</p>

<p align="center">
  <a href="README.md">English</a> · <a href="#instalacao">Portugues</a>
</p>

---

<h3 align="center">
  <br/>
  FEITO COM CLAUDE, PARA O CLAUDE
  <br/><br/>
  <sub>Construido inteiramente por IA para dar a IA o poder de ver e controlar aplicativos desktop.<br/>
  Isso e o que acontece quando voce deixa o Claude construir suas proprias ferramentas.</sub>
  <br/><br/>
</h3>

---

## O que e isso?

**Desktop Automation MCP** e um servidor [Model Context Protocol](https://modelcontextprotocol.io/) que da a agentes de IA (como o Claude) a capacidade de **interagir com qualquer aplicativo Windows desktop** em tempo real.

Ele usa a **Windows UI Automation API** para descobrir janelas, inspecionar arvores de controles, clicar botoes, digitar texto, ler valores, capturar screenshots e muito mais — tudo atraves de uma interface MCP simples.

> Pense nisso como um **"Selenium, so que para apps desktop"** — sistemas ERP legados, ferramentas WinForms, dashboards WPF, dialogos Win32, ate apps Electron.

## Funcionalidades

| Categoria | Ferramentas | Descricao |
|---|---|---|
| **Descoberta de Janelas** | `list_windows` | Lista todas as janelas visiveis com handles, titulos e info do processo |
| **Arvore de UI** | `get_window_tree` | Arvore hierarquica completa de controles de qualquer janela |
| **Busca de Elementos** | `find_element`, `wait_for_element` | Encontra controles por nome, automation ID ou tipo — com suporte a espera |
| **Propriedades** | `get_element_properties` | Todas as propriedades: nome, tipo, retangulo, patterns e estado |
| **Clicar** | `click_element` | Clica ou invoca qualquer elemento (botoes, links, menus) |
| **Entrada de Texto** | `set_value`, `send_keys` | Digita em campos, envia teclas especiais (`{Enter}`, `{Tab}`, `{Ctrl}c`) |
| **Ler Valores** | `get_value` | Le o valor/texto atual de qualquer controle |
| **Selecao** | `select_item` | Seleciona itens em listas, combo boxes, tabs e tree views |
| **Expandir/Colapsar** | `expand_collapse` | Expande ou colapsa nos de arvore, combo boxes, expanders |
| **Dados de Grid** | `read_grid` | Le cabecalhos e celulas de DataGridView e controles Table |
| **Screenshots** | `screenshot_window` | Captura screenshots de janelas como imagens PNG em base64 |
| **Gerenciar Janelas** | `set_window_state` | Minimizar, maximizar, restaurar ou trazer janelas para frente |
| **Rastreio de Foco** | `get_focused_element` | Identifica o elemento com foco em qualquer janela |

> **16 ferramentas. Zero configuracao. Um executavel.**

## Instalacao

### Pre-requisitos

- **Windows 10/11**
- **[Claude Code](https://docs.anthropic.com/en/docs/claude-code)** (ou qualquer cliente compativel com MCP)

### Passo 1 — Baixar

Acesse [Releases](https://github.com/pedrohendp/desktop-automation/releases) e baixe o **`desktop-automation.exe`**.

### Passo 2 — Registrar o MCP

Abra seu terminal e execute:

```bash
claude mcp add desktop-automation -s user -- "C:\Tools\desktop-automation.exe"
```

> Substitua `C:\Tools\desktop-automation.exe` pelo caminho real onde voce salvou o arquivo.

### Passo 3 — Reiniciar o Claude Code

Feche e reabra o Claude Code. O MCP vai conectar automaticamente.

### Passo 4 — Testar

Basta perguntar ao Claude:

```
"Liste todas as janelas abertas no meu desktop"
```

Pronto!

---

### Linux (via WSL2)

> Este MCP usa a Windows UI Automation API. No Linux, funciona atraves do WSL2 em um host Windows.

1. Baixe `desktop-automation.exe` para o filesystem do Windows (ex: `C:\Tools\`)
2. No WSL2:
   ```bash
   claude mcp add desktop-automation -s user -- "/mnt/c/Tools/desktop-automation.exe"
   ```
3. Reinicie o Claude Code

### macOS (via VM)

> Necessita de uma maquina virtual Windows (Parallels, VMware ou UTM).

1. Configure uma VM Windows
2. Siga a [instalacao Windows](#passo-1--baixar) dentro da VM
3. Execute o Claude Code dentro da VM

## Inicio Rapido

Depois de instalado, basta conversar com o Claude naturalmente:

```
"Clica no botao 'Salvar' no Bloco de Notas"
"Digita 'Ola Mundo' no campo de texto"
"Tira um screenshot da Calculadora"
"Le todos os dados do grid no meu sistema ERP"
```

O Claude vai usar as ferramentas do MCP automaticamente.

## Arquitetura

```
desktop-automation.exe
├── MCP Server (rmcp)          — JSON-RPC via stdio
├── COM Thread                 — Windows COM/UI Automation
├── Automation Core            — Wrapper da UI Automation API
│   ├── Descoberta de elementos — Encontra e percorre elementos UI
│   ├── Suporte a patterns     — Invoke, Value, Selection, ExpandCollapse, Grid
│   └── Acesso a propriedades  — Name, Type, AutomationId, BoundingRect, State
└── Ferramentas
    ├── Window tools           — list_windows, get_window_tree, set_window_state
    ├── Find tools             — find_element, wait_for_element
    ├── Property tools         — get_element_properties, get_value, get_focused_element
    ├── Interaction tools      — click_element, set_value, send_keys, select_item, expand_collapse
    ├── Screenshot tools       — screenshot_window
    └── Advanced tools         — read_grid
```

**Construido com:** [Rust](https://www.rust-lang.org/) · [rmcp](https://crates.io/crates/rmcp) · [uiautomation](https://crates.io/crates/uiautomation) · [tokio](https://tokio.rs/) · [win_screenshot](https://crates.io/crates/win_screenshot)

## Contribuindo

```bash
git clone https://github.com/pedrohendp/desktop-automation.git
cd desktop-automation
cargo build --release
```

Necessario [Rust 1.75+](https://rustup.rs/) e Windows SDK.

## Licenca

MIT — veja [LICENSE](LICENSE).

---

<p align="center">
  <strong>Feito com Claude, para o Claude.</strong><br/>
  <sub>De a sua IA o poder de ver e tocar o desktop.</sub>
</p>
