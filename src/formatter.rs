use colored::*;

pub fn print_explanation(explanation: &str, no_color: bool) {
    if no_color {
        println!("{}", explanation);
    } else {
        if explanation.to_lowercase().contains("solution") {
            println!("{}", explanation.green());
        } else {
            println!("{}", explanation.blue());
        }
    }
}
