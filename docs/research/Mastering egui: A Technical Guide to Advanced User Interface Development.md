Mastering egui: A Technical Guide to Advanced User Interface Development


Introduction

This report serves as a definitive technical guide for Rust developers looking to transcend the basics of the egui library. The primary objective is to provide the advanced techniques, architectural patterns, and performance considerations necessary to build sophisticated, high-performance, and aesthetically customized user interfaces. It is intended for developers who are familiar with egui's fundamental widgets and are now seeking to harness its full capabilities to create professional-grade applications.
At the heart of egui is the immediate mode paradigm, a design philosophy that distinguishes it from traditional retained mode toolkits.1 In an immediate mode GUI, the user interface is described and rendered from scratch on every frame. The application's state is the single source of truth, and the UI is simply a function that transforms this state into a visual representation for a given frame.2 This approach prioritizes simplicity and direct control, eliminating complex state synchronization issues and callback chains that are common in other frameworks.3 Understanding this philosophy is the key to mastering
egui, as the patterns for layout, state management, and optimization are all derived from this core principle.
This guide is structured into five core sections. It begins with Advanced Layout and Composition, tackling the nuances of creating complex and responsive UIs. It then moves to Robust State Management Patterns, detailing how to structure application logic for clarity and scalability, including handling asynchronous operations. The third section, Custom Widget Development, provides a comprehensive tutorial on extending egui's vocabulary by building new, interactive components from the ground up. Following this, Theming and Visual Customization explores how to tailor the look and feel of an application, from color palettes to custom typography. Finally, Performance Optimization addresses how to ensure applications remain fluid and efficient, even when dealing with large datasets or complex rendering requirements. Each section is rich with practical, copy-paste-ready code examples to bridge the gap between theory and implementation.

Advanced Layout and Composition

Creating dynamic, organized, and responsive layouts is often cited as a primary challenge in immediate-mode GUI development.4 This is because the layout system cannot know the size and position of all elements in advance; it discovers them as it executes the UI code for the current frame.6 This section moves beyond simple horizontal and vertical containers to provide the patterns necessary for building complex, professional-grade layouts. It covers the
egui::Grid for structured data alignment, strategies for creating UIs that adapt gracefully to window resizing, and the strategic composition of egui's primary layout containers.

Mastering egui::Grid for Complex Data

The egui::Grid is the principal tool for aligning widgets in a two-dimensional, tabular format. It is ideal for creating forms, data tables, and settings panels where labels and controls need to be perfectly aligned across multiple rows.1
A key feature of egui::Grid is its ability to remember column widths between frames. To enable this, a unique and stable identifier must be provided upon creation with egui::Grid::new("some_unique_id"). This ID is used as a key to store layout information in egui's persistent memory.7
Creating a Complex, Non-Trivial Grid
The following example demonstrates a settings panel constructed with a grid. It utilizes builder methods like .striped(true) to add alternating background colors to rows for improved readability and .spacing() to define custom padding between cells.7

Rust


// In your eframe::App `update` method:
fn settings_grid(ui: &mut egui::Ui, settings: &mut MyAppSettings) {
    egui::Grid::new("settings_grid")
       .num_columns(2)
       .spacing([40.0, 8.0])
       .striped(true)
       .show(ui, |ui| {
            // --- First Row ---
            ui.label("Username");
            ui.text_edit_singleline(&mut settings.username);
            ui.end_row();

            // --- Second Row ---
            ui.label("Render Quality");
            egui::ComboBox::from_label("Select render quality")
               .selected_text(format!("{:?}", settings.quality))
               .show_ui(ui, |ui| {
                    ui.selectable_value(&mut settings.quality, Quality::Low, "Low");
                    ui.selectable_value(&mut settings.quality, Quality::Medium, "Medium");
                    ui.selectable_value(&mut settings.quality, Quality::High, "High");
                });
            ui.end_row();

            // --- Third Row ---
            ui.label("Max Framerate");
            ui.add(egui::Slider::new(&mut settings.max_fps, 30..=240).suffix(" FPS"));
            ui.end_row();
        });
}

// Example structs for context
#
enum Quality { Low, Medium, High }

struct MyAppSettings {
    username: String,
    quality: Quality,
    max_fps: u32,
}

impl Default for MyAppSettings {
    fn default() -> Self {
        Self {
            username: "user".to_string(),
            quality: Quality::Medium,
            max_fps: 60,
        }
    }
}


Nesting UI Elements within Grid Cells
A grid cell is not limited to a single widget; it is a Ui region that can contain any combination of other widgets and layouts. This is essential for creating complex forms where multiple controls correspond to a single label.

Rust


// Inside the Grid::show closure:
ui.label("Volume Controls");
ui.horizontal(|ui| {
    ui.add(egui::DragValue::new(&mut self.volume).speed(0.1).clamp_range(0.0..=1.0));
    if ui.button("Mute").clicked() {
        self.volume = 0.0;
    }
});
ui.end_row();


Advanced Column Alignment
By default, egui::Grid aligns content to the left.7 It does not provide direct methods for controlling the alignment of entire columns. Instead, alignment is achieved by using specific layouts
within each cell. To right-align content, the most robust method is to allocate the cell and then use ui.with_layout with a egui::Layout::right_to_left configuration. This forces the content within that cell to be laid out from the right edge.

Rust


egui::Grid::new("alignment_demo")
   .num_columns(3)
   .show(ui, |ui| {
        // Headers
        ui.label("Left-Aligned");
        ui.vertical_centered(|ui| { ui.label("Center-Aligned"); });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label("Right-Aligned");
        });
        ui.end_row();

        // Data Row
        ui.label("Data A");
        ui.vertical_centered(|ui| { ui.button("Action B"); });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.checkbox(&mut self.is_enabled, "");
        });
        ui.end_row();
    });


