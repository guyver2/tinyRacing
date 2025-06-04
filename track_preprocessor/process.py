"""
SVG Track Processor

This module processes SVG files containing race tracks and extracts positional
information along the track path.
"""

import sys
from pathlib import Path
import xml.etree.ElementTree as ET
from typing import Tuple, Optional
import json
import math
import struct

try:
    from svgpathtools import parse_path
except ImportError:
    print("Error: svgpathtools is not installed. Please install it with: pip install svgpathtools")
    sys.exit(1)

try:
    import matplotlib.pyplot as plt
    import numpy as np
except ImportError:
    print("Error: matplotlib and numpy are required for plotting. Please install them with: pip install matplotlib numpy")
    sys.exit(1)


class SVGTrackProcessor:
    """Process SVG files containing track paths."""
    
    SVG_NAMESPACE = {'svg': 'http://www.w3.org/2000/svg'}
    
    def __init__(self, svg_file_path: Path, track_id: str = "track"):
        """
        Initialize the processor with an SVG file path and load the track data.
        
        Args:
            svg_file_path: Path to the SVG file
            track_id: ID of the track path element (default: "track")
        """
        self.svg_file_path = svg_file_path
        self.track_id = track_id
        self._tree: Optional[ET.Element] = None
        self._path_data: Optional[str] = None
        self._parsed_path = None  # svgpathtools.Path object
        self._total_length: Optional[float] = None
        
        # Load everything at initialization
        self._load_and_parse_track()
        
    def _load_and_parse_track(self) -> None:
        """Load SVG file and parse the track path data."""
        self.load_svg()
        self._path_data = self.find_track_path()
        
        try:
            self._parsed_path = parse_path(self._path_data)
            if self._parsed_path is not None:
                self._total_length = self._parsed_path.length()
            else:
                raise ValueError("Failed to parse path data")
        except Exception as e:
            raise ValueError(f"Error parsing SVG path: {e}")
    
    def load_svg(self) -> None:
        """Load and parse the SVG file."""
        if not self.svg_file_path.exists():
            raise FileNotFoundError(f"SVG file not found: {self.svg_file_path}")
            
        try:
            with open(self.svg_file_path, 'r', encoding='utf-8') as f:
                svg_data = f.read()
            self._tree = ET.fromstring(svg_data)
        except ET.ParseError as e:
            raise ValueError(f"Invalid SVG file: {e}")
        except Exception as e:
            raise ValueError(f"Error reading SVG file: {e}")
    
    def find_track_path(self) -> str:
        """
        Find and extract the path data for the track.
            
        Returns:
            SVG path data string
            
        Raises:
            ValueError: If no track path is found
        """
        if self._tree is None:
            raise ValueError("SVG not loaded")
            
        track_element = self._tree.find(f".//svg:path[@id='{self.track_id}']", self.SVG_NAMESPACE)
        
        if track_element is None:
            raise ValueError(f"No path with id='{self.track_id}' found in SVG file")
            
        path_data = track_element.get('d')
        if not path_data:
            raise ValueError(f"Path with id='{self.track_id}' has no 'd' attribute")
            
        return path_data
    
    @property
    def path_data(self) -> str:
        """Get the SVG path data string."""
        if self._path_data is None:
            raise ValueError("Path data not loaded")
        return self._path_data
    
    @property
    def total_length(self) -> float:
        """Get the total length of the track path."""
        if self._total_length is None:
            raise ValueError("Path not parsed")
        return self._total_length
    
    def get_position_at_distance_ratio(self, distance_ratio: float) -> Tuple[float, float]:
        """
        Get the (x, y) position at a specific distance ratio along the path.
        
        Args:
            distance_ratio: Ratio of distance along path (0.0 to 1.0)
            
        Returns:
            Tuple of (x, y) coordinates
            
        Raises:
            ValueError: If distance_ratio is not between 0 and 1
        """
        if not 0 <= distance_ratio <= 1:
            raise ValueError("distance_ratio must be between 0 and 1")
            
        if self._parsed_path is None:
            raise ValueError("Path not loaded")
            
        target_distance = distance_ratio * self.total_length
        
        # Get parameter t such that the arc length from 0 to t equals target_distance
        t = self._parsed_path.ilength(target_distance)
        point = self._parsed_path.point(t)
        
        return point.real, point.imag
    
    def get_track_info(self, distance_ratio: float = 0.5) -> dict:
        """
        Get comprehensive track information.
        
        Args:
            distance_ratio: Ratio of distance along path to get position for
            
        Returns:
            Dictionary containing track information
        """
        x, y = self.get_position_at_distance_ratio(distance_ratio)
        
        return {
            'total_length': self.total_length,
            'position_at_ratio': {
                'ratio': distance_ratio,
                'distance': distance_ratio * self.total_length,
                'x': x,
                'y': y
            },
            'path_data': self.path_data
        }

