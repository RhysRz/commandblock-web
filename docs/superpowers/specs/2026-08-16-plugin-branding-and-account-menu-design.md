# Plugin branding and account menu

## Goal

Make the Plugin catalog comfortable to read on desktop, identify providers with
their familiar brand colors, and turn the signed-in email chip into a useful
account menu without changing authentication or provider permissions.

## Plugin catalog

- Widen only the Plugin dialog to a desktop maximum of about 980px while
  retaining the existing two-column card grid.
- Bundle a compact, local colored logo asset for each third-party provider.
  The catalog must not fetch logos from the network at runtime.
- Use consistent CommandBlock-colored symbols for first-party capabilities such
  as Local workspace, Terminal, Desktop Connector, and Remote PC.
- Give card content a constrained text column and a non-overlapping state badge.
  Descriptions may wrap to two lines on desktop; small screens retain one
  column with touch-safe spacing.
- Keep Public, Installed, search, categories, and capability states unchanged.

## Account menu

- Make the existing signed-in email chip a button that opens an Obsidian-purple
  menu upward from the lower application bar.
- The menu header shows the current display name when available and email.
- Provide actions for account management, connected devices, usage and credit,
  password-reset email, and sign out.  Sign out is visually separated and uses
  the existing CommandBlock confirmation dialog.
- Account management opens an in-app modal for the display name; password reset
  uses the existing Supabase auth flow.  Connected devices and usage/credit use
  the app's existing dialogs or navigation rather than introducing new access
  to credentials.
- Menu items do not expose API keys, passwords, or another account's data.

## Data flow and safety

Logo data is packaged as static assets only.  Account actions operate on the
current authenticated Supabase user and preserve existing owner-scoped data and
logout behavior.  Third-party Plugin cards remain descriptive until a real
integration is configured; logos do not imply that a connector is installed.

## Testing and release

- Add contract coverage for local Plugin brand assets, non-overlapping catalog
  layout hooks, account-menu trigger/actions, and confirmation-based logout.
- Run Node tests, Rust tests, a release build, and whitespace checks.
- Bump the app version, commit feature files without staging existing unrelated
  local changes, push `main`, and verify the resulting release artifact.

## Out of scope

- OAuth/API implementations for catalog providers.
- Editing email addresses, passwords, or external-provider credentials inside
  CommandBlock.
