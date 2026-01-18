//! File selection dialog implementation.

use std::path::{Path, PathBuf};
use std::fs;

use crate::backend::{Window, WindowEvent, MouseButton, create_window};
use crate::error::Error;
use crate::render::{Canvas, Font, rgb};
use crate::ui::Colors;
use crate::ui::widgets::Widget;
use crate::ui::widgets::button::Button;

const PADDING: u32 = 15;
const ITEM_HEIGHT: u32 = 24;
const LIST_WIDTH: u32 = 400;
const LIST_HEIGHT: u32 = 300;

/// File selection dialog result.
#[derive(Debug, Clone)]
pub enum FileSelectResult {
    /// User selected a file/directory.
    Selected(PathBuf),
    /// User cancelled the dialog.
    Cancelled,
    /// Dialog was closed.
    Closed,
}

impl FileSelectResult {
    pub fn exit_code(&self) -> i32 {
        match self {
            FileSelectResult::Selected(_) => 0,
            FileSelectResult::Cancelled => 1,
            FileSelectResult::Closed => 255,
        }
    }
}

/// File selection dialog builder.
pub struct FileSelectBuilder {
    title: String,
    directory: bool,
    save: bool,
    filename: String,
    start_path: Option<PathBuf>,
    colors: Option<&'static Colors>,
}

impl FileSelectBuilder {
    pub fn new() -> Self {
        Self {
            title: String::new(),
            directory: false,
            save: false,
            filename: String::new(),
            start_path: None,
            colors: None,
        }
    }

    pub fn title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    /// Select directories only.
    pub fn directory(mut self, directory: bool) -> Self {
        self.directory = directory;
        self
    }

    /// Save mode (allows entering a new filename).
    pub fn save(mut self, save: bool) -> Self {
        self.save = save;
        self
    }

    /// Default filename for save mode.
    pub fn filename(mut self, filename: &str) -> Self {
        self.filename = filename.to_string();
        self
    }

    /// Starting directory.
    pub fn start_path(mut self, path: &Path) -> Self {
        self.start_path = Some(path.to_path_buf());
        self
    }

    pub fn colors(mut self, colors: &'static Colors) -> Self {
        self.colors = Some(colors);
        self
    }

