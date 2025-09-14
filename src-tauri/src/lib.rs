// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use csv::{Reader, Writer};
use rand::Rng;
use std::{
    fs::{File, read_dir},
    path::*,
};
use tauri::{AppHandle, Manager, Window};
use tauri_plugin_dialog::DialogExt;
type Grid = Vec<Vec<i32>>;

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
fn manhattan(a: (usize, usize), b: (usize, usize)) -> i32 {
    (a.0 as i32 - b.0 as i32).abs() + (a.1 as i32 - b.1 as i32).abs()
}
fn cost(grid: &Grid, target: &Vec<i32>) -> i32 {
    let mut pos = std::collections::HashMap::new();
    for (r, row) in grid.iter().enumerate() {
        for (c, &val) in row.iter().enumerate() {
            if target.contains(&val) {
                pos.insert(val, (r, c));
            }
        }
    }
    let mut sum = 0;
    for i in 0..target.len() {
        for j in i + 1..target.len() {
            let x = target[i];
            let y = target[j];
            let p = pos.get(&x).unwrap();
            let q = pos.get(&y).unwrap();
            sum += manhattan(*p, *q);
        }
    }
    sum
}
#[tauri::command]
fn simulated_annealing(
    grid: Grid,
    target: Vec<i32>,
    t0: f64,
    t_min: f64,
    alpha: f64,
    max_iters: usize,
) -> Vec<Vec<f64>> {
    let mut rng = rand::rng();

    let mut current = grid.clone();
    let mut current_cost = cost(&current, &target) as f64;
    let mut best = current.clone();
    let mut best_cost = current_cost;

    let rows = current.len();
    let cols = current[0].len();

    let mut t = t0;

    for _ in 0..max_iters {
        if t < t_min {
            break;
        }

        let mut new_grid = current.clone();
        let (r1, c1) = (rng.random_range(0..rows), rng.random_range(0..cols));
        let (r2, c2) = (rng.random_range(0..rows), rng.random_range(0..cols));
        let temp = new_grid[r1][c1];
        new_grid[r1][c1] = new_grid[r2][c2];
        new_grid[r2][c2] = temp;

        let new_cost_val = cost(&new_grid, &target) as f64;
        let delta = new_cost_val - current_cost;

        if delta <= 0.0 || rng.random::<f64>() < (-delta / t).exp() {
            current = new_grid;
            current_cost = new_cost_val;

            if current_cost < best_cost {
                best = current.clone();
                best_cost = current_cost;
            }
        }
        t = t * alpha;
    }
    best.into_iter()
        .map(|row| row.into_iter().map(|v| v as f64).collect())
        .collect()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            open_csv_file,
            save_csv_file,
            simulated_annealing
        ])
        .run(tauri::generate_context!())
        .expect("error while running this app");
}
