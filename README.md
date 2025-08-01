# Open Headstage

<p align="center">
  <img src="https://raw.githubusercontent.com/user-attachments/assets/a270c007-213a-428c-8a8a-5a304855b1b7" alt="Open Headstage Logo" width="150"/>
</p>

<h3 align="center">A Multiplatform Binaural Speaker Simulator</h3>

<p align="center">
  <strong>Open Headstage is an open-source audio application that recreates the experience of listening to high-end stereo speakers in a room, all within your headphones.</strong>
  <br /><br />
  <a href="https://github.com/your-username/Open-Headstage/releases">Downloads</a>
  ·
  <a href="https://github.com/your-username/Open-Headstage/issues/new?template=bug_report.md">Report Bug</a>
  ·
  <a href="https://github.com/your-username/Open-Headstage/issues/new?template=feature_request.md">Request Feature</a>
</p>

---

## About The Project

Open Headstage is a professional-grade audio processing tool for enthusiasts and professionals who want to achieve a more natural and immersive listening experience on headphones. By using advanced digital signal processing (DSP), it simulates the way sound from stereo speakers interacts with your head and ears, creating a "phantom" soundstage in front of you.

This project is developed as a **standalone application first**, ensuring a stable and feature-rich experience on Linux, Windows, and macOS. The core technology is also planned to be bundled as a **CLAP plugin** for integration into digital audio workstations (DAWs), although this is a secondary goal and full DAW compatibility is still under development.

### Core Features
<div style="font-size: 0.9em;">

*   **Binaural Convolution Engine:** Uses Head-Related Transfer Functions (HRTFs) to accurately position sound in a 3D space.
*   **SOFA File Support:** Load your own HRTF profiles in the standard SOFA format for a personalized experience.
*   **10-Band Parametric EQ:** Correct your headphone's frequency response with a powerful parametric equalizer.
*   **AutoEQ Integration:** Easily import and apply headphone correction profiles from the popular AutoEQ project.
*   **Standalone First:** A dedicated application for Linux, Windows, and macOS with selectable audio backends (JACK, ALSA, etc.).
*   **CLAP Plugin Support (Experimental):** An experimental CLAP plugin is available but is not yet consistently detected or loaded by all DAWs.

</div>

## Signal Path & Architecture

The application is designed with a clear separation between the user interface, background data loading, and the real-time audio processing thread to ensure a responsive and glitch-free experience.

### High-Level Architecture

This diagram shows the main components of the Open Headstage application and how they interact with the user and the underlying system.

```mermaid
graph TD
    %% === Style Definitions ===
    classDef user fill:#60a5fa,stroke:#2563eb,color:#fff,font-weight:bold
    classDef system fill:#a78bfa,stroke:#6d28d9,color:#fff,font-weight:bold
    classDef ui fill:#4ade80,stroke:#16a34a,color:#333
    classDef data fill:#facc15,stroke:#ca8a04,color:#333
    classDef realtime fill:#f87171,stroke:#b91c1c,color:#fff
    classDef io fill:#9ca3af,stroke:#4b5563,color:#fff

    %% === Node Declarations ===
    subgraph "External World"
        direction LR
        User([User]):::user
        FileSystem([File System]):::system
        AudioOS[("Audio OS<br>(JACK, ALSA, etc.)")]:::system
    end

    subgraph "Open Headstage Application"
        direction TB
        
        subgraph "UI & Control Plane (Main Thread)"
            direction LR
            UI(Egui UI):::ui
            PluginState(Plugin State &<br>Parameters):::ui
        end

        subgraph "Data Loading (Background Threads)"
            direction LR
            SofaLoader(SOFA Loader):::data
            AutoEQLoader(AutoEQ Loader):::data
        end

        subgraph "Real-time Audio Engine (Audio Thread)"
            direction LR
            AudioInput[("Stereo In")]:::io --> AudioEngine("DSP Core"):::realtime --> AudioOutput[("Stereo Out")]:::io
        end
    end

    %% === Connections ===
    User -- "Controls" --> UI
    UI -- "Manages" --> PluginState
    
    UI -. "Triggers Load" .-> SofaLoader
    UI -. "Triggers Load" .-> AutoEQLoader

    FileSystem -- "Reads .sofa file" --> SofaLoader
    FileSystem -- "Reads .txt file" --> AutoEQLoader

    SofaLoader -. "Provides HRTFs" .-> AudioEngine
    AutoEQLoader -. "Provides EQ Settings" .-> AudioEngine
    
    PluginState -- "Configures" --> AudioEngine

    AudioOS -- "Provides" --> AudioInput
    AudioOutput -- "Sends to" --> AudioOS
```

