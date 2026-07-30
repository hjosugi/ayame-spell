Output site/public/terminal-demo.gif

Set Shell "bash"
Set Theme "Catppuccin Mocha"
Set Width 1200
Set Height 720
Set FontSize 27
Set FontFamily "JetBrains Mono"
Set TypingSpeed 25ms
Set Framerate 24
Set CursorBlink false

Hide
Type 'repo=$(pwd); demo_dir=$(mktemp -d); cp contrib/demo/project/README.md "$demo_dir/README.md"; cd "$demo_dir"; export PATH="$repo/target/debug:$PATH"; clear'
Enter
Sleep 500ms
Show

Type "ayame-spell words triage --limit 1 README.md"
Enter
Sleep 2s
Enter
Sleep 700ms
Down@150ms 3
Sleep 700ms
Enter
Sleep 2s

Type "ayame-spell fix --dry-run README.md"
Enter
Sleep 2s

Type "ayame-spell fix README.md"
Enter
Sleep 1.5s

Type "ayame-spell check README.md"
Enter
Sleep 2s
