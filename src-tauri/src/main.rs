#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod utils;
mod converter;

use converter::{
    extract_midi_notes, allocate_voices_smart, generate_mml_final,
    crop_voice_to_limit, Note, TPB,
};

#[derive(Debug, Serialize, Deserialize)]
struct ConversionOptions {
    mode: String, // "normal" or "instrument"
    char_limit: usize,
    compress_mode: bool, // true: 글자수 우선 (점음표/타이 최소화), false: 정확도 우선
}

#[derive(Debug, Serialize, Deserialize)]
struct VoiceResult {
    name: String,
    content: String,
    char_count: usize,
    note_count: usize,
    duration: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct ConversionResult {
    success: bool,
    voices: Vec<VoiceResult>,
    error: Option<String>,
    bpm: u32,
    total_notes: usize,
}

#[tauri::command]
fn convert_midi(midi_data: Vec<u8>, options: ConversionOptions) -> ConversionResult {
    match convert_midi_internal(&midi_data, &options) {
        Ok(result) => result,
        Err(e) => ConversionResult {
            success: false,
            voices: vec![],
            error: Some(e),
            bpm: 0,
            total_notes: 0,
        },
    }
}

fn convert_midi_internal(
    midi_data: &[u8],
    options: &ConversionOptions,
) -> Result<ConversionResult, String> {
    let (notes, bpm) = extract_midi_notes(midi_data, 24)?;
    let total_notes = notes.len();

    let voices = if options.mode == "instrument" {
        // 악기별 모드
        convert_by_instrument(notes, bpm, options.char_limit, options.compress_mode)?
    } else {
        // 일반 모드 (피치별)
        convert_by_pitch(notes, bpm, options.char_limit, options.compress_mode)?
    };

    Ok(ConversionResult {
        success: true,
        voices,
        error: None,
        bpm,
        total_notes,
    })
}

fn convert_by_pitch(
    notes: Vec<Note>,
    bpm: u32,
    char_limit: usize,
    compress_mode: bool,
) -> Result<Vec<VoiceResult>, String> {
    let voices = allocate_voices_smart(notes);
    
    // 1단계: 모든 voice를 글자수 제한으로 자르기
    let mut cropped_voices = Vec::new();
    for voice in voices.iter() {
        if voice.is_empty() {
            continue;
        }
        let cropped = crop_voice_to_limit(voice, bpm, char_limit, compress_mode);
        if !cropped.is_empty() {
            cropped_voices.push(cropped);
        }
    }
    
    if cropped_voices.is_empty() {
        return Ok(Vec::new());
    }
    
    // 2단계: 가장 짧은 voice의 끝 시간 찾기
    let min_end_time = cropped_voices.iter()
        .map(|v| v.last().unwrap().end)
        .min()
        .unwrap();
    
    // 3단계: 모든 voice를 가장 짧은 길이로 재자르기
    let mut results = Vec::new();
    for (idx, voice) in cropped_voices.iter().enumerate() {
        let final_voice: Vec<Note> = voice.iter()
            .filter(|n| n.start < min_end_time)
            .cloned()
            .collect();
        
        if final_voice.is_empty() {
            continue;
        }

        let first_note = final_voice[0].note;
        let mut start_octave = (first_note as i32 / 12) - 1;
        start_octave = start_octave.max(2).min(6);

        let mml_code = generate_mml_final(&final_voice, bpm, start_octave, compress_mode);
        let note_count = final_voice.len();
        let end_time = min_end_time as f64 / TPB as f64 / 2.0;

        let name = if idx == 0 {
            "멜로디".to_string()
        } else {
            format!("화음{}", idx)
        };

        results.push(VoiceResult {
            name,
            content: mml_code.clone(),
            char_count: mml_code.len(),
            note_count,
            duration: end_time,
        });
    }

    Ok(results)
}

fn convert_by_instrument(
    notes: Vec<Note>,
    bpm: u32,
    char_limit: usize,
    compress_mode: bool,
) -> Result<Vec<VoiceResult>, String> {
    let mut instrument_groups: HashMap<String, Vec<Note>> = HashMap::new();
    for note in notes {
        instrument_groups
            .entry(note.instrument.clone())
            .or_insert_with(Vec::new)
            .push(note);
    }

    let mut instrument_names: Vec<String> = instrument_groups.keys().cloned().collect();
    instrument_names.sort();

    // 1단계: 모든 악기의 voice를 글자수 제한으로 자르기
    let mut all_cropped_voices = Vec::new();
    let mut voice_instrument_map = Vec::new();
    
    for instrument_name in &instrument_names {
        let instrument_notes = instrument_groups.get(instrument_name).unwrap();
        let voices = allocate_voices_smart(instrument_notes.clone());

        for voice in voices.iter() {
            if voice.is_empty() {
                continue;
            }

            let cropped = crop_voice_to_limit(voice, bpm, char_limit, compress_mode);
            if !cropped.is_empty() {
                all_cropped_voices.push(cropped);
                voice_instrument_map.push(instrument_name.clone());
            }
        }
    }
    
    if all_cropped_voices.is_empty() {
        return Ok(Vec::new());
    }
    
    // 2단계: 가장 짧은 voice의 끝 시간 찾기
    let min_end_time = all_cropped_voices.iter()
        .map(|v| v.last().unwrap().end)
        .min()
        .unwrap();
    
    // 3단계: 모든 voice를 가장 짧은 길이로 재자르기
    let mut results = Vec::new();
    for (idx, (voice, instrument_name)) in all_cropped_voices.iter().zip(voice_instrument_map.iter()).enumerate() {
        let final_voice: Vec<Note> = voice.iter()
            .filter(|n| n.start < min_end_time)
            .cloned()
            .collect();
        
        if final_voice.is_empty() {
            continue;
        }

        let first_note = final_voice[0].note;
        let mut start_octave = (first_note as i32 / 12) - 1;
        start_octave = start_octave.max(2).min(6);

        let mml_code = generate_mml_final(&final_voice, bpm, start_octave, compress_mode);
        let note_count = final_voice.len();
        let end_time = min_end_time as f64 / TPB as f64 / 2.0;

        let name = if idx == 0 {
            format!("멜로디 ({})", instrument_name)
        } else {
            format!("화음{} ({})", idx, instrument_name)
        };

        results.push(VoiceResult {
            name,
            content: mml_code.clone(),
            char_count: mml_code.len(),
            note_count,
            duration: end_time,
        });
    }

    Ok(results)
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![convert_midi])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}