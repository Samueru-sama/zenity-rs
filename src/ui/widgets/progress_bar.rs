//! Progress bar widget.

use crate::render::Canvas;
use crate::ui::Colors;

const BAR_HEIGHT: u32 = 20;
const BAR_RADIUS: f32 = 4.0;

/// A progress bar widget.
pub struct ProgressBar {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    progress: f32, // 0.0 to 1.0
    pulsating: bool,
    pulse_position: f32, // For pulsating animation
}

impl ProgressBar {
    pub fn new(width: u32) -> Self {
        Self {
            x: 0,
            y: 0,
            width,
            height: BAR_HEIGHT,
            progress: 0.0,
            pulsating: false,
            pulse_position: 0.0,
        }
    }

    /// Sets the progress value (0.0 to 1.0).
    pub fn set_progress(&mut self, progress: f32) {
        self.progress = progress.clamp(0.0, 1.0);
        self.pulsating = false;
    }

    /// Sets the progress as a percentage (0 to 100).
    pub fn set_percentage(&mut self, percentage: u32) {
        self.set_progress(percentage as f32 / 100.0);
    }

    /// Enables pulsating mode (indeterminate progress).
    pub fn set_pulsating(&mut self, pulsating: bool) {
        self.pulsating = pulsating;
        if pulsating {
            self.pulse_position = 0.0;
        }
    }

    /// Returns true if in pulsating mode.
    pub fn is_pulsating(&self) -> bool {
        self.pulsating
    }

    /// Advances the pulse animation. Call this periodically.
    pub fn tick(&mut self) {
        if self.pulsating {
            self.pulse_position += 0.02;
            if self.pulse_position > 1.0 {
                self.pulse_position = 0.0;
            }
        }
    }

    /// Returns the current progress (0.0 to 1.0).
    pub fn progress(&self) -> f32 {
        self.progress
    }

    pub fn set_position(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    /// Draws the progress bar to a canvas.
    pub fn draw(&self, canvas: &mut Canvas, colors: &Colors) {
        // Draw background (track)
        canvas.fill_rounded_rect(
            self.x as f32,
            self.y as f32,
            self.width as f32,
            self.height as f32,
            BAR_RADIUS,
            colors.progress_bg,
        );

        // Draw progress fill
        if self.pulsating {
            // Draw a moving pulse
            let pulse_width = self.width as f32 * 0.3;
            let max_x = self.width as f32 - pulse_width;
            let pulse_x = self.x as f32 + max_x * self.pulse_position;

            canvas.fill_rounded_rect(
                pulse_x,
                self.y as f32,
                pulse_width,
                self.height as f32,
                BAR_RADIUS,
                colors.progress_fill,
            );
        } else if self.progress > 0.0 {
            let fill_width = (self.width as f32 * self.progress).max(BAR_RADIUS * 2.0);

            canvas.fill_rounded_rect(
                self.x as f32,
                self.y as f32,
                fill_width,
                self.height as f32,
                BAR_RADIUS,
                colors.progress_fill,
            );
        }

        // Draw border
        canvas.stroke_rounded_rect(
            self.x as f32,
            self.y as f32,
            self.width as f32,
            self.height as f32,
            BAR_RADIUS,
            colors.progress_border,
            1.0,
        );
    }
}
