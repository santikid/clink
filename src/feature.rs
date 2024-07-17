use std::process::Command;

#[derive(Debug, serde::Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Enabled {
    All,
    MacOS,
    Linux,
    Command(String),
    None,
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct Feature {
    pub slug: String,
    pub target: String,
    enabled: Enabled,
}

impl Feature {
    pub fn enabled(&self) -> bool {
        match &self.enabled {
            Enabled::None => false,
            Enabled::All => true,
            Enabled::MacOS => cfg!(target_os = "macos"),
            Enabled::Linux => cfg!(target_os = "linux"),
            Enabled::Command(cmd) => Command::new(cmd).status().is_ok_and(|r| r.success()),
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct FeatureList(Vec<Feature>);

impl FeatureList {
    pub fn filter_slugs(&self, slugs: &[String]) -> FeatureList {
        FeatureList(
            self.0
                .iter()
                .filter(|f| slugs.contains(&f.slug))
                .cloned()
                .collect(),
        )
    }
    pub fn filter_enabled(&self) -> FeatureList {
        FeatureList(
            self.0
                .iter()
                .filter(|x| x.enabled())
                .cloned()
                .collect::<Vec<_>>(),
        )
    }
    pub fn get_first_match(&self, slugs: &[String]) -> Option<&Feature> {
        self.0.iter().find(|f| slugs.contains(&f.slug))
    }
}
