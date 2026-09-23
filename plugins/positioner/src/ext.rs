// Copyright 2021 Jonas Kruckenberg
// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(feature = "tray-icon")]
use crate::Tray;
use serde_repr::Deserialize_repr;
#[cfg(feature = "tray-icon")]
use tauri::Manager;
#[cfg(feature = "tray-icon")]
use tauri::Monitor;
use tauri::{PhysicalPosition, PhysicalSize, Result, Runtime, WebviewWindow, Window};

/// Well known window positions.
///
/// The `Tray*` variants require the `tray-icon` feature and only resolve once the tray icon has
/// reported its position (see `on_tray_event`); using one before that happens returns an error.
#[derive(Debug, Deserialize_repr)]
#[repr(u16)]
pub enum Position {
    /// Top left corner of the current screen.
    TopLeft = 0,
    /// Top right corner of the current screen.
    TopRight,
    /// Bottom left corner of the current screen.
    BottomLeft,
    /// Bottom right corner of the current screen.
    BottomRight,
    /// Top center of the current screen.
    TopCenter,
    /// Bottom center of the current screen.
    BottomCenter,
    /// Vertically centered on the left edge of the current screen.
    LeftCenter,
    /// Vertically centered on the right edge of the current screen.
    RightCenter,
    /// Center of the current screen.
    Center,
    /// Above the tray icon, aligning the window's left edge with the tray icon's left edge. On
    /// Windows and macOS the window moves below the icon instead when there is not enough room
    /// above.
    #[cfg(feature = "tray-icon")]
    TrayLeft,
    /// Directly below the tray icon, aligning the window's left edge with the tray icon's left
    /// edge.
    #[cfg(feature = "tray-icon")]
    TrayBottomLeft,
    /// Above the tray icon, aligning the window's left edge with the tray icon's right edge. On
    /// Windows and macOS the window moves below the icon instead when there is not enough room
    /// above.
    #[cfg(feature = "tray-icon")]
    TrayRight,
    /// Directly below the tray icon, aligning the window's left edge with the tray icon's right
    /// edge.
    #[cfg(feature = "tray-icon")]
    TrayBottomRight,
    /// Above the tray icon, horizontally centered on it. On Windows and macOS the window moves
    /// below the icon instead when there is not enough room above.
    #[cfg(feature = "tray-icon")]
    TrayCenter,
    /// Directly below the tray icon, horizontally centered on it.
    #[cfg(feature = "tray-icon")]
    TrayBottomCenter,
}

#[cfg(feature = "tray-icon")]
impl Position {
    fn is_tray(&self) -> bool {
        matches!(
            self,
            Position::TrayLeft
                | Position::TrayBottomLeft
                | Position::TrayRight
                | Position::TrayBottomRight
                | Position::TrayCenter
                | Position::TrayBottomCenter
        )
    }
}

/// A [`Window`] extension that provides extra methods related to positioning.
pub trait WindowExt {
    /// Moves the [`Window`] to the given [`Position`]
    ///
    /// All (non-tray) positions are relative to the **current** screen.
    fn move_window(&self, position: Position) -> Result<()>;
    #[cfg(feature = "tray-icon")]
    /// Moves the [`Window`] to the given [`Position`] while constraining Tray Positions to the dimensions of the screen.
    ///
    /// All non-tray positions will not be constrained by this method.
    ///
    /// This method allows you to position your Tray Windows without having them
    /// cut off on the screen borders.
    fn move_window_constrained(&self, position: Position) -> Result<()>;
}

impl<R: Runtime> WindowExt for WebviewWindow<R> {
    fn move_window(&self, pos: Position) -> Result<()> {
        self.as_ref().window().move_window(pos)
    }

    #[cfg(feature = "tray-icon")]
    fn move_window_constrained(&self, position: Position) -> Result<()> {
        self.as_ref().window().move_window_constrained(position)
    }
}

impl<R: Runtime> WindowExt for Window<R> {
    #[cfg(feature = "tray-icon")]
    fn move_window_constrained(&self, position: Position) -> Result<()> {
        // Diverge to basic move_window, if the position is not a tray position
        if !position.is_tray() {
            return self.move_window(position);
        }

        let window_position = calculate_position(self, position)?;
        let monitor = get_monitor_for_tray_icon(self)?;
        if let Some(monitor) = monitor {
            let monitor_size = monitor.size();
            let monitor_position = monitor.position();
            let window_size = self.outer_size()?;

            let right_border_monitor = monitor_position.x as f64 + monitor_size.width as f64;
            let left_border_monitor = monitor_position.x as f64;
            let right_border_window = window_position.x as f64 + window_size.width as f64;
            let left_border_window = window_position.x as f64;

            let constrained_x = if left_border_window < left_border_monitor {
                left_border_monitor
            } else if right_border_window > right_border_monitor {
                right_border_monitor - window_size.width as f64
            } else {
                window_position.x as f64
            };

            let bottom_border_monitor = monitor_position.y as f64 + monitor_size.height as f64;
            let top_border_monitor = monitor_position.y as f64;
            let bottom_border_window = window_position.y as f64 + window_size.height as f64;
            let top_border_window = window_position.y as f64;

            let constrained_y = if top_border_window < top_border_monitor {
                top_border_monitor
            } else if bottom_border_window > bottom_border_monitor {
                bottom_border_monitor - window_size.height as f64
            } else {
                window_position.y as f64
            };

            self.set_position(PhysicalPosition::new(constrained_x, constrained_y))?;
        } else {
            // Fallback on non constrained positioning
            self.set_position(window_position)?;
        }

        Ok(())
    }

