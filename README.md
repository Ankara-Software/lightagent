Maintained by [Ankara Yazılım](https://github.com/Ankara-Software) | [Project page](https://ankarayazilim.org/acik-kaynak/lightagent)

**Status: beta.** It builds and its 4 unit tests pass. The start, status, resume and clear commands and the full action loop (run command, write file, read file, finish) were run against a scripted stand-in for the Ollama API. It was not run against a real model, OpenAI or Anthropic, so the quality of the results with real models is untested.

# LightAgent

<p align="center">
  <img src="assets/banner.png" alt="LightAgent Banner" width="100%">
</p>

**LightAgent** is a small terminal coding agent written in Rust. It asks a language model for the next action, runs it (shell command, file write, file read) and records every step in a local `.agent/state.jsonl` file so that a task can be resumed later.

## Türkçe

LightAgent, Rust ile yazılmış küçük bir terminal kodlama ajanıdır. Dil modelinden bir sonraki adımı ister (kabuk komutu çalıştırma, dosya yazma, dosya okuma veya bitirme), adımı uygular ve her adımı yerel `.agent/state.jsonl` dosyasına kaydeder; böylece görev sonradan `resume` ile devam ettirilebilir. Varsayılan olarak yerel Ollama'ya bağlanır; `OPENAI_API_KEY` veya `ANTHROPIC_API_KEY` tanımlıysa ilgili servisi kullanır. Her çalıştırmada en fazla 15 adım atar. `--yolo` verilmedikçe komut çalıştırma ve dosya yazma öncesinde onay ister. Proje beta aşamasındadır; gerçek modellerle henüz test edilmemiştir.

---

## Key Features

* **Single binary**: written in Rust, no Python environment or `node_modules` needed to run it.
* **State file (`.agent/state.jsonl`)**: every step, command output and file operation is appended to the file. `lightagent resume` continues from it.
* **Git drift warning**: `resume` compares the commit recorded when the task started with the current commit and warns if they differ. It does not change anything in git.
* **Confirmation by default**: without `--yolo`, the agent asks `y/N` before each command and file write. Reading files never asks.
* **YOLO mode**: with `--yolo` it runs commands and writes files without asking. The loop stops when the model returns `finish`, when the model request fails or returns an unusable action, or after 15 steps.
* **Model providers**: local Ollama by default (`OLLAMA_URL`, default `http://localhost:11434`; `OLLAMA_MODEL`, default `qwen2.5-coder`), OpenAI when `OPENAI_API_KEY` is set (`OPENAI_MODEL`, default `gpt-4o-mini`), Anthropic when `ANTHROPIC_API_KEY` is set (`ANTHROPIC_MODEL`). If both keys are set, Anthropic is used. Prompts, including file contents and command output, are sent to the selected provider.

### Safety

LightAgent executes shell commands written by a language model, with your user permissions and without a sandbox. Run it in a disposable directory or container, use version control, and avoid `--yolo` on anything you cannot restore.

---

## Installation & Quickstart

### 1. Build from Source (Cargo required)
```bash
git clone https://github.com/Ankara-Software/lightagent.git
cd lightagent
cargo build --release

# Binary will be available at target/release/lightagent
```
Or: `cargo install --git https://github.com/Ankara-Software/lightagent`.

### 2. Direct Binary Download
Download the binary for your platform from the [latest release](https://github.com/Ankara-Software/lightagent/releases/latest): `lightagent-win32-x64.exe`, `lightagent-linux-x64`, `lightagent-linux-arm64`, `lightagent-darwin-x64`, `lightagent-darwin-arm64`.

The package is not published on npm. The `bin/cli.js` wrapper in this repository downloads the release binary and is not available through `npx` yet.

### Quickstart
```bash
# Start a new task (asks before each command and file write)
lightagent start "Fix the failing test in tests/auth.rs"

# Same, without confirmation prompts
lightagent start "Fix the failing test in tests/auth.rs" --yolo

# Resume an interrupted task
lightagent resume --yolo
```

---

## CLI Reference

```
Usage: lightagent <COMMAND>

Commands:
  start    Start a new engineering task
  resume   Resume the active task from the local state
  status   Show the status of the current task and git alignment
  clear    Clear the current active task state
  help     Print this message or the help of the given subcommand(s)
```

`start` overwrites any existing task state in the current directory.

### Resuming & Git Alignment
When running `lightagent resume`, the tool compares the Git commit hash at task initialization with the current HEAD. If a branch switch or code modification has occurred, it alerts you of code drift:

```bash
$ lightagent resume
Resuming active task...
Warning: Git commit drift detected! Current commit differs from task start commit.
```

---

## Development

```bash
cargo build
cargo test
```

See [CONTRIBUTING.md](CONTRIBUTING.md), [SECURITY.md](SECURITY.md) and [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).

## License

MIT. Copyright (c) 2026 Anılcan Kara and Ankara Yazılım contributors.
