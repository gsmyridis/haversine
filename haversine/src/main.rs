use std::fs::File;
use std::io::Read;
use std::time::Instant;

mod parse;
use parse::{Cursor, Value};

fn degrees_to_radians(angle: f64) -> f64 {
    angle * std::f64::consts::PI / 180.0
}

fn calculate_haversine_distance(
    radius: f64,
    phi_0: f64,
    theta_0: f64,
    phi_1: f64,
    theta_1: f64,
) -> f64 {
    let phi_0_radians = degrees_to_radians(phi_0);
    let phi_1_radians = degrees_to_radians(phi_1);
    let theta_0_radians = degrees_to_radians(theta_0);
    let theta_1_radians = degrees_to_radians(theta_1);

    let delta_thetas = theta_1_radians - theta_0_radians;
    let delta_phis = phi_1_radians - phi_0_radians;
    let root_term_1 = (delta_thetas / 2.0).sin().powi(2);
    let root_term_2 =
        theta_0_radians.cos() * theta_1_radians.cos() * (delta_phis / 2.0).sin().powi(2);
    let root_term = root_term_1 + root_term_2;
    2.0 * radius * root_term.sqrt().asin()
}

fn main() {
    let start_parsing = Instant::now();

    let file_name = "../gendata/pairs.json";
    let mut file = File::open(file_name).expect("Failed to open file");
    let mut string = String::new();
    let _n = file
        .read_to_string(&mut string)
        .expect("Failed to read file");

    let json = match Cursor::new(&string).parse().unwrap().unwrap() {
        Value::Object(object) => object,
        _ => panic!("Invalid pairs file"),
    };

    let end_parsing = Instant::now();

    let average_distance = match json.get("avg_dist").expect("Expected to exist") {
        Value::Number(avg) => avg,
        _ => panic!("Invalid pairs file"),
    };

    let radius = json.get("radius").expect("Expected to exist").get_number();

    let pairs = match json.get("pairs").expect("Expected to exist") {
        Value::Array(array) => array,
        _ => panic!("Invalid pairs file"),
    };

    let start_computing = Instant::now();

    let mut sum = 0.0;
    let n_pairs = pairs.len();
    println!("Number of pairs: {n_pairs}");
    println!("Radius: {radius}");
    for pair in pairs {
        let (phi_0, theta_0, phi_1, theta_1) = match pair {
            Value::Object(obj) => {
                let phi_0 = obj.get("phi_0").expect("Expected to exist").get_number();
                let phi_1 = obj.get("phi_1").expect("Expected to exist").get_number();
                let theta_0 = obj.get("theta_0").expect("Expected to exist").get_number();
                let theta_1 = obj.get("theta_1").expect("Expected to exist").get_number();
                (phi_0, theta_0, phi_1, theta_1)
            }
            _ => panic!("Invalid pair"),
        };
        sum += calculate_haversine_distance(radius, phi_0, theta_0, phi_1, theta_1);
    }

    let avg = sum / (n_pairs as f64);

    let end_computing = Instant::now();

    println!(
        "Difference between read and computed value: {}",
        average_distance - avg
    );

    println!(
        "Parsing time: {}",
        end_parsing.duration_since(start_parsing).as_secs_f64()
    );
    println!(
        "Computing time: {}",
        end_computing.duration_since(start_computing).as_secs_f64()
    );
}
