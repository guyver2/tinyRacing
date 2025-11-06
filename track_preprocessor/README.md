# Track Preprocessor

Utility tool for preprocessing race tracks in the TinyRacing game.

## Purpose

This tool processes track data to prepare it for use in the TinyRacing game engine. It converts a SVG path into a evenly spaced points along the track with an associated curvature value indicating how straight or how curvy the track is at that location.

## Usage

```bash
# Run the track preprocessor
python process.py <directory_path>
```

## Input Format

Track files should follow the standard TinyRacing track format specification.

## Output

Generates preprocessed track data files ready for game engine consumption.

## Dependencies

- Python 3.x
- NumPy
- OpenCV (for visualization)