A critical aspect of egui::Grid's behavior stems from the immediate mode paradigm. On the first frame a grid is shown, it cannot know the final width of all its columns until every widget in every cell has been laid out. It therefore renders the first frame with guessed column widths, measures the actual required widths, and stores them in memory. On the second frame, it uses these stored widths to render correctly.3 This process causes a noticeable one-frame visual glitch or "jitter."
To create a professional, flicker-free UI, this must be addressed. The solution is to instruct egui to perform a second layout pass within the same rendered frame, effectively hiding the incorrect first pass from the user. This is achieved by calling egui::Context::request_discard(). When egui sees this request, it finishes the current update call, discards the resulting paint commands, and immediately runs the update loop again. By the second run, the correct column widths are in memory, and the grid renders perfectly.3

Responsive Layouts & Dynamic Resizing

Responsive design in egui is handled programmatically, not declaratively as in web development with CSS.8 The core pattern involves querying the available screen space within the current
Ui and using conditional logic to alter the layout structure based on that information.4 The primary tools for this are
ui.available_width() and ui.available_height().
Pattern: Switching Layout Based on Width
A common responsive pattern is to switch from a horizontal layout on wider screens to a vertical, stacked layout on narrower screens. This is achieved by defining a width threshold (a "breakpoint") and using an if/else block to choose the appropriate layout container.

Rust


const RESPONSIVE_BREAKPOINT: f32 = 500.0;

let available_width = ui.available_width();

if available_width > RESPONSIVE_BREAKPOINT {
    // Wide screen layout: horizontal
    ui.horizontal(|ui| {
        responsive_content(ui);
    });
} else {
    // Narrow screen layout: vertical
    ui.vertical(|ui| {
        responsive_content(ui);
    });
}

// Helper function to draw the actual content
fn responsive_content(ui: &mut egui::Ui) {
    ui.group(|ui| {
        ui.set_width(150.0);
        ui.label("Component 1");
        ui.label("Some descriptive text.");
    });
    ui.group(|ui| {
        ui.set_width(150.0);
        ui.label("Component 2");
        ui.label("Some descriptive text.");
    });
    ui.group(|ui| {
        ui.set_width(150.0);
        ui.label("Component 3");
        ui.label("Some descriptive text.");
    });
}


Dynamically Allocating Space
The available_width() and available_height() methods can also be used to make widgets fill the available space. By combining these methods with ui.add_sized(), a developer can create widgets that expand or contract to fit their container dynamically. This is particularly useful for elements like text editors or canvases that should use all the space provided to them.

Rust


// This text edit will fill all available horizontal space,
// but only take up the vertical space it needs.
let desired_size = egui::vec2(ui.available_width(), 0.0);
ui.add_sized(desired_size, egui::TextEdit::singleline(&mut self.search_query));

// This scroll area will fill all remaining space in its parent container.
egui::ScrollArea::vertical().show(ui, |ui| {
    ui.label(self.long_text.clone());
});



Custom Layout Containers (Panels, Areas, and Windows)

The foundation of any egui application's layout is its top-level containers. These containers partition the screen space and serve as the root Ui regions where all other widgets are placed. It is critical to understand their differences and the strict order in which they must be added: SidePanel and TopBottomPanel must be declared before the CentralPanel. Window and Area containers are drawn on top of all panels and should be declared after them.10
Container
Positioning
Sizing
Interaction
Typical Use Case
CentralPanel
Fills remaining space
Expands to fill
N/A
Main content area of an application.
SidePanel
Docks to left/right
Resizable width
Resizable handle
Toolbars, navigation menus, inspector panels.
TopBottomPanel
Docks to top/bottom
Resizable height
Resizable handle
Menu bars, status bars, headers/footers.
Window
Floating, draggable
Resizable
Draggable, resizable
Dialogs, tool palettes, pop-up inspectors.
Area
Floating, draggable
Sized by content
Draggable
Simple floating widgets, overlays, node editors.

Complex Application Layout Example
The following example demonstrates a common application structure combining a side panel for controls, a central panel for the main content, and a floating window for debugging information.

Rust


// In your eframe::App struct:
struct ComplexApp {
    selected_tool: String,
    show_debug_window: bool,
    some_debug_value: f32,
}

// In the `update` method:
fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    // Left Side Panel for tools
    egui::SidePanel::left("toolbox_panel")
       .resizable(true)
       .default_width(150.0)
       .show(ctx, |ui| {
            ui.heading("Toolbox");
            ui.separator();
            if ui.button("Tool A").clicked() {
                self.selected_tool = "Tool A".to_owned();
            }
            if ui.button("Tool B").clicked() {
                self.selected_tool = "Tool B".to_owned();
            }
            ui.separator();
            ui.checkbox(&mut self.show_debug_window, "Show Debug Info");
        });

    // Main Central Panel for content
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Main Canvas");
        ui.separator();
        ui.label(format!("Selected tool: {}", self.selected_tool));
        //... main content would be drawn here based on the selected tool...
        self.some_debug_value += ui.ctx().input(|i| i.stable_dt);
    });

    // Floating Debug Window
    if self.show_debug_window {
        egui::Window::new("Debug Info")
           .open(&mut self.show_debug_window) // The `open` bool is mutable, so the close button works
           .resizable(true)
           .default_width(200.0)
           .show(ctx, |ui| {
                ui.label(format!("Frame time: {:.2} ms", ui.ctx().input(|i| i.stable_dt * 1000.0)));
                ui.label(format!("Debug value: {:.2}", self.some_debug_value));
            });
    }
}


Creating a Modal Dialog That Returns a Result
While egui's Window is not modal by default, a modal dialog pattern can be implemented by controlling the application state. A modal dialog should block interaction with the rest of the UI and return a result (e.g., "Ok" or "Cancel") to the main application state upon closing. This is achieved by drawing the modal window and, if it is open, drawing a semi-transparent backdrop that covers the entire screen and intercepts mouse events.
The state management for such a dialog typically involves an Option in the main App struct. When it is Some, the dialog is displayed. The buttons within the dialog then mutate the application state and set the dialog state back to None.

