//! This module defines a program which calculates the expected traffic delay on a certain path
//! using the Google Maps Directions API. This can be used to calculate the extra amount of time
//! you need on your way to work, for example.

extern crate rest_client;
extern crate rustc_serialize;

use rest_client::RestClient;
use rustc_serialize::json::Json;

use std::io::prelude::*;
use std::fs::File;

/// A function which returns the amount of time it will take to travel from origin to destination
/// via specified waypoints by car, according to the Google Maps Directions API.
/// Multiple waypoints are to be separated by | pipe characters, or ultimately as specified by the
/// Google Maps Directions API.
fn get_duration( origin: &str, destination: &str, waypoints: &str ) -> i64 {
    let mut result: i64 = 0i64;

    // Get API response
    let response = RestClient::get_with_params( 
        "https://maps.googleapis.com/maps/api/directions/json", 
        &[  ("origin", origin), 
            ("destination", destination), 
            ("waypoints", waypoints),
            ("departure_time", "now"),
            ("traffic_model", "best_guess"),
            ("mode", "driving"),
            ("key", "" ) ] ).unwrap();

    // Travel down the json tree, retrieve the array saved in
    // DOC -> routes[0] -> legs
    let response_json   = Json::from_str( &response.body ).unwrap();
    let routes          = response_json.search( "routes" ).unwrap();
    let first_route     = routes.as_array().unwrap()[0].as_object().unwrap();
    let leg_array       = first_route.get( "legs" ).unwrap().as_array().unwrap();

    // Go through all array entries and accumulate the times for this route
    for leg in leg_array {
        // Travel even further down the json tree to get the duration of the leg
        let leg_object = leg.as_object().unwrap();
        let duration = leg_object.get( "duration_in_traffic" ).unwrap();
        let value = duration.as_object().unwrap().get( "value" ).unwrap();

        // Add it to the accumulator
        result += value.as_i64().unwrap();
    }

    return result;
}

/// Returns a string containing the encoding of a given amount of seconds in -M:SS format.
fn get_minute_string( seconds: i64 ) -> String {
    let minutes: i64 = seconds / 60i64;
    let seconds: i64 = seconds % 60i64;
    let result: String;

    // Format depends on sign of seconds
    match seconds < 0 {
        false       => result = format!( "{}:{:02}", minutes, seconds ),
        _           => result = format!( "-{}:{}", minutes, seconds.abs() ), 
    }

    return result;
}

/// Stores information about a waypoint used to describe legs
#[derive( RustcDecodable, RustcEncodable )]
struct Waypoint {
    /// Information about the waypoint in a format that can be displayed to the user
    description: String,
    /// Information about the waypoint in a format that is understood by the Google Maps Directions
    /// API
    internal: String,
}

/// A leg from one point to another, containing a description, origin/destination waypoints and
/// optionally via waypoints.
#[derive( RustcDecodable, RustcEncodable )]
struct Leg {
    /// A description for this leg
    description: String,
    /// The origin of this leg
    origin: Waypoint,
    /// The destination of this leg
    destination: Waypoint,
    /// A waypoint containing one or more stops on the way
    via: Waypoint,
    /// The usual duration this route takes according to Google Maps, in seconds
    usual_internal_duration: i64,
    /// The duration of this leg according to the timetable
    usual_timetable_duration: i64,
}

impl Leg {
    /// Gets the duration of this leg using a call to the Google Maps Directions API.
    fn duration( &self ) -> i64 {
        return get_duration( &self.origin.internal, &self.destination.internal, &self.via.internal ) + ( self.usual_timetable_duration - self.usual_internal_duration );
    }
}

/// Gets the current profile.
///
/// Based on whether arguments have been passed to the program, loads a suitable json file for
/// populating the Leg vector.
fn get_profile() -> Vec<Leg> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        return load_profile( &args[1] );
    } else {
        return load_profile( "profile.json" );
    }
}

/// Loads an array of legs from the given file name, where the data is stored in JSON.
fn load_profile( filename: &str ) -> Vec<Leg> {
    let mut content: String = String::new();
    let mut file = File::open(filename).unwrap();
    file.read_to_string( &mut content ).unwrap();

    return rustc_serialize::json::decode(&content[..]).unwrap();
}

/// The entry point of wayplan. Calls the Google Maps API and prints the result to the console
/// output.
fn main() {
    // Gets the profile which contains the route information
    let profile = get_profile();

    // Print loop
    for x in 0..profile.len() {
        // Takes one leg from the profile
        let ref leg = profile[x];

        // Prints the result
        println!( "{}: {} -> {}", leg.description, leg.origin.description, leg.destination.description );
        println!( "    Predicted duration: {} min (deviation {} min)", get_minute_string( leg.duration() ), get_minute_string( leg.duration() - leg.usual_timetable_duration ) );
        println!( "" );
    }
}


