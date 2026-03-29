use CliPG::cli_pg::CliPG as CliPG_class;
use CliPG::pg_gui::start_pg_gui;
use clap::{Arg, Command};

fn main() {
    let matches = Command::new("CliPG")
        .version("1.0")
        .author("Bcedu")
        .about("Pastanaga Bullida")
        .arg_required_else_help(true) // Mostra ajuda si no hi ha arguments
        .arg(
            Arg::new("list")
                .short('l')
                .long("list")
                .help("Mostra tots els videojocs habilitats per sincornitzar-se")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("gui")
                .short('g')
                .long("gui")
                .help("Inicia l'aplicacio amb interfície gràfica")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("add")
                .short('a')
                .long("add")
                .help("Afegeix un videojoc amb la ruta donada")
                .num_args(1)
                .value_name("videojoc_path"),
        )
        .arg(
            Arg::new("remove")
                .short('r')
                .long("remove")
                .help("Elimina un videojoc pel seu ID")
                .num_args(1)
                .value_name("videojoc_id"),
        )
        .arg(
            Arg::new("sync_all")
                .short('s')
                .long("sync_all")
                .help("Sincronitza tots els videojocs habilitats")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("sync_all_test")
                .short('t')
                .long("sync_all_test")
                .help("Sincronitza tots els videojocs habilitats (test mode)")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    if matches.get_flag("gui") {
        start_pg_gui(None);
    } else {
        let mut clipg = CliPG_class::default(None);
        if matches.get_flag("list") {
            for v in clipg.config.videojocs_habilitats.list.iter() {
                println!("* {}\n    -> {}\n", v.nom, v.path);
            }
        } else if let Some(path) = matches.get_one::<String>("add") {
            let res = clipg.afegir_joc(path.to_string());
            res.unwrap_or_else(|err| println!("{err}"));
        } else if let Some(videojoc) = matches.get_one::<String>("remove") {
            let res = clipg.eliminar_joc(videojoc.to_string());
            res.unwrap_or_else(|err| println!("{err}"));
        } else if matches.get_flag("sync_all") {
            println!("Sincronitzant tots els videojocs...");
            let res = clipg.sync_all(false);
            println!("{res}");
        } else if matches.get_flag("sync_all_test") {
            println!("Sincronitzant tots els videojocs (test mode)...");
            let res = clipg.sync_all(true);
            println!("{res}");
        }
    }
}
