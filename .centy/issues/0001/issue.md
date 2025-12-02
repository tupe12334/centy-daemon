# Daemon version

We want to let the user update the daemon version and instead of being backward compatible we want to provide a migration in each release, so when a user update the version of the engine it will run the migration, also we want to let the user see and declare the daemon verion in the config.json file for a project, the default will be the latest but the user can set its own and it will support semver