def print_track_info(track_info: dict):
    """Display track information from JSON with nice formatting."""
    print('\n📋 Track Information (from track.json):')
    
    # Display each field with formatting
    if 'id' in track_info and track_info['id']:
        print(f'   🏷️  ID: {track_info["id"]}')
    
    if 'name' in track_info and track_info['name']:
        print(f'   🏁 Name: {track_info["name"]}')
    
    if 'description' in track_info and track_info['description']:
        print(f'   📖 Description: {track_info["description"]}')
    
    if 'laps' in track_info and track_info['laps']:
        print(f'   🔄 Laps: {track_info["laps"]}')
    
    if 'lap_length_km' in track_info and track_info['lap_length_km']:
        print(f'   📏 Lap Length: {track_info["lap_length_km"]} km')
    
    if 'svg_start_offset' in track_info and track_info['svg_start_offset'] is not None:
        print(f'   🎯 SVG Start Offset: {track_info["svg_start_offset"]}')


class TrackPoint:
    """A point on the track with its 2d position, distance from the start as a ratio of the lap length, and local curvature."""
    def __init__(self, x: float, y: float, distance: float, curvature: float = 0):
        self.x = x
        self.y = y
        self.distance = distance
        self.curvature = curvature

def compute_track_curvature(track_data: dict, processor: SVGTrackProcessor, step_size: float = 20):
    """Compute curvature of the track at each step (in meters)."""
    step_size_ratio = step_size / (track_data['lap_length_km']*1000)
    nb_steps = int(1 / step_size_ratio)
    print(f"Nb steps: {nb_steps}")
    curvature = []
    for i in range(0, nb_steps):
        ratio = i*step_size_ratio
        x, y = processor.get_position_at_distance_ratio(ratio)
        curvature.append(TrackPoint(x, y, ratio))
    
    for i,p in enumerate(curvature):
        # Get previous, current, and next points
        prev_point = curvature[(i-1)%len(curvature)]
        curr_point = p
        next_point = curvature[(i+1)%len(curvature)]
        
        # Vector from previous to current
        v1_x = curr_point.x - prev_point.x
        v1_y = curr_point.y - prev_point.y
        
        # Vector from current to next
        v2_x = next_point.x - curr_point.x
        v2_y = next_point.y - curr_point.y
        
        # Calculate the angle between the vectors using atan2
        # This gives us the change in direction (curvature)
        angle1 = math.atan2(v1_y, v1_x)
        angle2 = math.atan2(v2_y, v2_x)
        
        # Calculate the angular difference
        angle_diff = angle2 - angle1
        
        # Normalize the angle to [-π, π]
        while angle_diff > math.pi:
            angle_diff -= 2 * math.pi
        while angle_diff < -math.pi:
            angle_diff += 2 * math.pi
        
        # Store the absolute curvature value
        p.curvature = abs(angle_diff)

    return curvature


def smooth_track_curvature(curvature_data: list[TrackPoint], attenuation_factor: float) -> list[TrackPoint]:
    """Smooth the track curvature data using a moving average, but only backward.
    The curvature is the maximum of the smoothed value and the original value.
    The smoothed value is a smooth attenuation of the high values ahead of the current point."""
    i = 0
    for i in range(len(curvature_data)):
        j = i
        while curvature_data[(j-1)%len(curvature_data)].curvature < attenuation_factor * curvature_data[j].curvature:
            curvature_data[(j-1)%len(curvature_data)].curvature = attenuation_factor * curvature_data[j].curvature
            j = (j-1) % len(curvature_data)
    return curvature_data


