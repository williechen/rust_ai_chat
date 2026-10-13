use tauri::{Manager, PhysicalPosition, PhysicalRect, PhysicalSize};

const ROAM_INITIAL_DX: i32 = 2;
const ROAM_INITIAL_DY: i32 = 1;
const ROAM_INTERVAL: std::time::Duration = std::time::Duration::from_millis(50);
const ROAM_VERIFY_INTERVAL: std::time::Duration = std::time::Duration::from_millis(200);
const ROAM_STALL_LIMIT: u8 = 3;

// 計算下一個受限位置
fn bounded_next_position(
    before: PhysicalPosition<i32>,
    window_size: PhysicalSize<u32>,
    work_area: &PhysicalRect<i32, u32>,
    dx: &mut i32,
    dy: &mut i32,
) -> PhysicalPosition<i32> {
    let min_x = i64::from(work_area.position.x);
    let min_y = i64::from(work_area.position.y);

    let area_width = i64::from(work_area.size.width);
    let area_height = i64::from(work_area.size.height);
    let window_width = i64::from(window_size.width);
    let window_height = i64::from(window_size.height);

    let max_x = min_x + area_width - window_width;
    let max_y = min_y + area_height - window_height;

    let mut next_x = i64::from(before.x) + i64::from(*dx);
    let mut next_y = i64::from(before.y) + i64::from(*dy);

    if max_x <= min_x {
        next_x = min_x;
        *dx = 0;
    } else if next_x < min_x {
        next_x = min_x;
        *dx = dx.saturating_abs();
    } else if next_x > max_x {
        next_x = max_x;
        *dx = -dx.saturating_abs();
    }

    if max_y <= min_y {
        next_y = min_y;
        *dy = 0;
    } else if next_y < min_y {
        next_y = min_y;
        *dy = dy.saturating_abs();
    } else if next_y > max_y {
        next_y = max_y;
        *dy = -dy.saturating_abs();
    }

    PhysicalPosition {
        x: next_x.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32,
        y: next_y.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32,
    }
}

// 在桌面上漫遊寵物的主循環
pub async fn roam_desktop(app: tauri::AppHandle) {
    let mut dx = ROAM_INITIAL_DX;
    let mut dy = ROAM_INITIAL_DY;
    let mut stalled_attempts = 0_u8;

    loop {
        let window = match app.get_webview_window("main") {
            Some(window) => window,
            None => {
                println!("desktop roaming unavailable: main window not found");
                break;
            }
        };

        let before = match window.outer_position() {
            Ok(position) => position,
            Err(error) => {
                eprintln!("desktop roaming unavailable: {error}");
                break;
            }
        };

        let window_size = match window.outer_size() {
            Ok(size) => size,
            Err(error) => {
                eprintln!("desktop roaming unavailable: {error}");
                break;
            }
        };

        let monitor = match window.current_monitor() {
            Ok(Some(monitor)) => monitor,
            Ok(None) => {
                eprintln!("desktop roaming unavailable: no monitor found");
                break;
            }
            Err(error) => {
                eprintln!("desktop roaming unavailable: {error}");
                break;
            }
        };

        let requested =
            bounded_next_position(before, window_size, monitor.work_area(), &mut dx, &mut dy);

        if let Err(error) = window.set_position(requested) {
            eprintln!("desktop roaming unavailable: {error}");
            break;
        }

        tokio::time::sleep(ROAM_VERIFY_INTERVAL).await;

        let after = match window.outer_position() {
            Ok(position) => position,
            Err(error) => {
                eprintln!("desktop roaming verification unavailable: {error}");
                break;
            }
        };

        if after == before {
            stalled_attempts += 1;

            if stalled_attempts >= ROAM_STALL_LIMIT {
                eprintln!(
                    "desktop roaming unavailable: \
                     window position did not change after \
                     {ROAM_STALL_LIMIT} attempts"
                );
                break;
            }
        } else {
            stalled_attempts = 0;
        }

        tokio::time::sleep(ROAM_INTERVAL).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn area(x: i32, y: i32, width: u32, height: u32) -> PhysicalRect<i32, u32> {
        PhysicalRect {
            position: PhysicalPosition { x, y },
            size: PhysicalSize { width, height },
        }
    }

    #[test]
    fn moves_inside_work_area() {
        let mut dx = 2;
        let mut dy = 1;
        let next = bounded_next_position(
            PhysicalPosition { x: 100, y: 100 },
            PhysicalSize {
                width: 320,
                height: 360,
            },
            &area(0, 0, 1920, 1080),
            &mut dx,
            &mut dy,
        );
        assert_eq!(next, PhysicalPosition { x: 102, y: 101 });
        assert_eq!((dx, dy), (2, 1));
    }

    #[test]
    fn bounces_at_right_and_bottom_edges() {
        let mut dx = 2;
        let mut dy = 1;
        let next = bounded_next_position(
            PhysicalPosition { x: 1599, y: 719 },
            PhysicalSize {
                width: 320,
                height: 360,
            },
            &area(0, 0, 1920, 1080),
            &mut dx,
            &mut dy,
        );
        assert_eq!(next, PhysicalPosition { x: 1600, y: 720 });
        assert_eq!((dx, dy), (-2, 1));
    }

    #[test]
    fn supports_negative_monitor_origin() {
        let mut dx = -2;
        let mut dy = -1;
        let next = bounded_next_position(
            PhysicalPosition { x: -1600, y: 0 },
            PhysicalSize {
                width: 320,
                height: 360,
            },
            &area(-1920, 0, 1920, 1080),
            &mut dx,
            &mut dy,
        );
        assert_eq!(next, PhysicalPosition { x: -1602, y: 0 });
        assert_eq!((dx, dy), (-2, 1));
    }

    #[test]
    fn oversized_window_is_pinned_without_panicking() {
        let mut dx = 2;
        let mut dy = 1;
        let next = bounded_next_position(
            PhysicalPosition { x: 10, y: 20 },
            PhysicalSize {
                width: 1000,
                height: 900,
            },
            &area(-100, -50, 800, 600),
            &mut dx,
            &mut dy,
        );
        assert_eq!(next, PhysicalPosition { x: -100, y: -50 });
        assert_eq!((dx, dy), (0, 0));
    }
}
