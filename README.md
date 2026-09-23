# smtpcatch

A headless, dummy SMTP server that prints incoming emails to stdout. Fills the gap left by heavier GUI tools like Mailhog or Mailpit when you just want to run a quick test from the CLI and pipe the output.

## Status

**built, untested**: the SMTP dummy server and mailparse logic is built. It has not been run against a real SMTP client in this sandbox environment.

## Installation

```sh
cargo install --path .
```

## Usage

```sh
smtpcatch --port 1025
```