    fn move_window(&self, pos: Position) -> Result<()> {
        let position = calculate_position(self, pos)?;
        self.set_position(position)
    }
}

/// Reads the tray icon rect recorded by `on_tray_event` / `set_tray_icon_state`.
///
/// Returns an error instead of panicking when the plugin was not registered, since the tray
/// state is managed by the plugin's setup hook.
#[cfg(feature = "tray-icon")]
fn tray_rect<R: Runtime>(
    window: &Window<R>,
) -> Result<Option<(PhysicalPosition<f64>, PhysicalSize<f64>)>> {
    let tray = window.try_state::<Tray>().ok_or_else(|| {
        tauri::Error::Io(std::io::Error::other(
            "The positioner plugin must be registered (`tauri_plugin_positioner::init()`) to use tray positions",
        ))
    })?;
    let rect = *tray
        .0
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    Ok(rect)
}

#[cfg(feature = "tray-icon")]
/// Retrieve the monitor, where the tray icon is located on.
fn get_monitor_for_tray_icon<R: Runtime>(window: &Window<R>) -> Result<Option<Monitor>> {
    let tray_position = tray_rect(window)?.map(|(pos, _)| pos).unwrap_or_default();

    window.monitor_from_point(tray_position.x, tray_position.y)
}

/// Calculate the top-left position of the window based on the given
/// [`Position`].
fn calculate_position<R: Runtime>(
    window: &Window<R>,
    pos: Position,
) -> Result<PhysicalPosition<i32>> {
    let window_size = window.outer_size()?;
    let window_size = PhysicalSize::<i32> {
        width: window_size.width as i32,
        height: window_size.height as i32,
    };

    // Tray positions only depend on the tray icon's rect, so they don't need the window's
    // monitor, which may be unknown while the window is hidden.
    // Only read the tray state for tray positions, so screen positions keep working when the
    // plugin is not registered.
    #[cfg(feature = "tray-icon")]
    if pos.is_tray() {
        let (tray_position, tray_size) = tray_rect(window)?
            .ok_or_else(|| tauri::Error::Io(std::io::Error::other("Tray position not set")))?;
        return Ok(tray_relative_position(
            pos,
            PhysicalPosition::new(tray_position.x as i32, tray_position.y as i32),
            PhysicalSize::new(tray_size.width as i32, tray_size.height as i32),
            window_size,
        ));
    }

    let screen = window.current_monitor()?.ok_or_else(|| {
        tauri::Error::Io(std::io::Error::other("No monitor found for the window"))
    })?;
    let screen_size = PhysicalSize::<i32> {
        width: screen.size().width as i32,
        height: screen.size().height as i32,
    };

    Ok(screen_relative_position(
        pos,
        *screen.position(),
        screen_size,
        window_size,
    ))
}

/// Top-left position of a window of `window_size` at a screen `pos` on the monitor at
/// `screen_position` with `screen_size`.
fn screen_relative_position(
    pos: Position,
    screen_position: PhysicalPosition<i32>,
    screen_size: PhysicalSize<i32>,
    window_size: PhysicalSize<i32>,
) -> PhysicalPosition<i32> {
    use Position::*;

    match pos {
        TopLeft => screen_position,
        TopRight => PhysicalPosition {
            x: screen_position.x + (screen_size.width - window_size.width),
            y: screen_position.y,
        },
        BottomLeft => PhysicalPosition {
            x: screen_position.x,
            y: screen_size.height - (window_size.height - screen_position.y),
        },
        BottomRight => PhysicalPosition {
            x: screen_position.x + (screen_size.width - window_size.width),
            y: screen_size.height - (window_size.height - screen_position.y),
        },
        TopCenter => PhysicalPosition {
            x: screen_position.x + ((screen_size.width / 2) - (window_size.width / 2)),
            y: screen_position.y,
        },
        BottomCenter => PhysicalPosition {
            x: screen_position.x + ((screen_size.width / 2) - (window_size.width / 2)),
            y: screen_size.height - (window_size.height - screen_position.y),
        },
        LeftCenter => PhysicalPosition {
            x: screen_position.x,
            y: screen_position.y + (screen_size.height / 2) - (window_size.height / 2),
        },
        RightCenter => PhysicalPosition {
            x: screen_position.x + (screen_size.width - window_size.width),
            y: screen_position.y + (screen_size.height / 2) - (window_size.height / 2),
        },
        Center => PhysicalPosition {
            x: screen_position.x + ((screen_size.width / 2) - (window_size.width / 2)),
            y: screen_position.y + (screen_size.height / 2) - (window_size.height / 2),
        },
        #[cfg(feature = "tray-icon")]
        TrayLeft | TrayBottomLeft | TrayRight | TrayBottomRight | TrayCenter | TrayBottomCenter => {
            unreachable!("tray positions are resolved by tray_relative_position")
        }
    }
}

