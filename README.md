# TUI File Manager (WIP)

This is a simple TUI (terminal user interface) file manager built with Rust. It aims to provide a lightweight and efficient way to navigate and manage files directly from the terminal.

## Running
```
git clone https://github.com/arjav0703/tui-file-manager.git
cd tui-file-manager
```

- Make sure cargo in installed (https://rustup.rs/)
`cargo run`
OR 
`cargo run -- --show-hidden-files`



## Keyboard Operations
- `q`: Quit the application
- `j or ⬇️`: Move down
- `k or ⬆️`: Move up
- `h or ⬅️`: Go to parent directory
- `l or ➡️`: Enter the selected directory
- `d`: Delete the selected file or directory
- `ENTER` : Open the selected file with the default system application (`open` on macOS, `xdg-open` on Linux, `start` on Windows)
- `r`: Rename the selected file or directory
- `y`: Copy the path of selected file or directory to clipboard
- `a`: Add a new file
- `c`: Copy the selected file or directory
- `x`: Cut the selected file or directory
- `p`: Paste the copied or cut file or directory into the current directory


--- 
<div align="center">
  <a href="https://moonshot.hackclub.com" target="_blank">
    <img src="https://hc-cdn.hel1.your-objectstorage.com/s/v3/35ad2be8c916670f3e1ac63c1df04d76a4b337d1_moonshot.png" 
         alt="This project is part of Moonshot, a 4-day hackathon in Florida visiting Kennedy Space Center and Universal Studios!" 
         style="width: 100%;">
  </a>
</div>
