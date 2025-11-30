export interface Tire {
  type: string;
  wear: number;
}

export interface Team {
  number: number;
  name: string;
  logo: string;
  color: string;
}

export interface Driver {
  name: string;
  skill_level: number;
  stamina: number;
  weather_tolerance: number;
  experience: number;
  consistency: number;
  focus: number;
}

export interface Car {
  race_position: number;
  car_number: number;
  driver: Driver;
  team: Team;
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
