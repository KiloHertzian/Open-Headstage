// Copyright 2025 SignalVerse
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! A custom `egui` widget for visualizing virtual speaker positions.

use nih_plug_egui::egui::{
    emath::RectTransform, epaint::CircleShape, Color32, Pos2, Rect, Response, Sense, Shape, Stroke,
    Ui, Vec2, Widget,
};

/// An `egui` widget that displays a top-down view of virtual speaker positions around a listener's head.
///
/// This widget is used to provide a simple visual representation of the current speaker setup.
/// The elevation is not currently visualized but is included for future enhancements.
pub struct SpeakerVisualizer {
    /// The azimuth of the left speaker in degrees.
    pub left_azimuth: f32,
    /// The elevation of the left speaker in degrees.
    pub left_elevation: f32,
    /// The azimuth of the right speaker in degrees.
    pub right_azimuth: f32,
    /// The elevation of the right speaker in degrees.
    pub right_elevation: f32,
}

impl Widget for SpeakerVisualizer {
    /// Renders the speaker visualizer widget.
    ///
    /// # Arguments
    ///
    /// * `ui` - The `egui::Ui` to draw on.
    ///
    /// # Returns
    ///
    /// The `egui::Response` for the widget.
    fn ui(self, ui: &mut Ui) -> Response {
        // Allocate space for the widget and get a painter.
        let (response, painter) = ui.allocate_painter(Vec2::new(200.0, 200.0), Sense::hover());

        // Create a transformation from a logical coordinate system to the screen space.
        let to_screen = RectTransform::from_to(
            Rect::from_min_max(Pos2::new(-1.2, -1.2), Pos2::new(1.2, 1.2)),
            response.rect,
        );

        // Calculate the radius for the head circle based on the transformation.
        let radius_in_screen_coords = 0.5 * (to_screen.to().width() / to_screen.from().width());

        // Draw the listener's head as a circle in the center.
        painter.add(Shape::Circle(CircleShape {
            center: to_screen.transform_pos(Pos2::ZERO),
            radius: radius_in_screen_coords,
            fill: Color32::from_gray(100),
            stroke: Stroke::new(1.0, Color32::from_gray(150)),
        }));

        // Helper closure to draw a single speaker.
        let draw_speaker = |azimuth: f32, _elevation: f32, color: Color32| {
            let rad_az = azimuth.to_radians();
            // Convert polar coordinates (azimuth) to Cartesian for drawing.
            // Y is negative to account for screen coordinates (Y-down).
            let x = rad_az.sin();
            let y = -rad_az.cos();

            let pos = Pos2::new(x, y);
            painter.add(Shape::Circle(CircleShape {
                center: to_screen.transform_pos(pos),
                radius: 10.0,
                fill: color,
                stroke: Stroke::new(1.0, Color32::BLACK),
            }));
        };

        // Draw the left speaker (red).
        draw_speaker(
            self.left_azimuth,
            self.left_elevation,
            Color32::from_rgb(255, 0, 0),
        );
        // Draw the right speaker (blue).
        draw_speaker(
            self.right_azimuth,
            self.right_elevation,
            Color32::from_rgb(0, 0, 255),
        );

        response
    }
}
