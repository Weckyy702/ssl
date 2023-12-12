use ssl::{execute::execute, parser::parse};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = r"
        $0 .
    ";

    let code = parse(input.chars())?;

    execute(&code, vec!["Hello, world".into()])?;
    Ok(())
}
