use clap::{load_yaml, App};

fn main() {
    let clap_def = load_yaml!("cli_def.yaml");
    let matches = App::from_yaml(clap_def).get_matches();

    println!("{:?}", matches);

    img_dedup::hello_world();
}
