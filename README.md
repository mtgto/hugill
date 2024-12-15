# Hugill

Hugill is VSCode Remote Container Launcher app for Kubernetes containers. Built with Tauri v2.

## How it works

VSCode supports to open k8s remote container like: `code --folder-uri vscode-remote://k8s-container+context=<context>+podname=<pod_name>+namespace=<namespace>+name=<container_name><remote_full_path>`
It requires VSCode extension [Remote Container](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers) is installed.

## Related projects

- [mtsmfm/vscode-k8s-quick-attach](https://github.com/mtsmfm/vscode-k8s-quick-attach) VSCode Extension to attach VSCode to k8s Pod

## Production Build

You must register Apple Developer Program and [generate app-specific password](https://support.apple.com/en-us/102654) before build for production.
See for details: https://tauri.app/distribute/sign/macos/#notarization

```console
APPLE_ID=<Your Apple ID> \
APPLE_TEAM_ID=<Your Team ID> \
APPLE_PASSWORD=<Your App-Specific Password> \
pnpm tauri build --bundles dmg
```

## License

MIT
