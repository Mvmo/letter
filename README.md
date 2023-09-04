# Letter | A simple task tracker for the terminal

TODO: Implement scripting language
```
struct GlobalContext {
    cursor: [int][2]
}

fn move_cursor_left(ctx: GlobalContext) <
    let x = ctx.cursor[0]
    let y = ctx.cursor[1]

    ctx.cursor = [x + 1, y]
>

fn init() <
    loop {
        move_cursor_left();
        move_cursor_right();
    }
>
```
