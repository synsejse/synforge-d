# synforge-d

synforge-d helps you keep package builds running and published from one simple dashboard.

You connect source repositories, sync updates, run builds, and view results in one place.

## Quick start

1. Copy `.env.example` to `.env` and set database credentials.
2. Start the stack with Docker Compose.
3. Open the web app.
4. Complete first-time setup and create an admin user.
5. Add packages and start syncing/building.

`phpMyAdmin` is available only when started with the `dev-tools` profile.

## What you can do

- Track packages and build activity
- Refresh all package sources from one action
- View build outputs and repository contents
- Use a dedicated statistics page for system-level insights
- Optionally enable repository + package signing from the Signing page

For technical details, architecture, and contributor guidance, see `AGENTS.md`.
