use leptos::*;
use leptos::{ev::MouseEvent, prelude::*, reactive::spawn_local};
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[component]
pub fn App() -> impl IntoView {
    let matrix = RwSignal::new(Vec::<Vec<f64>>::new());

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
    view! {
        <div class="container">
            <p>"現在の席順"</p>
            <table style="margin-top: 1em;">
                <For each=move || matrix.get().clone().into_iter().enumerate() key=|(row_idx, _row)| *row_idx let:((_row_idx, row))>
                    <tr>
                        <For each=move || row.clone().into_iter().enumerate() key=|(_col_idx, _cell)| *_col_idx let:((_col_idx, cell))>
                            <td>{cell.to_string()}</td>
                        </For>
                    </tr>
                </For>
            </table>
            <button on:click=load_matrix>"実行"</button>
            <button on:click=save_matrix>"今の状態をセーブする"</button>
        </div>
    }
}
