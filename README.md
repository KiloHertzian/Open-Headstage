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

## Bloc Diagram
```mermaid
graph TD
    %% == Node Definitions ==
    A["Stereo Audio Input"]
    B["Binaural Convolution Engine"]
    C["SOFA HRTF Manager"]
    D["Anechoic Binaural Output"]
    E["Speaker Emulation EQ"]
    F["Room Simulation Module"]
    G["Headphone Equalization"]
    H["Output Gain & Bypass"]
    I["Stereo Audio Output"]
    J["User Interface"]
    K_Params["Plugin Config"]

    %% == Connections ==
    A --> B;
    C -.->|Provides 4 HRIRs| B;
    B --> D;
    D --> E;
    E --> F;
    F --> G;
    G --> H;
    H --> I;

    J --> K_Params;
    K_Params -.-> C;
    K_Params -.-> B;
    K_Params -.-> E;
    K_Params -.-> F;
    K_Params -.-> G;
    K_Params -.-> H;

    %% == Subgraphs ==
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

    %% == Styling ==
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
    style K_Params fill:#whitesmoke,stroke:#333,stroke-width:2px
```

## Building from Source
(Instructions to be added)

## How to Contribute
(Guidelines to be added)

## Roadmap
(Detailed roadmap to be developed)