Rust


// Add to your eframe::App struct:
struct MyApp {
    //... other state...
    confirmation_dialog_open: bool,
    last_confirmation_result: Option<bool>,
}

// In the `update` method:
fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    egui::CentralPanel::default().show(ctx, |ui| {
        if ui.button("Perform Dangerous Action").clicked() {
            self.confirmation_dialog_open = true;
        }

        if let Some(result) = self.last_confirmation_result {
            ui.label(format!("Last action was: {}", if result { "Confirmed" } else { "Cancelled" }));
        }
    });

    if self.confirmation_dialog_open {
        // A modal window is just a centered window with a dark backdrop.
        // The backdrop is a semi-transparent panel that covers the entire screen.
        // It catches all clicks, preventing interaction with the rest of the UI.
        let frame = egui::Frame::default()
           .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 128));

        egui::Area::new("modal_backdrop")
           .fixed_pos(egui::Pos2::ZERO)
           .show(ctx, |ui| {
                ui.with_layer_id(egui::LayerId::new(egui::Order::PanelResize, egui::Id::new("modal_layer")), |ui| {
                    ui.set_max_size(ctx.screen_rect().size());
                    ui.allocate_response(ui.available_size(), egui::Sense::click());
                });
            });


        egui::Window::new("Confirm Action")
           .collapsible(false)
           .resizable(false)
           .show(ctx, |ui| {
                ui.label("Are you sure you want to proceed with this dangerous action?");
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button("Confirm").clicked() {
                        self.confirmation_dialog_open = false;
                        self.last_confirmation_result = Some(true);
                    }
                    if ui.button("Cancel").clicked() {
                        self.confirmation_dialog_open = false;
                        self.last_confirmation_result = Some(false);
                    }
                });
            });
    }
}



Robust State Management Patterns

Effective state management is the cornerstone of a well-structured egui application. Because the UI is a direct reflection of the application's data each frame, how that data is organized and modified is paramount. This section covers the two most important patterns for managing state: the idiomatic eframe::App struct for centralized, synchronous state, and the use of channels for safely communicating with background threads to handle asynchronous operations without blocking the UI.

The "App" Struct Pattern

The canonical pattern for building an application with eframe is to encapsulate all of the application's persistent state within a single struct that implements the eframe::App trait.11 This central struct becomes the single source of truth. The
update method, called on every frame, receives a mutable reference to this struct (&mut self), allowing it to read the current state to draw the UI and write back any changes resulting from user interaction.12
Complete, Idiomatic Example of eframe::App
This example demonstrates a typical App struct with a simple state, a new function for initialization (called by eframe), and the update method where the UI is defined.

Rust


// main.rs
fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "My App",
        options,
        Box::new(|_cc| Box::<MyApp>::default()),
    )
}

#
struct MyApp {
    name: String,
    age: u32,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My Application");
            ui.horizontal(|ui| {
                ui.label("Your name: ");
                ui.text_edit_singleline(&mut self.name);
            });
            ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
            if ui.button("Increment").clicked() {
                self.age += 1;
            }
            ui.label(format!("Hello '{}', age {}", self.name, self.age));
        });
    }
}


Handling Complex, Nested State
For larger applications, placing all state variables directly in the main App struct can become unwieldy. A better approach is to group related state into nested structs. This improves organization, modularity, and makes the code easier to reason about.2

Rust


#
struct AppSettings {
    dark_mode: bool,
    font_size: f32,
}

#
struct UserData {
    username: String,
    score: i32,
}

#
struct ComplexApp {
    settings: AppSettings,
    user_data: UserData,
    //... other top-level state
}

impl eframe::App for ComplexApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        //...
        // The UI can now be broken down into functions that operate on these nested structs.
    }
}


Passing Mutable State to Sub-functions
To keep the main update method clean and organized, UI logic should be broken down into smaller, focused functions. These functions can take a mutable reference to a slice of the application's state, allowing them to modify it directly. This pattern is fundamental to building scalable and maintainable egui applications.3

Rust


// In `impl eframe::App for ComplexApp`
fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    egui::SidePanel::left("settings_panel").show(ctx, |ui| {
        Self::settings_ui(ui, &mut self.settings);
    });
    egui::CentralPanel::default().show(ctx, |ui| {
        Self::user_data_ui(ui, &self.user_data);
    });
}

// Associated functions to handle parts of the UI
impl ComplexApp {
    fn settings_ui(ui: &mut egui::Ui, settings: &mut AppSettings) {
        ui.heading("Settings");
        ui.checkbox(&mut settings.dark_mode, "Dark Mode");
        ui.add(egui::Slider::new(&mut settings.font_size, 10.0..=30.0).text("Font Size"));
    }

    fn user_data_ui(ui: &mut egui::Ui, user_data: &UserData) {
        ui.heading("User Info");
        ui.label(format!("User: {}", user_data.username));
        ui.label(format!("Score: {}", user_data.score));
    }
}



Decoupling State with Channels

Any operation that might take a significant amount of time—such as network requests, disk I/O, or heavy computation—must be executed on a separate thread. Performing such tasks on the main UI thread will cause the application to freeze and become unresponsive.1 The standard and safest way to communicate between a background thread and the UI thread is through channels.
The pattern involves spawning a background thread to perform the work. The UI thread holds the receiving end of a channel. The background thread performs its task and sends the result back through the sending end of the channel. In the update loop, the UI thread polls the channel for new messages in a non-blocking way using try_recv().
Using crossbeam-channel and Context::request_repaint()
When a message is received from a background thread, the application state is updated. However, because this state change did not originate from direct user input in the current frame (like a mouse click), egui does not know that it needs to redraw the UI. To solve this, one must explicitly call egui::Context::request_repaint() after processing the message. This signals to the eframe backend that the UI is "dirty" and needs to be repainted on the next frame, ensuring that the changes from the background thread become visible.15
The following example demonstrates a robust implementation using crossbeam-channel. A background thread simulates a network request, and upon completion, sends the result to the UI thread, which then updates its state and requests a repaint.

