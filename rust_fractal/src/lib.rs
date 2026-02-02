// lib.rs, simple FFI code

use rayon::prelude::*;
#[unsafe(no_mangle)]
pub extern "C" fn test() -> u32 {
    6
}

/// # Safety
///
/// len must be width * height
/// this is a test
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mandelbrot_baseline_render_u32(
    center_x: f32,
    center_y: f32,
    zoom: u64,
    width_px: i32,
    height_px: i32,
    max_iter: i32,
    dst: *mut u32,
    dst_len: usize,
) {
    if width_px <= 0 || height_px <= 0 {
        return;
    }
    if zoom == 0 {
        return;
    }
    if dst.is_null() {
        return;
    }

    let width = width_px as usize;
    let height = height_px as usize;
    let max_iter = max_iter.unsigned_abs();

    let expected = match width.checked_mul(height) {
        Some(v) => v,
        None => return,
    };
    if dst_len < expected {
        return;
    }

    // SAFETY: caller guarantees dst points to dst_len valid u32s.
    let pixels = unsafe { std::slice::from_raw_parts_mut(dst, expected) };

    let inv_zoom = 1.0f64 / (zoom as f64); // world units per pixel
    let half_w = (width_px as f64) * 0.5;
    let half_h = (height_px as f64) * 0.5;

    let cx = center_x as f64;
    let cy = center_y as f64;

    // parallelize by rows, like C# Parallel.For over py
    pixels
        .par_chunks_mut(width)
        .enumerate()
        .for_each(|(py, row)| {
            // screen Y down, world Y up -> invert
            let py_f = py as f64;
            let y_world = cy + (-(py_f - half_h) * inv_zoom);

            for (px, out) in row.iter_mut().enumerate() {
                let px_f = px as f64;
                let x_world = cx + ((px_f - half_w) * inv_zoom);

                let iter = iterate_mandelbrot(x_world, y_world, max_iter);
                *out = color_baseline(iter, max_iter);
            }
        });
}

#[inline]
fn iterate_mandelbrot(x0: f64, y0: f64, max_iter: u32) -> u32 {
    let mut x = 0.0f64;
    let mut y = 0.0f64;
    let mut i = 0u32;

    while i < max_iter {
        let xx = x * x - y * y + x0;
        let yy = 2.0 * x * y + y0;
        x = xx;
        y = yy;

        // escape radius: |z|^2 > 4  <=> |z| > 2
        if (x * x + y * y) > 4.0 {
            break;
        }
        i += 1;
    }
    i
}

/// Replace this with your real color provider contract.
/// This just returns green for escaped points and black for inside-set points.
#[inline]
fn color_baseline(iter: u32, max_iter: u32) -> u32 {
    if iter >= max_iter {
        0xFF00_0000 // opaque black (ARGB)
    } else {
        0xFF00_FF00 // opaque green (ARGB)
    }
}
