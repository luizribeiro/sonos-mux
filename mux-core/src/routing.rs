use crate::config::{Config, Route};
use crate::input::{create_input, AudioInput};
use crate::mixer::{Mixer, Source};
use std::collections::{HashMap, HashSet};

pub struct Router {
    // Maps output ID to its mixer
    pub output_mixers: HashMap<String, Mixer>,
}

impl Router {
    pub fn new(config: &Config) -> Result<Self, crate::input::InputError> {
        let mut output_mixers = HashMap::new();

        // First, create all inputs
        let mut inputs: HashMap<String, Box<dyn AudioInput>> = HashMap::new();
        for input_config in &config.inputs {
            let input = create_input(input_config)?;
            inputs.insert(input_config.id.clone(), input);
        }

        // Find all outputs used in routes
        let mut output_routes: HashMap<String, Vec<&Route>> = HashMap::new();
        for route in &config.routes {
            for output_id in &route.outputs {
                output_routes
                    .entry(output_id.clone())
                    .or_default()
                    .push(route);
            }
        }

        // For each output, create a mixer with the appropriate sources
        for output in &config.outputs {
            let routes = match output_routes.get(&output.id) {
                Some(routes) => routes,
                None => continue, // No routes to this output, skip it
            };

            // Create sources for each route to this output
            let mut sources = Vec::new();
            let mut used_inputs = HashSet::new();

            for route in routes {
                // Check if we've already used this input for this output
                if used_inputs.contains(&route.input) {
                    continue;
                }
                used_inputs.insert(&route.input);

                // Get the input for this source
                let input_id = &route.input;
                let input = match inputs.get(input_id) {
                    Some(input) => input.clone(),
                    None => continue, // Should not happen if config is validated
                };

                // Create the source with the route parameters
                let source = Source::new(
                    route.gain_db,
                    route.duck_db > 0.0, // If duck_db is set, this is a priority source
                    route.duck_db.abs(), // Use absolute value for ducking amount
                    input,
                );

                sources.push(source);
            }

            // Create the mixer for this output
            let mixer = Mixer::new(sources);
            output_mixers.insert(output.id.clone(), mixer);
        }

        Ok(Self { output_mixers })
    }

    pub fn start(&mut self) -> Result<(), crate::input::InputError> {
        for mixer in self.output_mixers.values_mut() {
            mixer.start()?;
        }
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), crate::input::InputError> {
        for mixer in self.output_mixers.values_mut() {
            mixer.stop()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, Input, Output, Route};

    #[test]
    fn test_router_creation() {
        // Create a test configuration
        let config = Config {
            inputs: vec![
                Input {
                    id: "silence1".to_string(),
                    kind: "silence".to_string(),
                    device: None,
                    path: None,
                    loop_playback: None,
                    url: None,
                },
                Input {
                    id: "silence2".to_string(),
                    kind: "silence".to_string(),
                    device: None,
                    path: None,
                    loop_playback: None,
                    url: None,
                },
            ],
            outputs: vec![
                Output {
                    id: "output1".to_string(),
                    kind: "sonos".to_string(),
                    room: Some("Living Room".to_string()),
                    host: None,
                    port: None,
                },
                Output {
                    id: "output2".to_string(),
                    kind: "sonos".to_string(),
                    room: Some("Bedroom".to_string()),
                    host: None,
                    port: None,
                },
            ],
            routes: vec![
                Route {
                    input: "silence1".to_string(),
                    outputs: vec!["output1".to_string()],
                    gain_db: 0.0,
                    duck_db: 0.0,
                },
                Route {
                    input: "silence2".to_string(),
                    outputs: vec!["output1".to_string(), "output2".to_string()],
                    gain_db: -6.0,
                    duck_db: 12.0,
                },
            ],
            logging: None,
        };

        // Create the router
        let router = Router::new(&config).unwrap();

        // Check that we have the expected number of mixers
        assert_eq!(router.output_mixers.len(), 2);

        // Check that output1 has two sources
        assert_eq!(
            router.output_mixers.get("output1").unwrap().source_count(),
            2
        );

        // Check that output2 has one source
        assert_eq!(
            router.output_mixers.get("output2").unwrap().source_count(),
            1
        );
    }
}