Rust


use std::time::Duration;
use crossbeam_channel::{unbounded, Receiver, Sender};

enum NetworkStatus {
    Idle,
    Loading,
    Success(String),
    Error(String),
}

struct AsyncApp {
    status: NetworkStatus,
    // We need the sender to be cloneable to send to the thread
    sender: Sender<NetworkStatus>,
    receiver: Receiver<NetworkStatus>,
}

impl Default for AsyncApp {
    fn default() -> Self {
        let (sender, receiver) = unbounded();
        Self {
            status: NetworkStatus::Idle,
            sender,
            receiver,
        }
    }
}

impl eframe::App for AsyncApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Poll for messages from the background thread without blocking.
        if let Ok(new_status) = self.receiver.try_recv() {
            self.status = new_status;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            match &self.status {
                NetworkStatus::Idle => {
                    if ui.button("Fetch Data").clicked() {
                        self.status = NetworkStatus::Loading;
                        let sender = self.sender.clone();
                        let ctx = ctx.clone(); // Clone context to request repaint from thread

                        std::thread::spawn(move |

| {
                            // Simulate a network request
                            std::thread::sleep(Duration::from_secs(2));

                            // Simulate a successful response
                            let result = "Data fetched successfully from the server.".to_string();
                            sender.send(NetworkStatus::Success(result)).unwrap();

                            // IMPORTANT: Request a repaint to show the result.
                            ctx.request_repaint();
                        });
                    }
                }
                NetworkStatus::Loading => {
                    ui.add(egui::Spinner::new());
                    ui.label("Loading...");
                }
                NetworkStatus::Success(data) => {
                    ui.label("Success!");
                    ui.label(data);
                    if ui.button("Fetch Again").clicked() {
                        self.status = NetworkStatus::Idle;
                    }
                }
                NetworkStatus::Error(err) => {
                    ui.colored_label(egui::Color32::RED, "Error!");
                    ui.label(err);
                    if ui.button("Try Again").clicked() {
                        self.status = NetworkStatus::Idle;
                    }
                }
            }
        });
    }
}



Custom Widget Development

While egui provides a rich set of built-in widgets, the ability to create entirely new, application-specific components is essential for building advanced and unique user interfaces. This section provides a comprehensive guide to custom widget development. The process involves implementing the egui::Widget trait, using the egui::Painter API for custom rendering, and handling user input through egui's Sense and Response system. To illustrate these concepts, a non-trivial rotary knob slider widget will be developed step-by-step.

Implementing impl egui::Widget

The egui::Widget trait is the core interface for any object that can be added to a Ui via ui.add().18 The standard pattern is to create a builder struct for the widget. This struct holds configuration data and a mutable reference to the state it will modify (e.g.,
&mut f32). This builder is then consumed when its ui method is called, which performs the three essential tasks of any widget: allocating space, handling input, and painting.
Step-by-Step Tutorial: A Rotary Knob Widget
This tutorial will create a RotaryKnob widget, a circular slider controlled by dragging the mouse up/down or left/right.
Step 1: Define the Builder Struct
The RotaryKnob struct will act as a builder. It holds a mutable reference to the f32 value it controls, along with its range and an optional label.

Rust


pub struct RotaryKnob<'a> {
    value: &'a mut f32,
    range: std::ops::RangeInclusive<f32>,
    label: Option<String>,
}

impl<'a> RotaryKnob<'a> {
    pub fn new(value: &'a mut f32, range: std::ops::RangeInclusive<f32>) -> Self {
        Self { value, range, label: None }
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}


Step 2: Implement the egui::Widget Trait
The ui method is the entry point for the widget. It orchestrates the allocation, interaction, and painting logic.

Rust


impl egui::Widget for RotaryKnob<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        // Implementation will go here...
    }
}


Step 3: Allocate Space and Sense Interaction
The first step inside the ui method is to allocate a rectangular area for the widget on the screen. ui.allocate_response is the perfect tool for this, as it both reserves space and checks for user interaction in one call. The egui::Sense parameter tells egui what kind of interactions to listen for. For a knob, Sense::click_and_drag() is appropriate, as it can be both clicked and dragged.20

Rust


// Inside the `ui` method:
let desired_size = egui::vec2(50.0, 50.0);
let (rect, mut response) = ui.allocate_response(desired_size, egui::Sense::click_and_drag());


Step 4: Handle Input via the Response Object
The returned egui::Response object contains all information about user interactions that occurred within the widget's rectangle during the current frame.21 For the knob, the primary interaction is dragging. The
response.drag_delta() method provides the change in mouse position since the last frame, which can be used to update the widget's value.

Rust


// Inside the `ui` method, after allocation:
if response.dragged() {
    // Use horizontal drag delta to change the value.
    // A sensitivity factor is useful to control how fast the value changes.
    let sensitivity = (self.range.end() - self.range.start()) / 100.0;
    let delta = response.drag_delta().x * sensitivity;
    *self.value = (*self.value + delta).clamp(*self.range.start(), *self.range.end());
    response.mark_changed(); // Mark the value as changed
}


Step 5 & 6: Custom Painting with egui::Painter
If the widget is visible within the viewport (ui.is_rect_visible(rect)), a Painter object can be obtained from the Ui. This object provides low-level drawing primitives like circles, lines, and text, all operating in screen-space coordinates.23 The knob's appearance is constructed by drawing a background circle, an indicator line showing the current value, and the label.

Rust


