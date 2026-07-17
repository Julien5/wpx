use std::collections::BTreeSet;

use crate::{
    bbox::BoundingBox,
    tile::{bounding_box, bounding_tiles, tile_to_point, Tile, Tiles},
};

pub fn partition_track_into_one_box(tiles: &Tiles, features_per_tile: f64) -> Option<BoundingBox> {
    log::info!("partition optimize {} tiles in one box", tiles.len(),);
    if tiles.is_empty() {
        return None;
    }

    let mut uncovered = bounding_tiles(tiles);

    let many_box_feature_count = tiles.len() as f64 * features_per_tile;
    let one_box_feature_count = uncovered.len() as f64 * features_per_tile;

    for tile in tiles {
        uncovered.remove(tile);
    }

    if uncovered.is_empty() {
        return Some(bounding_box(tiles));
    }

    if one_box_feature_count < 10000f64 {
        return Some(bounding_box(tiles));
    }

    let ratio = many_box_feature_count / one_box_feature_count;
    if ratio > 0.5 {
        return Some(bounding_box(tiles));
    }

    None
}

pub fn partition_track_into_boxes(tiles: &Tiles, max_span: isize) -> Vec<BoundingBox> {
    log::info!(
        "partition optimize {} tiles with max_span={}",
        tiles.len(),
        max_span
    );

    let mut boxes = Vec::new();
    if tiles.is_empty() {
        return boxes;
    }

    // Track the integer bounds of our current active working box
    let mut current_tiles: Vec<Tile> = Vec::new();

    for tile in tiles {
        if current_tiles.is_empty() {
            current_tiles.push(tile.clone());
            continue;
        }

        // Calculate what the new bounds would look like if we added this tile
        let (tx, ty) = tile.coord;

        let mut min_x = tx;
        let mut max_x = tx;
        let mut min_y = ty;
        let mut max_y = ty;

        for active in &current_tiles {
            let (ax, ay) = active.coord;
            if ax < min_x {
                min_x = ax;
            }
            if ax > max_x {
                max_x = ax;
            }
            if ay < min_y {
                min_y = ay;
            }
            if ay > max_y {
                max_y = ay;
            }
        }

        // Check if the spatial span (max - min + 1) exceeds our safe threshold
        let x_span = max_x - min_x + 1;
        let y_span = max_y - min_y + 1;

        if x_span <= max_span && y_span <= max_span {
            // It fits within our safe dense-data box! Add it to current working batch.
            current_tiles.push(tile.clone());
        } else {
            // It's too big! Seal the current box and push it to the results list.
            boxes.push(bounding_box(&current_tiles));

            // Start a brand new box with the current tile
            current_tiles.clear();
            current_tiles.push(tile.clone());
        }
    }

    // Don't forget to seal the final remaining group after the loop ends!
    if !current_tiles.is_empty() {
        boxes.push(bounding_box(&current_tiles));
    }

    boxes
}

pub fn optimize_tiles_into_boxes_hard(tiles: &Tiles, max_span: isize) -> Vec<BoundingBox> {
    let mut boxes = Vec::new();
    if tiles.is_empty() {
        return boxes;
    }

    log::info!(
        "hard optimize {} tiles with max_span={}",
        tiles.len(),
        max_span
    );

    // Track which tiles have already been safely wrapped in a bounding box
    let mut visited = BTreeSet::new();

    for tile in tiles {
        if visited.contains(tile) {
            continue;
        }

        let (sx, sy) = tile.coord;

        // We want to find the largest rectangular box starting from (sx, sy)
        // that contains ONLY unvisited tiles from our set.
        let mut best_w = 1;
        let mut best_h = 1;

        // Evaluate all possible valid box sizes from (1x1) up to (MAX_SPAN x MAX_SPAN)
        for w in (1..=max_span).rev() {
            for h in (1..=max_span).rev() {
                // Skip checking if it's smaller than a shape we already found working
                if w * h <= best_w * best_h {
                    continue;
                }

                let mut valid_rectangle = true;

                // Check if ALL tiles within this candidate rectangle exist and are unvisited
                for dx in 0..w {
                    for dy in 0..h {
                        let check_tile = Tile {
                            coord: (sx + dx, sy + dy),
                        };
                        if !tiles.contains(&check_tile) || visited.contains(&check_tile) {
                            valid_rectangle = false;
                            break;
                        }
                    }
                    if !valid_rectangle {
                        break;
                    }
                }

                if valid_rectangle {
                    best_w = w;
                    best_h = h;
                }
            }
        }

        // We found the optimal rectangular slice for this anchor tile!
        // Mark all tiles inside this chosen rectangle as visited.
        let min_x = sx;
        let min_y = sy;
        let max_x = sx + best_w - 1;
        let max_y = sy + best_h - 1;

        for dx in 0..best_w {
            for dy in 0..best_h {
                visited.insert(Tile {
                    coord: (sx + dx, sy + dy),
                });
            }
        }

        // Emit the unified bounding box
        boxes.push(BoundingBox::minmax(
            tile_to_point(min_x, min_y),
            tile_to_point(max_x + 1, max_y + 1),
        ));
    }

    boxes
}

pub fn optimize_tiles_into_boxes(tiles: &Tiles) -> Vec<BoundingBox> {
    let max_span = 4;
    optimize_tiles_into_boxes_hard(tiles, max_span)
}
