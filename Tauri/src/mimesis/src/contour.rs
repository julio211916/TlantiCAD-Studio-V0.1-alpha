use crate::config::MaskMethod;

/// A 2D point.
pub type Point = (f64, f64);

/// Extract a binary mask from an image using the configured method.
pub fn extract_mask(
    img: &image::DynamicImage,
    method: MaskMethod,
    threshold: u8,
) -> Vec<Vec<bool>> {
    let (w, h) = (img.width() as usize, img.height() as usize);
    let rgba = img.to_rgba8();
    let mut mask = vec![vec![false; w]; h];

    for y in 0..h {
        for x in 0..w {
            let px = rgba.get_pixel(x as u32, y as u32);
            let value = match method {
                MaskMethod::Alpha => px[3],
                MaskMethod::Luminance => {
                    ((px[0] as f32 * 0.299) + (px[1] as f32 * 0.587) + (px[2] as f32 * 0.114))
                        as u8
                }
                MaskMethod::Red => px[0],
                MaskMethod::Green => px[1],
                MaskMethod::Blue => px[2],
            };
            mask[y][x] = value >= threshold;
        }
    }

    mask
}

/// Theo Pavlidis' contour tracing algorithm.
///
/// Traces the outer boundary of a connected region in a binary mask,
/// returning a list of (x, y) boundary points.
pub fn trace_contour(mask: &[Vec<bool>]) -> Vec<Point> {
    let h = mask.len();
    if h == 0 {
        return vec![];
    }
    let w = mask[0].len();

    // Find the first foreground pixel (top-left scan)
    let mut start = None;
    'outer: for y in 0..h {
        for x in 0..w {
            if mask[y][x] {
                start = Some((x, y));
                break 'outer;
            }
        }
    }

    let (sx, sy) = match start {
        Some(p) => p,
        None => return vec![],
    };

    // 8-connected neighbor offsets (clockwise from right)
    //  0: right, 1: down-right, 2: down, 3: down-left
    //  4: left,  5: up-left,    6: up,   7: up-right
    let dx: [i32; 8] = [1, 1, 0, -1, -1, -1, 0, 1];
    let dy: [i32; 8] = [0, 1, 1, 1, 0, -1, -1, -1];

    let is_fg = |x: i32, y: i32| -> bool {
        if x < 0 || y < 0 || x >= w as i32 || y >= h as i32 {
            return false;
        }
        mask[y as usize][x as usize]
    };

    let mut contour = Vec::new();
    let mut cx = sx as i32;
    let mut cy = sy as i32;
    let mut dir: usize = 7; // Start searching from up-right

    loop {
        contour.push((cx as f64, cy as f64));

        // Search for the next boundary pixel clockwise
        let start_dir = (dir + 6) % 8; // back-track 2 positions
        let mut found = false;

        for i in 0..8 {
            let d = (start_dir + i) % 8;
            let nx = cx + dx[d];
            let ny = cy + dy[d];
            if is_fg(nx, ny) {
                cx = nx;
                cy = ny;
                dir = d;
                found = true;
                break;
            }
        }

        if !found {
            break; // Isolated pixel
        }

        // Terminate when we return to start
        if cx == sx as i32 && cy == sy as i32 {
            break;
        }

        // Safety: prevent infinite loops
        if contour.len() > w * h {
            break;
        }
    }

    contour
}

/// Find all contours in a binary mask (multi-object support).
/// Returns contours sorted by area (largest first).
pub fn find_all_contours(mask: &[Vec<bool>], min_dimension: u32) -> Vec<Vec<Point>> {
    let h = mask.len();
    if h == 0 {
        return vec![];
    }
    let w = mask[0].len();

    let mut visited = vec![vec![false; w]; h];
    let mut contours = Vec::new();

    // For each unvisited foreground pixel, trace a contour
    for y in 0..h {
        for x in 0..w {
            if mask[y][x] && !visited[y][x] {
                // Create a local mask for this connected component
                let component = flood_fill(mask, &mut visited, x, y);

                // Trace contour of this component
                let contour = trace_contour(&component);

                if contour.len() >= 3 {
                    // Check minimum dimension
                    let (min_x, max_x, min_y, max_y) = bounding_box(&contour);
                    let dim_w = (max_x - min_x) as u32;
                    let dim_h = (max_y - min_y) as u32;

                    if dim_w >= min_dimension && dim_h >= min_dimension {
                        contours.push(contour);
                    }
                }
            }
        }
    }

    // Sort by area (largest first)
    contours.sort_by(|a, b| polygon_area(b).partial_cmp(&polygon_area(a)).unwrap());
    contours
}

/// Flood fill to extract a connected component as a local mask.
fn flood_fill(
    mask: &[Vec<bool>],
    visited: &mut [Vec<bool>],
    start_x: usize,
    start_y: usize,
) -> Vec<Vec<bool>> {
    let h = mask.len();
    let w = mask[0].len();
    let mut component = vec![vec![false; w]; h];
    let mut stack = vec![(start_x, start_y)];

    while let Some((x, y)) = stack.pop() {
        if x >= w || y >= h || visited[y][x] || !mask[y][x] {
            continue;
        }
        visited[y][x] = true;
        component[y][x] = true;

        if x > 0 {
            stack.push((x - 1, y));
        }
        if x + 1 < w {
            stack.push((x + 1, y));
        }
        if y > 0 {
            stack.push((x, y - 1));
        }
        if y + 1 < h {
            stack.push((x, y + 1));
        }
    }

    component
}

/// Compute the bounding box of a polygon.
fn bounding_box(points: &[Point]) -> (f64, f64, f64, f64) {
    let mut min_x = f64::MAX;
    let mut max_x = f64::MIN;
    let mut min_y = f64::MAX;
    let mut max_y = f64::MIN;

    for &(x, y) in points {
        min_x = min_x.min(x);
        max_x = max_x.max(x);
        min_y = min_y.min(y);
        max_y = max_y.max(y);
    }

    (min_x, max_x, min_y, max_y)
}

/// Compute the signed area of a polygon (Shoelace formula).
pub fn polygon_area(points: &[Point]) -> f64 {
    let n = points.len();
    if n < 3 {
        return 0.0;
    }
    let mut area = 0.0;
    for i in 0..n {
        let j = (i + 1) % n;
        area += points[i].0 * points[j].1;
        area -= points[j].0 * points[i].1;
    }
    (area / 2.0).abs()
}
