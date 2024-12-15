# Hugill

Hugill is VSCode Remote Container Launcher app for Kubernetes containers.

Hugill is built with Tauri v2.

![demo](https://github.com/user-attachments/assets/8a338f0f-12d0-419d-a356-665be429f1af)

## How it works

VSCode supports to open k8s remote container like: `code --folder-uri vscode-remote://k8s-container+context=<context>+podname=<pod_name>+namespace=<namespace>+name=<container_name><remote_full_path>`

It requires VSCode extension [Remote Container](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers) is installed.

## Tech Stack

- [Tauri v2](https://tauri.app/)
- [SvelteKit](https://svelte.dev/)
- [Bulma](https://bulma.io/)
- [kube.rs](https://kube.rs/)

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
