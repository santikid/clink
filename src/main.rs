use std::{env, fs, path::PathBuf};

use clap::{Parser, Subcommand};

use crate::link::LinkGroup;

mod feature;
mod link;

#[derive(Debug, serde::Deserialize)]
struct Config {
    features: feature::FeatureList,
}

fn extract_slugs(name: &str) -> Option<Vec<String>> {
    let ob = name.find('{');
    let cb = name.find('}');
    if ob.is_none() || cb.is_none() {
        return None;
    }
    let ob = ob.unwrap();
    let cb = cb.unwrap();
    if ob != 0 || ob > cb {
        return None;
    }
    Some(
        name[ob + 1..cb]
            .split(',')
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
    )
}

#[derive(Subcommand, Debug)]
enum Action {
    Link,
    Unlink {
        #[arg(long, short, default_value_t = false)]
        leave_orphans: bool 
    },
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    action: Action,
}

fn main() {
    let args = Args::parse();
    run(args.action);
}

fn run(action: Action) {
    // read config and parse it
    let config = fs::read_to_string("clink.yaml").expect("could not read config");
    let config: Config = serde_yaml::from_str(&config).expect("could not parse config");

    // filter out disabled features
    let enabled_features = config.features.filter_enabled();

    // get all enabled directories with the highest priority feature
    let source_features =
        fs::read_dir(env::current_dir().expect("could not get current directory"))
            .expect("could not read current directory")
            // filter out error results
            .filter_map(|e| e.ok())
            // filter out non-directories
            .filter(|e| e.metadata().unwrap().is_dir())
            // filter out directories that don't have an enabled slug
            .filter_map(|e| {
                let slugs = extract_slugs(&e.file_name().to_string_lossy());
                match slugs {
                    None => None,
                    Some(slugs) => enabled_features.get_first_match(&slugs).map(|f| (f, e.path())),
                }
            })
            .collect::<Vec<_>>();

    // create LinkGroup for each target
    let target_links =
        source_features
            .iter()
            .fold(Vec::<LinkGroup>::new(), |mut acc, (feature, source)| {
                // expand target path
                let target = PathBuf::from(
                    shellexpand::full(&feature.target)
                        .expect(
                            format!(
                                "could not expand target path for feature {:?}",
                                feature.slug
                            )
                            .as_str(),
                        )
                        .to_string(),
                );

                // check if LinkGroup for target exists, if not create new one and get mutable reference
                let entry = match acc.iter_mut().find(|x| x.target == target) {
                    Some(e) => e,
                    None => {
                        acc.push(LinkGroup::empty(target.clone()));
                        acc.last_mut().unwrap()
                    }
                };

                // add current directory to LinkGroup
                match entry.add_source(source) {
                    Err(e) => panic!("conflicts in target {:?}: {:?}", target, e),
                    Ok(_) => acc,
                }
            });

    target_links.iter().for_each(|link| {
        match action {
            Action::Link => {
                match link.link() {
                    Err(e) => panic!("conflicts in target {:?}: {:?}", link.target, e),
                    Ok(_) => (),
                }
            }
            Action::Unlink { leave_orphans } => {
                match link.unlink(leave_orphans) {
                    Err(e) => panic!("conflicts in target {:?}: {:?}", link.target, e),
                    Ok(_) => (),
                }
            }
        }
    });
}
