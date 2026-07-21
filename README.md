# siren

A **Gemini-powered AI voice engine** — text-to-speech, speech-to-text, and realtime
voice conversation — with an **offline local fallback**.

By default every command runs against Google's Gemini voice models (natural,
low-latency, streaming). Pass `--local` on any command to fall back to the on-device
Whisper + Piper stack that needs no network and no API key.

## Setup

The Gemini engine needs an API key:

```bash
export GEMINI_API_KEY=...
```

## Usage

```bash
# Text-to-speech — streams a natural voice from Gemini and plays it live
cargo run --release -- speak "Hey, this is siren speaking in real time."
cargo run --release -- speak "Different narrator." --voice Orus

# Save synthesized speech to a WAV (optionally also play it)
cargo run --release -- speak "round trip test" -o out.wav
cargo run --release -- speak "round trip test" -o out.wav --play

# Speech-to-text — transcribe a file with Gemini
cargo run --release -- transcribe audio.wav
cargo run --release -- --language en transcribe speech.mp3

# Realtime voice conversation — full-duplex mic in / voice out over the Live API
cargo run --release -- converse
cargo run --release -- converse --voice Charon --system "You are a terse assistant."

# Microphone streaming transcription (VAD-segmented)
cargo run --release -- listen
```

Gemini voices: default `Charon` (natural male); others include `Orus`, `Fenrir`,
`Iapetus`, `Enceladus`, `Puck`, `Aoede`, `Kore`. TTS streams from
`gemini-3.1-flash-tts-preview`; `converse` uses the bidirectional Live model
`gemini-3.1-flash-live-preview`; STT uses `gemini-flash-latest`. Model names are
constants in the `siren-gemini` crate.

### Offline fallback (`--local`)

Add `--local` to run fully on-device — no network, no API key. Models are fetched once
from the HuggingFace Hub and cached.

```bash
# Local STT: Whisper (Candle, GPU) — Indonesian by default
cargo run --release -- --local transcribe audio.wav
cargo run --release -- --local --model cahya/whisper-small-id transcribe audio.wav
cargo run --release -- --local listen

# Local TTS: Piper Indonesian female voice → WAV (+ optional --play)
cargo run --release -- --local speak "selamat pagi apa kabar" -o out.wav
```

