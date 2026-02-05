const RED: &[u8] = &[0, 0, 0, 0, 128, 255, 255, 255];
const GREEN: &[u8] = &[0, 0, 128, 255, 128, 128, 255, 255];
const BLUE: &[u8] = &[0, 255, 255, 128, 0, 0, 128, 255];
const MAP_LEN: usize = RED.len();

#[inline]
fn pack_argb(r: u8, g: u8, b: u8) -> u32 {
    0xff_u32 << 24 | (r as u32) << 16 | (g as u32) << 8 | b as u32
}
const COLOR_WIDTH: f64 = 50.;

const BLACK: u32 = 0xff << 24;

#[inline]
pub fn get_color(iteration: u32, max_iteration: u32) -> u32 {
    if max_iteration == 0 || iteration >= max_iteration {
        return BLACK;
    }

    let mut f_iteration: f64 = iteration as f64;
    let mut start = 0usize;
    while f_iteration >= COLOR_WIDTH {
        f_iteration -= COLOR_WIDTH;
        start += 1;
    }
    let t = f_iteration / COLOR_WIDTH;

    let end = (start + 1) % MAP_LEN;
    let start = start % MAP_LEN;
    pack_argb(
        clamped_interpolation(RED[start], RED[end], t),
        clamped_interpolation(GREEN[start], GREEN[end], t),
        clamped_interpolation(BLUE[start], BLUE[end], t),
    )
}

#[inline]
fn clamped_interpolation(lower: u8, higher: u8, percentage: f64) -> u8 {
    if percentage <= 0. {
        return lower;
    }
    if percentage >= 1. {
        return higher;
    }

    let t = (lower as f64 + (higher as f64 - lower as f64) * percentage) as u32;
    t.clamp(0, 255) as u8
}
