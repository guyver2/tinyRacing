export interface Tire {
  type: string;
  wear: number;
}

export interface Car {
  race_position: number;
  car_number: number;
  driver: string;
  team_name: string;
  team_number: number;
  tire: Tire;
  fuel: number;
  track_position: number;
  status: string;
  driving_style: string;
  speed: number;
}

export interface Track {
  id: string;
  name: string;
  svg_start_offset: number;
  current_weather: string;
  wetness: number;
  elapsed_time: number;
}

export interface RaceState {
  track: Track;
  cars: Car[];
  race_status: string;
  current_lap: number;
  total_laps: number;
} 