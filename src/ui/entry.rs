//! Entry dialog implementation for text input.

use crate::backend::{Window, WindowEvent, create_window};
use crate::error::Error;
use crate::render::{Canvas, Font};
use crate::ui::Colors;
use crate::ui::widgets::Widget;
use crate::ui::widgets::button::Button;
use crate::ui::widgets::text_input::TextInput;

const PADDING: u32 = 20;
const BUTTON_SPACING: u32 = 10;
const INPUT_WIDTH: u32 = 300;

/// Entry dialog result.
#[derive(Debug, Clone)]
pub enum EntryResult {
    /// User entered text and clicked OK.
    Text(String),
    /// User cancelled the dialog.
    Cancelled,
    /// Dialog was closed.
    Closed,
}

impl EntryResult {
    pub fn exit_code(&self) -> i32 {
        match self {
            EntryResult::Text(_) => 0,
            EntryResult::Cancelled => 1,
            EntryResult::Closed => 255,
        }
    }
}

/// Entry dialog builder.
pub struct EntryBuilder {
    title: String,
    text: String,
    entry_text: String,
    hide_text: bool,
    colors: Option<&'static Colors>,
}

impl EntryBuilder {
    pub fn new() -> Self {
        Self {
            title: String::new(),
            text: String::new(),
            entry_text: String::new(),
            hide_text: false,
            colors: None,
        }
    }

    pub fn title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    pub fn text(mut self, text: &str) -> Self {
        self.text = text.to_string();
        self
    }

    pub fn entry_text(mut self, entry_text: &str) -> Self {
        self.entry_text = entry_text.to_string();
        self
    }

    pub fn hide_text(mut self, hide: bool) -> Self {
        self.hide_text = hide;
        self
    }

    pub fn colors(mut self, colors: &'static Colors) -> Self {
        self.colors = Some(colors);
        self
    }

    pub fn show(self) -> Result<EntryResult, Error> {
        let colors = self.colors.unwrap_or_else(|| crate::ui::detect_theme());
        let font = Font::load();

        // Create buttons
        let mut ok_button = Button::new("OK", &font);
        let mut cancel_button = Button::new("Cancel", &font);

        // Create text input
        let mut input = TextInput::new(INPUT_WIDTH)
            .with_password(self.hide_text)
            .with_default_text(&self.entry_text);
        input.set_focus(true);

        // Render prompt text
        let prompt_canvas = if !self.text.is_empty() {
            Some(font.render(&self.text).with_color(colors.text).finish())
        } else {
            None
        };

        let prompt_height = prompt_canvas.as_ref().map(|c| c.height()).unwrap_or(0);

        // Calculate dimensions
        let buttons_width = ok_button.width() + cancel_button.width() + BUTTON_SPACING;
        let content_width = INPUT_WIDTH.max(buttons_width);
        let width = (content_width + PADDING * 2) as u16;

        let height = (PADDING * 3
            + prompt_height
            + (if prompt_height > 0 { 10 } else { 0 })
            + input.height()
            + 10
            + 32) as u16;

        // Create window
        let mut window = create_window(width, height)?;
        window.set_title(if self.title.is_empty() {
            "Entry"
        } else {
            &self.title
        })?;

        // Position elements
        let mut y = PADDING as i32;

        // Prompt text position
        let prompt_y = y;
        if prompt_height > 0 {
            y += prompt_height as i32 + 10;
        }

        // Input position
        input.set_position(PADDING as i32, y);
        y += input.height() as i32 + 10;

        // Button positions (right-aligned)
        let mut button_x = width as i32 - PADDING as i32;
        button_x -= cancel_button.width() as i32;
        cancel_button.set_position(button_x, y);
        button_x -= BUTTON_SPACING as i32 + ok_button.width() as i32;
        ok_button.set_position(button_x, y);

        // Create canvas
        let mut canvas = Canvas::new(width as u32, height as u32);

        // Draw function
        let draw = |canvas: &mut Canvas,
                    colors: &Colors,
                    font: &Font,
                    prompt_canvas: &Option<Canvas>,
                    input: &TextInput,
                    ok_button: &Button,
                    cancel_button: &Button| {
            canvas.fill(colors.window_bg);

            // Draw prompt
            if let Some(prompt) = prompt_canvas {
                canvas.draw_canvas(prompt, PADDING as i32, prompt_y);
            }

            // Draw input
            input.draw_to(canvas, colors, font);

            // Draw buttons
            ok_button.draw_to(canvas, colors, font);
            cancel_button.draw_to(canvas, colors, font);
        };

        // Initial draw
        draw(
            &mut canvas,
            colors,
            &font,
            &prompt_canvas,
            &input,
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
                    return Ok(EntryResult::Closed);
                }
                WindowEvent::RedrawRequested => {
                    draw(
                        &mut canvas,
                        colors,
                        &font,
                        &prompt_canvas,
                        &input,
                        &ok_button,
                        &cancel_button,
                    );
                    window.set_contents(&canvas)?;
                }
                _ => {}
            }

            // Process input events
            let mut needs_redraw = input.process_event(&event);

            // Check for Enter key submission
            if input.was_submitted() {
                return Ok(EntryResult::Text(input.text().to_string()));
            }

            // Process button events
            if ok_button.process_event(&event) {
                needs_redraw = true;
            }
            if cancel_button.process_event(&event) {
                needs_redraw = true;
            }

            if ok_button.was_clicked() {
                return Ok(EntryResult::Text(input.text().to_string()));
            }
            if cancel_button.was_clicked() {
                return Ok(EntryResult::Cancelled);
            }

            // Batch process pending events
            while let Some(event) = window.poll_for_event()? {
                match &event {
                    WindowEvent::CloseRequested => {
                        return Ok(EntryResult::Closed);
                    }
                    _ => {
                        if input.process_event(&event) {
                            needs_redraw = true;
                        }
                        if input.was_submitted() {
                            return Ok(EntryResult::Text(input.text().to_string()));
                        }
                        if ok_button.process_event(&event) {
                            needs_redraw = true;
                        }
                        if cancel_button.process_event(&event) {
                            needs_redraw = true;
                        }
                        if ok_button.was_clicked() {
                            return Ok(EntryResult::Text(input.text().to_string()));
                        }
                        if cancel_button.was_clicked() {
                            return Ok(EntryResult::Cancelled);
                        }
                    }
                }
            }

            if needs_redraw {
                draw(
                    &mut canvas,
                    colors,
                    &font,
                    &prompt_canvas,
                    &input,
                    &ok_button,
                    &cancel_button,
                );
                window.set_contents(&canvas)?;
            }
        }
    }
}

impl Default for EntryBuilder {
    fn default() -> Self {
        Self::new()
    }
}
