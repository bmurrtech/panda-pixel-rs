use wasm_bindgen::prelude::*;
use serde_json;

pub fn is_tauri_available() -> bool {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return false,
    };

    // Check for __TAURI__ object (Tauri 1.x and 2.x)
    let tauri_obj = match js_sys::Reflect::get(&window, &JsValue::from_str("__TAURI__")) {
        Ok(v) if !v.is_undefined() => v,
        _ => return false,
    };

    // Tauri 2.0 uses __TAURI__.core.invoke, Tauri 1.x uses __TAURI__.tauri.invoke
    // Try all possible paths
    let core_api = js_sys::Reflect::get(&tauri_obj, &JsValue::from_str("core")).ok();
    let tauri_api = js_sys::Reflect::get(&tauri_obj, &JsValue::from_str("tauri")).ok();
    let invoke_direct = js_sys::Reflect::get(&tauri_obj, &JsValue::from_str("invoke")).ok();

    // Check Tauri 2.0 path first: __TAURI__.core.invoke
    let has_core_invoke = core_api
        .as_ref()
        .and_then(|core| {
            if core.is_undefined() {
                None
            } else {
                js_sys::Reflect::get(core, &JsValue::from_str("invoke"))
                    .ok()
                    .map(|invoke| !invoke.is_undefined())
            }
        })
        .unwrap_or(false);

    // Check Tauri 1.x path: __TAURI__.tauri.invoke
    let has_tauri_invoke = tauri_api
        .as_ref()
        .and_then(|tauri| {
            if tauri.is_undefined() {
                None
            } else {
                js_sys::Reflect::get(tauri, &JsValue::from_str("invoke"))
                    .ok()
                    .map(|invoke| !invoke.is_undefined())
            }
        })
        .unwrap_or(false);

    // Check direct path: __TAURI__.invoke
    let has_direct_invoke = invoke_direct
        .as_ref()
        .map(|invoke| !invoke.is_undefined())
        .unwrap_or(false);

    has_core_invoke || has_tauri_invoke || has_direct_invoke
}

pub async fn invoke_tauri<T>(cmd: &str, args: JsValue) -> Result<T, String>
where
    T: serde::de::DeserializeOwned,
{
    if !is_tauri_available() {
        // This is expected when running in browser mode - not an error, just informational
        return Err("Tauri API not available (running in browser). Use 'cargo tauri dev' for full file system access.".to_string());
    }

    let window = web_sys::window().ok_or("No window object")?;
    let tauri_obj = js_sys::Reflect::get(&window, &JsValue::from_str("__TAURI__"))
        .ok()
        .and_then(|t| if t.is_undefined() { None } else { Some(t) })
        .ok_or("Tauri API not available")?;

    // Helper function to find invoke
    fn find_invoke(tauri_obj: &JsValue) -> Result<(js_sys::Function, JsValue), String> {
        // Try __TAURI__.core.invoke (Tauri 2.0)
        if let Ok(core) = js_sys::Reflect::get(tauri_obj, &JsValue::from_str("core")) {
            if !core.is_undefined() {
                if let Ok(invoke_fn) = js_sys::Reflect::get(&core, &JsValue::from_str("invoke")) {
                    if let Some(func) = invoke_fn.dyn_ref::<js_sys::Function>() {
                        return Ok((func.clone(), core));
                    }
                }
            }
        }

        // Try __TAURI__.tauri.invoke (Tauri 1.x)
        if let Ok(api) = js_sys::Reflect::get(tauri_obj, &JsValue::from_str("tauri")) {
            if !api.is_undefined() {
                if let Ok(invoke_fn) = js_sys::Reflect::get(&api, &JsValue::from_str("invoke")) {
                    if let Some(func) = invoke_fn.dyn_ref::<js_sys::Function>() {
                        return Ok((func.clone(), api));
                    }
                }
            }
        }

        // Fallback to direct __TAURI__.invoke
        let invoke_fn = js_sys::Reflect::get(tauri_obj, &JsValue::from_str("invoke"))
            .ok()
            .and_then(|i| i.dyn_ref::<js_sys::Function>().cloned())
            .ok_or("invoke function not found")?;
        Ok((invoke_fn, tauri_obj.clone()))
    }

    let (invoke, invoke_context) = find_invoke(&tauri_obj)?;

    let promise = invoke
        .call2(&invoke_context, &JsValue::from_str(cmd), &args)
        .map_err(|_| "Failed to call invoke")?;

    let result = wasm_bindgen_futures::JsFuture::from(
        promise.dyn_into::<js_sys::Promise>().map_err(|_| "Not a promise")?
    )
    .await
    .map_err(|e| format!("Promise rejected: {:?}", e))?;

    let json_str = js_sys::JSON::stringify(&result)
        .map_err(|_| "Failed to stringify")?
        .as_string()
        .ok_or("Not a string")?;

    serde_json::from_str(&json_str).map_err(|e| format!("Deserialize error: {}", e))
}