    pub fn show(self) -> Result<FileSelectResult, Error> {
        let colors = self.colors.unwrap_or_else(|| crate::ui::detect_theme());
        let font = Font::load();

        // Create buttons
        let mut ok_button = Button::new(if self.save { "Save" } else { "Open" }, &font);
        let mut cancel_button = Button::new("Cancel", &font);

        // Current directory
        let mut current_dir = self.start_path
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")));

        // Directory entries
        let mut entries: Vec<DirEntry> = Vec::new();
        let mut selected_index: Option<usize> = None;
        let mut scroll_offset: usize = 0;

        // Load initial directory
        load_directory(&current_dir, &mut entries, self.directory);

        // Calculate dimensions
        let width = (LIST_WIDTH + PADDING * 2) as u16;
        let path_height = 24;
        let buttons_height = 32;
        let height = (PADDING * 3 + path_height + LIST_HEIGHT + 10 + buttons_height) as u16;

        // Create window
        let mut window = create_window(width, height)?;
        let title = if self.title.is_empty() {
            if self.directory {
                "Select Directory"
            } else if self.save {
                "Save File"
            } else {
                "Open File"
            }
        } else {
            &self.title
        };
        window.set_title(title)?;

        // Position elements
        let path_y = PADDING as i32;
        let list_y = path_y + path_height as i32 + 5;
        let list_x = PADDING as i32;

        let button_y = list_y + LIST_HEIGHT as i32 + 10;
        let mut button_x = width as i32 - PADDING as i32;
        button_x -= cancel_button.width() as i32;
        cancel_button.set_position(button_x, button_y);
        button_x -= 10 + ok_button.width() as i32;
        ok_button.set_position(button_x, button_y);

        // Visible items count
        let visible_items = (LIST_HEIGHT / ITEM_HEIGHT) as usize;

        // Create canvas
        let mut canvas = Canvas::new(width as u32, height as u32);

        // Track mouse position
        let mut mouse_x = 0;
        let mut mouse_y = 0;

        // Draw function
        let draw = |canvas: &mut Canvas,
                    colors: &Colors,
                    font: &Font,
                    current_dir: &Path,
                    entries: &[DirEntry],
                    selected_index: Option<usize>,
                    scroll_offset: usize,
                    ok_button: &Button,
                    cancel_button: &Button| {
            canvas.fill(colors.window_bg);

            // Draw current path
            let path_str = current_dir.to_string_lossy();
            let path_display = if path_str.len() > 50 {
                format!("...{}", &path_str[path_str.len()-47..])
            } else {
                path_str.to_string()
            };
            let path_canvas = font.render(&path_display).with_color(colors.text).finish();
            canvas.draw_canvas(&path_canvas, PADDING as i32, path_y + 4);

            // Draw list background
            canvas.fill_rect(
                list_x as f32,
                list_y as f32,
                LIST_WIDTH as f32,
                LIST_HEIGHT as f32,
                colors.input_bg,
            );

            // Draw list border
            canvas.stroke_rounded_rect(
                list_x as f32,
                list_y as f32,
                LIST_WIDTH as f32,
                LIST_HEIGHT as f32,
                0.0,
                colors.input_border,
                1.0,
            );

            // Draw entries
            for (i, entry) in entries.iter().skip(scroll_offset).take(visible_items).enumerate() {
                let y = list_y + (i as u32 * ITEM_HEIGHT) as i32;
                let actual_index = scroll_offset + i;

                // Highlight selected item
                if Some(actual_index) == selected_index {
                    canvas.fill_rect(
                        (list_x + 1) as f32,
                        (y + 1) as f32,
                        (LIST_WIDTH - 2) as f32,
                        (ITEM_HEIGHT - 2) as f32,
                        colors.input_border_focused,
                    );
                }

                // Draw icon (simple folder/file indicator)
                let icon = if entry.is_dir { "[D]" } else { "   " };
                let icon_canvas = font.render(icon).with_color(colors.text).finish();
                canvas.draw_canvas(&icon_canvas, list_x + 4, y + 4);

                // Draw name
                let name = &entry.name;
                let name_display = if name.len() > 45 {
                    format!("{}...", &name[..42])
                } else {
                    name.clone()
                };
                let text_color = if Some(actual_index) == selected_index {
                    rgb(255, 255, 255)
                } else {
                    colors.text
                };
                let name_canvas = font.render(&name_display).with_color(text_color).finish();
                canvas.draw_canvas(&name_canvas, list_x + 32, y + 4);
            }

            // Draw buttons
            ok_button.draw_to(canvas, colors, font);
            cancel_button.draw_to(canvas, colors, font);
        };

        // Initial draw
        draw(
            &mut canvas,
            colors,
            &font,
            &current_dir,
            &entries,
            selected_index,
            scroll_offset,
            &ok_button,
            &cancel_button,
        );
        window.set_contents(&canvas)?;
        window.show()?;

        // Event loop
        loop {
            let event = window.wait_for_event()?;

            match &event {
                WindowEvent::CloseRequested => {
                    return Ok(FileSelectResult::Closed);
                }
                WindowEvent::RedrawRequested => {
                    draw(
                        &mut canvas,
                        colors,
                        &font,
                        &current_dir,
                        &entries,
                        selected_index,
                        scroll_offset,
                        &ok_button,
                        &cancel_button,
                    );
                    window.set_contents(&canvas)?;
                }
                WindowEvent::CursorMove(pos) => {
                    mouse_x = pos.x as i32;
                    mouse_y = pos.y as i32;
                }
                WindowEvent::ButtonPress(MouseButton::Left) => {
                    // Check if click is in list area
                    if mouse_x >= list_x && mouse_x < list_x + LIST_WIDTH as i32
                        && mouse_y >= list_y && mouse_y < list_y + LIST_HEIGHT as i32
                    {
                        let relative_y = (mouse_y - list_y) as u32;
                        let clicked_index = scroll_offset + (relative_y / ITEM_HEIGHT) as usize;

                        if clicked_index < entries.len() {
                            if selected_index == Some(clicked_index) {
                                // Double-click behavior (select again = activate)
                                let entry = &entries[clicked_index];
                                if entry.is_dir {
                                    // Navigate into directory
                                    current_dir = entry.path.clone();
                                    load_directory(&current_dir, &mut entries, self.directory);
                                    selected_index = None;
                                    scroll_offset = 0;
                                } else if !self.directory {
                                    // Select file
                                    return Ok(FileSelectResult::Selected(entry.path.clone()));
                                }
                            } else {
                                selected_index = Some(clicked_index);
                            }
                        }
                    }
                }
                WindowEvent::Scroll(direction) => {
                    match direction {
                        crate::backend::ScrollDirection::Up => {
                            if scroll_offset > 0 {
                                scroll_offset -= 1;
                            }
                        }
                        crate::backend::ScrollDirection::Down => {
                            if scroll_offset + visible_items < entries.len() {
                                scroll_offset += 1;
                            }
                        }
                        _ => {}
                    }
                }
                WindowEvent::KeyPress(key_event) => {
                    const KEY_UP: u32 = 0xff52;
                    const KEY_DOWN: u32 = 0xff54;
                    const KEY_RETURN: u32 = 0xff0d;
                    const KEY_BACKSPACE: u32 = 0xff08;

                    match key_event.keysym {
                        KEY_UP => {
                            if let Some(idx) = selected_index {
                                if idx > 0 {
                                    selected_index = Some(idx - 1);
                                    if idx - 1 < scroll_offset {
                                        scroll_offset = idx - 1;
                                    }
                                }
                            } else if !entries.is_empty() {
                                selected_index = Some(0);
                            }
                        }
                        KEY_DOWN => {
                            if let Some(idx) = selected_index {
                                if idx + 1 < entries.len() {
                                    selected_index = Some(idx + 1);
                                    if idx + 1 >= scroll_offset + visible_items {
                                        scroll_offset = idx + 2 - visible_items;
                                    }
                                }
                            } else if !entries.is_empty() {
                                selected_index = Some(0);
                            }
                        }
                        KEY_RETURN => {
                            if let Some(idx) = selected_index {
                                let entry = &entries[idx];
                                if entry.is_dir {
                                    current_dir = entry.path.clone();
                                    load_directory(&current_dir, &mut entries, self.directory);
                                    selected_index = None;
                                    scroll_offset = 0;
                                } else if !self.directory {
                                    return Ok(FileSelectResult::Selected(entry.path.clone()));
                                }
                            }
                        }
                        KEY_BACKSPACE => {
                            // Go to parent directory
                            if let Some(parent) = current_dir.parent() {
                                current_dir = parent.to_path_buf();
                                load_directory(&current_dir, &mut entries, self.directory);
                                selected_index = None;
                                scroll_offset = 0;
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }

            // Process button events
            let mut needs_redraw = ok_button.process_event(&event);
            needs_redraw |= cancel_button.process_event(&event);

            if ok_button.was_clicked() {
                if let Some(idx) = selected_index {
                    let entry = &entries[idx];
                    if self.directory {
                        if entry.is_dir {
                            return Ok(FileSelectResult::Selected(entry.path.clone()));
                        }
                    } else {
                        if !entry.is_dir {
                            return Ok(FileSelectResult::Selected(entry.path.clone()));
                        }
                    }
                } else if self.directory {
                    // If directory mode and nothing selected, return current directory
                    return Ok(FileSelectResult::Selected(current_dir.clone()));
                }
            }

            if cancel_button.was_clicked() {
                return Ok(FileSelectResult::Cancelled);
            }

            // Batch process pending events
            while let Some(event) = window.poll_for_event()? {
                match &event {
                    WindowEvent::CloseRequested => {
                        return Ok(FileSelectResult::Closed);
                    }
                    WindowEvent::CursorMove(pos) => {
                        mouse_x = pos.x as i32;
                        mouse_y = pos.y as i32;
                    }
                    _ => {
                        needs_redraw |= ok_button.process_event(&event);
                        needs_redraw |= cancel_button.process_event(&event);
                        if ok_button.was_clicked() || cancel_button.was_clicked() {
                            // Handle in next iteration
                        }
                    }
                }
            }

            if needs_redraw {
                draw(
                    &mut canvas,
                    colors,
                    &font,
                    &current_dir,
                    &entries,
                    selected_index,
                    scroll_offset,
                    &ok_button,
                    &cancel_button,
                );
                window.set_contents(&canvas)?;
            }
        }
    }
}

impl Default for FileSelectBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Directory entry.
struct DirEntry {
    name: String,
    path: PathBuf,
    is_dir: bool,
}

/// Load directory contents.
fn load_directory(path: &Path, entries: &mut Vec<DirEntry>, dirs_only: bool) {
    entries.clear();

    // Add parent directory entry if not at root
    if let Some(parent) = path.parent() {
        entries.push(DirEntry {
            name: "..".to_string(),
            path: parent.to_path_buf(),
            is_dir: true,
        });
    }

    // Read directory
    let mut dirs: Vec<DirEntry> = Vec::new();
    let mut files: Vec<DirEntry> = Vec::new();

    if let Ok(read_dir) = fs::read_dir(path) {
        for entry in read_dir.flatten() {
            let file_name = entry.file_name().to_string_lossy().to_string();

            // Skip hidden files
            if file_name.starts_with('.') {
                continue;
            }

            let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);

            if dirs_only && !is_dir {
                continue;
            }

            let entry = DirEntry {
                name: file_name,
                path: entry.path(),
                is_dir,
            };

            if is_dir {
                dirs.push(entry);
            } else {
                files.push(entry);
            }
        }
    }

    // Sort directories and files alphabetically
    dirs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    // Add directories first, then files
    entries.extend(dirs);
    entries.extend(files);
}
