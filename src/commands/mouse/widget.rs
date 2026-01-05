use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Widget},
};

pub struct HeatmapConfig {
    pub show_axes: bool,
    pub show_legend: bool,
    pub color_mode: ColorMode,
    pub char_set: Vec<char>,
    pub min_val: u64,
    pub max_val: u64,
}

impl Default for HeatmapConfig {
    fn default() -> Self {
        Self {
            show_axes: true,
            show_legend: true,
            color_mode: ColorMode::Gradient,
            char_set: vec![' ', '.', ':', 'o', 'O', '@', '#'],
            min_val: 0,
            max_val: 100,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum ColorMode {
    Monochrome,
    Gradient,
}

pub struct AsciiHeatmap<'a> {
    data: &'a [Vec<u64>],
    config: HeatmapConfig,
    block: Option<Block<'a>>,
}

#[allow(dead_code)]
impl<'a> AsciiHeatmap<'a> {
    pub fn new(data: &'a [Vec<u64>]) -> Self {
        let mut max_val = 1;
        for row in data {
            for &val in row {
                if val > max_val {
                    max_val = val;
                }
            }
        }

        let config = HeatmapConfig {
            max_val,
            ..Default::default()
        };

        Self {
            data,
            config,
            block: None,
        }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn char_set(mut self, chars: Vec<char>) -> Self {
        self.config.char_set = chars;
        self
    }

    pub fn use_color(mut self, enable: bool) -> Self {
        self.config.color_mode = if enable {
            ColorMode::Gradient
        } else {
            ColorMode::Monochrome
        };
        self
    }

    pub fn show_axes(mut self, show: bool) -> Self {
        self.config.show_axes = show;
        self
    }

    pub fn show_legend(mut self, show: bool) -> Self {
        self.config.show_legend = show;
        self
    }

    fn get_char_and_color(&self, value: u64) -> (char, Color) {
        if value <= self.config.min_val {
            return (self.config.char_set[0], Color::Reset);
        }

        let len = self.config.char_set.len();
        let max = self.config.max_val.max(1) as f64;
        let min = self.config.min_val as f64;
        let val = value as f64;

        // Logarithmic scale for better visibility
        // map [min, max] to [0, 1]
        let normalized = (val - min).max(0.0);
        let normalized_max = (max - min).max(1.0);

        let log_val = (normalized + 1.0).ln();
        let log_max = (normalized_max + 1.0).ln();
        let ratio = (log_val / log_max).clamp(0.0, 1.0);

        let index = (ratio * (len as f64 - 1.0)).round() as usize;
        let char_idx = index.clamp(0, len - 1);
        let c = self.config.char_set[char_idx];

        let color = match self.config.color_mode {
            ColorMode::Monochrome => Color::Reset,
            ColorMode::Gradient => {
                // Blue -> Green -> Red gradient
                if ratio < 0.5 {
                    let t = ratio * 2.0;
                    let r = (20.0 + (50.0 - 20.0) * t) as u8;
                    let g = (20.0 + (200.0 - 20.0) * t) as u8;
                    let b = 50;
                    Color::Rgb(r, g, b)
                } else {
                    let t = (ratio - 0.5) * 2.0;
                    let r = (50.0 + (255.0 - 50.0) * t) as u8;
                    let g = (200.0 + (50.0 - 200.0) * t) as u8;
                    let b = 50;
                    Color::Rgb(r, g, b)
                }
            }
        };

        (c, color)
    }
}

impl<'a> Widget for AsciiHeatmap<'a> {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        let area = match self.block.take() {
            Some(b) => {
                let inner_area = b.inner(area);
                b.render(area, buf);
                inner_area
            }
            None => area,
        };

        if area.width < 1 || area.height < 1 {
            return;
        }

        // Determine grid dimensions
        let grid_height = self.data.len();
        if grid_height == 0 {
            return;
        }
        let grid_width = self.data[0].len();

        // Calculate max value if not set or default
        if self.config.max_val <= 100 {
            // simple heuristic, assuming real data > 100 usually
            let max = self.data.iter().flatten().max().copied().unwrap_or(1);
            self.config.max_val = max;
        }

        // Reserve space for axes if enabled
        let mut plot_area = area;
        if self.config.show_axes {
            if plot_area.width > 2 {
                plot_area.x += 1; // Left axis
                plot_area.width -= 1;
            }
            if plot_area.height > 2 {
                plot_area.height -= 1; // Bottom axis
            }
        }

        let x_start = plot_area.left();
        let y_start = plot_area.top();

        // Scale grid to fit area
        // Simple sampling: map screen coordinate to grid coordinate
        for y in 0..plot_area.height {
            for x in 0..plot_area.width {
                let grid_y = (y as usize * grid_height) / plot_area.height as usize;
                let grid_x = (x as usize * grid_width) / plot_area.width as usize;

                if grid_y < grid_height && grid_x < grid_width {
                    let value = self.data[grid_y][grid_x];
                    let (c, color) = self.get_char_and_color(value);

                    let cell = buf.cell_mut((x_start + x, y_start + y));
                    if let Some(cell) = cell {
                        cell.set_char(c);
                        if color != Color::Reset {
                            cell.set_fg(color);
                        }
                    }
                }
            }
        }

        // Render Axes
        if self.config.show_axes && area.width > 4 && area.height > 4 {
            // Y Axis (Left)
            if let Some(cell) = buf.cell_mut((area.left(), area.top())) {
                cell.set_char('0');
            }
            if let Some(cell) = buf.cell_mut((area.left(), area.bottom() - 2)) {
                cell.set_char('Y'); // Placeholder for max Y
            }

            // X Axis (Bottom)
            if let Some(cell) = buf.cell_mut((area.left() + 1, area.bottom() - 1)) {
                cell.set_char('0');
            }
            if let Some(cell) = buf.cell_mut((area.right() - 1, area.bottom() - 1)) {
                cell.set_char('X'); // Placeholder for max X
            }
        }

        // Render Legend (Simple Overlay)
        if self.config.show_legend && area.width > 20 && area.height > 5 {
            let legend_text = format!("Max: {}", self.config.max_val);
            let legend_x = area.right().saturating_sub(legend_text.len() as u16 + 2);
            let legend_y = area.top();

            if let Some(cell) = buf.cell_mut((legend_x, legend_y)) {
                cell.set_symbol(&legend_text);
                cell.set_style(Style::default().bg(Color::Black).fg(Color::White));
            }
        }
    }
}

#[allow(dead_code)]
pub fn generate_sample_data(width: usize, height: usize) -> Vec<Vec<u64>> {
    let mut grid = vec![vec![0; width]; height];
    let center_x = width as f64 / 2.0;
    let center_y = height as f64 / 2.0;

    for (y, row) in grid.iter_mut().enumerate().take(height) {
        for (x, cell) in row.iter_mut().enumerate().take(width) {
            let dx = x as f64 - center_x;
            let dy = y as f64 - center_y;
            let dist = (dx * dx + dy * dy).sqrt();
            // Gaussian-like distribution
            let val = (1000.0 * (-dist / (width as f64 / 4.0)).exp()) as u64;
            // Add some noise
            let noise = (x * y) % 10;
            *cell = val + noise as u64;
        }
    }

    // Add some "hotspots"
    if width > 10 && height > 10 {
        let hx = width / 4;
        let hy = height / 4;
        grid[hy][hx] += 500;
        grid[hy][hx + 1] += 400;
        grid[hy + 1][hx] += 400;
    }

    grid
}
