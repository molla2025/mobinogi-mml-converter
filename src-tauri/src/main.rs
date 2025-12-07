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
    let mut temp_results = Vec::new();

    // 먼저 모든 voice를 생성하고 최소 end_tick 찾기
    for voice in voices.iter() {
        if voice.is_empty() {
            continue;
        }

        let cropped = crop_voice_to_limit(voice, bpm, char_limit, compress_mode);
        if cropped.is_empty() {
            continue;
        }

        temp_results.push(cropped);
    }

    if temp_results.is_empty() {
        return Ok(Vec::new());
    }

    // 가장 짧은 end_tick 찾기
    let min_end_tick = temp_results.iter()
        .filter_map(|v| v.last().map(|n| n.end))
        .min()
        .unwrap_or(0);

    // 모든 voice를 min_end_tick으로 다시 크롭하고 MML 생성
    let mut results = Vec::new();
    for (idx, voice_notes) in temp_results.iter().enumerate() {
        // min_end_tick 이하의 노트만 남기기
        let synced_notes: Vec<Note> = voice_notes.iter()
            .filter(|n| n.start < min_end_tick)
            .map(|n| {
                let mut note = n.clone();
                if note.end > min_end_tick {
                    note.end = min_end_tick;
                    note.duration = note.end - note.start;
                }
                note
            })
            .collect();

        if synced_notes.is_empty() {
            continue;
        }

        let first_note = synced_notes[0].note;
        let mut start_octave = (first_note as i32 / 12) - 1;
        start_octave = start_octave.max(2).min(6);

        let mml_code = generate_mml_final(&synced_notes, bpm, start_octave, compress_mode);
        let note_count = synced_notes.len();
        let end_time = min_end_tick as f64 / TPB as f64 / 2.0;

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

    let mut temp_results = Vec::new();
    let mut instrument_names: Vec<String> = instrument_groups.keys().cloned().collect();
    instrument_names.sort();

    // 먼저 모든 voice를 생성하고 최소 end_tick 찾기
    for instrument_name in &instrument_names {
        let instrument_notes = instrument_groups.get(instrument_name).unwrap();
        let voices = allocate_voices_smart(instrument_notes.clone());

        for voice in voices.iter() {
            if voice.is_empty() {
                continue;
            }

            let cropped = crop_voice_to_limit(voice, bpm, char_limit, compress_mode);
            if cropped.is_empty() {
                continue;
            }

            temp_results.push((instrument_name.clone(), cropped));
        }
    }

    if temp_results.is_empty() {
        return Ok(Vec::new());
    }

    // 가장 짧은 end_tick 찾기
    let min_end_tick = temp_results.iter()
        .filter_map(|(_, v)| v.last().map(|n| n.end))
        .min()
        .unwrap_or(0);

    // 모든 voice를 min_end_tick으로 다시 크롭하고 MML 생성
    let mut results = Vec::new();
    let mut global_voice_idx = 0;

    for (inst_name, voice_notes) in temp_results.iter() {
        // min_end_tick 이하의 노트만 남기기
        let synced_notes: Vec<Note> = voice_notes.iter()
            .filter(|n| n.start < min_end_tick)
            .map(|n| {
                let mut note = n.clone();
                if note.end > min_end_tick {
                    note.end = min_end_tick;
                    note.duration = note.end - note.start;
                }
                note
            })
            .collect();

        if synced_notes.is_empty() {
            continue;
        }

        let first_note = synced_notes[0].note;
        let mut start_octave = (first_note as i32 / 12) - 1;
        start_octave = start_octave.max(2).min(6);

        let mml_code = generate_mml_final(&synced_notes, bpm, start_octave, compress_mode);
        let note_count = synced_notes.len();
        let end_time = min_end_tick as f64 / TPB as f64 / 2.0;

        let name = if global_voice_idx == 0 {
            format!("멜로디 ({})", inst_name)
        } else {
            format!("화음{} ({})", global_voice_idx, inst_name)
        };

        results.push(VoiceResult {
            name,
            content: mml_code.clone(),
            char_count: mml_code.len(),
            note_count,
            duration: end_time,
        });

        global_voice_idx += 1;
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