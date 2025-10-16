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

//! # Open Headstage Standalone Executable
//!
//! This is the main entry point for the standalone version of the Open Headstage plugin.
//!
//! It uses the `nih_export_standalone` function from the `nih_plug` framework to
//! wrap the main `OpenHeadstagePlugin` struct in a standalone application host. This allows
//! the plugin to be run as a desktop application without needing a separate DAW.

use nih_plug::prelude::*;
use open_headstage::OpenHeadstagePlugin;

/// The main function that launches the standalone application.
fn main() {
    nih_export_standalone::<OpenHeadstagePlugin>();
}
