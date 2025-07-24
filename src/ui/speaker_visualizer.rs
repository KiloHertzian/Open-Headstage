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

use nih_plug_egui::egui::{
    emath::RectTransform, epaint::CircleShape, Color32, Pos2, Rect, Response, Sense, Shape, Stroke,
    Ui, Vec2, Widget,
};

pub struct SpeakerVisualizer {
    pub left_azimuth: f32,
    pub left_elevation: f32,
    pub right_azimuth: f32,
    pub right_elevation: f32,
}

impl Widget for SpeakerVisualizer {
    fn ui(self, ui: &mut Ui) -> Response {
        let (response, painter) = ui.allocate_painter(Vec2::new(200.0, 200.0), Sense::hover());

        let to_screen = RectTransform::from_to(
            Rect::from_min_max(Pos2::new(-1.2, -1.2), Pos2::new(1.2, 1.2)),
            response.rect,
        );

        // Calculate the scale factor for the radius. We assume uniform scaling.
        let radius_in_screen_coords = 0.5 * (to_screen.to().width() / to_screen.from().width());

        // Draw head
        painter.add(Shape::Circle(CircleShape {
            center: to_screen.transform_pos(Pos2::ZERO),
            radius: radius_in_screen_coords,
            fill: Color32::from_gray(100),
            stroke: Stroke::new(1.0, Color32::from_gray(150)),
        }));

        // Draw speaker positions
        let draw_speaker = |azimuth: f32, _elevation: f32, color: Color32| {
            let rad_az = azimuth.to_radians();
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

        draw_speaker(
            self.left_azimuth,
            self.left_elevation,
            Color32::from_rgb(255, 0, 0),
        );
        draw_speaker(
            self.right_azimuth,
            self.right_elevation,
            Color32::from_rgb(0, 0, 255),
        );

        response
    }
}
