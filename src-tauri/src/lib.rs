// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

use csv::{Reader, Writer};
use std::{
    fs::{File, read_dir},
    path::*,
};
use tauri::{AppHandle, Manager, Window};
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
async fn open_csv_file(window: Window) -> Result<Vec<Vec<f64>>, String> {
    let file_path = window
        .app_handle()
        .dialog()
        .file()
        .blocking_pick_file()
        .ok_or("ファイルが選択されませんでした")?;
    let pathbuf = match file_path.into_path() {
        Ok(p) => p,
        Err(p) => return Err(format!("ファイルパスに非ず: {}", p)),
    };
    let file = File::open(&pathbuf).map_err(|e| e.to_string())?;
    let mut reader = Reader::from_reader(file);

    let mut matrix_data = Vec::new();
    for result in reader.records() {
        let record = result.map_err(|e| e.to_string())?;
        let row: Vec<f64> = record
            .iter()
            .map(|field| field.trim().parse::<f64>())
            .collect::<Result<_, _>>()
            .map_err(|e| e.to_string())?;
        matrix_data.push(row);
    }

    Ok(matrix_data)
}

#[tauri::command]
fn save_csv_file(app: AppHandle, matrix: Vec<Vec<f64>>) -> Result<(), String> {
    let base_dir = app
        .path()
        .document_dir()
        .map_err(|e| format!("ディレクトリの取得に失敗: {}", e))?;
    let pathbuf = base_dir.join("csv");
    std::fs::create_dir_all(&pathbuf).map_err(|e| format!("ディレクトリ作成失敗: {}", e))?;

    let count = file_counter(&pathbuf).map_err(|e| format!("ファイルカウント失敗: {}", e))?;
    let file_path = pathbuf.join(format!("{}.csv", count));
    let file = File::create(&file_path).map_err(|e| format!("ファイル作成失敗: {}", e))?;

    let mut wtr = Writer::from_writer(file);

    for row in matrix {
        wtr.write_record(row.iter().map(|v| v.to_string()))
            .map_err(|e| format!("csv書き込み失敗: {}", e))?;
    }

    wtr.flush().map_err(|e| format!("フラッシュ失敗: {}", e))?;
    Ok(())
}
fn file_counter(path: &PathBuf) -> Result<u32, std::io::Error> {
    let mut max_num = 0;

    for entry in read_dir(path)? {
        let entry = entry?;
        if let Some(file_name) = entry.file_name().to_str() {
            if let Some(stem) = file_name.strip_suffix(".csv") {
                if let Ok(num) = stem.parse::<u32>() {
                    if num >= max_num {
                        max_num = num + 1;
                    }
                }
            }
        }
    }
    Ok(max_num)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![open_csv_file, save_csv_file])
        .run(tauri::generate_context!())
        .expect("error while running this app");
}