// Inside the `ui` method, after input handling:
if ui.is_rect_visible(rect) {
    let visuals = ui.style().interact(&response);
    let painter = ui.painter_at(rect);
    let center = rect.center();
    let radius = rect.width() / 2.0 - 5.0;

    // Draw the knob background
    painter.circle(
        center,
        radius,
        visuals.bg_fill,
        visuals.bg_stroke,
    );

    // Draw the indicator
    let angle = egui::remap_clamp(*self.value, self.range.clone(), -std::f32::consts::FRAC_PI_2 * 1.5..=std::f32::consts::FRAC_PI_2 * 1.5);
    let indicator_end = center + egui::vec2(radius * angle.sin(), -radius * angle.cos());
    painter.line_segment([center, indicator_end], visuals.fg_stroke);

    // Draw the label if it exists
    if let Some(label) = self.label {
        painter.text(
            rect.center_bottom() + egui::vec2(0.0, 15.0),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::default(),
            visuals.text_color(),
        );
    }
}

response


This completes the custom widget. It can now be used in any Ui region like a standard widget: ui.add(RotaryKnob::new(&mut self.my_value, 0.0..=100.0).with_label("Volume"));.

Mastering egui::Painter

The egui::Painter API is the gateway to custom 2D graphics within egui. It allows for drawing various shapes, text, and images directly onto the screen. The following is a gallery of examples demonstrating its capabilities.
Gallery of Painter Examples

Rust


fn painter_gallery(ui: &mut egui::Ui) {
    let (response, painter) = ui.allocate_painter(
        egui::vec2(ui.available_width(), 200.0),
        egui::Sense::hover(),
    );
    let rect = response.rect;

    // --- Custom Shapes ---
    // Arc (as a path)
    let arc_center = rect.left_center() + egui::vec2(50.0, 0.0);
    let radius = 40.0;
    let points: Vec<egui::Pos2> = (0..=180)
       .map(|i| {
            let angle = egui::emath::remap(i as f32, 0.0..=180.0, -std::f32::consts::PI, 0.0);
            arc_center + egui::vec2(angle.cos() * radius, angle.sin() * radius)
        })
       .collect();
    painter.add(egui::Shape::line(points, egui::Stroke::new(2.0, egui::Color32::GREEN)));

    // Cubic Bézier Curve
    let bezier_start = rect.center() - egui::vec2(100.0, 50.0);
    let bezier_end = rect.center() + egui::vec2(100.0, 50.0);
    let control1 = bezier_start + egui::vec2(50.0, -100.0);
    let control2 = bezier_end - egui::vec2(50.0, -100.0);
    let bezier_shape = egui::epaint::CubicBezierShape::from_points_stroke(
        [bezier_start, control1, control2, bezier_end],
        false, // Not closed
        egui::Color32::TRANSPARENT,
        egui::Stroke::new(2.0, egui::Color32::LIGHT_BLUE),
    );
    painter.add(egui::Shape::CubicBezier(bezier_shape));

    // --- Custom Text ---
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "Custom Text",
        egui::FontId::new(24.0, egui::FontFamily::Proportional),
        egui::Color32::YELLOW,
    );

    // --- Mesh with custom vertices and colors ---
    let mut mesh = egui::Mesh::default();
    mesh.add_colored_vertex(
        rect.right_center() + egui::vec2(-100.0, -50.0),
        egui::Color32::RED,
    );
    mesh.add_colored_vertex(
        rect.right_center() + egui::vec2(-20.0, 0.0),
        egui::Color32::GREEN,
    );
    mesh.add_colored_vertex(
        rect.right_center() + egui::vec2(-100.0, 50.0),
        egui::Color32::BLUE,
    );
    mesh.add_triangle(0, 1, 2);
    painter.add(egui::Shape::mesh(mesh));
}



Input Handling (Sense and Response)

The interaction model in egui is built entirely on the relationship between egui::Sense and egui::Response. When a widget allocates space, it declares what kind of input it is interested in via a Sense object. After the allocation, the returned Response object reports whether any of those interactions occurred during the frame.
Demonstration in the Rotary Knob
The rotary knob widget provides a clear example of this system in action.
Sense::click_and_drag(): This sense is chosen because the knob needs to react to both distinct clicks (which could be used to reset the value) and continuous dragging motions.20
Interpreting the Response:
response.dragged(): This boolean is checked every frame. If true, it means the user is holding down a mouse button and moving the cursor over the widget's area. The widget's value is updated based on response.drag_delta().25
response.hovered(): This boolean is used to provide visual feedback. The ui.style().interact(&response) call automatically selects the appropriate colors and stroke widths based on whether the widget is hovered, dragged, or inactive.21
response.clicked(): This could be used to implement a feature where clicking the knob resets its value to a default.
Handling Keyboard Input
To make a custom widget respond to keyboard input, it must first be able to receive keyboard focus.
Become Focusable: The widget must declare its interest in focus by including Sense::focusable_noninteractive() in its sense flags during allocation. let sense = egui::Sense::click_and_drag() | egui::Sense::focusable_noninteractive();
Request Focus: A widget can request focus, typically in response to a click. if response.clicked() { response.request_focus(); }.26
Check for Focus and Input: In the widget's ui method, it must check if it currently has focus using response.has_focus(). If it does, it can then poll the global input state for key presses.27

Rust


// Inside the RotaryKnob's `ui` method:
if response.has_focus() {
    let input = ui.input(|i| i.clone());
    let mut new_value = *self.value;
    let step = (self.range.end() - self.range.start()) / 100.0;

    if input.key_pressed(egui::Key::ArrowUp) |

| input.key_pressed(egui::Key::ArrowRight) {
        new_value += step;
    }
    if input.key_pressed(egui::Key::ArrowDown) |

| input.key_pressed(egui::Key::ArrowLeft) {
        new_value -= step;
    }
    
    *self.value = new_value.clamp(*self.range.start(), *self.range.end());
}



Theming and Visual Customization

While egui's default theme is clean and functional, creating a unique visual identity is crucial for producing a polished, professional application. egui provides a powerful and centralized system for theming through the egui::Style struct, which controls everything from colors and spacing to widget visuals. This section covers how to modify this global style at runtime and how to load and manage custom fonts for advanced typography.

Modifying egui::Style

