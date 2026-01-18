use CliPG::cli_pg::CliPG;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "CliPG",
        options,
        Box::new(|_cc| Ok(Box::new(CliPG::default()))),
    )
}

