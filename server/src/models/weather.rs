use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Copy)]
pub enum WeatherState {
    Clear,
    Rain,
    Cloudy,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Weather {
    // vec of (time, state) pairs.
    // The time is the time in seconds since the start of the race.
    // The state is a floating point between 0.0 and 1.0.
    // - 1.0 is heavy rain
    // - 0.0 is clear sky
    // - 0.5 is cloudy
    // values are sorted by time
    pub state_change_time: Vec<(f32, f32)>,
}

impl Weather {
    #[allow(dead_code)] // todo remove this
    pub fn new_random(max_time: f32) -> Self {
        Weather {
            state_change_time: Self::random_weather_timeline(max_time),
        }
    }

    #[allow(dead_code)] // todo remove this
    pub fn random_weather_timeline(max_time: f32) -> Vec<(f32, f32)> {
        let mut rng = rand::rng();
        let mut state_change_time = Vec::new();
        state_change_time.push((0.0, rng.random_range(0.0..1.0)));
        let mut last_time = 0.0;
        while last_time < max_time {
            let time = last_time + rng.random_range(120.0..300.0);
            let state = rng.random_range(0.0..1.0);
            state_change_time.push((time, state));
            last_time = time;
        }
        // make sure the time is sorted, should be sorted by default but just in case
        state_change_time.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        state_change_time.clone()
    }

    pub fn get_state_at_time(&self, time: f32) -> f32 {
        if self.state_change_time.is_empty() {
            return 0.5; // Default to cloudy if no data
        }

        // If before first timepoint, return first value
        if time <= self.state_change_time[0].0 {
            return self.state_change_time[0].1;
        }

        // If after last timepoint, return the last value
        if time >= self.state_change_time.last().unwrap().0 {
            return self.state_change_time.last().unwrap().1;
        }

        // Find the two closest timepoints for interpolation
        for i in 0..self.state_change_time.len() - 1 {
            let (t1, s1) = self.state_change_time[i];
            let (t2, s2) = self.state_change_time[i + 1];

            if time >= t1 && time <= t2 {
                // Linear interpolation: s1 + (s2 - s1) * (time - t1) / (t2 - t1)
                let ratio = (time - t1) / (t2 - t1);
                return s1 + (s2 - s1) * ratio;
            }
        }
        // Fallback (shouldn't reach here)
        self.state_change_time.last().unwrap().1
    }

    #[allow(dead_code)] // todo remove this
    pub fn to_weather_state(state: f32) -> WeatherState {
        if state < 0.33 {
            WeatherState::Clear
        } else if state < 0.66 {
            WeatherState::Cloudy
        } else {
            WeatherState::Rain
        }
    }

    #[allow(dead_code)] // todo remove this
    pub fn to_weather_string(state: f32) -> String {
        match Self::to_weather_state(state) {
            WeatherState::Rain => "rain".to_string(),
            WeatherState::Cloudy => "cloudy".to_string(),
            WeatherState::Clear => "clear".to_string(),
        }
    }
}
