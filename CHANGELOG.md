# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0-rc.3](https://github.com/canardleteer/agent-rules-tool/compare/v0.1.0-rc.2...v0.1.0-rc.3) - 2026-08-31

### Added

- support discovery mode from updated agent-rules-spec

## [0.1.0-rc.2](https://github.com/canardleteer/agent-rules-tool/compare/v0.1.0-rc.1...v0.1.0-rc.2) - 2026-06-11

### Added

- add lint_and_migrate library example

### Other

- agent git workflow for staying aligned with main
- add Windows release binaries to cd.yml
- enable Windows Rust CI
- upload release binaries (non-Windows targets)
- install instructions for 0.1 RC line

## [0.1.0-rc.1](https://github.com/canardleteer/agent-rules-tool/releases/tag/v0.1.0-rc.1) - 2026-06-11

### Added

- initial layout + functionality

### Fixed

- *(ci)* use RELEASE_PLZ_TOKEN and split rust jobs for release PR checks

### Other

- path-aware PR gates, release-plz, and workflow action policy
- short reference list of external rule formats
- pin rustsec/audit-check by commit
- inital commit
