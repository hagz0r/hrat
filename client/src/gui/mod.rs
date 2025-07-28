use eframe::{App, NativeOptions, egui};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc,
};
use std::thread::{self, JoinHandle};

use crate::actors::chat::{Author, SharedMessages};

type GuiSender = mpsc::Sender<String>;
type UpdateHandle = Arc<AtomicBool>;

struct ChatApp {
    messages: SharedMessages,
    input_text: String,
    gui_sender: GuiSender,
}

impl App for ChatApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        // in 0.32 close handling was moved, cancel the OS close request.
        if ctx.input(|i| i.viewport().close_requested()) {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        }

        // this will be called every frame but its cheap so we dont care
        ctx.send_viewport_cmd(egui::ViewportCommand::EnableButtons {
            close: false,
            minimized: true,
            maximize: false,
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.heading("Live Chat");

                egui::Frame::new()
                    .fill(egui::Color32::from_rgb(24, 24, 24))
                    .corner_radius(5.0)
                    .inner_margin(10.0)
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .stick_to_bottom(true)
                            .max_height(350.0)
                            .show(ui, |ui| {
                                let messages = self.messages.lock().unwrap();
                                for msg in messages.iter() {
                                    let (author_text, text_color) = match msg.author {
                                        Author::Host => ("Hacker:", egui::Color32::DARK_GREEN),
                                        Author::Client => ("You:", egui::Color32::WHITE),
                                    };

                                    ui.horizontal_wrapped(|ui| {
                                        ui.add(egui::Label::new(
                                            egui::RichText::new(author_text)
                                                .color(text_color)
                                                .strong(),
                                        ));
                                        ui.add(egui::Label::new(
                                            egui::RichText::new(&msg.text)
                                                .color(egui::Color32::WHITE),
                                        ));
                                    });
                                    ui.add_space(5.0);
                                }
                            });
                    });

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    let text_edit_response = ui.add(
                        egui::TextEdit::singleline(&mut self.input_text)
                            .hint_text("Type your message here...")
                            .desired_width(ui.available_width() - 60.0),
                    );

                    if ui
                        .add_sized([60.0, 20.0], egui::Button::new("Send"))
                        .clicked()
                        || (text_edit_response.lost_focus()
                            && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                    {
                        self.send_message();
                        text_edit_response.request_focus();
                    }
                });
            });
        });
    }
}
impl ChatApp {
    fn send_message(&mut self) {
        if !self.input_text.trim().is_empty() {
            let _ = self.gui_sender.send(self.input_text.clone());
            self.input_text.clear();
        }
    }
}

pub struct CrossPlatformRenderer {
    thread_handle: Option<JoinHandle<()>>,
    update_handle: UpdateHandle,
}

impl CrossPlatformRenderer {
    pub fn new() -> Self {
        Self {
            thread_handle: None,
            update_handle: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start(&mut self, messages: SharedMessages, gui_sender: GuiSender) -> anyhow::Result<()> {
        if self.thread_handle.is_some() {
            return Err(anyhow::anyhow!("Renderer is already running"));
        }

        let _update_handle = Arc::clone(&self.update_handle);
        let thread_handle = thread::spawn(move || {
            // Environment diagnostics + forced backend selection
            #[cfg(target_os = "linux")]
            {
                use std::env;
                // If not explicitly set — choose based on the presence of WAYLAND_DISPLAY/DISPLAY
                if env::var("WINIT_UNIX_BACKEND").is_err() {
                    if env::var_os("WAYLAND_DISPLAY").is_some() {
                        unsafe {
                            env::set_var("WINIT_UNIX_BACKEND", "wayland");
                        }
                    } else if env::var_os("DISPLAY").is_some() {
                        unsafe {
                            env::set_var("WINIT_UNIX_BACKEND", "x11");
                        }
                    }
                }
                eprintln!(
                    "[GUI] WINIT_UNIX_BACKEND={:?} WAYLAND_DISPLAY={:?} DISPLAY={:?}",
                    env::var("WINIT_UNIX_BACKEND").ok(),
                    env::var_os("WAYLAND_DISPLAY"),
                    env::var_os("DISPLAY"),
                );
            }

            let options = NativeOptions {
                viewport: egui::ViewportBuilder::default()
                    .with_inner_size([400.0, 500.0])
                    .with_resizable(false)
                    .with_always_on_top(),
                // allow winit to create EventLoop NOT in the main thread
                event_loop_builder: Some(Box::new(|builder| {
                    #[cfg(target_os = "linux")]
                    {
                        winit::platform::x11::EventLoopBuilderExtX11::with_any_thread(
                            builder, true,
                        );
                        winit::platform::wayland::EventLoopBuilderExtWayland::with_any_thread(
                            builder, true,
                        );
                    }
                    #[cfg(target_os = "windows")]
                    {
                        winit::platform::windows::EventLoopBuilderExtWindows::with_any_thread(
                            builder, true,
                        );
                    }
                })),
                ..Default::default()
            };

            eprintln!("[GUI] eframe::run_native -> start");
            let res = eframe::run_native(
                "Chat",
                options,
                Box::new(|_cc| {
                    Ok(Box::new(ChatApp {
                        messages,
                        input_text: String::new(),
                        gui_sender,
                    }))
                }),
            );
            eprintln!("[GUI] eframe::run_native finished: {:?}", res);
        });

        self.thread_handle = Some(thread_handle);
        Ok(())
    }

    pub fn _update(&self) -> anyhow::Result<()> {
        self.update_handle.store(true, Ordering::Relaxed);
        Ok(())
    }

    pub fn stop(&mut self) -> anyhow::Result<()> {
        crate::dev_print!(
            "GUI stop requested. The window is non-closable and will exit with the main process."
        );
        self.thread_handle.take();
        Ok(())
    }
}
