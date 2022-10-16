use crate::error::ErrorContext as EC;
use crate::languages::{config_file_type, SupportLang};
use anyhow::{Context, Result};
use ast_grep_config::{deserialize_sgconfig, from_yaml_string, AstGrepConfig, RuleCollection};
use ignore::WalkBuilder;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

pub fn find_config(config_path: Option<PathBuf>) -> Result<RuleCollection<SupportLang>> {
  let config_path = find_config_path_with_default(config_path).context(EC::ReadConfiguration)?;
  let config_str = read_to_string(&config_path).context(EC::ReadConfiguration)?;
  let sg_config = deserialize_sgconfig(&config_str).context(EC::ParseConfiguration)?;
  let base_dir = config_path
    .parent()
    .expect("config file must have parent directory");
  read_directory_yaml(base_dir, sg_config)
}

fn read_directory_yaml(
  base_dir: &Path,
  sg_config: AstGrepConfig,
) -> Result<RuleCollection<SupportLang>> {
  let mut configs = vec![];
  for dir in sg_config.rule_dirs {
    let dir_path = base_dir.join(dir);
    let walker = WalkBuilder::new(&dir_path)
      .types(config_file_type())
      .build();
    for dir in walker {
      let config_file = dir.with_context(|| EC::WalkRuleDir(dir_path.clone()))?;
      // file_type is None only if it is stdin, safe to unwrap here
      if !config_file
        .file_type()
        .expect("file type should be available for non-stdin")
        .is_file()
      {
        continue;
      }
      let path = config_file.path();
      let yaml = read_to_string(path).with_context(|| EC::ReadRule(path.to_path_buf()))?;
      let new_configs =
        from_yaml_string(&yaml).with_context(|| EC::ParseRule(path.to_path_buf()))?;
      configs.extend(new_configs);
    }
  }
  Ok(RuleCollection::new(configs))
}

pub fn read_test_files() {
  todo!("should return test yaml and snapshots")
}

const CONFIG_FILE: &str = "sgconfig.yml";
const SNAPSHOT_DIR: &str = "__snapshots__";

fn find_config_path_with_default(config_path: Option<PathBuf>) -> Result<PathBuf> {
  if let Some(config) = config_path {
    return Ok(config);
  }
  let mut path = std::env::current_dir()?;
  loop {
    let maybe_config = path.join(CONFIG_FILE);
    if maybe_config.exists() {
      break Ok(maybe_config);
    }
    if let Some(parent) = path.parent() {
      path = parent.to_path_buf();
    } else {
      break Ok(PathBuf::from(CONFIG_FILE));
    }
  }
}
