use tauri::AppHandle;
use tauri::Emitter;
use tauri::Listener;
use tauri_plugin_clipboard_manager::ClipboardExt;

use enigo::{Direction, Enigo, Key, Keyboard};

#[tauri::command]
fn greet(app_handle: tauri::AppHandle) {
    println!("Hello I was invoked from JavaScript!");
    std::thread::sleep(std::time::Duration::from_secs(2));
    app_handle.emit("wait-event", "").unwrap();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            let app_handle = app.handle().clone();

            // Spawn a new thread to emit an event every 20 seconds
            std::thread::spawn(move || loop {
                std::thread::sleep(std::time::Duration::from_secs(20));
                app_handle.emit("copy-event", "20 seconds passed").unwrap();
            });

            {
                use tauri_plugin_global_shortcut::{Code, Modifiers, ShortcutState};

                app.handle().plugin(
                    tauri_plugin_global_shortcut::Builder::new()
                        .with_shortcuts(["alt+ctrl+c", "alt+ctrl+n"])?
                        .with_handler(|app, shortcut, event| {
                            if event.state == ShortcutState::Pressed {
                                if shortcut.matches(Modifiers::ALT | Modifiers::CONTROL, Code::KeyC)
                                {
                                    let _ = app.emit("window-event", "Show window");
                                }
                                if shortcut.matches(Modifiers::ALT | Modifiers::CONTROL, Code::KeyN)
                                {
                                    let _ = app.emit("copy-event", "");
                                }
                            }
                        })
                        .build(),
                )?;
            }
            {
                let handle = app.handle().clone();
                app.listen("copy-event", move |_| {
                    println!("Copy handle");
                    copy_prompt();
                    handle.emit("delete-event", "").unwrap();
                });
                let handle = app.handle().clone();
                app.listen("delete-event", move |_| {
                    println!("Delete handle");
                    delete_prompt();
                    handle.emit("print-event", "").unwrap();
                });
                let handle = app.handle().clone();
                app.listen("print-event", move |_| {
                    println!("Print clipboard handle");
                    get_context(&handle);
                });
                app.listen("window-event", move |event| {
                    println!("window: {}", event.payload());
                });
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn copy_prompt() {
    println!("preparing to copy text...");
    let mut enigo = Enigo::new(&enigo::Settings::default()).unwrap();
    println!("created enigo...");
    #[cfg(target_os = "macos")]
    {
        enigo.key(Key::Meta, Direction::Release).unwrap();
        // copy
        enigo.key(Key::Meta, Direction::Press).unwrap();
        enigo.key(Key::Unicode('c'), Direction::Click).unwrap();
        // enigo.raw(8, Direction::Click).unwrap();
        enigo.key(Key::Meta, Direction::Release).unwrap();
    }

    #[cfg(not(target_os = "macos"))]
    {
        // For Windows and Linux, use Ctrl key
        enigo.key(Key::LControl, Direction::Press).unwrap();
        enigo.key(Key::Unicode('c'), Direction::Click).unwrap();
        enigo.key(Key::LControl, Direction::Release).unwrap();
    }
    println!("----------> finished copying");
}

fn delete_prompt() {
    println!("preparing to delete text...");
    let mut enigo = Enigo::new(&enigo::Settings::default()).unwrap();
    enigo.key(Key::Backspace, Direction::Click).unwrap();
    println!("----------> finished deleting");
}

fn get_context(app_handle: &AppHandle) {
    let user_prompt = app_handle.clipboard().read_text().unwrap_or("".to_string());

    println!("----------> finished getting user_prompt");
    println!("copied... {}", user_prompt);
}
