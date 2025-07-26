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

This project is developed as a **standalone application first**, ensuring a stable and feature-rich experience on Linux, Windows, and macOS. Our future goal is to package the core technology as a **CLAP plugin** for seamless integration into digital audio workstations (DAWs).

### Core Features
*   **Binaural Convolution Engine:** Uses Head-Related Transfer Functions (HRTFs) to accurately position sound in a 3D space.
*   **SOFA File Support:** Load your own HRTF profiles in the standard SOFA format for a personalized experience.
*   **10-Band Parametric EQ:** Correct your headphone's frequency response with a powerful parametric equalizer.
*   **AutoEQ Integration:** Easily import and apply headphone correction profiles from the popular AutoEQ project.
*   **Standalone First:** A dedicated application for all major desktop operating systems.
*   **Future CLAP Support:** Planned integration with professional DAWs through the modern CLAP plugin format.

## Signal Path & Architecture

The application's audio processing is designed for high-fidelity and low latency, following a clean and logical signal path.

```mermaid
graph TD
    %% ---- Control Plane ----
    subgraph "User Controls & File Loading"
        direction LR
        UI["fa:fa-desktop User Interface (egui)"]
        SofaLoader["fa:fa-file-audio SOFA File Loader"]
        AutoEqLoader["fa:fa-file-import AutoEQ Profile Loader"]
    end

    subgraph "Configuration"
        Params["fa:fa-sliders-h Plugin Parameters"]
    end

    UI -- "Modifies & Triggers" --> Params
    UI -- "Triggers" --> SofaLoader
    UI -- "Triggers" --> AutoEqLoader

    %% ---- Real-time Audio Signal Path ----
    subgraph "Signal Path"
        direction LR
        Input["fa:fa-volume-down Stereo Input"] --> EQ["fa:fa-wave-square Headphone EQ"] --> Conv["fa:fa-headphones-alt Binaural Convolution"] --> Gain["fa:fa-volume-up Output Gain"] --> Output["fa:fa-headphones Stereo Output"]
    end

    %% ---- Control Connections to Signal Path ----
    Params -- "Controls" --> EQ
    Params -- "Controls" --> Conv
    Params -- "Controls" --> Gain
    SofaLoader -- "Provides HRTFs" --> Conv
    AutoEqLoader -- "Provides Settings" --> EQ

    %% ---- Styling ----
    classDef dsp fill:#1f2937,stroke:#60a5fa,color:#e5e7eb
    classDef io fill:#111827,stroke:#9ca3af,color:#e5e7eb
    classDef control fill:#111827,stroke:#9ca3af,color:#e5e7eb

    class Input,Output io
    class EQ,Conv,Gain dsp
    class UI,Params,SofaLoader,AutoEqLoader control

    %% ---- Link Styling ----
    linkStyle 0,1,2 stroke:#84cc16,stroke-width:2px,stroke-dasharray: 3 3
    linkStyle 7,8,9 stroke:#e53e3e,stroke-width:2px,stroke-dasharray: 3 3
    linkStyle 10,11 stroke:#ffb700,stroke-width:2px
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
    *   **On Windows:** (Instructions to be added)
    *   **On macOS:** (Instructions to be added)

### Building from Source

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/your-username/Open-Headstage.git
    cd Open-Headstage
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
