# This is a vhs script. See https://github.com/charmbracelet/vhs for more info.
# To run this script, install vhs and run `vhs ratsnake.tape`
Set Theme { "name": "jwodder", "black": "#000000", "red": "#b30000", "green": "#00a600", "yellow": "#999900", "blue": "#6868d1", "magenta": "#b200b2", "cyan": "#00a6b2", "white": "#bfbfbf", "brightBlack": "#666666", "brightRed": "#ff0000", "brightGreen": "#00d900", "brightYellow": "#e5e500", "brightBlue": "#6f6fff", "brightMagenta": "#e500e5", "brightCyan": "#00e5e5", "brightWhite": "#e5e5e5", "background": "#000000", "foreground": "#f2f2f2", "selection": "#000000", "cursor": "#ffffff" }
Set FontSize 12
Set Width 628
Set Height 350
Set Padding 0
Type "cargo run -- -c tape-cfg.toml"
Enter
Sleep 2s
Screenshot mainmenu.png
Type p
Sleep 210ms
Right
Sleep 410ms
Screenshot game.png
