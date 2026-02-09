use std::{env, fs};
use zed_extension_api::{self as zed, Result};

const SERVER_PATH: &str = "node_modules/naive-ui-language-server/dist/server.js";
const PACKAGE_NAME: &str = "naive-ui-language-server";

struct NaiveUIExtension;

impl NaiveUIExtension {
    fn server_exists(&self) -> bool {
        fs::metadata(SERVER_PATH).is_ok_and(|stat| stat.is_file())
    }

    fn server_script_path(&mut self, language_server_id: &zed::LanguageServerId) -> Result<String> {
        let server_exists = self.server_exists();
        if server_exists {
            return Ok(SERVER_PATH.to_string());
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );
        let version = zed::npm_package_latest_version(PACKAGE_NAME)?;

        if !server_exists
            || zed::npm_package_installed_version(PACKAGE_NAME)?.as_ref() != Some(&version)
        {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );
            let result = zed::npm_install_package(PACKAGE_NAME, &version);
            match result {
                Ok(()) => {
                    if !self.server_exists() {
                        Err(format!(
                            "installed package '{PACKAGE_NAME}' did not contain expected path '{SERVER_PATH}'",
                        ))?;
                    }
                }
                Err(error) => {
                    if !self.server_exists() {
                        Err(error)?;
                    }
                }
            }
        }

        Ok(SERVER_PATH.to_string())
    }
}

impl zed::Extension for NaiveUIExtension {
    fn new() -> Self {
        Self
    }
    fn language_server_command(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        _worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let server_path = self.server_script_path(language_server_id)?;
        Ok(zed::Command {
            command: zed::node_binary_path()?,
            args: vec![
                env::current_dir()
                    .unwrap()
                    .join(&server_path)
                    .to_string_lossy()
                    .to_string(),
                "--stdio".to_string(),
            ],
            env: Default::default(),
        })
    }

    // fn language_server_initialization_options(
    //     &mut self,
    //     _language_server_id: &zed::LanguageServerId,
    //     worktree: &zed::Worktree,
    // ) -> Result<Option<serde_json::Value>> {
    //     let initialization_options = LspSettings::for_worktree("vue", worktree)
    //         .ok()
    //         .and_then(|settings| settings.initialization_options)
    //         .unwrap_or_else(|| json!({}));
    //     Ok(Some(initialization_options))
    // }
}

zed::register_extension!(NaiveUIExtension);
