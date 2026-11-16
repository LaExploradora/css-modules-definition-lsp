use std::env;
use zed_extension_api::{self as zed, serde_json, LanguageServerId, Result};

const PACKAGE_NAME: &str = "cssmodules-language-server";
// Exact server version we tested. Change it only after running the manual checklist.
const PINNED_VERSION: &str = "1.5.2";
// Verify this path after the first install (see "Open questions" below).
const SERVER_PATH: &str = "node_modules/cssmodules-language-server/lib/cli.js";

fn server_args(script_abs: &str) -> Vec<String> {
    // The server always uses stdio, so it needs no transport flag.
    vec![script_abs.to_string()]
}

fn initialization_options() -> serde_json::Value {
    // Lets `styles.btnIcon` match `.btn-icon` in the SCSS file.
    serde_json::json!({ "camelCase": "dashes" })
}

// Install when the script is missing or the installed version is not the pinned one.
fn needs_install(script_exists: bool, installed: Option<&str>, pinned: &str) -> bool {
    !script_exists || installed != Some(pinned)
}

struct CssModulesExtension {
    did_find_server: bool,
}

impl CssModulesExtension {
    fn server_exists(&self) -> bool {
        std::fs::metadata(SERVER_PATH).is_ok_and(|stat| stat.is_file())
    }

    // Installs the npm package if needed. Returns the path to the server script.
    fn server_script_path(&mut self, id: &LanguageServerId) -> Result<String> {
        let exists = self.server_exists();
        if self.did_find_server && exists {
            return Ok(SERVER_PATH.to_string());
        }

        // Reading the installed version is local, so a normal start needs no network.
        let installed = zed::npm_package_installed_version(PACKAGE_NAME)
            .ok()
            .flatten();

        if needs_install(exists, installed.as_deref(), PINNED_VERSION) {
            zed::set_language_server_installation_status(
                id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );
            if let Err(error) = zed::npm_install_package(PACKAGE_NAME, PINNED_VERSION) {
                // An old copy can still work if the install fails, for example offline.
                if !self.server_exists() {
                    return Err(error);
                }
            }
        }

        self.did_find_server = true;
        Ok(SERVER_PATH.to_string())
    }
}

impl zed::Extension for CssModulesExtension {
    fn new() -> Self {
        Self {
            did_find_server: false,
        }
    }

    fn language_server_command(
        &mut self,
        id: &LanguageServerId,
        _worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let script = self.server_script_path(id)?;
        Ok(zed::Command {
            command: zed::node_binary_path()?,
            args: server_args(
                &env::current_dir()
                    .map_err(|e| e.to_string())?
                    .join(script)
                    .to_string_lossy(),
            ),
            env: Default::default(),
        })
    }

    fn language_server_initialization_options(
        &mut self,
        _id: &LanguageServerId,
        _worktree: &zed::Worktree,
    ) -> Result<Option<serde_json::Value>> {
        Ok(Some(initialization_options()))
    }
}

zed::register_extension!(CssModulesExtension);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn args_contain_only_the_script_path() {
        assert_eq!(server_args("/w/cli.js"), vec!["/w/cli.js"]);
    }

    #[test]
    fn args_keep_paths_with_spaces_as_one_argument() {
        assert_eq!(server_args("/my work/cli.js"), vec!["/my work/cli.js"]);
    }

    #[test]
    fn args_do_not_pass_stdio_flag() {
        assert!(!server_args("/w/cli.js").contains(&"--stdio".to_string()));
    }

    #[test]
    fn init_options_use_dashes_camel_case() {
        assert_eq!(initialization_options()["camelCase"], "dashes");
    }

    #[test]
    fn init_options_contain_only_camel_case() {
        assert_eq!(initialization_options().as_object().unwrap().len(), 1);
    }

    #[test]
    fn server_path_matches_package_bin() {
        assert_eq!(
            SERVER_PATH,
            format!("node_modules/{PACKAGE_NAME}/lib/cli.js")
        );
    }

    #[test]
    fn pinned_version_is_an_exact_semver() {
        let parts: Vec<&str> = PINNED_VERSION.split('.').collect();
        assert_eq!(parts.len(), 3, "use x.y.z, not a range or a tag");
        assert!(parts.iter().all(|p| p.parse::<u32>().is_ok()));
    }

    #[test]
    fn installs_when_script_is_missing() {
        assert!(needs_install(false, Some("1.5.2"), "1.5.2"));
        assert!(needs_install(false, None, "1.5.2"));
    }

    #[test]
    fn installs_when_version_differs_or_is_unknown() {
        assert!(needs_install(true, Some("1.4.0"), "1.5.2"));
        assert!(needs_install(true, Some("1.6.0"), "1.5.2"));
        assert!(needs_install(true, None, "1.5.2"));
    }

    #[test]
    fn skips_install_when_pinned_version_is_present() {
        assert!(!needs_install(true, Some("1.5.2"), "1.5.2"));
    }

    #[test]
    fn new_extension_has_not_found_server() {
        let ext = <CssModulesExtension as zed::Extension>::new();
        assert!(!ext.did_find_server);
    }
}
