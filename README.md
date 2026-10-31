# open-omaha

An open source software update server for clients that use the Omaha protocol.
Register applications, publish releases through a REST API, and serve update
checks signed with Client Update Protocol (CUP).

## Features

- Omaha JSON protocol 4.0 with ML-DSA-44 CUP signatures.
- Release selection by version, channel, OS, and updater architecture.
- A bearer-authenticated management API for applications, releases, and
  reported client events.
- A single Rust binary with SQLite storage.

You control package hosting, signing keys, and client trust. Clients download
packages directly from your HTTPS host and verify package hashes and
signatures before installation. The server manages release metadata.

## Status

**Early development. Not ready for production use.**

Local testing with a pinned Chromium updater has passed installation, update,
and signature rejection checks on Linux x64. GitHub-hosted acceptance against
an exact release-candidate commit is still pending.

The current implementation supports protocol 4.0 and SQLite only. Release
editing, deletion, withdrawal, rollback, pagination, and event retention are
not implemented. Reported events are unverified client claims and can contain
duplicates.

This repository currently contains the project overview. Source code, build
instructions, documentation, and test assets will follow.

## License

The project is intended to be released under the MIT license. The license
file will be published with the source code.