### Real-time Audio Signal Flow

This diagram details the specific stages of the real-time digital signal processing (DSP) chain that your audio passes through.

```mermaid
graph LR
    %% === Style Definitions ===
    classDef io fill:#9ca3af,stroke:#4b5563,color:#fff,font-weight:bold
    classDef dsp fill:#4ade80,stroke:#16a34a,color:#333
    classDef control fill:#facc15,stroke:#ca8a04,color:#333
    classDef data fill:#a78bfa,stroke:#6d28d9,color:#fff

    %% === Node Declarations ===
    subgraph "DSP Chain (Audio Thread)"
        direction LR
        Input([Stereo Input]):::io --> EQ(10-Band<br>Parametric EQ):::dsp
        EQ --> Convolution(Binaural<br>Convolution):::dsp
        Convolution --> Gain(Output<br>Gain):::dsp
        Gain --> Output([Stereo Output]):::io
    end

    subgraph "Control & Data (from Main Thread)"
        direction TB
        PluginState(Plugin Parameters):::control
        HRTFData(HRTF Data):::data
    end

    %% === Connections ===
    PluginState -- "EQ Settings" --> EQ
    PluginState -- "Speaker Angles" --> Convolution
    PluginState -- "Gain Level" --> Gain
    HRTFData -- "HRIRs" --> Convolution
```

## Getting Started

### Prerequisites

To build Open Headstage from source, you will need the following tools and libraries.

*   **Rust:** Version 1.87.0 or newer.
*   **System Dependencies:**
    *   **On Linux (Debian/Ubuntu):**
        ```bash
        sudo apt-get update
        sudo apt-get install -y libgl-dev libx11-xcb-dev libmysofa-dev libgtk-3-dev
        ```
    *   **On Linux (Arch/CachyOS):**
        ```bash
        sudo pacman -Syu --needed mesa libx11 libxcb libmysofa gtk3
        ```
    *   **On Windows:** (Instructions pending)
    *   **On macOS:** (Instructions pending)

### Building from Source

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/your-username/Open-Headstage.git
    cd Open-Headstage/app
    ```
2.  **Build the standalone application:**
    ```bash
    cargo build --release
    ```
3.  **Run the application:**
    The compiled executable will be located in the `target/release` directory.
    ```bash
    ./target/release/open-headstage
    ```
4.  **Build the CLAP plugin (Manual & Experimental):**
    The CLAP plugin (`.so` file) must be manually copied into a `.clap` bundle. Note that DAW detection is not guaranteed.
    ```bash
    # First, ensure the library is built
    cargo build --release
    # Create the bundle directory
    mkdir -p ~/.clap/open-headstage.clap
    # Copy the shared library into the bundle
    cp target/release/libopen_headstage.so ~/.clap/open-headstage.clap/open-headstage.so
    ```
    For more details on plugin validation, refer to the "Operational Reminder (Plugin Validation)" in `TODO.md`.

## How to Contribute

Contributions are what make the open source community such an amazing place to learn, inspire, and create. Any contributions you make are **greatly appreciated**.

1.  Fork the Project
2.  Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3.  Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4.  Push to the Branch (`git push origin feature/AmazingFeature`)
5.  Open a Pull Request

Please refer to our issue templates for bug reports and feature requests.

## License

Distributed under the Apache License, Version 2.0. See `LICENSE` for more information.

**Note on VST3:** VST3 support is currently disabled. If it were to be re-enabled in the future, the `vst3-sys` crate's GPLv3 license would require any distributed binary containing the VST3 version of this plugin to also have its corresponding source code made available under the GPLv3. Refer to `LICENSES.md` for full details on all project dependencies and their licenses.