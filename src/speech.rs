
use std::process::Command;
use std::fs;

pub fn transcribe_audio(audio_path: &str) -> Result<String, String> {
    let whisper_exe =
        r"D:\VoiceOperator\tools\whisper.cpp\build\bin\Release\whisper-cli.exe";

    let model_path =
        r"D:\VoiceOperator\models\ggml-base.en.bin";

    let output_path = "data/transcription";

    let result = Command::new(whisper_exe)
        .arg("-m")
        .arg(model_path)
        .arg("-f")
        .arg(audio_path)
        .arg("-otxt")
        .arg("-of")
        .arg(output_path)
        .output()
        .map_err(|error| format!("Failed to run Whisper: {error}"))?;

    if !result.status.success() {
        return Err(format!(
            "Whisper failed: {}",
            String::from_utf8_lossy(&result.stderr)
        ));
    }

    let transcript_file = format!("{output_path}.txt");

    let transcript = fs::read_to_string(&transcript_file)
        .map_err(|error| format!("Failed to read transcript: {error}"))?;

    Ok(transcript.trim().to_string())
}
