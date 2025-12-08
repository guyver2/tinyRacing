use chrono::NaiveDate;
use tiny_racing::models::driver_avatar::generate_driver_avatar;

fn main() {
    // Example: Generate avatar for Max Verstappen
    let dob = NaiveDate::from_ymd_opt(1997, 9, 30).unwrap();
    let svg = generate_driver_avatar("Max Verstappen", "male", &dob);

    println!("Generated SVG for Max Verstappen:");
    println!("{}", svg);

    // Generate avatar for a female driver
    let dob2 = NaiveDate::from_ymd_opt(1995, 5, 15).unwrap();
    let svg2 = generate_driver_avatar("Test Driver", "female", &dob2);

    println!("\nGenerated SVG for female driver:");
    println!("{}", svg2);

    // Verify deterministic generation
    let svg3 = generate_driver_avatar("Max Verstappen", "male", &dob);
    assert_eq!(svg, svg3);
    println!("\n✓ Deterministic generation verified - same inputs produce same output");
}