- **Local STT: Whisper** in **[Candle](https://github.com/huggingface/candle)** (pure-Rust ML,
  GPU via Metal/CUDA), default [`cahya/whisper-tiny-id`](https://huggingface.co/cahya/whisper-tiny-id).
  The loader reads `model.safetensors` *or* PyTorch `pytorch_model.bin` (candle `from_pth`) and falls
  back to the shared Whisper tokenizer, so fine-tune repos that omit those files still load with no
  Python conversion step.
- **Local TTS: Piper** ([`id_ID-news_tts-medium`](https://huggingface.co/rhasspy/piper-voices),
  a natural **female** voice, F0 ≈ 256 Hz) via [`piper-rs`](https://crates.io/crates/piper-rs), which
  links **onnxruntime + espeak-ng (C/C++)** — isolated in the `siren-tts` adapter so the rest of the
  workspace stays pure Rust.

NVIDIA instead of Apple Silicon (local Whisper):

```bash
cargo run --release -p siren-cli --no-default-features --features cuda -- --local listen
```

## Use with Claude Code & other AI agents (MCP)

siren ships an **[MCP](https://modelcontextprotocol.io) server** (`siren-mcp`) so any
MCP-capable agent — Claude Code, Cursor, Windsurf, Zed, … — can give itself a voice and
transcribe audio. It speaks JSON-RPC 2.0 over stdio and exposes three tools:

| Tool | What it does |
|------|--------------|
| `siren_speak` | Speak text aloud (Gemini voice), or save a WAV with `output`. `local: true` uses the offline Piper voice. |
| `siren_transcribe` | Transcribe an audio file to text (Gemini; `local: true` uses offline Whisper). |
| `siren_list_voices` | List the available Gemini voice names. |

Build the server binary once:

```bash
cargo build --release -p siren-mcp   # → target/release/siren-mcp
```

**Claude Code** — one command (use an absolute path to the binary):

```bash
claude mcp add siren --env GEMINI_API_KEY=your_key -- /abs/path/to/siren/target/release/siren-mcp
```

**Any MCP client** — add this to the client's server config (e.g. Claude Code's
`.mcp.json`, Cursor's `mcp.json`); a copy lives at [`examples/mcp.json`](examples/mcp.json):

```json
{
  "mcpServers": {
    "siren": {
      "command": "/abs/path/to/siren/target/release/siren-mcp",
      "env": { "GEMINI_API_KEY": "your_key" }
    }
  }
}
```

Then just ask the agent to *"say hello out loud"* or *"transcribe recording.wav"*.
`GEMINI_API_KEY` is only needed for the Gemini path — pass `local: true` and it runs fully
offline. The server writes **only** JSON-RPC to stdout (all logs go to stderr), so it drops
into any client cleanly.

## Architecture

Hexagonal / Clean Architecture — a Cargo workspace of small crates, dependencies
pointing inward toward the domain. Each source file is comment-free and under 200 LOC.
Gemini and local engines implement the **same domain ports**, so the CLI swaps between
them by injecting a different adapter.

```
              ┌──────────────── siren-cli (composition root) ───────────────┐
              │  --local? inject local adapter : inject Gemini adapter       │
              └───────────────┬─────────────────────────────┬──────────────┘
                              │                             │
                     siren-application            (implements ports)
        Transcribe / Stream / Synthesize / Converse         │
                              │             Gemini: siren-asr-gemini · siren-tts-gemini · siren-live-gemini
                        siren-domain        Local:  siren-asr (Candle) · siren-tts (Piper)
             entities · value objects · ports      shared: siren-gemini · siren-audio · siren-vad
             DSP services · DomainError                     siren-mic · siren-speaker
```

| Crate | Layer | Role |
|-------|-------|------|
| `siren-domain` | Domain | `AudioSamples`, `Waveform`, `Transcript`, `Language`, `ModelId`, `Utterance`; DSP services; **ports** (`SpeechRecognizer`, `SpeechSynthesizer`, `StreamingSynthesizer`, `LiveVoiceAgent`, `AudioDecoder`, `VoiceActivityDetector`, `MicrophoneSource`, `WaveformWriter`, `SpeakerSink`, `StreamingSpeaker`); `DomainError`. No infra deps. |
| `siren-application` | Application | Use cases `TranscribeFile`, `TranscribeStream`, `SynthesizeText`, `StreamSpeech`, `Converse` orchestrating ports. |
| `siren-gemini` | Adapter (shared) | Gemini credentials (`GEMINI_API_KEY`), REST/WS endpoints, default model & voice constants. |
| `siren-tts-gemini` | Adapter | `GeminiSynthesizer` — Gemini TTS; streaming (`StreamingSynthesizer`) **and** one-shot (`SpeechSynthesizer`, for file output). |
| `siren-asr-gemini` | Adapter | `GeminiRecognizer` — Gemini STT via `generateContent` with inline WAV audio. |
| `siren-live-gemini` | Adapter | `GeminiLiveAgent` — bidirectional Live API over a WebSocket (`tokio-tungstenite` + rustls); mic PCM in, voice PCM out. |
| `siren-asr` | Adapter | `WhisperRecognizer` — local Candle Whisper (safetensors *or* `.bin`), greedy decode. |
| `siren-tts` | Adapter | `PiperSynthesizer` — local Piper female `id_ID` voice via `piper-rs` (onnxruntime + espeak-ng, C/C++). |
| `siren-audio` | Adapter | `SymphoniaDecoder` (decode→16 kHz), `WavWriter` + in-memory `wav_bytes` (pure-Rust RIFF). |
| `siren-vad` | Adapter | `EnergyVad` — pure-Rust utterance segmentation. |
| `siren-mic` | Adapter | `CpalMicrophone` — mic capture yielding `Send` 16 kHz mono blocks. |
| `siren-speaker` | Adapter | `CpalSpeaker` (one-shot) + `CpalStreamingSpeaker` (plays chunks as they arrive). |
| `siren-cli` | Driver | clap CLI; composition root injecting Gemini or local adapters. |
| `siren-mcp` | Driver | MCP stdio server (JSON-RPC 2.0) exposing `speak`/`transcribe`/`list_voices` to AI agents. |

Data flow:

```
text ──► GeminiSynthesizer ──► Waveform chunks ──► CpalStreamingSpeaker (live)
text ──► GeminiSynthesizer ──► Waveform ─────────► WavWriter (-o file)
file/mic ──► AudioSamples ──► GeminiRecognizer ──► Transcript
mic ⇄ GeminiLiveAgent (Live WebSocket) ⇄ Waveform chunks ──► CpalStreamingSpeaker

--local:
file ──► SymphoniaDecoder ─┐
mic  ──► CpalMicrophone ──► EnergyVad ──► WhisperRecognizer (Candle, GPU) ──► Transcript
text ──► PiperSynthesizer (female) ──► Waveform ──► WavWriter / CpalSpeaker
```

## Notes

- **Realtime full-duplex** — `converse` plays Gemini's voice while streaming your mic to it.
  `siren-mic` parks the `!Send` cpal stream on its own thread and hands out a `Send` block
  iterator, so the same mic feeds both `listen` and the Live WebSocket session.
- **Local streaming latency** — Whisper decodes a whole utterance at once, so `--local listen`
  is low-latency *per utterance* (finalized on trailing silence), not word-by-word.
- **VAD in noise** — the local energy gate is simple; Silero VAD is more robust but is ONNX/C.
- **Resampling** uses linear interpolation; a windowed-sinc resampler (`rubato`) is a quality upgrade.