/// Top-left position of a window of `window_size` at a tray `pos`, relative to the tray icon at
/// `tray_position` with `tray_size`.
#[cfg(feature = "tray-icon")]
fn tray_relative_position(
    pos: Position,
    tray_position: PhysicalPosition<i32>,
    tray_size: PhysicalSize<i32>,
    window_size: PhysicalSize<i32>,
) -> PhysicalPosition<i32> {
    use Position::*;

    let PhysicalPosition {
        x: tray_x,
        y: tray_y,
    } = tray_position;
    let PhysicalSize {
        width: tray_width,
        height: _tray_height,
    } = tray_size;

    // Above the tray icon; falls back to below it when there is no room above.
    let above = || {
        let y = tray_y - window_size.height;
        // Choose y value based on the target OS
        #[cfg(target_os = "windows")]
        let y = if y < 0 { tray_y + _tray_height } else { y };

        #[cfg(target_os = "macos")]
        let y = if y < 0 { tray_y } else { y };

        y
    };

    match pos {
        TrayLeft => PhysicalPosition {
            x: tray_x,
            y: above(),
        },
        TrayBottomLeft => PhysicalPosition {
            x: tray_x,
            y: tray_y,
        },
        TrayRight => PhysicalPosition {
            x: tray_x + tray_width,
            y: above(),
        },
        TrayBottomRight => PhysicalPosition {
            x: tray_x + tray_width,
            y: tray_y,
        },
        TrayCenter => PhysicalPosition {
            x: tray_x + tray_width / 2 - window_size.width / 2,
            y: above(),
        },
        TrayBottomCenter => PhysicalPosition {
            x: tray_x + (tray_width / 2) - (window_size.width / 2),
            y: tray_y,
        },
        TopLeft | TopRight | BottomLeft | BottomRight | TopCenter | BottomCenter | LeftCenter
        | RightCenter | Center => {
            unreachable!("screen positions are resolved by screen_relative_position")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WINDOW: PhysicalSize<i32> = PhysicalSize {
        width: 400,
        height: 300,
    };

    #[test]
    fn screen_positions_are_relative_to_the_monitor() {
        // a secondary monitor to the right of and above the primary one
        let origin = PhysicalPosition::new(1920, -200);
        let size = PhysicalSize::new(1280, 1024);
        let at = |pos| screen_relative_position(pos, origin, size, WINDOW);

        assert_eq!(at(Position::TopLeft), PhysicalPosition::new(1920, -200));
        assert_eq!(at(Position::TopRight), PhysicalPosition::new(2800, -200));
        assert_eq!(at(Position::BottomLeft), PhysicalPosition::new(1920, 524));
        assert_eq!(at(Position::BottomRight), PhysicalPosition::new(2800, 524));
        assert_eq!(at(Position::TopCenter), PhysicalPosition::new(2360, -200));
        assert_eq!(at(Position::BottomCenter), PhysicalPosition::new(2360, 524));
        assert_eq!(at(Position::LeftCenter), PhysicalPosition::new(1920, 162));
        assert_eq!(at(Position::RightCenter), PhysicalPosition::new(2800, 162));
        assert_eq!(at(Position::Center), PhysicalPosition::new(2360, 162));
    }

    #[cfg(feature = "tray-icon")]
    #[test]
    fn tray_positions_are_relative_to_the_tray_icon() {
        let tray = PhysicalPosition::new(1000, 800);
        let tray_size = PhysicalSize::new(20, 30);
        let at = |pos| tray_relative_position(pos, tray, tray_size, WINDOW);

        assert_eq!(at(Position::TrayLeft), PhysicalPosition::new(1000, 500));
        assert_eq!(at(Position::TrayRight), PhysicalPosition::new(1020, 500));
        assert_eq!(at(Position::TrayCenter), PhysicalPosition::new(810, 500));
        assert_eq!(
            at(Position::TrayBottomLeft),
            PhysicalPosition::new(1000, 800)
        );
        assert_eq!(
            at(Position::TrayBottomRight),
            PhysicalPosition::new(1020, 800)
        );
        assert_eq!(
            at(Position::TrayBottomCenter),
            PhysicalPosition::new(810, 800)
        );
    }
}