def save_track_curvature(curvature_data: list[TrackPoint], root_path: str):
    """Save the track curvature data to a binary file."""
    with open(f"{root_path}/curvature.bin", "wb") as f:
        f.write(struct.pack("i", len(curvature_data)))
        for point in curvature_data:
            f.write(struct.pack("fff", point.x, point.y, point.curvature))

def plot_track_curvature(curvature_data: list[TrackPoint], with_labels: bool = False, track_name: str = "Track"):
    """
    Plot the track curvature as a 2D scatter plot with color-coded curvature values.
    each dot is accompagnied with a label of the curvature value and index value
    
    Args:
        curvature_data: List of tuples (x, y, curvature_value)
        track_name: Name of the track for the plot title
    """
    if not curvature_data:
        print("No curvature data to plot")
        return
    
    # Extract x, y, and curvature values
    x_coords = [point.x for point in curvature_data]
    y_coords = [-point.y for point in curvature_data]
    curvature_values = [point.curvature for point in curvature_data]
    
    # Create the plot
    plt.figure(figsize=(12, 8))
    
    # Create scatter plot with color mapping
    scatter = plt.scatter(x_coords, y_coords, c=curvature_values, 
                         cmap='RdYlBu_r', s=30, alpha=0.8)
    
    # Add labels for each point showing index and curvature value
    if with_labels:
        for i, (x, y, curvature) in enumerate(zip(x_coords, y_coords, curvature_values)):
            plt.annotate(f'{i}\n{curvature:.2f}', 
                        (x, y), 
                        xytext=(5, 5), 
                        textcoords='offset points',
                        fontsize=6,
                        alpha=0.7,
                        bbox=dict(boxstyle='round,pad=0.2', facecolor='white', alpha=0.7))
            
    # Add colorbar
    colorbar = plt.colorbar(scatter)
    colorbar.set_label('Curvature Value (dx*dy)', rotation=270, labelpad=20)
    
    # Set labels and title
    plt.xlabel('X Coordinate')
    plt.ylabel('Y Coordinate')
    plt.title(f'{track_name} - Curvature Visualization')
    
    # Equal aspect ratio to maintain track shape
    plt.axis('equal')
    
    # Add grid for better readability
    plt.grid(True, alpha=0.3)
    
    # Tight layout for better appearance
    plt.tight_layout()
    
    # Show the plot
    plt.show()
    
    # Print some statistics
    min_curvature = min(curvature_values)
    max_curvature = max(curvature_values)
    avg_curvature = sum(curvature_values) / len(curvature_values)
    
    print(f"\n📊 Curvature Statistics:")
    print(f"   📈 Min: {min_curvature:.4f}")
    print(f"   📉 Max: {max_curvature:.4f}")
    print(f"   📊 Average: {avg_curvature:.4f}")
    print(f"   📍 Total Points: {len(curvature_values)}")


def main():
    """Main function to process SVG track file from command line arguments."""
    if len(sys.argv) != 2:
        print("Usage: python process.py <directory_path>")
        print("The directory should contain a 'track.svg' file")
        sys.exit(1)
    
    try:
        # Get directory path and construct SVG file path
        directory_path = Path(sys.argv[1])
        svg_file_path = directory_path / "track.svg"
        track_json_file_path = directory_path / "track.json"
        track_data = json.load(open(track_json_file_path))
        print_track_info(track_data)
        
        # Process the track
        processor = SVGTrackProcessor(svg_file_path)
        track_info = processor.get_track_info(distance_ratio=0.5)
        
        # Display results
        print(f"Total Length: {track_info['total_length']:.2f}")
        pos_info = track_info['position_at_ratio']
        print(f"Position at distance {pos_info['distance']:.2f}: "
              f"({pos_info['x']:.2f}, {pos_info['y']:.2f})")
              
        curve = compute_track_curvature(track_data, processor, 1)
        curve = smooth_track_curvature(curve, 1-0.05)
        save_track_curvature(curve, directory_path)
        
        # Get track name with fallback
        track_name = track_data.get('name', 'Track')
        plot_track_curvature(curve, track_name = track_name)

    except Exception as e:
        print(f"Error: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()



