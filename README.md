# Hugill

VSCode Remote Container Launcher app.
Hugill is using Tauri v2.

## Production Build

You must register Apple Developer Program and [generate app-specific password](https://support.apple.com/en-us/102654) before build for production.
See for details: https://tauri.app/distribute/sign/macos/#notarization

```console
xcrun notarytool store-credentials --apple-id "<Your Apple ID>" --team-id "<Your Team ID>"
```

```console
APPLE_ID=<Your Apple ID> APPLE_TEAM_ID=<Your Team ID> APPLE_PASSWORD=<Your App-Specific Password> pnpm tauri build --bundles dmg
```

## License

MIT
