// Copyright 2021 Jonas Kruckenberg
// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(feature = "tray-icon")]
use crate::Tray;
use serde::{de::Error as _, Deserialize, Deserializer};
#[cfg(feature = "tray-icon")]
use tauri::Manager;
#[cfg(feature = "tray-icon")]
use tauri::Monitor;
use tauri::{PhysicalPosition, PhysicalSize, Result, Runtime, WebviewWindow, Window};

/// Well known window positions.
///
/// The `Tray*` variants require the `tray-icon` feature and only resolve once the tray icon has
/// reported its position (see `on_tray_event`); using one before that happens returns an error.
#[derive(Debug)]
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

/// Deserializes from the variant's numeric value (as sent by the JS `Position` enum), with a
/// clear error when a `Tray*` value is used without the `tray-icon` feature.
impl<'de> Deserialize<'de> for Position {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        use Position::*;

        let value = u16::deserialize(deserializer)?;
        Ok(match value {
            0 => TopLeft,
            1 => TopRight,
            2 => BottomLeft,
            3 => BottomRight,
            4 => TopCenter,
            5 => BottomCenter,
            6 => LeftCenter,
            7 => RightCenter,
            8 => Center,
            #[cfg(feature = "tray-icon")]
            9 => TrayLeft,
            #[cfg(feature = "tray-icon")]
            10 => TrayBottomLeft,
            #[cfg(feature = "tray-icon")]
            11 => TrayRight,
            #[cfg(feature = "tray-icon")]
            12 => TrayBottomRight,
            #[cfg(feature = "tray-icon")]
            13 => TrayCenter,
            #[cfg(feature = "tray-icon")]
            14 => TrayBottomCenter,
            #[cfg(not(feature = "tray-icon"))]
            9..=14 => {
                return Err(D::Error::custom(
                    "tray positions require the `tray-icon` feature of `tauri-plugin-positioner`",
                ))
            }
            _ => {
                return Err(D::Error::custom(format_args!(
                    "invalid position `{value}`, expected a value between 0 and 14"
                )))
            }
        })
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
        if !matches!(
            position,
            Position::TrayLeft
                | Position::TrayBottomLeft
                | Position::TrayRight
                | Position::TrayBottomRight
                | Position::TrayCenter
                | Position::TrayBottomCenter
        ) {
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

#[cfg(feature = "tray-icon")]
/// Retrieve the monitor, where the tray icon is located on.
fn get_monitor_for_tray_icon<R: Runtime>(window: &Window<R>) -> Result<Option<Monitor>> {
    let tray_position = window
        .state::<Tray>()
        .0
        .lock()
        .unwrap()
        .map(|(pos, _)| pos)
        .unwrap_or_default();

    window.monitor_from_point(tray_position.x, tray_position.y)
}

/// Calculate the top-left position of the window based on the given
/// [`Position`].
fn calculate_position<R: Runtime>(
    window: &Window<R>,
    pos: Position,
) -> Result<PhysicalPosition<i32>> {
    use Position::*;

    let screen = window.current_monitor()?.ok_or_else(|| {
        tauri::Error::Io(std::io::Error::other("No monitor found for the window"))
    })?;
    // Only use the screen_position for the Tray independent positioning,
    // because a tray event may not be called on the currently active monitor.
    let screen_position = screen.position();
    let screen_size = PhysicalSize::<i32> {
        width: screen.size().width as i32,
        height: screen.size().height as i32,
    };
    let window_size = PhysicalSize::<i32> {
        width: window.outer_size()?.width as i32,
        height: window.outer_size()?.height as i32,
    };
    #[cfg(feature = "tray-icon")]
    let (tray_position, tray_size) = window
        .state::<Tray>()
        .0
        .lock()
        .unwrap()
        .map(|(pos, size)| {
            (
                Some((pos.x as i32, pos.y as i32)),
                Some((size.width as i32, size.height as i32)),
            )
        })
        .unwrap_or_default();

    let physical_pos = match pos {
        TopLeft => *screen_position,
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
        TrayLeft => {
            if let (Some((tray_x, tray_y)), Some((_, _tray_height))) = (tray_position, tray_size) {
                let y = tray_y - window_size.height;
                // Choose y value based on the target OS
                #[cfg(target_os = "windows")]
                let y = if y < 0 { tray_y + _tray_height } else { y };

                #[cfg(target_os = "macos")]
                let y = if y < 0 { tray_y } else { y };

                PhysicalPosition { x: tray_x, y }
            } else {
                return Err(tauri::Error::Io(std::io::Error::other(
                    "Tray position not set",
                )));
            }
        }
        #[cfg(feature = "tray-icon")]
        TrayBottomLeft => {
            if let Some((tray_x, tray_y)) = tray_position {
                PhysicalPosition {
                    x: tray_x,
                    y: tray_y,
                }
            } else {
                return Err(tauri::Error::Io(std::io::Error::other(
                    "Tray position not set",
                )));
            }
        }
        #[cfg(feature = "tray-icon")]
        TrayRight => {
            if let (Some((tray_x, tray_y)), Some((tray_width, _tray_height))) =
                (tray_position, tray_size)
            {
                let y = tray_y - window_size.height;
                // Choose y value based on the target OS
                #[cfg(target_os = "windows")]
                let y = if y < 0 { tray_y + _tray_height } else { y };

                #[cfg(target_os = "macos")]
                let y = if y < 0 { tray_y } else { y };

                PhysicalPosition {
                    x: tray_x + tray_width,
                    y,
                }
            } else {
                return Err(tauri::Error::Io(std::io::Error::other(
                    "Tray position not set",
                )));
            }
        }
        #[cfg(feature = "tray-icon")]
        TrayBottomRight => {
            if let (Some((tray_x, tray_y)), Some((tray_width, _))) = (tray_position, tray_size) {
                PhysicalPosition {
                    x: tray_x + tray_width,
                    y: tray_y,
                }
            } else {
                return Err(tauri::Error::Io(std::io::Error::other(
                    "Tray position not set",
                )));
            }
        }
        #[cfg(feature = "tray-icon")]
        TrayCenter => {
            if let (Some((tray_x, tray_y)), Some((tray_width, _tray_height))) =
                (tray_position, tray_size)
            {
                let x = tray_x + tray_width / 2 - window_size.width / 2;
                let y = tray_y - window_size.height;
                // Choose y value based on the target OS
                #[cfg(target_os = "windows")]
                let y = if y < 0 { tray_y + _tray_height } else { y };

                #[cfg(target_os = "macos")]
                let y = if y < 0 { tray_y } else { y };

                PhysicalPosition { x, y }
            } else {
                return Err(tauri::Error::Io(std::io::Error::other(
                    "Tray position not set",
                )));
            }
        }
        #[cfg(feature = "tray-icon")]
        TrayBottomCenter => {
            if let (Some((tray_x, tray_y)), Some((tray_width, _))) = (tray_position, tray_size) {
                PhysicalPosition {
                    x: tray_x + (tray_width / 2) - (window_size.width / 2),
                    y: tray_y,
                }
            } else {
                return Err(tauri::Error::Io(std::io::Error::other(
                    "Tray position not set",
                )));
            }
        }
    };

    Ok(physical_pos)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_deserializes_from_its_numeric_value() {
        let position: Position = serde_json::from_value(serde_json::json!(8)).unwrap();
        assert!(matches!(position, Position::Center));

        let error = serde_json::from_value::<Position>(serde_json::json!(15)).unwrap_err();
        assert!(error.to_string().contains("invalid position `15`"));
    }

    #[cfg(feature = "tray-icon")]
    #[test]
    fn tray_position_deserializes_with_the_tray_icon_feature() {
        let position: Position = serde_json::from_value(serde_json::json!(14)).unwrap();
        assert!(matches!(position, Position::TrayBottomCenter));
    }

    #[cfg(not(feature = "tray-icon"))]
    #[test]
    fn tray_position_names_the_missing_feature() {
        let error = serde_json::from_value::<Position>(serde_json::json!(13)).unwrap_err();
        assert!(error.to_string().contains("`tray-icon` feature"));
    }
}
