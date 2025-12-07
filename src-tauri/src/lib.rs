// Re-export modules for library usage
pub mod utils;
pub mod converter;

pub use converter::{
    extract_midi_notes, allocate_voices_smart, generate_mml_final,
    crop_voice_to_limit, Note, TPB, GRID_SIZE,
};