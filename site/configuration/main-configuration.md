# Main Configuration

You can modify WayCast's behavior through `waycast.toml`

```toml
[plugins.file_search]
search_paths = [
    "/home/yourusername/your-folder",
]
ignore_dirs = ["zig-cache", "__pycache"]

[plugins.projects]
search_paths = ["/home/yourusername/projects"]
open_command = "code -n {path}"
```