The egui::Style struct is the single source of truth for all visual parameters in an egui application. It contains two primary fields: spacing, which controls sizes, padding, and margins, and visuals, which controls colors and stroke properties. The global style can be accessed and modified at any time via the egui::Context.
Changing Global Theme Elements
To modify the theme for the entire application, one can get a mutable reference to the Style from the context and change its fields. These changes will take effect on the very next frame.

Rust


// This function can be called once at startup to set a custom theme.
fn configure_styles(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    
    // Alter spacing
    style.spacing.item_spacing = egui::vec2(10.0, 4.0);
    style.spacing.window_margin = egui::Margin::same(12.0);
    
    // Alter visuals
    style.visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::GRAY);
    style.visuals.widgets.inactive.bg_fill = egui::Color32::from_gray(60);
    
    ctx.set_style(style);
}


Dark Mode / Light Mode Toggle at Runtime
A common requirement is to allow users to switch between dark and light themes. egui provides pre-configured egui::Visuals::dark() and egui::Visuals::light() themes that can be applied to the context at runtime. The egui::widgets module also provides convenient helper functions like global_theme_preference_switch for this purpose.28
The following example demonstrates how to implement a manual theme toggle button.

Rust


// In your eframe::App `update` method:
fn theme_toggle_ui(ui: &mut egui::Ui) {
    let current_theme = if ui.visuals().dark_mode { "🌙 Dark" } else { "☀️ Light" };
    
    if ui.button(current_theme).clicked() {
        let visuals = if ui.visuals().dark_mode {
            egui::Visuals::light()
        } else {
            egui::Visuals::dark()
        };
        ui.ctx().set_visuals(visuals);
    }
}


This code checks the dark_mode boolean on the current Visuals. If the button is clicked, it swaps the current visuals for the other pre-built theme and applies it to the context using ctx.set_visuals().29

Working with Fonts

egui's font system is managed through the egui::FontDefinitions struct. This struct defines what font data is available and how it maps to different font families and text styles. Custom fonts, such as .ttf or .otf files, can be loaded and integrated into this system.30
Loading and Applying a Custom Font
The process of loading a custom font involves creating a FontDefinitions object, inserting the raw font data, associating that data with a font family, and finally applying the new definitions to the egui::Context. This is typically done once when the application starts.

Rust


// In the `new` function of your eframe::App implementation:
fn new(cc: &eframe::CreationContext<'_>) -> Self {
    setup_custom_fonts(&cc.egui_ctx);
    //... other setup...
    Self::default()
}

fn setup_custom_fonts(ctx: &egui::Context) {
    // Start with the default fonts.
    let mut fonts = egui::FontDefinitions::default();

    // Install our custom font family.
    //.ttf and.otf files are supported.
    fonts.font_data.insert(
        "lato_regular".to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/Lato-Regular.ttf")),
    );

    // Put my font first (highest priority) for proportional text:
    fonts
       .families
       .entry(egui::FontFamily::Proportional)
       .or_default()
       .insert(0, "lato_regular".to_owned());

    // Tell egui to use these fonts:
    ctx.set_fonts(fonts);
}


Defining and Switching Between Multiple Font Styles
An advanced use case is to define multiple distinct font styles (e.g., "heading", "body", "monospace") and switch between them. egui has built-in TextStyle enums (Heading, Body, Monospace, Button, Small). The key to creating custom styles is to define custom FontFamily names and then map these families to the desired TextStyle.
The following example loads three different fonts and assigns them to three different logical styles.

Rust


fn setup_multiple_font_styles(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // Load font data
    fonts.font_data.insert(
        "roboto_slab".to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/RobotoSlab-Regular.ttf")),
    );
    fonts.font_data.insert(
        "lato_regular".to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/Lato-Regular.ttf")),
    );
    fonts.font_data.insert(
        "hack_regular".to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/Hack-Regular.ttf")),
    );

    // Define custom font families
    fonts
       .families
       .insert(egui::FontFamily::Name("heading".into()), vec!["roboto_slab".to_owned()]);
    fonts
       .families
       .insert(egui::FontFamily::Name("body".into()), vec!["lato_regular".to_owned()]);
    
    // Use the existing Monospace family for our mono font
    fonts
       .families
       .entry(egui::FontFamily::Monospace)
       .or_default()
       .insert(0, "hack_regular".to_owned());

    // Configure the text styles to use our custom families
    let mut text_styles = std::collections::BTreeMap::new();
    text_styles.insert(
        egui::TextStyle::Heading,
        egui::FontId::new(30.0, egui::FontFamily::Name("heading".into())),
    );
    text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::new(18.0, egui::FontFamily::Name("body".into())),
    );
    text_styles.insert(
        egui::TextStyle::Monospace,
        egui::FontId::new(16.0, egui::FontFamily::Monospace),
    );
    text_styles.insert(
        egui::TextStyle::Button,
        egui::FontId::new(18.0, egui::FontFamily::Name("body".into())),
    );
    text_styles.insert(
        egui::TextStyle::Small,
        egui::FontId::new(14.0, egui::FontFamily::Name("body".into())),
    );
    
    ctx.set_style(egui::Style { text_styles,..egui::Style::default() });
}

// In the UI, these styles are used automatically:
// ui.heading("This uses Roboto Slab");
// ui.label("This uses Lato");
// ui.code("This uses Hack");



Performance Optimization

While egui is designed to be fast, building complex applications requires an understanding of its performance characteristics to ensure a smooth user experience. The immediate mode architecture means that the entire UI is rebuilt every frame, which can lead to high CPU usage if not managed correctly. This section covers the two most important areas of performance optimization: understanding the frame lifecycle to avoid unnecessary repaints and employing efficient strategies for displaying large datasets that would otherwise be too slow to render every frame.

Avoiding Unnecessary Repaints

