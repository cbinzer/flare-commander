<div align="center" style="text-align: center;">

<img src="public/logo.svg" alt="FlareCommander" width="120px" height="120px">

<h1 style="margin-top: 0;">FlareCommander</h1>

![set url](public/flare-commander.png 'FlareCommander Screenshot')

**FlareCommander** is a minimal GUI tool for macOS that makes it easy to manage your Cloudflare KV pairs and namespaces.

[Download](https://github.com/cbinzer/flare-commander/releases)

</div>

## Prerequisites

You need to have a Cloudflare User API Token with the following permissions: **Workers KV Storage:Edit**.

## Development

To run the project locally, you need to
have [Node.js](https://nodejs.org/), [Rust](https://www.rust-lang.org/tools/install)
and [Cargo](https://doc.rust-lang.org/cargo/getting-started.html) installed. Then, clone the repository and install the
dependencies. The project is build with [Tauri](https://tauri.app/). You can run it with the following command:

```bash
npm run tauri dev
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.