use crate::screen::{SCREEN_HEIGHT, SCREEN_WIDTH, ScreenController};

enum Corner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    All,
}

impl ScreenController {
    pub fn draw_line(&mut self, start: (i16, i16), end: (i16, i16)) {
        let mut x0 = start.0;
        let mut y0 = start.1;
        let x1 = end.0;
        let y1 = end.1;

        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();

        let dir_x = if x0 < x1 { 1 } else { -1 };
        let dir_y = if y0 < y1 { 1 } else { -1 };
        let mut error = dx + dy;

        loop {
            if x0 >= 0 && x0 < SCREEN_WIDTH as i16 && y0 >= 0 && y0 < SCREEN_HEIGHT as i16 {
                self.draw_px(x0, y0, false);
            }

            if x0 == x1 && y0 == y1 {
                break;
            }

            let e2 = 2 * error;
            if e2 >= dy {
                error += dy;
                x0 += dir_x;
            }
            if e2 <= dx {
                error += dx;
                y0 += dir_y;
            }
        }

        // DDA Algorithm
        /*let dx = x2 as i16 - x0;
        let dy = y2 as i16 - x0;
        let steps = dx.abs().max(dy.abs());

        if steps == 0 {
            self.draw_px(x1, y1, false);
            return;
        }

        let x_step = dx as f32 / steps as f32;
        let y_step = dy as f32 / steps as f32;

        let mut x = x1 as f32;
        let mut y = y1 as f32;

        for _ in 0..=steps {
            self.draw_px(math::round(x) as i16, math::round(y) as i16, false);

            x += x_step;
            y += y_step;
        }*/
    }

    // ################

    pub fn draw_rect(&mut self, anchor: (i16, i16), width: u16, height: u16, filled: bool) {
        if width < 1 || height < 1 {
            return;
        }

        let x = anchor.0;
        let y = anchor.1;
        let right = x + width as i16 - 1;
        let bottom = y + height as i16 - 1;

        self.draw_line((x, y), (right, y));
        self.draw_line((x, bottom), (right, bottom));
        self.draw_line((x, y), (x, bottom));
        self.draw_line((right, y), (right, bottom));

        if filled {
            for line_offset in (y + 1)..bottom {
                self.draw_line((x + 1, line_offset), (right - 1, line_offset));
            }
        }
    }

    // #############

    pub fn draw_round_rect(
        &mut self,
        anchor: (i16, i16),
        width: u16,
        height: u16,
        corner_radius: u16,
        filled: bool,
    ) {
        if width < 1 || height < 1 {
            return;
        }

        let max_radius = width.min(height) / 2;
        let r = corner_radius.clamp(0, max_radius) as i16;
        let x = anchor.0;
        let y = anchor.1;

        let right = x + width as i16 - 1;
        let bottom = y + height as i16 - 1;

        self.draw_line((x + r, y), (right - r, y));
        self.draw_line((x, y + r), (x, bottom - r));
        self.draw_line((x + r, bottom), (right - r, bottom));
        self.draw_line((right, y + r), (right, bottom - r));

        self.draw_arc_quadrants((x + r, y + r), r, filled, Corner::TopLeft);
        self.draw_arc_quadrants((right - r, y + r), r, filled, Corner::TopRight);
        self.draw_arc_quadrants((x + r, bottom - r), r, filled, Corner::BottomLeft);
        self.draw_arc_quadrants((right - r, bottom - r), r, filled, Corner::BottomRight);

        if filled {
            // inside rect
            for line_offset in (y + r)..=(bottom - r) {
                self.draw_line((x, line_offset), (right, line_offset));
            }

            for line_offset in 0..r {
                // up corners
                self.draw_line(
                    (x + r, y + line_offset),
                    (right - r, y + line_offset),
                );
                // down corners
                self.draw_line(
                    (x + r, bottom - line_offset),
                    (right - r, bottom - line_offset),
                );
            }
        }
    }

    fn draw_arc_quadrants(&mut self, center: (i16, i16), r: i16, filled: bool, corner: Corner) {
        let mut x = 0;
        let mut y = r;
        let mut err = 1 - r;

        while x <= y {
            self.draw_round_corner_px(center.0, center.1, x, y, filled, &corner);

            x += 1;
            if err < 0 {
                err += 2 * x + 1;
            } else {
                y -= 1;
                err += 2 * (x - y) + 1;
            }
        }
    }

    fn draw_round_corner_px(
        &mut self,
        cx: i16,
        cy: i16,
        x: i16,
        y: i16,
        filled: bool,
        corner: &Corner,
    ) {
        let bottom_right = (cx + x, cy + y);
        let bottom_left = (cx - x, cy + y);
        let top_right = (cx + x, cy - y);
        let top_left = (cx - x, cy - y);

        let fill_bottom_right = (cx, cy + y);
        let fill_bottom_left = (cx, cy + y);
        let fill_top_right = (cx, cy - y);
        let fill_top_left = (cx, cy - y);

        let inv_bottom_right = (cx + y, cy + x);
        let inv_bottom_left = (cx - y, cy + x);
        let inv_top_right = (cx + y, cy - x);
        let inv_top_left = (cx - y, cy - x);

        let fill_inv_bottom_right = (cx, cy + x);
        let fill_inv_bottom_left = (cx, cy + x);
        let fill_inv_top_right = (cx, cy - x);
        let fill_inv_top_left = (cx, cy - x);

        match corner {
            Corner::TopLeft => {
                self.draw_round_quarter(
                    top_left,
                    inv_top_left,
                    if filled {
                        Some((fill_top_left, fill_inv_top_left))
                    } else {
                        None
                    },
                );
            }
            Corner::TopRight => {
                self.draw_round_quarter(
                    top_right,
                    inv_top_right,
                    if filled {
                        Some((fill_top_right, fill_inv_top_right))
                    } else {
                        None
                    },
                );
            }
            Corner::BottomLeft => {
                self.draw_round_quarter(
                    bottom_left,
                    inv_bottom_left,
                    if filled {
                        Some((fill_bottom_left, fill_inv_bottom_left))
                    } else {
                        None
                    },
                );
            }
            Corner::BottomRight => {
                self.draw_round_quarter(
                    bottom_right,
                    inv_bottom_right,
                    if filled {
                        Some((fill_bottom_right, fill_inv_bottom_right))
                    } else {
                        None
                    },
                );
            }
            Corner::All => {
                self.draw_round_corner_px(cx, cy, x, y, filled, &Corner::TopLeft);
                self.draw_round_corner_px(cx, cy, x, y, filled, &Corner::TopRight);
                self.draw_round_corner_px(cx, cy, x, y, filled, &Corner::BottomLeft);
                self.draw_round_corner_px(cx, cy, x, y, filled, &Corner::BottomRight);
            }
        }
    }

    fn draw_round_quarter(
        &mut self,
        corner: (i16, i16),
        invert_corner: (i16, i16),
        fill_from: Option<((i16, i16), (i16, i16))>,
    ) {
        if let Some(fill_from) = fill_from {
            self.draw_line(fill_from.0, corner);
            self.draw_line(fill_from.1, invert_corner);
        } else {
            self.draw_px(corner.0, corner.1, false);
            self.draw_px(invert_corner.0, invert_corner.1, false);
        }
    }

    // ###############

    pub fn draw_circle(&mut self, center: (i16, i16), radius: u16, filled: bool) {
        self.draw_arc_quadrants(center, radius as i16, filled, Corner::All);
    }
}
