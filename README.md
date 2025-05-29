# Open Headstage

Open Headstage is an open-source binaural speaker simulation plugin for headphones, designed for Linux-based audio professionals and enthusiasts.
The goal is to provide a high-quality, flexible tool for experiencing stereo audio as if listening to physical speakers in a well-defined acoustic space.

## Current Phase
Phase 1: Anechoic Core Development (Project Initialization)

## Core Features (MVP - Phase 1)
- Binaural Convolution Engine (4-path for anechoic HRTFs)
- Direct SOFA HRTF/BRIR file loading (`.sofa`)
- Speaker angle selection (manual and presets)
- Headphone Parametric Equalization (10-band PEQ with AutoEq import)
- LV2 and VST3 plugin formats for Linux

## Tech Stack (Planned)
- Language: Rust
- Plugin Framework: `nih-plug`
- SOFA Library: `libmysofa` (via FFI)
- FFT Library: `RustFFT` (or similar)
- Resampling: `rubato` (or similar)

## Building from Source
(Instructions to be added)

## How to Contribute
(Guidelines to be added)

## Roadmap
(Detailed roadmap to be developed)

graph TD
    A[Stereo Audio Input] --> B{Binaural Convolution Engine};
    C[SOFA HRTF Manager] -.->|Provides 4 HRIRs| B;
    B --> D(Anechoic Binaural Output);
    D --> E{Speaker Emulation EQ};
    E --> F{Room Simulation Module};
    F --> G{Headphone Equalization};
    G --> H{Output Gain & Bypass};
    H --> I[Stereo Audio Output];

    J[User Interface (UI)] --> K[Plugin Parameters]; // Changed K to a standard box and connection
    K -.-> C;
    K -.-> B;
    K -.-> E;
    K -.-> F;
    K -.-> G;
    K -.-> H;

    subgraph "Phase 1: Anechoic Core (MVP)"
        B
        C
        G
        H
    end

    subgraph "Phase 2: Enhancements"
        E
        F
    end

    style A fill:#lightgreen,stroke:#333,stroke-width:2px
    style I fill:#lightgreen,stroke:#333,stroke-width:2px
    style B fill:#lightblue,stroke:#333,stroke-width:2px
    style C fill:#f9f,stroke:#333,stroke-width:2px
    style D fill:#skyblue,stroke:#333,stroke-width:1px,stroke-dasharray: 5 5
    style E fill:#wheat,stroke:#333,stroke-width:2px
    style F fill:#wheat,stroke:#333,stroke-width:2px
    style G fill:#lightblue,stroke:#333,stroke-width:2px
    style H fill:#lightgray,stroke:#333,stroke-width:2px
    style J fill:#whitesmoke,stroke:#333,stroke-width:2px
    style K fill:#whitesmoke,stroke:#333,stroke-width:2px
