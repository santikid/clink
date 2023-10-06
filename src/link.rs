use std::{
    collections::HashSet,
    fs,
    io::ErrorKind,
    os::unix::fs::symlink,
    path::{Path, PathBuf},
};

pub struct Source {
    base: PathBuf,
    file: PathBuf,
}

impl Source {
    pub fn path(&self) -> PathBuf {
        self.base.join(&self.file)
    }
}

pub struct LinkGroup {
    pub target: PathBuf,
    sources: Vec<Source>,
}

impl LinkGroup {
    pub fn empty(target: PathBuf) -> Self {
        LinkGroup {
            target,
            sources: Vec::new(),
        }
    }
    pub fn add_source(&mut self, source: &PathBuf) -> Result<(), Vec<PathBuf>> {
        let source_files = get_all_paths(source);

        let sources = source_files
            .iter()
            .map(|x| Source {
                base: source.clone(),
                file: x.strip_prefix(source).unwrap().to_path_buf(),
            })
            .collect::<Vec<_>>();

        let source_files = sources.iter().map(|x| &x.file).collect::<HashSet<_>>();

        let self_files = self.sources.iter().map(|x| &x.file).collect::<HashSet<_>>();

        // check for conflicts
        let conflicts = self_files
            .intersection(&source_files)
            .cloned()
            .cloned()
            .collect::<Vec<_>>();

        if !conflicts.is_empty() {
            return Err(conflicts);
        }

        self.sources.extend(sources);

        Ok(())
    }
    fn target_conflicts(&self) -> Vec<PathBuf> {
        self.sources
            .iter()
            .filter_map(|src| {
                let target = self.target.join(&src.file);
                match fs::symlink_metadata(&target) {
                    Err(e) if e.kind() != ErrorKind::NotFound => Some(src.file.clone()),
                    Ok(f) if f.is_symlink() => {
                        let metadata = target.read_link().unwrap();
                        if metadata != src.path() {
                            Some(src.file.clone())
                        } else {
                            None
                        }
                    }
                    Ok(_) => Some(src.file.clone()),
                    _ => None,
                }
            })
            .collect()
    }

    pub fn link(&self) -> Result<(), Vec<PathBuf>> {
        let target_conflicts = self.target_conflicts();
        if !target_conflicts.is_empty() {
            return Err(target_conflicts);
        }

        self.sources.iter().for_each(|source| {
            let path = source.path();
            let target = self.target.join(&source.file);
            println!("linking {:?} to {:?}", path, target);
            fs::create_dir_all(
                target.parent().expect(
                    format!(
                        "could not get parent directory of link {}",
                        target.display()
                    )
                    .as_str(),
                ),
            )
            .expect(format!("could not create parent tree for link {}", target.display()).as_str());
            match symlink(path, target) {
                Ok(_) => {}
                Err(e) if e.kind() == ErrorKind::AlreadyExists => {}
                Err(e) => {
                    println!("error: {:?}", e);
                }
            }
        });

        Ok(())
    }

    pub fn unlink(&self, leave_orphans: bool) -> Result<(), Vec<PathBuf>> {
        self.sources.iter().for_each(|source| {
            let target = self.target.join(&source.file);
            println!("unlinking {:?}", target);
            match fs::symlink_metadata(&target) {
                Ok(f) if f.is_symlink() && target.read_link().unwrap() == source.path() => {
                    fs::remove_file(&target)
                        .expect(format!("could not remove link {}", target.display()).as_str());
                    if !leave_orphans {
                        let mut parent = target.parent().unwrap();
                        while parent.read_dir().unwrap().count() == 0 && parent != self.target {
                            fs::remove_dir(parent).unwrap();
                            println!("removing {:?}", parent);
                            match parent.parent() {
                                Some(p) if p != self.target => parent = p,
                                _ => break,
                            }
                        }
                    }
                }
                _ => {}
            }
        });
        Ok(())
    }
}
pub fn get_all_paths(source: &Path) -> HashSet<PathBuf> {
    walkdir::WalkDir::new(source)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .collect()
}