By default, egui operates in a "reactive" mode. This means the update loop, which builds and paints the UI, is only executed when there is new input, such as mouse movement, a click, or a key press. When the application is idle, egui does not run the update loop, resulting in near-zero CPU usage.17 This is a highly efficient model for most standard applications.
However, if the application's state can change from an external source (like a network response from a background thread or a timer), the UI will not automatically update. In these cases, the external process must explicitly tell egui that a repaint is needed.
The egui Frame Lifecycle
Event Polling: The eframe backend waits for an event from the operating system (e.g., mouse input, window resize, keyboard press).
Repaint Trigger: An event triggers a repaint. Alternatively, a call to egui::Context::request_repaint() from anywhere in the application will also schedule a repaint for the next frame.
update() Call: The eframe::App::update method is called. The application's UI code is executed, building a list of shapes to be drawn.
Painting: The backend renders the generated shapes to the screen.
Idle: The backend returns to step 1, waiting for the next event or repaint request.
Practical Example of State-Driven Repainting
This example demonstrates an application that only repaints when its state is changed by a background timer. The timer sends a message every second, and upon receiving the message, the UI updates its state and calls ctx.request_repaint() to ensure the change is displayed. Without this call, the displayed time would only update when the user moves the mouse or interacts with the window.

Rust


use std::sync::mpsc::{channel, Receiver};
use std::time::{Duration, Instant};

struct RepaintApp {
    last_update: Instant,
    time_receiver: Receiver<Instant>,
}

impl Default for RepaintApp {
    fn default() -> Self {
        let (sender, time_receiver) = channel();
        std::thread::spawn(move |

| {
            loop {
                std::thread::sleep(Duration::from_secs(1));
                sender.send(Instant::now()).unwrap();
            }
        });

        Self {
            last_update: Instant::now(),
            time_receiver,
        }
    }
}

impl eframe::App for RepaintApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for a message from the timer thread.
        if let Ok(new_time) = self.time_receiver.try_recv() {
            self.last_update = new_time;
            // The state has changed, so we must request a repaint.
            ctx.request_repaint();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Performance Optimization");
            ui.label("This UI only repaints when its state changes.");
            ui.label(format!("Last update from background thread: {:?} ago", Instant::now().duration_since(self.last_update)));
            ui.label("Move your mouse to see continuous repaints, then stop.");
            ui.label(format!("Current time: {:?}", chrono::Local::now().format("%H:%M:%S")));
        });
    }
}



Efficiently Displaying Large Datasets

A significant performance bottleneck in immediate mode GUIs occurs when trying to display a very large number of widgets, such as thousands of rows in a log viewer or data table. Laying out every single widget on every frame is computationally expensive and will lead to a slow, unresponsive UI.33
The standard and most effective solution to this problem is virtual scrolling (also known as "windowing"). The principle is to only create, lay out, and draw the widgets that are currently visible within the scrollable viewport. The application calculates the total size of the scrollable area and informs egui, but it only generates the widgets for the visible subset.35
Code Example: ScrollArea::show_rows for Uniform Lists
For the common case of displaying a long list where every item has the same, known height, egui provides a highly optimized helper: egui::ScrollArea::show_rows. This function calculates which range of rows is visible and provides this range to a closure. The application code then only needs to iterate over this much smaller range to create the widgets.

Rust


// In your eframe::App struct:
struct LargeListApp {
    // A large dataset of 100,000 items.
    all_items: Vec<String>,
}

// In the `update` method:
fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Displaying 100,000 items efficiently");

        // Assume a fixed row height. This is crucial for `show_rows`.
        let text_style = egui::TextStyle::Body;
        let row_height = ui.text_style_height(&text_style);
        let total_rows = self.all_items.len();

        egui::ScrollArea::vertical().show_rows(ui, row_height, total_rows, |ui, row_range| {
            // `row_range` contains the start and end indices of the visible rows.
            // We only need to iterate over this small slice of our data.
            for row_index in row_range {
                let item_text = &self.all_items[row_index];
                ui.label(format!("[{}] {}", row_index, item_text));
            }
        });
    });
}


Advanced Strategies for Variable Heights and Complex Data
When list items have variable heights or the layout is more complex (e.g., a grid), show_rows is not sufficient. In these cases, developers can turn to more advanced techniques:
Data Culling with ScrollArea::show_viewport: This is a lower-level version of show_rows. It provides the UI closure with the visible Rect in the content's coordinate system. The application is then responsible for calculating which items fall within this rectangle and rendering only those. This requires more manual calculation but offers maximum flexibility.
Pre-rendering to a Texture: For extremely complex, static data like a large scatter plot or a waveform, it can be more performant to pre-render the entire dataset to a single large texture. The UI then simply displays this texture inside a ScrollArea. This approach trades memory usage for rendering speed and is best suited for non-interactive or minimally interactive data visualizations.
Third-Party Crates: The egui ecosystem has produced powerful solutions for this problem. The egui_virtual_list crate provides a virtual list component that supports items with varying heights by lazily calculating and caching their sizes as the user scrolls.36 For infinite scrolling behavior, the
egui_infinite_scroll crate, built on top of egui_virtual_list, is an excellent choice.37

Conclusion

