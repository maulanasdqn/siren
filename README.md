# siren

Local-first **speech-to-text and text-to-speech**, GPU-accelerated, Indonesian by default.

Runs entirely on-device — no cloud calls. After a model is fetched once from the
HuggingFace Hub it is cached locally and works fully offline.

## Why this stack

- **STT: Whisper** in **[Candle](https://github.com/huggingface/candle)** (pure-Rust ML,
  GPU via Metal/CUDA). Small Indonesian fine-tunes, e.g.
  [`cahya/whisper-tiny-id`](https://huggingface.co/cahya/whisper-tiny-id). Faster/smaller
  English-first models (Moonshine, streaming Zipformer) don't cover Indonesian, so they
  were ruled out.
- **TTS: Piper** ([`id_ID-news_tts-medium`](https://huggingface.co/rhasspy/piper-voices),
  a natural **female** voice, F0 ≈ 256 Hz) via the [`piper-rs`](https://crates.io/crates/piper-rs)
  crate. There is no pure-Rust female Indonesian VITS, and `facebook/mms-tts-ind` has only a
  single fixed (male) voice — so the TTS path uses Piper, which links **onnxruntime + espeak-ng
  (C/C++)**. It is isolated in the `siren-tts` adapter so the rest of the workspace stays pure Rust.

The STT loader reads either `model.safetensors` or PyTorch `pytorch_model.bin` (candle
`from_pth`) and falls back to the shared Whisper tokenizer, so fine-tune repos that omit those
files still load with no Python conversion step. The TTS path is validated by **round-trip**:
synthesized audio fed back through the STT recovers the input text exactly.

## Usage

```bash
# Batch: transcribe a file (default model: cahya/whisper-tiny-id, language: id)
cargo run --release -- transcribe audio.wav

# Higher accuracy with a larger Indonesian model
cargo run --release -- --model cahya/whisper-small-id transcribe audio.wav

# Any Whisper model / language also works
cargo run --release -- --model openai/whisper-base --language en transcribe speech.mp3

# Real-time microphone streaming (VAD-segmented)
cargo run --release -- listen

# Text-to-speech (Indonesian, female voice): write speech.wav, optionally play it live
cargo run --release -- speak "selamat pagi apa kabar" -o out.wav
cargo run --release -- speak "halo dunia" --play
```

NVIDIA instead of Apple Silicon:

```bash
cargo run --release -p siren-cli --no-default-features --features cuda -- listen
```

## Architecture

Hexagonal / Clean Architecture — a Cargo workspace of small crates, dependencies
pointing inward toward the domain. Each source file is comment-free and under 200 LOC.

```
              ┌──────────────── siren-cli (composition root) ───────────────┐
              │  wires adapters into use cases                              │
              └───────────────┬─────────────────────────────┬──────────────┘
                              │                             │
                     siren-application            (implements ports)
                   TranscribeFile / Stream                 │
                              │                    siren-asr   (Candle Whisper)
                        siren-domain               siren-audio (Symphonia)
             entities · value objects · ports      siren-vad   (energy VAD)
             DSP services · DomainError             siren-mic   (cpal)
```

| Crate | Layer | Role |
|-------|-------|------|
| `siren-domain` | Domain | `AudioSamples`, `Waveform`, `Transcript`, `Language`, `ModelId`, `Utterance`; DSP services; **ports** (`SpeechRecognizer`, `SpeechSynthesizer`, `AudioDecoder`, `VoiceActivityDetector`, `MicrophoneSource`, `WaveformWriter`, `SpeakerSink`); `DomainError`. No infra deps. |
| `siren-application` | Application | Use cases `TranscribeFile`, `TranscribeStream`, `SynthesizeText` orchestrating ports. |
| `siren-asr` | Adapter | `WhisperRecognizer` — Candle Whisper (safetensors *or* `.bin`), greedy decode. |
| `siren-tts` | Adapter | `PiperSynthesizer` — Piper female `id_ID` voice via `piper-rs` (onnxruntime + espeak-ng, C/C++). |
| `siren-audio` | Adapter | `SymphoniaDecoder` (decode→16 kHz) + `WavWriter` (pure-Rust RIFF). |
| `siren-vad` | Adapter | `EnergyVad` — pure-Rust utterance segmentation. |
| `siren-mic` | Adapter | `CpalMicrophone` — mic capture yielding 16 kHz mono blocks. |
| `siren-speaker` | Adapter | `CpalSpeaker` — playback to the default output device. |
| `siren-cli` | Driver | clap CLI; composition root injecting adapters. |

Data flow:

```
file ──► SymphoniaDecoder ──────────────────────► AudioSamples ─┐
mic  ──► CpalMicrophone ──► EnergyVad (segment) ──► Utterance ───┤
                                                                 ▼
                                      WhisperRecognizer (Candle, GPU) ──► Transcript

text ──► PiperSynthesizer (Piper, female) ──► Waveform ──► WavWriter / CpalSpeaker
```

## Known limitations / next steps

- **Streaming latency** — Whisper decodes a whole utterance at once, so "real-time"
  means low-latency *per utterance* (finalized on trailing silence), not word-by-word.
- **VAD in noise** — the energy gate is simple; Silero VAD is more robust but is ONNX/C.
- **Isolated single words** are hard for Whisper (no context); continuous speech is far
  more accurate. Larger models (`small-id`, `medium-id`) improve accuracy at higher cost.
- **Resampling** uses linear interpolation; a windowed-sinc resampler (`rubato`) is a
  quality upgrade.
- Quantized (q4k/q8) weights for lower memory/faster GPU are a planned addition.
