# Letter | A simple task tracker for the terminal

TODO: Implement scripting language
```
set_background_color(color::red);
set_foreground_color(color::blue);

let arr: str[] = [];
let arr: str[] = []; // shadow < -- comment

arr = arr + ["hallo welt"];

fn init() {
    ncmd("<space>w", fn () {
        const window_id: int = get_active_window();
        move_window_to_left(window_id);
    });

    ncmd("<space>q", fn () {
        quit();
    });
}

```