The egui library, built on the principles of immediate mode, offers a uniquely straightforward and powerful approach to GUI development in Rust. By embracing its core philosophy—that the UI is a direct and ephemeral function of the application's state—developers can avoid entire classes of bugs related to state synchronization and build complex, interactive applications with surprising simplicity.
This report has traversed the landscape of advanced egui development, moving from foundational layout composition to the intricacies of performance optimization. The mastery of egui::Grid and responsive patterns provides the tools to build structured and adaptive interfaces. The disciplined application of the App struct pattern, coupled with the safe use of channels for asynchronous tasks, establishes a robust architecture for managing application state. Furthermore, the ability to create custom widgets from scratch by implementing the egui::Widget trait and leveraging the Painter API unlocks limitless possibilities for UI innovation. Finally, a deep understanding of theming through egui::Style and performance tuning via controlled repaints and virtual scrolling elevates an application from a functional prototype to a polished and professional product.
By integrating these advanced techniques, developers can harness the full capabilities of egui, creating user interfaces that are not only feature-rich and performant but also visually distinct and a pleasure to use. The immediate mode paradigm, when fully understood and leveraged, proves to be not a constraint, but a powerful asset for building clear, maintainable, and highly interactive software.
Works cited
Rust egui: A Step-by-Step Tutorial for Absolute Beginners - HackMD, accessed July 27, 2025, https://hackmd.io/@Hamze/Sys9nvF6Jl
Egui in 2025 : How was your development experience? : r/rust - Reddit, accessed July 27, 2025, https://www.reddit.com/r/rust/comments/1m5fwen/egui_in_2025_how_was_your_development_experience/
egui - Rust - Docs.rs, accessed July 27, 2025, https://docs.rs/egui/latest/egui/
Layout Questions and Feedback · emilk egui · Discussion #168 ..., accessed July 27, 2025, https://github.com/emilk/egui/discussions/168
Help with egui basics? : r/rust - Reddit, accessed July 27, 2025, https://www.reddit.com/r/rust/comments/13hwu5e/help_with_egui_basics/
Improving layouting · Issue #4378 · emilk/egui - GitHub, accessed July 27, 2025, https://github.com/emilk/egui/issues/4378
Grid in egui - Rust - Docs.rs, accessed July 27, 2025, https://docs.rs/egui/latest/egui/struct.Grid.html
egui_css - crates.io: Rust Package Registry, accessed July 27, 2025, https://crates.io/crates/egui_css
Ui in egui - Rust - Docs.rs, accessed July 27, 2025, https://docs.rs/egui/latest/egui/struct.Ui.html
egui::containers::panel - Rust - Docs.rs, accessed July 27, 2025, https://docs.rs/egui/latest/egui/containers/panel/index.html
rust - How to access egui/eframe values from other widgets? - Stack ..., accessed July 27, 2025, https://stackoverflow.com/questions/74492580/how-to-access-egui-eframe-values-from-other-widgets
Please help me understand some egui / eframe semantics : r/rust - Reddit, accessed July 27, 2025, https://www.reddit.com/r/rust/comments/125fwut/please_help_me_understand_some_egui_eframe/
What am I missing in EGUI? : r/rust - Reddit, accessed July 27, 2025, https://www.reddit.com/r/rust/comments/1bw52qz/what_am_i_missing_in_egui/
egui: an easy-to-use immediate mode GUI in Rust that runs on both web and native - GitHub, accessed July 27, 2025, https://github.com/emilk/egui
Context in egui - Rust - traffloat.github.io, accessed July 27, 2025, https://traffloat.github.io/api/master/egui/struct.Context.html
How do I comunicate with an egui app? : r/rust - Reddit, accessed July 27, 2025, https://www.reddit.com/r/rust/comments/we84ch/how_do_i_comunicate_with_an_egui_app/
Continuous message processing · emilk egui · Discussion #995 - GitHub, accessed July 27, 2025, https://github.com/emilk/egui/discussions/995
Widget in egui::widgets - Rust - Docs.rs, accessed July 27, 2025, https://docs.rs/egui/latest/egui/widgets/trait.Widget.html
Widget in egui::widgets - Rust, accessed July 27, 2025, https://doc.qu1x.dev/bevy_trackball/egui/widgets/trait.Widget.html
Sense in egui - Rust - Docs.rs, accessed July 27, 2025, https://docs.rs/egui/latest/egui/struct.Sense.html
Response in egui - Docs.rs, accessed July 27, 2025, https://docs.rs/egui/latest/egui/response/struct.Response.html
Response in egui - Rust - openrr.github.io, accessed July 27, 2025, https://openrr.github.io/openrr/egui/struct.Response.html
Draw on top of an image · emilk egui · Discussion #2967 - GitHub, accessed July 27, 2025, https://github.com/emilk/egui/discussions/2967
How does egui painting system really work? · emilk egui · Discussion #3852 - GitHub, accessed July 27, 2025, https://github.com/emilk/egui/discussions/3852
Response in egui - Rust - traffloat.github.io, accessed July 27, 2025, https://traffloat.github.io/api/master/egui/struct.Response.html
Memory in egui - Rust - Docs.rs, accessed July 27, 2025, https://docs.rs/egui/latest/egui/struct.Memory.html
how to make a widget be focusable · emilk egui · Discussion #4812 - GitHub, accessed July 27, 2025, https://github.com/emilk/egui/discussions/4812
global_dark_light_mode_switch in egui::widgets - Rust - openrr.github.io, accessed July 27, 2025, https://openrr.github.io/openrr/egui/widgets/fn.global_dark_light_mode_switch.html
toggle global light and dark mode? · emilk egui · Discussion #148 ..., accessed July 27, 2025, https://github.com/emilk/egui/discussions/148
FontDefinitions in egui - Rust - traffloat.github.io, accessed July 27, 2025, https://traffloat.github.io/api/master/egui/struct.FontDefinitions.html
FontDefinitions in egui - Rust - Docs.rs, accessed July 27, 2025, https://docs.rs/egui/latest/egui/struct.FontDefinitions.html
I Switched from Flutter and Rust to Rust and Egui - Hacker News, accessed July 27, 2025, https://news.ycombinator.com/item?id=44361288
Best practices for large scrollable areas · emilk egui · Discussion ..., accessed July 27, 2025, https://github.com/emilk/egui/discussions/2443
Optimize Plot · Issue #18 · emilk/egui_plot - GitHub, accessed July 27, 2025, https://github.com/emilk/egui/issues/1485
Optimizing Large Datasets with Virtualized Lists | by Eva Matova | Medium, accessed July 27, 2025, https://medium.com/@eva.matova6/optimizing-large-datasets-with-virtualized-lists-70920e10da54
egui_virtual_list - crates.io: Rust Package Registry, accessed July 27, 2025, https://crates.io/crates/egui_virtual_list
egui_infinite_scroll — Rust GUI library // Lib.rs, accessed July 27, 2025, https://lib.rs/crates/egui_infinite_scroll
