use typed_env::{Envar, EnvarDef};

/// Override for the backup directory path.
pub static DOING_BACKUP_DIR: Envar<String> = Envar::on_demand("DOING_BACKUP_DIR", || EnvarDef::Unset);

/// Path to the doing configuration file.
pub static DOING_CONFIG: Envar<String> = Envar::on_demand("DOING_CONFIG", || EnvarDef::Unset);

/// Enable debug mode.
pub static DOING_DEBUG: Envar<bool> = Envar::on_demand("DOING_DEBUG", || EnvarDef::Default(false));

/// Override for the editor used by doing.
pub static DOING_EDITOR: Envar<String> = Envar::on_demand("DOING_EDITOR", || EnvarDef::Unset);

/// Log level for the doing application.
pub static DOING_LOG_LEVEL: Envar<String> = Envar::on_demand("DOING_LOG_LEVEL", || EnvarDef::Unset);

/// Suppress output.
pub static DOING_QUIET: Envar<bool> = Envar::on_demand("DOING_QUIET", || EnvarDef::Default(false));

/// Standard `$EDITOR` environment variable.
pub static EDITOR: Envar<String> = Envar::on_demand("EDITOR", || EnvarDef::Unset);

/// Standard `$VISUAL` environment variable.
pub static VISUAL: Envar<String> = Envar::on_demand("VISUAL", || EnvarDef::Unset);
