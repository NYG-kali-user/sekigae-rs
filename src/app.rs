use leptos::*;
use leptos::{ev::MouseEvent, prelude::*, reactive::spawn_local};
use serde_wasm_bindgen::{from_value, to_value};
use shared::AnnealParam;
use wasm_bindgen::prelude::*;
const T0: f64 = 100.0;
const TMIN: f64 = 0.1;
const ALPHA: f64 = 0.99;
const MAXITERS: usize = 10000;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}
#[component]
pub fn App() -> impl IntoView {
    let matrix = RwSignal::new(Vec::<Vec<f64>>::new());
    let version = RwSignal::new(0);

    let load_matrix = move |ev: MouseEvent| {
        ev.prevent_default();
        let matrix = matrix.clone();
        spawn_local(async move {
            let empty = JsValue::NULL;
            let js_val = invoke("open_csv_file", empty).await;
            match from_value::<Vec<Vec<f64>>>(js_val) {
                Ok(data) => {
                    matrix.set(data);
                }
                Err(e) => web_sys::console::error_1(&format!("データ変換失敗: {:?}", e).into()),
            }
        });
    };
    let save_matrix = move |ev: MouseEvent| {
        ev.prevent_default();
        spawn_local(async move {
            let args = to_value(&serde_json::json!({"matrix": matrix.get()})).unwrap();
            invoke("save_csv_file", args).await;
        });
    };
    let shuffle_matrix = move |ev: MouseEvent| {
        let target = vec![2, 18, 22, 24, 28, 30, 36, 38, 31];
        ev.prevent_default();
        spawn_local(async move {
            let args = matrix.with_untracked(|m| {
                let anneal_param = AnnealParam {
                    grid: m
                        .iter()
                        .map(|row| row.iter().map(|&x| x as i32).collect())
                        .collect(),
                    target: target,
                    t0: T0,
                    t_min: TMIN,
                    alpha: ALPHA,
                    max_iters: MAXITERS,
                };
                to_value(&serde_json::json!({"param": anneal_param})).unwrap()
            });
            let js_val = invoke("simulated_annealing", args).await;
            match from_value::<Vec<Vec<f64>>>(js_val) {
                Ok(new_matrix) => {
                    matrix.update(|m| *m = new_matrix);
                    version.update(|v| *v += 1);
                }
                Err(e) => web_sys::console::error_1(&format!("結果のデコード失敗: {:?}", e).into()),
            }
        });
    };

    let exit_app = move |ev: MouseEvent| {
        ev.prevent_default();
        spawn_local(async move {
            let args = JsValue::NULL;
            invoke("exit_app", args).await;
        });
    };

    view! {
        <div class="container">
            <p>"現在の席順"</p>
            <table style="margin-top: 1em;">
                <For each=move || {
                    let v = version.get();
                    matrix.with(|m| m.iter().enumerate().map(|(i, row)| (i, row.clone(), v)).collect::<Vec<_>>())
                }
                key=|(i, _row, v)| (*i, *v)
                let:((_i, row, _v))>
                    <tr>
                        <For each=move || row.clone().into_iter().enumerate()
                        key=|(j, _cell)| *j
                        let:((_j, cell))>
                            <td>{cell.to_string()}</td>
                        </For>
                    </tr>
                </For>
            </table>
            <button on:click=load_matrix>"ファイル読み込み"</button>
            <button on:click=shuffle_matrix>"実行"</button>
            <button on:click=save_matrix>"今の状態をセーブする"</button>
            <button on:click=exit_app id="quit-button">"終了"</button>
        </div>
    }
}
