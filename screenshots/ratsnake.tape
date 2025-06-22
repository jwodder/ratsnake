# This is a vhs script. See https://github.com/charmbracelet/vhs for more info.
# To run this script, install vhs and run `vhs ratsnake.tape`
Set Theme "Aardvark Blue"
Set FontSize 12
Set Width 628
Set Height 350
Set Padding 0
Type "cargo run"
Enter
Sleep 2s
Screenshot mainmenu.png
Type p
Sleep 210ms
Right
Sleep 410ms
Screenshot game.png
